# Components and state

This specifies Group 2 behavior. The compiler and runtime that will implement
it do not exist yet. See the [grammar](grammar.md) for syntax.

## A complete component

```qg
export component Counter(initial = 0) {
  state count = initial;
  computed doubled = count * 2;

  function increment() {
    count = count + 1;
  }

  view {
    <section>
      <p>Count: {count}</p>
      <p>Double: {doubled}</p>
      <button type="button" on:click={increment}>Add one</button>
    </section>
  }
}
```

Each mounted instance owns its state and derived values. Clicking one instance's
button changes only that instance. The view reads current values; there is no
dependency array or application-facing `effect` call.

## Parameters are props

A component name starts with an uppercase letter. Its parameters declare named,
read-only props. A parent supplies them as attributes:

```qg
<Counter initial={5} />
```

A parameter without a default is required. An omitted optional prop uses its
default expression; `null` is a supplied value and does not select the default.
Defaults are pure and may reference imports, module constants, and earlier
parameters. Required parameters must precede optional parameters. Duplicate
parameters, unknown props, and assignment to a prop are errors.

Parent prop expressions update as their dependencies change. State initializers
run once for each mounted instance: changing `initial` later does not reset
`count`. A computed expression reading a prop does update. This distinction keeps
initial values separate from values controlled by a parent.

Components are rendered through elements, not invoked as ordinary functions.
Nested component declarations and component children/slots are deferred; a
component use must be self-closing in the first subset.

## Declaration and initialization rules

Component members precede exactly one final `view` declaration. Supported members
are `state`, `computed`, `const`, and named `function` declarations. All member
names are visible to functions and the view; declarations may not share names
with props or imported/enclosing names.

State and constant initializers are evaluated in source order and must be pure.
They may read props, module constants, or earlier initialized state/constants.
They may not read a component computed value or a later initialized binding,
even indirectly through a function. This prevents accidental initialization
cycles. Function declarations are available regardless of their position.

Computed dependencies can reference other computed declarations regardless of
source order. Reject cycles with the dependency path, including cycles hidden
inside called functions. Reads in conditional paths must not be omitted by the
implementation's dependency tracking.

## State and local variables

| Declaration | Meaning | Assignment |
| --- | --- | --- |
| `state count = 0;` | Per-instance reactive value | Component functions may replace it |
| `computed doubled = count * 2;` | Reactive, read-only derived value | Forbidden |
| `const step = 1;` | Value initialized once | Forbidden |
| `let next = count + step;` | Function-local temporary | Allowed inside its scope |
| Component parameter | Value supplied by a parent | Forbidden |

`let` is allowed inside function bodies, not as a component member or module
declaration. Both `let` and `const` require initializers. Assignment targets are
simple identifiers. Assignment to an object property or array index is outside
the first subset; replace the state value instead:

```qg
state profile = { name: "Justin" };

function rename() {
  profile = { name: "Angelo" };
}
```

Arrays and plain objects are immutable values, including nested values. No
mutating methods are provided in the subset. Sharing a value with another prop
or binding therefore does not create an undocumented state-mutation route.
Assigning a state value updates downstream computations and views; local `let`
assignment does not create a reactive binding.

## Computed values and functions

Computed expressions are pure: they may read values and call pure Quagmire
functions, but may not change state, perform I/O, install listeners, or call an
effectful function indirectly. Pure functions may use local `let` temporaries
and local assignments. Purity is inferred through the call graph, not declared
by an extra keyword. Imports preserve the same rules.

Normal component functions may change their owning component's state. The
compiler must reject a state-changing function used by a computed expression,
initializer, default, condition, loop source, key, prop expression, or rendering
expression. Render reads must never perform writes. Module-level functions
cannot access a component instance's state.

Functions use positional arguments. Missing required arguments and extra
arguments are errors. Defaults run at call time and may read earlier parameters
and already initialized enclosing bindings; required parameters precede defaults.
`return expression;` returns a value; `return;` and falling off the end return
`null`. Calls are synchronous. A code `for (item in items)` iterates array values
in order; its loop variable is read-only and scoped to that iteration. `return`
inside a loop or conditional exits the containing function.

```qg
function clamp(value, minimum, maximum) {
  if (value < minimum) {
    return minimum;
  } else if (value > maximum) {
    return maximum;
  }
  return value;
}
```

This pure helper can be used in a computed expression. A helper that writes
component state cannot. Conditions require booleans; arrays are not truth tests.

## Update and ownership contract

State replacement takes effect immediately for reads in the same function call.
Derived reads must reflect those changes without returning a stale cached value.
Multiple writes in one synchronous event handler may share one DOM update after
that handler completes. Before the next browser paint, observable rendered output
must match the final state. No exact scheduler algorithm is mandated here.

Removing a component disposes its subscriptions and event listeners. Conditional
removal and list removal have the same ownership rule. A later mount is a new
instance unless its identity was preserved through the list key rules.

## Invalid examples

```qg
computed doubled = count * 2;
function breakDerived() {
  doubled = 10;
}
```

Assignment to a computed value is invalid. `initial = 4;` is also invalid because
props are read-only. `profile.name = "New";` is unsupported property assignment.
`computed doubled = increment();` is invalid if `increment` writes state.
`let count = 0;` at component scope is invalid; use `state` for reactive data.
