# JoeLang Studio 1.0

JoeLang Studio is a static browser IDE built on the portable JoeLang 1.0 language pack in `dist/pack/`. The UI owns files, layout, and host input; the language pack owns parsing, diagnostics, formatting, projection, and execution.

## Use

Open `dist/index.html` through a static web server or the deployed Studio URL. The Studio keeps the current workspace in browser storage and provides explicit transfer controls:

- **Open workspace** / **Save workspace**: portable `.joeworkspace.json` files.
- **Import .joe** / **Download file**: individual source files.
- **Language pack**: a downloadable release bundle for other IDE authors.

The Studio does not upload source or input history to a server.
