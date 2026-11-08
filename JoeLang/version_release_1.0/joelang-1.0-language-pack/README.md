# JoeLang 1.0 language pack

This folder is the portable, browser-ready JoeLang implementation. It contains no Studio UI code. Any static site can include `dist/joelang-core.js`, then use the `JoeLangCore` global to analyze, format, project, and run JoeLang source.

## Integration

```html
<script src="joelang-core.js"></script>
<script>
  const result = JoeLangCore.analyze(source);
  if (result.diagnostics.some((item) => item.level === "error")) return;
  await JoeLangCore.run(source, {
    onOutput: ({ text }) => console.log(text),
    inputProvider: async ({ prompt }) => window.prompt(prompt)
  });
</script>
```

`run()` is asynchronous so that `input(...)` can suspend execution safely. Hosts that do not support input receive a `JOE610` runtime diagnostic when an input expression is reached.

## Contents

- `dist/joelang-core.js` — portable browser-global runtime and analysis API
- `dist/joelang-language.json` — version, capability, and IDE contract manifest
- `dist/examples/` — small 1.0 programs

The pack exposes no filesystem, network, DOM, cookie, or persistent-storage capability to programs.
