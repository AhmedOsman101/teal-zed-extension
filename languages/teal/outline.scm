(function_statement
  "function" @context
  name: (_) @name) @item

(record_declaration
  "record" @context
  name: (identifier) @name) @item

(interface_declaration
  "interface" @context
  name: (identifier) @name) @item

(enum_declaration
  "enum" @context
  name: (identifier) @name) @item

(type_declaration
  "type" @context
  name: (identifier) @name) @item

(macroexp_declaration
  "macroexp" @context
  name: (identifier) @name) @item
