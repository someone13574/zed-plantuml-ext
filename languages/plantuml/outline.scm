(class_declaration
  abstract: (_)? @context
  "class" @context
  name: (_) @name) @item

(interface_declaration
  "interface" @context
  name: (_) @name) @item

(enum_declaration
  "enum" @context
  name: (_) @name) @item

(entity_declaration
  kind: (_) @context
  name: (_) @name) @item

(member
  (method
    name: (_) @name)) @item

(member
  (attribute
    name: (_) @name)) @item

(package_block
  "package" @context
  name: (_) @name) @item

(namespace_block
  "namespace" @context
  name: (_) @name) @item

(participant_declaration
  kind: _ @context
  name: (_) @name) @item

(box_block
  "box" @context
  [
    name: (_) @name
    label: (_) @name
  ]) @item

(frame_block
  kind: _ @context
  label: (_) @name) @item

(partition_block
  "partition" @context
  name: (_) @name) @item

(swimlane
  name: (_) @name) @item
