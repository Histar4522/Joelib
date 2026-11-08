# JoeLang

JoeLang is the object-facing programming-language project in **Joelib**. It turns the project’s Joe-related vocabulary and writing conventions into a small, browser-ready language with a familiar Java/C++-style structure: typed variables, explicit scopes, objects, methods, arrays, dynamic lists, conditionals, loops, input, packages, diagnostics, and a JavaScript inspection projection.

The language is designed to be usable in more than one editor. Its language rules and runtime live in a separate **language pack**; JoeLang Studio is one IDE that consumes that pack rather than owning a second copy of the interpreter.

## What JoeLang is based on

JoeLang is part of the broader JoeLib collection. Its terminology, object-facing focus, and project identity are informed by the Joe-related documents and creative work collected in the parent repository, including Joelements, Joeglish material, the Joe Wiki, Joelet Speech, JOESII, and Joekemons. The programming-language specification itself is documented in the manuals and 1.0 design records below.

## Start here

| I want to… | Go to… |
| --- | --- |
| Read the current language reference | [JoeLang 1.0 User Manual](documents/1.0/JoeLang_1.0_User_Manual.pdf) |
| Read the original beta manual | [JoeLang beta 0.9 User Manual](documents/beta0.9/JoeLang_User_Manual.pdf) |
| Read the discovery/patent-style document | [JoeLang Patent](documents/JoeLang_Patent.pdf) |
| Study the executable 1.0 specification | [`documents/ver1.0-dev/`](documents/ver1.0-dev/) |
| Run or extend the 1.0 Studio | [`version_release_1.0/joelang-1.0-studio/`](version_release_1.0/joelang-1.0-studio/) |
| Build another JoeLang IDE | [`version_release_1.0/joelang-1.0-language-pack/`](version_release_1.0/joelang-1.0-language-pack/) |
| Review the original beta implementation | [`version_beta_0.9/`](version_beta_0.9/) |

The hosted 1.0 Studio is available at:

<https://joelang-studio-1-0-20261107.nurturingpeony1.chatgpt.site>

## Releases and language packs

### JoeLang 1.0

`version_release_1.0/` is the current release line.

- `joelang-1.0-language-pack/` is the standalone language engine. Its `dist/` directory contains `joelang-core.js`, the API manifest, and runnable examples.
- `joelang-1.0-studio/` is the browser IDE. It includes file and workspace import/export, diagnostics, syntax colors, autocomplete, automatic pairs and indentation, an input panel, runtime output, Joebject inspection, and JavaScript projection.
- The Studio also ships a portable [JoeLang 1.0 language-pack ZIP](version_release_1.0/joelang-1.0-studio/dist/downloads/joelang-language-pack-1.0.0.zip).

### JoeLang beta 0.9

`version_beta_0.9/` preserves the earlier console, Studio, and pack. It is useful for historical comparison and compatibility work, but new language and IDE work should target the 1.0 pack.

## Write a JoeLang program

```joe
jinport "std/joemath" as math

joebject Greeter {
  has strjin greeting = "Hello"

  does greet(strjin person) {
    show greeting + ", " + person
    give math.max(1, 3)
  }
}

newjoe Greeter as app
show app.greet("Joe")
```

JoeLang 1.0 uses `joebject` and `does` for object structure, `has` for typed declarations, `newjoe` for instance creation, and `show` for output. The 1.0 manual covers types, scopes, arrays, `JoerayList`, `input`, `joeach` loops, packages, and diagnostics in detail.

## Build a third-party IDE

Third-party editors should use the pack instead of copying parser or runtime code. This keeps editors compatible with future language-pack updates.

1. Copy the 1.0 language pack’s `dist/` directory into the IDE, or distribute the pack ZIP with the IDE.
2. Load `joelang-core.js` before the editor application. It exposes the browser global `JoeLangCore`.
3. Check API compatibility, then delegate language work to the pack:

```html
<script src="joelang-core.js"></script>
<script>
  if (!JoeLangCore.isCompatible(2)) throw new Error("JoeLang pack API is incompatible");

  const analysis = JoeLangCore.analyze(source);
  const suggestions = JoeLangCore.completions(prefix);
  const projection = JoeLangCore.compile(source, { fileName: "main.joe" });

  const result = await JoeLangCore.run(source, {
    onOutput: ({ text }) => console.log(text),
    inputProvider: async ({ prompt }) => window.prompt(prompt)
  });
</script>
```

`run()` is asynchronous because a program may pause at `input(...)`. A full IDE should provide an `inputProvider`, show `onOutput` text in its console, and display the diagnostics returned by `analyze()` or `compile()`. The Studio source is a working reference implementation.

The language pack deliberately does **not** expose arbitrary filesystem access, network access, browser DOM access, cookies, or persistent storage to JoeLang programs. File handling, themes, panels, project format, and UI design belong to each IDE.

## Updating an IDE safely

The language pack has its own API version and manifest (`joelang-language.json`). An IDE can upgrade parsing, keywords, diagnostics, formatting, projection, and runtime behavior by replacing the pack files and checking `JoeLangCore.isCompatible(...)`; its UI does not need to reimplement the language. Keep the pack version visible to users and test any update against the examples in `dist/examples/`.

## Repository layout

```text
JoeLang/
├── documents/                 Manuals, PDFs, LaTeX sources, and 1.0 design records
├── version_beta_0.9/          Historical beta console, Studio, and language pack
└── version_release_1.0/
    ├── joelang-1.0-language-pack/  Reusable runtime and IDE contract
    └── joelang-1.0-studio/         Browser IDE built on that runtime
```
