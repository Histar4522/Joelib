# JoeLang Projection Console

A buildless static console UI. Its language behavior is deliberately not embedded here: it loads the portable [JoeLang Language Pack](../joelang-language-pack/) from the stable hosted `joelang-core.js` URL.

The package owns keyword data, diagnostics, formatting, JavaScript projection, and execution. This console owns only presentation and editor interactions, so compatible language-pack updates can improve it without a console UI release.
