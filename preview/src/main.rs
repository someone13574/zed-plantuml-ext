use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{RecvTimeoutError, Sender};
use lsp_server::{Connection, ErrorCode, Message, Notification, Request, Response};
use lsp_types::{
    CodeAction, CodeActionOrCommand, CodeActionParams, CodeActionProviderCapability, Command,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    ExecuteCommandOptions, ExecuteCommandParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument,
        Notification as LspNotification,
    },
    request::{CodeActionRequest, ExecuteCommand, Request as LspRequest},
};

mod render;

const START_COMMAND: &str = "plantuml-preview.start";
const STOP_COMMAND: &str = "plantuml-preview.stop";

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let plantuml = env::args()
        .skip_while(|arg| arg != "--plantuml")
        .nth(1)
        .map(PathBuf::from)
        .ok_or("missing `--plantuml <path>` argument")?;

    let (connection, io_threads) = Connection::stdio();
    let capabilities = ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
        execute_command_provider: Some(ExecuteCommandOptions {
            commands: vec![START_COMMAND.to_string(), STOP_COMMAND.to_string()],
            ..Default::default()
        }),
        ..Default::default()
    };
    connection.initialize(serde_json::to_value(capabilities)?)?;

    let (renders, jobs) = crossbeam_channel::unbounded();
    let render_thread = thread::spawn(move || render::run(plantuml, jobs));

    Server {
        documents: HashMap::new(),
        previewing: HashSet::new(),
        pending: HashMap::new(),
        renders,
    }
    .run(&connection)?;

    render_thread.join().expect("render thread panicked");
    drop(connection);
    io_threads.join()?;
    Ok(())
}

struct Server {
    documents: HashMap<String, String>,
    previewing: HashSet<String>,
    pending: HashMap<String, Instant>,
    renders: Sender<render::Job>,
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
            CodeActionRequest::METHOD => {
                parse_params(params).and_then(|params| self.code_actions(params))
            }
            ExecuteCommand::METHOD => {
                parse_params(params).and_then(|params| self.execute_command(params))
            }
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

    fn code_actions(&self, params: CodeActionParams) -> Result<serde_json::Value, String> {
        let uri = params.text_document.uri.to_string();
        if file_path(&uri).is_none() {
            return Ok(serde_json::json!([]));
        }

        let (title, command) = if self.previewing.contains(&uri) {
            ("Stop live preview", STOP_COMMAND)
        } else {
            ("Start live preview", START_COMMAND)
        };
        let action = CodeActionOrCommand::CodeAction(CodeAction {
            title: title.to_string(),
            command: Some(Command {
                title: title.to_string(),
                command: command.to_string(),
                arguments: Some(vec![serde_json::Value::String(uri)]),
            }),
            ..Default::default()
        });
        serde_json::to_value(vec![action]).map_err(|err| err.to_string())
    }

    fn execute_command(
        &mut self,
        params: ExecuteCommandParams,
    ) -> Result<serde_json::Value, String> {
        let uri = params
            .arguments
            .first()
            .and_then(|arg| arg.as_str())
            .ok_or("expected a document uri")?;

        match params.command.as_str() {
            START_COMMAND => {
                self.previewing.insert(uri.to_string());
                self.pending.insert(uri.to_string(), Instant::now());
            }
            STOP_COMMAND => {
                self.previewing.remove(uri);
                self.pending.remove(uri);
            }
            command => return Err(format!("unknown command `{command}`")),
        }
        Ok(serde_json::Value::Null)
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
                if self.previewing.contains(&uri) {
                    self.pending.insert(uri.clone(), Instant::now());
                }
                self.documents.insert(uri, params.text_document.text);
            }
            DidChangeTextDocument::METHOD => {
                const DEBOUNCE: Duration = Duration::from_millis(150);

                let params: DidChangeTextDocumentParams =
                    notification.extract(DidChangeTextDocument::METHOD)?;
                let uri = params.text_document.uri.to_string();
                if let Some(change) = params.content_changes.into_iter().last() {
                    if self.previewing.contains(&uri) {
                        self.pending.insert(uri.clone(), Instant::now() + DEBOUNCE);
                    }
                    self.documents.insert(uri, change.text);
                }
            }
            DidCloseTextDocument::METHOD => {
                let params: DidCloseTextDocumentParams =
                    notification.extract(DidCloseTextDocument::METHOD)?;
                let uri = params.text_document.uri.to_string();
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

fn file_path(uri: &str) -> Option<PathBuf> {
    url::Url::parse(uri).ok()?.to_file_path().ok()
}

fn parse_params<P: serde::de::DeserializeOwned>(params: serde_json::Value) -> Result<P, String> {
    serde_json::from_value(params).map_err(|err| err.to_string())
}
