;; Primitives
(boolean) @boolean
(comment) @comment
(shebang_comment) @comment
(identifier) @variable
((identifier) @variable.special
  (#eq? @variable.special "self"))
(nil) @constant.builtin
(number) @number
(string) @string
(table_constructor ["{" "}"] @constructor)
(varargs "..." @constant.builtin)
[ "," "." ":" ";" ] @punctuation.delimiter

(escape_sequence) @string.escape
(format_specifier) @string.escape

;; Properties. These stay before the function-call patterns so a call such
;; as `obj.method()` resolves its name to @function rather than @property.
(table_entry key: (identifier) @property)
(field key: (identifier) @property)
(index key: (identifier) @property)
(attribute) @attribute

;; Basic statements and keywords
[ "if" "then" "elseif" "else" ] @keyword.control
[ "for" "while" "repeat" "until" ] @keyword.control
[ "in" "local" "return" (break) (goto) "do" "end" ] @keyword
(label) @label

;; `global` is contextual, not a reserved word.
(var_declaration "global" @keyword)
(type_declaration "global" @keyword)
(function_statement "global" @keyword)
(record_declaration "global" @keyword)
(interface_declaration "global" @keyword)
(enum_declaration "global" @keyword)

(macroexp_statement "macroexp" @keyword)
(macroexp_declaration "macroexp" @keyword)

;; Operators
(bin_op (op) @operator)
(unary_op (op) @operator)
[ "=" "as" ] @operator

;; Functions
(function_statement
  "function" @keyword
  . name: (_) @function)
(anon_function
  "function" @keyword)
(function_body "end" @keyword)

(arg name: (identifier) @variable.parameter)

(function_signature
  (arguments
    . (arg name: (identifier) @variable.special))
  (#eq? @variable.special "self"))

(typeargs
  "<" @punctuation.bracket
  . (_) @type.parameter
  . ("," . (_) @type.parameter)*
  . ">" @punctuation.bracket)

(function_call
  (identifier) @function . (arguments))
(function_call
  (index (_) key: (identifier) @function) . (arguments))
(function_call
  (method_index (_) key: (identifier) @function) . (arguments))

;; Types

; Contextual keywords in record bodies
(record_declaration
  . [ "record" ] @keyword
  name: (identifier) @type)
(anon_record . "record" @keyword)
(record_body
  (record_declaration
    . [ "record" ] @keyword
    . name: (identifier) @type))
(record_body
  (enum_declaration
    . [ "enum" ] @keyword
    . name: (identifier) @type))
(record_body
  (interface_declaration
    . [ "interface" ] @keyword
    . name: (identifier) @type))
(record_body
  (typedef
    . "type" @keyword
    . name: (identifier) @type . "="))
(record_body
  (macroexp_declaration
    . [ "macroexp" ] @keyword))
(record_body (metamethod "metamethod" @keyword))
(record_body (userdata) @keyword)

; Contextual keywords in interface bodies
(interface_declaration
  . [ "interface" ] @keyword
  name: (identifier) @type)
(anon_interface . "interface" @keyword)
(interface_body
  (record_declaration
    . [ "record" ] @keyword
    . name: (identifier) @type))
(interface_body
  (enum_declaration
    . [ "enum" ] @keyword
    . name: (identifier) @type))
(interface_body
  (interface_declaration
    . [ "interface" ] @keyword
    . name: (identifier) @type))
(interface_body
  (typedef
    . "type" @keyword
    . name: (identifier) @type . "="))
(interface_body
  (macroexp_declaration
    . [ "macroexp" ] @keyword))
(interface_body (metamethod "metamethod" @keyword))
(interface_body (userdata) @keyword)

(enum_declaration
  "enum" @keyword
  name: (identifier) @type)

(type_declaration "type" @keyword)
(type_declaration (identifier) @type)
(simple_type) @type
(type_index) @type
(type_union "|" @operator)
(function_type "function" @type)

;; Variable declarators
(var_declaration
  declarators: (var_declarators
      (var name: (identifier) @variable)))
(var_declaration
  declarators: (var_declarators
    (var
      "<" @punctuation.bracket
      . attribute: (attribute) @attribute
      . ">" @punctuation.bracket)))

[ "(" ")" "[" "]" "{" "}" ] @punctuation.bracket
