(function_statement
  body: (function_body) @function.inside) @function.around

(anon_function
  body: (function_body) @function.inside) @function.around

(record_declaration
  record_body: (record_body) @class.inside) @class.around

(anon_record
  record_body: (record_body) @class.inside) @class.around

(interface_declaration
  interface_body: (interface_body) @class.inside) @class.around

(anon_interface
  interface_body: (interface_body) @class.inside) @class.around

(enum_declaration
  enum_body: (enum_body) @class.inside) @class.around

(comment)+ @comment.around
