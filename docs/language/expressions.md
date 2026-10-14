# Expressions and values

These rules define the initial language subset. Examples are specifications,
not output from an implemented evaluator. See the [grammar](grammar.md).

## Values

Values are numbers (IEEE 754 binary64), strings, booleans, `null`, immutable
arrays, immutable plain objects, and named function references. There is no
`undefined` value in source. Components have a separate name category and are
not callable expression values. Array holes and trailing argument holes are
invalid; a single trailing comma is allowed in arrays, objects, calls, imports,
and parameter lists.

```qg
const title = "Quagmire";
const enabled = true;
const fallback = null;
const sizes = [1, 2, 3];
const person = { name: "Justin", active: true };
```

Plain object keys are identifier or string literals followed by `:`. Duplicate
keys are errors, including equivalent decoded strings. Keys named `__proto__`,
`prototype`, and `constructor` are rejected in the first subset. Objects expose
only their own declared keys; JavaScript prototypes are not a source-level API.
Property shorthand, methods, computed keys, and spread are deferred.

## Operators and precedence

From strongest to weakest:

| Level | Operators | Association and requirements |
| --- | --- | --- |
| Grouping | `(expression)` | Explicitly overrides precedence |
| Postfix | `.name`, `[index]`, `(arguments)` | Left to right |
| Unary | `!`, `-`, `+` | Right to left; boolean for `!`, number for signs |
| Multiplication | `*`, `/`, `%` | Left to right; numeric operands |
| Addition | `+`, `-` | Left to right; `+` takes two numbers or two strings |
| Ordering | `<`, `<=`, `>`, `>=` | One per comparison; numeric operands |
| Equality | `===`, `!==` | Left to right; strict type-sensitive equality |
| Logical and | `&&` | Left to right; boolean operands, short-circuit |
| Logical or | `||` | Left to right; boolean operands, short-circuit |

`1 + 2 * 3` is `7`. `(1 + 2) * 3` is `9`. `a < b < c` is invalid;
use `(a < b) && (b < c)`. Assignment is a statement, not an expression.
`==`, `!=`, bitwise operators, increments, compound assignment, and ternaries
are outside the subset.

There is no implicit numeric/string conversion: `"Count: " + 2` is a type error;
write separate view text and interpolation instead. Conditions and logical
operators require actual booleans, not JavaScript truthiness. Ordering strings
is deferred. Numeric division and remainder use binary64 behavior, including
nonfinite results from arithmetic; `%` is remainder, not a mathematical modulo.

Equality compares primitives by value and arrays, objects, and functions by
identity. Values of different categories compare unequal. Numeric `NaN` compares
unequal to itself; positive and negative zero compare equal. This does not promise
deep object comparison.

## Access and calls

`person.name` reads an own object property. `person["name"]` does the same.
`sizes[0]` indexes an array with a nonnegative integer; array and string `.length`
are supported. String length counts UTF-16 code units to match the browser output.
String indexing and methods are deferred.

Reading a missing object key or an out-of-range array index returns `null`.
Access on `null`, invalid index types, and attempts to read JavaScript prototype
members raise a runtime error if static analysis cannot reject them earlier.
An object's own `length` key has ordinary object semantics.

`clamp(value, 0, 10)` calls a named Quagmire function. Function references may
be passed as props and bound to events. Every call must check callability and
argument count when it cannot be established statically. Unknown identifiers
are compile-time errors. Arbitrary JavaScript globals and native methods are
not automatically imported into Quagmire.

## Valid and invalid examples

```qg
const items = [{ id: 1, name: "First" }];
const first = items[0];
const name = first.name;
const ready = (items.length > 0) && (name !== null);
```

The following are invalid:

```qg
const brokenObject = { name };
const brokenArray = [1,,2];
const looseEquality = 1 == "1";
const mutation = (count = 1);
```

The first uses unsupported shorthand, the second an array hole, the third a
nonexistent operator, and the fourth an assignment expression. `if (items)` is
syntactically parseable but semantically invalid because an array is not boolean.

Errors known from source should be compile-time diagnostics. Uncertain runtime
type errors must retain the original source location through generated mappings;
they must not silently coerce values into a different category.
