# The Joe Programming Language

> **The Chronicles of Joe:** An absurdist narrative, dynamically typed, single-threaded programming language where every program reads like a biographical chapter in the chaotic, mundane, yet surprisingly Turing-complete life of a guy named Joe.

In JoeLang, programs are not compiled into cold silicon instructions. They are endured by Joe as a series of mental episodes, routines, and existential crises.

---

## Key Highlights

- **Literary Narrative Grammar:** Code is structured as grammatical English sentences ending with periods (`.`) and colons (`:`).
- **Cycle-Tracking Garbage Collection:** Backed by **`dumpster`**, cyclic social circles (e.g. Alice and Bob referencing each other as codependent friends) are automatically tracked and collected without memory leaks.
- **Synchronous "Cup of Joe" Standard Library:** Single-threaded, synchronous modules for terminal I/O (`std.io`), file system (`std.file`), HTTP requests (`std.net`), math (`std.math`), random (`std.random`), text utilities (`std.strings`), collections (`std.collections`), JSON (`std.json`), time (`std.time`), and OS (`std.os`).
- **Declarable & File Modules:** Joe can establish self-contained domains in-code (`Joe establishes MathEngine: ... That's MathEngine.`) or import `.joe` files from disk via `Joe calls for <module>.`.
- **Class Keyword & Operator Overrides:** Classes can intercept core keywords (e.g., `Joe yells msg to logger.`, `Joe whispers msg to "file.log".`) and overload operators (`+`, `-`, `*`, `/`, `%`, `==`).

---

## The Three Personas (Binaries)

JoeLang provides three dedicated command-line binaries:

| Binary | Persona | Role |
| :--- | :--- | :--- |
| **`joe`** | **The Story Runner & REPL** | Executes `.joe` scripts, offers an interactive REPL (`joe> `), and features a documentary voiceover mode (`--narrate`). |
| **`cjoe`** | **Joe's Conscience** | Static & semantic reality checker that detects amnesia (undefined variables), unused memories, and illegal escapes. Supports `--ast` and `--json`. |
| **`djoe`** | **Joe's Therapist** | Interactive step-debugger where the programmer acts as Joe's therapist (`therapist> `) to step through his life, inspect his mental baggage (`mind`), and examine his call stack (`trauma`). |

---

## Quickstart

### 1. Build the Project
```bash
cargo build --release
```

### 2. Run a Story
```bash
cargo run --bin joe -- examples/morning_routine.joe
```

### 3. Run with Documentary Voiceover Mode
```bash
cargo run --bin joe -- --narrate examples/fibonacci.joe
```

### 4. Interactive REPL
```bash
cargo run --bin joe
```
```text
=== The Chronicles of Joe (Interactive Mode) ===
Type your story or 'Joe rests.' to leave him alone.

joe> Joe remembers x as 42.
=> 42
joe> Joe yells x * 2.
84
=> nothing
joe> Joe rests.
Joe goes to sleep. Farewell.
```

### 5. Check Code with Joe's Conscience (`cjoe`)
```bash
cargo run --bin cjoe -- examples/troubled_joe.joe
```
```text
Error (Reality Check): Existential Error: 'Joe gives up.' outside of a loop. Joe cannot escape reality that easily.
Error (Reality Check): Confusion Error: 'Joe delivers' outside of any habit/function. Deliver to whom?
Error (Reality Check): Amnesia Error: Joe tries to change 'mysterious_debt', but Joe has no recollection of it.
Warning (Conscience): Joe remembered 'unused_money' at 2:1, but never thought about it again. Typical Joe.

✖ 3 error(s) found.
```

### 6. Interactive Therapy Session (`djoe`)
```bash
cargo run --bin djoe -- examples/morning_routine.joe
```
```text
=== Joe's Therapy Session (djoe debugger) ===
Welcome. Joe is currently in a state of suspended animation.
Type 'help' to review your therapeutic options.

therapist> s
Joe advanced to line 4:
  -> Joe yells "--- Joe Wakes Up ---".
therapist> mind
Joe's current mental baggage:
  bank_account = 30
  coffee_price = 5
  coffees_bought = 0
therapist> c
Joe concluded his journey without further interruption.
```

---

## Syntax Cheat Sheet

| Traditional Construct | JoeLang Narrative Syntax | Description |
| :--- | :--- | :--- |
| `let x = 10;` | `Joe remembers x as 10.` | Variable declaration |
| `x = 20;` | `x turns into 20.`<br>`Joe sets x to 20.` | Variable mutation |
| `println(msg);` | `Joe yells msg.` | Print with newline |
| `print(msg);` | `Joe whispers msg.` | Print without newline |
| `println(to_file);` | `Joe yells msg to "out.log".`<br>`Joe yells msg to my_logger.` | Directed yell (keyword override) |
| `let s = read();` | `Joe listens to s.` | Read line from stdin |
| `if cond { ... } else { ... }` | `If Joe believes cond:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`...`<br>`Otherwise:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`...`<br>`That is that.` | Conditional branching |
| `while cond { ... }` | `While Joe worries about cond:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`...`<br>`Joe moves on.` | While loop |
| `for x in list { ... }` | `For every x in list:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`...`<br>`Joe moves on.` | For loop |
| `break;` | `Joe gives up.` | Exit loop |
| `continue;` | `Joe tries anyway.` | Next loop iteration |
| `fn f(a, b) { return a + b; }` | `To f with a, b:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`Joe delivers a + b.`<br>`That's how Joe does it.` | Function declaration (Habit) |
| `class Person { ... }` | `Meet a Person:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`It has name, friend.`<br>&nbsp;&nbsp;&nbsp;&nbsp;`To greet with o:`<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;`Joe yells jthis.name + " greets " + o.`<br>&nbsp;&nbsp;&nbsp;&nbsp;`That's how Joe does it.`<br>`That's a Person.` | Class declaration (Archetype) |
| `module Mod { ... }` | `Joe establishes Mod:`<br>&nbsp;&nbsp;&nbsp;&nbsp;`...`<br>`That's Mod.` | Declarable module |
| `import math;` | `Joe calls for std.math.` | Import module |
| `true` / `false` | `Joe agrees` / `Joe disagrees` | Booleans |
| `null` / `nil` | `nothing` | Nil value |
| `panic("boom");` | `Joe panics "boom".` | Runtime panic |

---

## Code Example: "A Morning with Coffee"

```joe
Joe calls for std.math.

Joe yells "--- Joe Wakes Up ---".

Joe remembers bank_account as 30.
Joe remembers coffee_price as 5.
Joe remembers coffees_bought as 0.

While Joe worries about bank_account >= coffee_price:
    bank_account turns into bank_account - coffee_price.
    coffees_bought turns into coffees_bought + 1.
    Joe yells "Joe drank cup #" + coffees_bought + ", remaining funds: $" + bank_account.

    If Joe believes coffees_bought >= 3:
        Joe yells "Joe is vibrating at high frequencies!".
        Joe gives up.
    That is that.
Joe moves on.

Joe yells "Joe survived the morning.".
```

---

## Documentation

Full language specification and standard library API documentation can be found in the wiki:
- **[`wiki/index.html`](wiki/index.html)**: Clean, self-contained HTML manual covering all syntax, stdlib modules, and tooling options.

---

## Testing

Run the test suite:
```bash
cargo test
```
All 21 unit and integration tests cover the lexer, Pratt parser, dumpster cycle collection, tree-walk interpreter, stdlib modules, conscience checker, and keyword overrides.

---

## License

JoeLang is open source licensed under the **MIT License**.
