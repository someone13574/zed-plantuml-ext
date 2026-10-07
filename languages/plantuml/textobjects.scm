[
  (class_declaration
    body: (entity_body
      "{"
      (_)* @class.inside
      "}"))
  (interface_declaration
    body: (entity_body
      "{"
      (_)* @class.inside
      "}"))
  (enum_declaration
    body: (entity_body
      "{"
      (_)* @class.inside
      "}"))
  (entity_declaration
    body: (entity_body
      "{"
      (_)* @class.inside
      "}"))
  (class_declaration !body)
  (interface_declaration !body)
  (enum_declaration !body)
  (entity_declaration !body)
  (package_block
    "{"
    (_)* @class.inside
    "}")
  (namespace_block
    "{"
    (_)* @class.inside
    "}")
  (partition_block
    "{"
    (_)* @class.inside
    "}")
] @class.around

(member
  (method)) @function.around

(comment)+ @comment.around
