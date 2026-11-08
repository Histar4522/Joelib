# JoeLang Language Pack

The JoeLang Language Pack is the portable language engine used by JoeLang Studio and the JoeLang Projection Console. It has no UI framework or build dependency, so another browser-based IDE can include the same engine directly.

## Use it in an IDE

```html
<script src="https://joelang-language-pack-20261107.nurturingpeony1.chatgpt.site/joelang-core.js"></script>
```

The script exposes `window.JoeLangCore`.

```js
const analysis = JoeLangCore.analyze(source);
const suggestions = JoeLangCore.completions();
const javascript = JoeLangCore.compile(source, { fileName: "main.joe" }).code;
const result = JoeLangCore.run(source, { onOutput: console.log });
```

The stable API is versioned separately from the language. IDEs should check `JoeLangCore.isCompatible(1)` before calling the API. When the pack is updated at its stable URL, existing compatible IDEs receive new keywords, diagnostics, formatting, projection, and execution without a UI release.

## Current scope

This is JoeLang beta `0.9.0`, API `1`. Behaviors can use parameters. Input, counted loops, types, and expanded collection operations are recorded as future language features but are deliberately not reserved keywords or executable syntax yet.

## Distribute it

Share the `dist/` folder for an offline copy, or link to the hosted `joelang-core.js`. The pack is intentionally a browser global instead of a bundled application, so it can be used by a custom editor, console, teaching tool, or another IDE.
