(comment) @comment
(block_comment) @comment

[
  "@startuml"
  "@enduml"
] @keyword.directive

[
  "class"
  "interface"
  "enum"
  "package"
  "namespace"
  "as"
  "note"
  "of"
  "on"
  "link"
  "end note"
  "hide"
  "show"
  "remove"
  "restore"
  "together"
  "extends"
  "implements"
  "participant"
  "actor"
  "boundary"
  "control"
  "entity"
  "database"
  "collections"
  "queue"
  "alt"
  "opt"
  "loop"
  "par"
  "break"
  "critical"
  "group"
  "else"
  "end"
  "start"
  "stop"
  "kill"
  "detach"
  "label"
  "goto"
  "backward"
  "if"
  "then"
  "is"
  "equals"
  "not"
  "elseif"
  "endif"
  "while"
  "endwhile"
  "repeat"
  "switch"
  "case"
  "endswitch"
  "fork"
  "split"
  "again"
  "merge"
  "partition"
  "activate"
  "deactivate"
  "create"
  "destroy"
  "return"
  "ref over"
  "end ref"
  "box"
  "end box"
] @keyword

(divider) @punctuation.special
(participant_declaration name: (identifier) @type)
(participant_declaration alias: (identifier) @type)

(entity_list (identifier) @type)
(color) @string.special
(hyperlink) @string.special.url
(member_separator) @punctuation.special
(qualified_name (identifier) @type)
(member_ref (identifier) @type)

(note_position) @keyword.modifier

(abstract) @keyword.modifier
(modifier) @keyword.modifier

(entity_kind) @keyword
(entity_declaration name: (identifier) @type)
(entity_declaration alias: (identifier) @type)
(colon_member entity: (identifier) @type)
(tag) @attribute
(qualifier) @string.special
(role) @string.special

(class_declaration name: (identifier) @type)
(class_declaration alias: (identifier) @type)
(interface_declaration name: (identifier) @type)
(interface_declaration alias: (identifier) @type)
(enum_declaration name: (identifier) @type)
(enum_declaration alias: (identifier) @type)
(relation left: (identifier) @type)
(relation right: (identifier) @type)
(package_block name: (identifier) @module)
(namespace_block name: (identifier) @module)

(method name: (identifier) @function.method)
(method name: (cpp_method_name) @function.method)
(attribute name: (identifier) @variable.member)
(type) @type
(generics) @type
(stereotype) @attribute

(string) @string
(cardinality) @string.special
(label) @string
(diagram_name) @string.special

(relation_operator) @operator
(visibility) @operator

[
  "{"
  "}"
  "("
  ")"
] @punctuation.bracket

[
  ":"
  ","
] @punctuation.delimiter

(activation) @operator
(number) @number

(action_text) @string
(lane_text) @module

(condition) @string.special
(branch_label) @string
(fork_label) @string
(arrow_operator) @operator
(connector) @label
(delay_text) @comment
(activity_control target: (identifier) @label)
(partition_block name: (identifier) @module)
(partition_block name: (string) @module)
(lifecycle_statement target: (identifier) @type)
(reference target: (entity_list (identifier) @type))
