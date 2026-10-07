use std::{
    borrow::Cow,
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{RecvTimeoutError, Sender};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams, Hover,
    HoverContents, HoverParams, HoverProviderCapability, LogMessageParams, MarkupContent,
    MarkupKind, MessageType, Position, Range, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, LogMessage,
        Notification as LspNotification,
    },
    request::{HoverRequest, Request as LspRequest},
};

mod render;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let plantuml = env::args()
        .skip_while(|arg| arg != "--plantuml")
        .nth(1)
        .map(PathBuf::from)
        .ok_or("missing `--plantuml <path>` argument")?;

    let (connection, io_threads) = Connection::stdio();
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        hover_provider: Some(HoverProviderCapability::Simple(true)),
        ..Default::default()
    };
    connection.initialize(serde_json::to_value(capabilities)?)?;
    log(
        &connection.sender,
        MessageType::INFO,
        format!("started with plantuml `{}`", plantuml.display()),
    );

    let (renders, jobs) = crossbeam_channel::unbounded();
    let render_logger = connection.sender.clone();
    let render_thread = thread::spawn(move || render::run(plantuml, jobs, render_logger));

    Server {
        documents: HashMap::new(),
        pending: HashMap::new(),
        renders,
        logger: connection.sender.clone(),
    }
    .run(&connection)?;

    render_thread.join().expect("render thread panicked");
    drop(connection);
    io_threads.join()?;
    Ok(())
}

struct Server {
    documents: HashMap<String, String>,
    pending: HashMap<String, Instant>,
    renders: Sender<render::Job>,
    logger: Sender<Message>,
}

impl Server {
    fn run(mut self, connection: &Connection) -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            let message = match self.pending.values().min() {
                Some(deadline) => {
                    let timeout = deadline.saturating_duration_since(Instant::now());
                    match connection.receiver.recv_timeout(timeout) {
                        Ok(message) => Some(message),
                        Err(RecvTimeoutError::Timeout) => None,
                        Err(RecvTimeoutError::Disconnected) => return Ok(()),
                    }
                }
                None => match connection.receiver.recv() {
                    Ok(message) => Some(message),
                    Err(_) => return Ok(()),
                },
            };

            match message {
                Some(Message::Request(request)) => {
                    if connection.handle_shutdown(&request)? {
                        return Ok(());
                    }
                    let response = self.handle_request(request);
                    connection.sender.send(response.into())?;
                }
                Some(Message::Notification(notification)) => {
                    self.handle_notification(notification)?
                }
                Some(Message::Response(_)) | None => {}
            }

            self.send_due_renders()?;
        }
    }

    fn handle_request(&mut self, request: Request) -> Response {
        let id = request.id;
        let params = request.params;
        let result = match request.method.as_str() {
            HoverRequest::METHOD => parse_params(params).and_then(|params| self.hover(params)),
            method => {
                return Response::new_err(
                    id,
                    ErrorCode::MethodNotFound as i32,
                    format!("unhandled method `{method}`"),
                );
            }
        };

        match result {
            Ok(value) => Response::new_ok(id, value),
            Err(message) => Response::new_err(id, ErrorCode::InvalidParams as i32, message),
        }
    }

    fn hover(&self, params: HoverParams) -> Result<serde_json::Value, String> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;
        let Some(path) = file_path(&uri) else {
            return Ok(serde_json::Value::Null);
        };
        let text = match self.documents.get(&uri) {
            Some(text) => Cow::Borrowed(text.as_str()),
            None => match fs::read_to_string(&path) {
                Ok(text) => {
                    log(
                        &self.logger,
                        MessageType::LOG,
                        format!(
                            "hover requested before open, read `{}` from disk",
                            path.display()
                        ),
                    );
                    Cow::Owned(text)
                }
                Err(_) => return Ok(serde_json::Value::Null),
            },
        };

        let Some((idx, (line, content))) = text
            .lines()
            .enumerate()
            .filter(|(_, content)| content.starts_with("@start"))
            .enumerate()
            .find(|(_, (line, _))| *line == position.line as usize)
        else {
            return Ok(serde_json::Value::Null);
        };
        let len = content.split_whitespace().next().map_or(0, str::len);
        if position.character as usize > len {
            return Ok(serde_json::Value::Null);
        }
        let Some(target) =
            render::png_path(&path, idx).and_then(|png| url::Url::from_file_path(png).ok())
        else {
            return Ok(serde_json::Value::Null);
        };

        log(
            &self.logger,
            MessageType::LOG,
            format!("hover preview link for diagram {idx} of `{uri}`"),
        );
        let hover = Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("[Open preview]({target})"),
            }),
            range: Some(Range {
                start: Position::new(line as u32, 0),
                end: Position::new(line as u32, len as u32),
            }),
        };
        serde_json::to_value(hover).map_err(|err| err.to_string())
    }

    fn handle_notification(
        &mut self,
        notification: Notification,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        match notification.method.as_str() {
            DidOpenTextDocument::METHOD => {
                let params: DidOpenTextDocumentParams =
                    notification.extract(DidOpenTextDocument::METHOD)?;
                let uri = params.text_document.uri.to_string();
                log(&self.logger, MessageType::LOG, format!("opened `{uri}`"));
                self.pending.insert(uri.clone(), Instant::now());
                self.documents.insert(uri, params.text_document.text);
            }
            DidChangeTextDocument::METHOD => {
                const DEBOUNCE: Duration = Duration::from_millis(150);

                let params: DidChangeTextDocumentParams =
                    notification.extract(DidChangeTextDocument::METHOD)?;
                let uri = params.text_document.uri.to_string();
                if let Some(change) = params.content_changes.into_iter().last() {
                    self.pending.insert(uri.clone(), Instant::now() + DEBOUNCE);
                    self.documents.insert(uri, change.text);
                }
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams =
                    notification.extract(DidCloseTextDocument::METHOD)?;
                let uri = params.text_document.uri.to_string();
                log(&self.logger, MessageType::LOG, format!("closed `{uri}`"));
                self.documents.remove(&uri);
                self.pending.remove(&uri);
            }
            _ => {}
        }
        Ok(())
    }

    fn send_due_renders(&mut self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let now = Instant::now();
        let due: Vec<String> = self
            .pending
            .iter()
            .filter(|(_, deadline)| **deadline <= now)
            .map(|(uri, _)| uri.clone())
            .collect();

        for uri in due {
            self.pending.remove(&uri);
            if let (Some(path), Some(text)) = (file_path(&uri), self.documents.get(&uri)) {
                self.renders.send(render::Job {
                    path,
                    text: text.clone(),
                })?;
            }
        }
        Ok(())
    }
}

fn log(logger: &Sender<Message>, typ: MessageType, message: String) {
    let params = LogMessageParams { typ, message };
    logger
        .send(Notification::new(LogMessage::METHOD.to_string(), params).into())
        .ok();
}

fn file_path(uri: &str) -> Option<PathBuf> {
    url::Url::parse(uri).ok()?.to_file_path().ok()
}

fn parse_params<P: serde::de::DeserializeOwned>(params: serde_json::Value) -> Result<P, String> {
    serde_json::from_value(params).map_err(|err| err.to_string())
}
