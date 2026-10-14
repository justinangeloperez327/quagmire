# Views, events, and control flow

This specifies the Group 2 view language. There is no renderer yet. The
[grammar](grammar.md) defines syntax; [components](components.md) defines state
and ownership. View expressions must be pure.

## Elements and text

A component has one final `view` block containing exactly one root element.
Lowercase tags name native HTML elements; uppercase tags name in-scope Quagmire
components. Native paired tags must close explicitly and match exactly. Void
elements use self-closing syntax, such as `<input />`. Component uses must also
be self-closing; children/slots and fragments are deferred.

```qg
view {
  <main>
    <h1>Hello, {name}</h1>
    <input type="text" value={name} />
    <Counter initial={2} />
  </main>
}
```

Raw text is preserved as source text, except that indentation-only text runs
containing a line ending are omitted. Inline whitespace-only runs are retained.
Normal browser CSS determines visible whitespace collapsing. Raw text decodes
`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&apos;`, and decimal/hexadecimal numeric
character references. Numeric references must name valid Unicode scalar values;
unknown or malformed references are errors. Literal braces can use `&#123;` and
`&#125;`, or string interpolation. Entity decoding produces text, never markup.

Inside raw view text, `//` and `/* */` are literal characters, not comments.
HTML comments are deferred. `if (` and `for (` at a node boundary introduce
directives; to display those exact characters, write `{"if (example)"}`.

## Interpolation and attributes

`{expression}` inserts text. Strings, numbers, and booleans have their normal
JavaScript string representation; `null` produces an empty text value. Arrays,
objects, function references, and components cannot be interpolated as text.
Use a loop for collections and explicit property access for an object.

Interpolation must create/update a text node. A value such as
`"<script>alert(1)</script>"` appears as text; it must not be assigned to
`innerHTML`. A value's quotes or angle brackets cannot create new attributes.
Raw HTML interpolation is outside the first subset.

```qg
<button type="button" disabled={busy}>Save</button>
<p class="notice" title={message}>{message}</p>
```

Native attributes support a quoted string, `{expression}`, or a bare boolean
attribute. Quoted attribute values use source string escaping, not HTML entity
decoding. Duplicate attributes are errors. A bare `disabled` means `true`;
`disabled={false}` removes the boolean attribute. Native boolean attributes
require booleans. For ordinary attributes, `null` means remove the attribute;
string/number/boolean values serialize to text. Objects, arrays, and functions
are invalid ordinary attribute values. Form `value` and `checked` bindings update
the live DOM properties as well as the appropriate initial presentation.

`aria-*` and `data-*` are ordinary attributes, so `aria-hidden={false}` means the
text value `"false"`, not removal. `class` takes a string. Object style bindings
are deferred. Component attributes instead supply declared props and may carry
any allowed Quagmire value. A component boolean prop may use a bare attribute.

Reject `onClick`/`onclick` attributes, raw inline event strings, `innerHTML`,
`outerHTML`, `srcdoc`, and native `<script>` elements in this subset. URL-bearing
attributes must use a centralized safety policy: allow relative URLs and
`http:`/`https:`; additionally allow `mailto:`/`tel:` for anchor `href`. Reject
other schemes including `javascript:` and `data:` after normalization, including
leading whitespace/control-character tricks. Do not claim URL safety from
text escaping alone. SVG and custom native elements require later designs.

## Events

Use `on:event={handler}` with a function reference, for example:

```qg
<button type="button" on:click={increment}>Add one</button>
```

Handlers execute when the event occurs. `on:click={increment()}` is invalid;
it invokes a function while evaluating the view instead of passing a handler.
The first subset supports native `click`, `input`, `change`, and `submit` events.
Reject unknown event names and event bindings on component elements; component
callbacks are ordinary function-valued props.

Event handlers declare zero or one required parameter, with no defaults. A
zero-parameter handler ignores the event. A one-parameter handler receives an
immutable event record, not unrestricted access to the browser's event object:

| Field | Value |
| --- | --- |
| `type` | Event name |
| `value` | Current string value for input/textarea/select targets; otherwise `null` |
| `checked` | Boolean for checkbox/radio input targets; otherwise `null` |

```qg
function updateName(event) {
  name = event.value;
}
```

This is valid when `name` is writable state and the handler is bound to an input.
No default prevention, propagation modifiers, async handlers, or event-object
methods are exposed yet. A `submit` binding alone does not prevent the browser's
default submission; custom asynchronous form workflows are deferred.

## Conditional views

```qg
<section>
  if (busy) {
    <p>Loading</p>
  } else if (failed) {
    <p>Unable to load</p>
  } else {
    <p>Ready</p>
  }
</section>
```

Conditions require booleans. Each branch may contain zero or more view nodes.
Only the selected branch is mounted. Leaving a branch removes its nodes and
disposes its component instances and listeners. Entering it again creates new
instances; conditional rendering is not hidden-state caching.

## Keyed lists

```qg
<ul>
  for (item in items) {
    <li key={item.id}>{item.name}</li>
  }
</ul>
```

A view loop iterates array values in order. Its body has exactly one root
element with a required `key={expression}`. The key is compiler/runtime metadata,
not a DOM attribute or component prop; reserve `key` as a component parameter
name. Loop variables are read-only and scoped to the body, including its key.
Nested loops follow the same rules. The source expression cannot read its own
loop variable.

Keys must be non-null strings or finite numbers, unique within that loop's
current sibling list. Equal numeric keys include positive/negative zero; string
`"1"` and number `1` are different keys. Duplicate or invalid dynamic keys raise
an error rather than silently using an array index. Key changes replace that
item's instance; stable keys preserve state across reorderings and update props
for retained instances. Removed keys dispose their owned resources. A key must
describe identity and should not be based on mutable display text.

## Invalid examples

- `<p>{items}</p>`: an array cannot be rendered as text.
- `<div><span>Text</div>`: mismatched closing tag.
- `<input>`: void elements require `/>`.
- `<Counter unknown={1} />`: prop not declared by the component.
- `<Counter>Text</Counter>`: component children are deferred.
- A view loop without `key`: item identity is missing.
- `on:click="increment()"`: string event handlers are forbidden.
- `{count = count + 1}`: assignment is not a rendering expression.

These rules define expected compiler and runtime behavior. Browser tests must be
added when those implementations exist; repository link checks do not prove them.
