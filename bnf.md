```
<expr>  ::= 
<bool> ::= <bool2> | <bool> "&&" <bool2>
<bool2> ::= <add-sub> | <bool2> "||" <add-sub>
<add-sub> ::= <term> | <expr> "+" <term> | <expr> "-" <term>
<term>     ::= <negative> | <term> "*" <negative> | <term> "/" <negative>
<negative> ::= "-" <factor>
<factor>   ::= <number> | "(" <expr> ")" | <function-call> | <identifier>
<number>   ::= <digit> | <digit> <number>
<function-call>  ::= <identifier> "(" <expression-list> ")"

<identifier-inner> ::= <letter> | <digit> | <letter> <identifier-inner> | <digit> <identifier-inner>
<identifier> ::= <letter> <identifier-inner>

<hex-digit> ::= "A" | "B" | "C" | "D" | "E" | "F" | <digit>

<hex> ::= <hex-digit> | <hex-digit> <hex>

<hex-prefix> ::= "0x" <hex>

<octal-digit> ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"

<octal> ::= <octal-digit> | <octal-digit> <octal>

<octal-prefix> ::= "0" <octal>

<digit>    ::= [0-9]
<letter>   ::= [A-Z] | [a..z]
<empty>    ::= ""

<primitive> ::= "u0" | "u8" | "u16" | "u32" | "i8" | "i16" | "i32" | "f8" | "f16" | "f32"

<struct> ::= "struct" <identifier> "{" <declaration-list> "}"

<union> ::= "union" <identifier> "{" <declaration-list> "}"

<base-types> ::= <primitive> | <struct> | <union>

<!-- TODO: Handle case of array of pointers -->
<array> ::= <identifier> | <identifier> "[" <digit> "]" | <array> "[" <digit> "]"

<pointer> ::= <array> | "*" <array> | "*" <pointer> 

<!-- TODO: prevent run time expressions (up to parser?) -->
<declaration> ::= <base-types> <pointer>

<declaration-list> ::= <declaration> | <declaration-list> "," <declaration>

<declaration-statement> ::= "let" <declaration> | "let" <declaration> "=" <expr> 
| "let" <pointer> | "let" <pointer> "=" <expr>

<expression-list> ::= <expr> | <expr> "," <expression-list>

<function-def> ::= "fn" <identifier> "->" <type> "(" <declaration-list> ")" <block>

<assignment> ::= <identifier> "=" <expr>

<block>          ::= "{" <statements> "}"
<statements>     ::= <statement> | <statement> <statements>
<!-- ? Label -->
<statement>      ::= (<ident>":")? (<assignment> ";" | <expr> | <return-stmt>)

<new-statement> ::= "new" <identifier>

<free-statement> ::= "free" <identifier>



<return-stmt>    ::= "return" <expression> ";" | <expression> "\n"
```
