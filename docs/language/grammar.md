# Quagmire grammar

This is the Group 2 language specification for the first supported subset.
It is an implementation contract, not a working compiler. Examples labeled
valid conform to this specification; they have not been compiled yet.

Read [components](components.md), [expressions](expressions.md), and
[views](views.md) for the semantic rules that accompany the syntax.

## Source and tokens

Source files use `.qg`, UTF-8, and case-sensitive names. An identifier matches
`[A-Za-z_][A-Za-z0-9_]*`. Keywords cannot be identifiers:

```text
component state computed function view if else for in return
import export from let const true false null
```

Outside view text, spaces, tabs, and line endings separate tokens. `//` comments
end at a line ending; `/* ... */` comments do not nest. An unterminated string or
comment is an error. Comments are recognized in code, not inside strings or raw
view text. Use the longest matching operator token, such as `===` before `=`.

Numbers are decimal digits with an optional fractional part and exponent:
`[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?`. A leading sign is a unary operator.
Reject leading zeroes except `0`, `0.xxx`, and `0e...`; reject source literals
that cannot be represented as finite binary64 values. No hexadecimal, numeric
separators, or big integers in the first subset.

Strings use matching single or double quotes. Supported escapes are `\\`,
`\"`, `\'`, `\n`, `\r`, `\t`, and `\uXXXX` (four hexadecimal digits).
Reject unknown escapes and unescaped line endings. Backtick strings and string
interpolation inside a string literal are deferred.

Semicolons terminate imports, variable declarations, computed declarations,
assignments, calls, and returns. They are required; there is no automatic
semicolon insertion. Component, function, view, `if`, and `for` blocks do not
require a trailing semicolon.

## Grammar notation

The following extended Backus-Naur form uses quoted terminals, `|` alternatives,
`[ ... ]` optional content, and `{ ... }` repetition. `IDENTIFIER`, `NUMBER`, and
`STRING` are the tokens described above. `TEXT`, `TAG`, and `ATTRIBUTE` are view
tokens described below. End of input is required after `module`.

```ebnf
module          = { import-decl }, { module-decl } ;
import-decl     = "import", "{", import-item, { ",", import-item }, [ "," ],
                  "}", "from", STRING, ";" ;
import-item     = IDENTIFIER ;
module-decl     = [ "export" ], ( component-decl | function-decl | const-decl ) ;
component-decl  = "component", IDENTIFIER, "(", [ parameters ], ")", "{",
                  { component-member }, view-decl, "}" ;
component-member = state-decl | computed-decl | const-decl | function-decl ;
parameters      = parameter, { ",", parameter }, [ "," ] ;
parameter       = IDENTIFIER, [ "=", expression ] ;
state-decl      = "state", IDENTIFIER, "=", expression, ";" ;
computed-decl   = "computed", IDENTIFIER, "=", expression, ";" ;
const-decl      = "const", IDENTIFIER, "=", expression, ";" ;
let-decl        = "let", IDENTIFIER, "=", expression, ";" ;
function-decl   = "function", IDENTIFIER, "(", [ parameters ], ")", block ;
block           = "{", { statement }, "}" ;
statement       = const-decl | let-decl | assignment | call-statement
                | return-statement | code-if | code-for ;
assignment      = IDENTIFIER, "=", expression, ";" ;
call-statement  = postfix, ";" ;
return-statement = "return", [ expression ], ";" ;
code-if         = "if", "(", expression, ")", block,
                  [ "else", ( code-if | block ) ] ;
code-for        = "for", "(", IDENTIFIER, "in", expression, ")", block ;

expression      = logical-or ;
logical-or      = logical-and, { "||", logical-and } ;
logical-and     = equality, { "&&", equality } ;
equality        = comparison, { ( "===" | "!==" ), comparison } ;
comparison      = additive, [ ( "<" | "<=" | ">" | ">=" ), additive ] ;
additive        = multiplicative, { ( "+" | "-" ), multiplicative } ;
multiplicative  = unary, { ( "*" | "/" | "%" ), unary } ;
unary           = ( "!" | "-" | "+" ), unary | postfix ;
postfix         = primary, { ".", IDENTIFIER | "[", expression, "]"
                | "(", [ arguments ], ")" } ;
arguments       = expression, { ",", expression }, [ "," ] ;
primary         = IDENTIFIER | NUMBER | STRING | "true" | "false" | "null"
                | "(", expression, ")" | array | object ;
array           = "[", [ arguments ], "]" ;
object          = "{", [ property, { ",", property }, [ "," ] ], "}" ;
property        = ( IDENTIFIER | STRING ), ":", expression ;

view-decl       = "view", "{", element, "}" ;
element         = "<", TAG, { attribute }, "/>"
                | "<", TAG, { attribute }, ">", { view-node }, "</", TAG, ">" ;
attribute       = ATTRIBUTE, [ "=", ( STRING | interpolation ) ] ;
view-node       = element | TEXT | interpolation | view-if | view-for ;
interpolation   = "{", expression, "}" ;
view-block      = "{", { view-node }, "}" ;
view-if         = "if", "(", expression, ")", view-block,
                  [ "else", ( view-if | view-block ) ] ;
view-for        = "for", "(", IDENTIFIER, "in", expression, ")",
                  "{", element, "}" ;
```

`call-statement` is syntactically broad to keep postfix parsing uniform. Semantic
analysis must require its outermost operation to be a call. `user.name;` is not
a valid statement. Likewise, syntax alone does not establish assignability,
name resolution, purity, tag matching, attribute safety, or loop key validity.

## Parsing decisions

| Situation | Rule |
| --- | --- |
| `{}` after `=` or inside an expression | Object literal; keys require `:` |
| `{}` after a function, `if`, or code `for` header | Statement block |
| `{}` after `view` | View block containing exactly one root element |
| `{}` in an element's children or attribute value | Expression interpolation; nested `{}` may be an object |
| `<` in an expression | Comparison operator; elements are never expression values |
| `<` while reading view children | Start an element; use `&lt;` for literal text |
| `if (` or `for (` at a view node boundary | View control directive; use string interpolation to display that text literally |
| Closing element | Must match the opening tag exactly |
| `else` | Belongs to the nearest preceding `if`; braces are always required |

`TAG` is a lowercase native name matching `[a-z][a-z0-9-]*`, or a component
identifier starting with an uppercase letter. `ATTRIBUTE` matches
`[A-Za-z_][A-Za-z0-9_-]*`, optionally followed by `:` and a second such name.
Only the `on:` namespace has meaning in this subset. Native element/attribute
validation and component prop validation are additional semantic checks.

View scanning switches into code for an interpolation or control header and
back into view children for a view control block. Balanced braces, brackets,
parentheses, strings, and comments determine expression boundaries; searching
for the next `}` with a regular expression is insufficient. After a nested
element closes, scanning resumes at a new child node boundary.

`TEXT` is a nonempty raw child-text run, terminated by `<`, `{`, `}`, or a view
directive at a node boundary. A literal closing brace uses `&#125;` or string
interpolation. Entity decoding and whitespace rules are in [views](views.md).

## Modules and names

Imports precede other module declarations. Named imports and explicit `export`
are supported; default imports, aliases, wildcard exports, packages, dynamic
imports, and extension inference are deferred. Paths must be relative, include
`.qg`, and resolve inside the application source root. Reject import cycles in
the first subset with a diagnostic identifying the cycle.

Each module has its own scope. Imported bindings and top-level constants are
read-only. All top-level declaration names are collected before checking bodies,
so functions can call later-declared functions. Constant initialization follows
source order and cannot read an uninitialized constant, directly or through a
function call. Component initialization follows [components](components.md).
Reject duplicate names in a scope and shadowing of an enclosing/imported name.

```qg
import { Counter } from "./counter.qg";

export component App() {
  view {
    <main><Counter initial={2} /></main>
  }
}
```

This is valid if `counter.qg` exports `Counter(initial = 0)`. Importing a missing
export, omitting `.qg`, or using `<counter />` to reference that component is invalid.

## First subset and diagnostics

The first subset includes synchronous functions, local state, computed values,
native elements, named components, events, conditional views, and keyed array
lists. Types are checked at runtime where static analysis cannot determine them.
Unsupported syntax must produce a diagnostic, not pass through as JavaScript.

Defer classes, type annotations, async/await, closures, anonymous functions,
destructuring, object shorthand, spread, assignment expressions, compound
assignment, increments, ternaries, optional chaining, raw HTML, fragments,
component children/slots, style blocks, two-way binding, and lifecycle APIs.
Ordinary CSS can be supplied by the host application in the first milestone.

Invalid examples include `state count;` (initializer missing),
`computed double = count * 2` (semicolon missing), `if count { ... }`
(parentheses missing), and `count++;` (unsupported operator).

Diagnostics must distinguish lexical errors, expected-token errors, unresolved
names, invalid assignments, impure computations, and invalid views. Report the
original `.qg` span and expected syntax. Parser recovery may resume at a semicolon
or balanced declaration boundary, but invalid input must never emit runnable code.
