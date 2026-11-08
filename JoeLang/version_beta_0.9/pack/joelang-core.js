/* JoeLang Language Pack | browser-ready core | v0.9.0 */
(function attachJoeLangCore(global) {
  "use strict";

  const API_VERSION = 1;
  const VERSION = "0.9.0";
  const INDENT = "  ";
  const keywordCatalog = Object.freeze([
    { word: "joebject", detail: "Declare an object-facing blueprint.", category: "declaration" },
    { word: "has", detail: "Give a Joebject an owned state value.", category: "state" },
    { word: "does", detail: "Define behavior a Joebject can receive.", category: "behavior" },
    { word: "newjoe", detail: "Create a Joebject instance.", category: "creation" },
    { word: "as", detail: "Give an instance a local name.", category: "creation" },
    { word: "show", detail: "Write a value to the console.", category: "output" },
    { word: "give", detail: "Return a value from behavior.", category: "output" },
    { word: "when", detail: "Run a block when a condition is true.", category: "decision" },
    { word: "otherwise", detail: "Provide the alternate condition path.", category: "decision" },
    { word: "each", detail: "Repeat for each value in a list.", category: "loop" },
    { word: "in", detail: "Name the collection used by each.", category: "loop" },
    { word: "ask", detail: "Request an approved Joelement.", category: "import" },
    { word: "from", detail: "Identify a requested Joelement package.", category: "import" },
    { word: "truejoe", detail: "The true logical value.", category: "literal" },
    { word: "falsejoe", detail: "The false logical value.", category: "literal" },
    { word: "nonejoe", detail: "An intentional empty value.", category: "literal" }
  ]);

  const language = Object.freeze({
    name: "JoeLang",
    version: VERSION,
    apiVersion: API_VERSION,
    keywords: keywordCatalog,
    featureFlags: Object.freeze({
      parameters: true,
      input: false,
      countLoop: false,
      dataTypes: false,
      collections: false
    }),
    roadmap: Object.freeze(["typed state", "input", "count loops", "collection operations"])
  });

  const identifier = "[A-Za-z_][A-Za-z0-9_]*";
  const identRe = new RegExp(`^${identifier}$`);
  const own = (object, key) => Object.prototype.hasOwnProperty.call(object, key);
  const lineAt = (text, index) => text.slice(0, Math.max(0, index)).split("\n").length;
  const diagnostic = (level, message, line, code = "JOE") => ({ level, message, line: Math.max(1, line || 1), code });

  function commentlessLine(line) {
    let quote = null;
    for (let index = 0; index < line.length - 1; index += 1) {
      const char = line[index];
      if ((char === '"' || char === "'") && line[index - 1] !== "\\") quote = quote === char ? null : (quote || char);
      if (!quote && char === "/" && line[index + 1] === "/") return line.slice(0, index);
    }
    return line;
  }

  function matchingBrace(text, opening) {
    let depth = 0;
    let quote = null;
    for (let index = opening; index < text.length; index += 1) {
      const char = text[index];
      if (quote) {
        if (char === quote && text[index - 1] !== "\\") quote = null;
        continue;
      }
      if (char === '"' || char === "'") { quote = char; continue; }
      if (char === "/" && text[index + 1] === "/") {
        const end = text.indexOf("\n", index + 2);
        index = end === -1 ? text.length : end;
        continue;
      }
      if (char === "{") depth += 1;
      if (char === "}") { depth -= 1; if (depth === 0) return index; }
    }
    return -1;
  }

  function maskSpan(text, start, end) {
    return text.slice(0, start) + " ".repeat(Math.max(0, end - start)) + text.slice(end);
  }

  function splitTopLevel(text, separator = ",") {
    const pieces = [];
    let start = 0, round = 0, square = 0, curly = 0, quote = null;
    for (let index = 0; index < text.length; index += 1) {
      const char = text[index];
      if (quote) { if (char === quote && text[index - 1] !== "\\") quote = null; continue; }
      if (char === '"' || char === "'") { quote = char; continue; }
      if (char === "(") round += 1;
      else if (char === ")") round -= 1;
      else if (char === "[") square += 1;
      else if (char === "]") square -= 1;
      else if (char === "{") curly += 1;
      else if (char === "}") curly -= 1;
      else if (char === separator && round === 0 && square === 0 && curly === 0) { pieces.push(text.slice(start, index).trim()); start = index + 1; }
    }
    pieces.push(text.slice(start).trim());
    return pieces.filter((piece) => piece.length > 0);
  }

  function parse(source) {
    const text = String(source || "").replace(/\r\n?/g, "\n");
    const diagnostics = [];
    const types = [];
    const typeMap = new Map();
    let programMask = text;
    const declaration = new RegExp(`\\bjoebject\\s+(${identifier})\\s*\\{`, "g");
    let match;

    while ((match = declaration.exec(text))) {
      const name = match[1];
      const opening = text.indexOf("{", match.index);
      const closing = matchingBrace(text, opening);
      const startLine = lineAt(text, match.index);
      if (closing === -1) { diagnostics.push(diagnostic("error", `Expected closing brace for Joebject ${name}.`, startLine, "JOE100")); break; }
      if (typeMap.has(name)) diagnostics.push(diagnostic("error", `Joebject ${name} is declared more than once.`, startLine, "JOE101"));
      const body = text.slice(opening + 1, closing);
      const states = [];
      const methods = [];
      const stateNames = new Set();
      const methodNames = new Set();
      let bodyMask = body;
      const methodRe = new RegExp(`\\bdoes\\s+(${identifier})\\s*\\(([^)]*)\\)\\s*\\{`, "g");
      let methodMatch;
      while ((methodMatch = methodRe.exec(body))) {
        const methodOpening = methodMatch.index + methodMatch[0].lastIndexOf("{");
        const methodClosing = matchingBrace(body, methodOpening);
        const methodLine = lineAt(text, opening + 1 + methodMatch.index);
        if (methodClosing === -1) { diagnostics.push(diagnostic("error", `Expected closing brace for behavior ${methodMatch[1]}.`, methodLine, "JOE102")); break; }
        const methodName = methodMatch[1];
        if (methodNames.has(methodName)) diagnostics.push(diagnostic("error", `Behavior ${methodName} is declared more than once on ${name}.`, methodLine, "JOE103"));
        methodNames.add(methodName);
        const params = splitTopLevel(methodMatch[2]).map((value) => value.trim());
        if (params.some((value) => !identRe.test(value)) || new Set(params).size !== params.length) diagnostics.push(diagnostic("error", `Behavior ${methodName} has an invalid parameter list.`, methodLine, "JOE104"));
        methods.push({ name: methodName, params, body: body.slice(methodOpening + 1, methodClosing), line: methodLine });
        bodyMask = maskSpan(bodyMask, methodMatch.index, methodClosing + 1);
        methodRe.lastIndex = methodClosing + 1;
      }
      const stateRe = new RegExp(`^\\s*has\\s+(${identifier})\\s*=\\s*(.+?)\\s*;?\\s*$`, "gm");
      let stateMatch;
      while ((stateMatch = stateRe.exec(bodyMask))) {
        const stateLine = lineAt(text, opening + 1 + stateMatch.index);
        const stateName = stateMatch[1];
        if (stateNames.has(stateName)) diagnostics.push(diagnostic("error", `State ${stateName} is declared more than once on ${name}.`, stateLine, "JOE105"));
        stateNames.add(stateName);
        states.push({ name: stateName, expression: stateMatch[2].replace(/;$/, "").trim(), line: stateLine });
      }
      bodyMask = bodyMask.replace(new RegExp(`^\\s*has\\s+${identifier}\\s*=.*$`, "gm"), "");
      bodyMask.split("\n").forEach((line, offset) => {
        const clean = commentlessLine(line).trim();
        if (clean) diagnostics.push(diagnostic("error", `Unexpected member in Joebject ${name}: ${clean.slice(0, 32)}.`, lineAt(text, opening + 1) + offset, "JOE106"));
      });
      const descriptor = { name, states, methods, line: startLine };
      typeMap.set(name, descriptor);
      types.push(descriptor);
      programMask = maskSpan(programMask, match.index, closing + 1);
      declaration.lastIndex = closing + 1;
    }

    const aliases = new Map();
    const statements = [];
    programMask.split("\n").forEach((line, index) => {
      const clean = commentlessLine(line).trim().replace(/;$/, "").trim();
      if (!clean) return;
      const lineNo = index + 1;
      const creation = new RegExp(`^newjoe\\s+(${identifier})\\s+as\\s+(${identifier})$`).exec(clean);
      if (creation) {
        if (!typeMap.has(creation[1])) diagnostics.push(diagnostic("error", `Unknown Joebject ${creation[1]}.`, lineNo, "JOE200"));
        if (aliases.has(creation[2])) diagnostics.push(diagnostic("error", `Receiver ${creation[2]} already exists.`, lineNo, "JOE201"));
        aliases.set(creation[2], creation[1]);
        statements.push({ kind: "new", type: creation[1], alias: creation[2], line: lineNo });
        return;
      }
      const visible = /^show\s+(.+)$/.exec(clean);
      if (visible) { statements.push({ kind: "show", expression: visible[1], line: lineNo }); return; }
      const call = new RegExp(`^(${identifier})\\.(${identifier})\\s*\\((.*)\\)$`).exec(clean);
      if (call) {
        const observedType = aliases.get(call[1]);
        if (!observedType) diagnostics.push(diagnostic("error", `Unknown receiver ${call[1]}. Create it with newjoe first.`, lineNo, "JOE202"));
        else if (!typeMap.get(observedType).methods.some((method) => method.name === call[2])) diagnostics.push(diagnostic("error", `Behavior ${call[2]} is not observed on ${observedType}.`, lineNo, "JOE203"));
        statements.push({ kind: "call", receiver: call[1], method: call[2], args: call[3], line: lineNo });
        return;
      }
      if (/^(has|does|when|otherwise|give|each)\b/.test(clean)) diagnostics.push(diagnostic("error", "This keyword belongs inside a Joebject behavior or declaration.", lineNo, "JOE204"));
      else diagnostics.push(diagnostic("error", "Could not read this top-level Joe statement.", lineNo, "JOE205"));
    });
    if (!types.length && text.trim()) diagnostics.push(diagnostic("error", "A Joe file needs at least one Joebject declaration.", 1, "JOE206"));
    return { source: text, diagnostics, types, typeMap, aliases, statements };
  }

  function analyze(source) {
    const ast = parse(source);
    return { ast, diagnostics: ast.diagnostics, summary: { joebjects: ast.types.length, statements: ast.statements.length }, language };
  }

  function codeSegments(value, transform) {
    return String(value).split(/("(?:\\.|[^"\\])*")/g).map((segment, index) => index % 2 ? segment : transform(segment)).join("");
  }

  function expressionToJavaScript(expression, context = {}) {
    return codeSegments(String(expression).trim().replace(/;$/, ""), (part) => {
      let output = part.replace(/\btruejoe\b/g, "true").replace(/\bfalsejoe\b/g, "false").replace(/\bnonejoe\b/g, "null");
      output = output.replace(/\bnot\s+/g, "!").replace(/\band\b/g, "&&").replace(/\bor\b/g, "||");
      (context.states || []).forEach((name) => {
        const re = new RegExp(`\\b${name}\\b`, "g");
        output = output.replace(re, (word, offset, whole) => whole[offset - 1] === "." ? word : `this.${word}`);
      });
      (context.methods || []).forEach((name) => {
        const re = new RegExp(`\\b${name}(?=\\s*\\()`, "g");
        output = output.replace(re, (word, offset, whole) => whole[offset - 1] === "." ? word : `this.${word}`);
      });
      return output;
    });
  }

  function translateMethod(method, descriptor) {
    const states = descriptor.states.map((state) => state.name);
    const methods = descriptor.methods.map((item) => item.name);
    const lines = method.body.split("\n").map((raw) => {
      const rawWithoutComment = commentlessLine(raw);
      const lead = (/^\s*/.exec(rawWithoutComment) || [""])[0];
      const line = rawWithoutComment.trim().replace(/;$/, "");
      if (!line) return "";
      let match;
      if ((match = /^when\s+(.+)\s*\{$/.exec(line))) return `${lead}if (${expressionToJavaScript(match[1], { states, methods })}) {`;
      if (/^}\s*otherwise\s*\{$/.test(line)) return `${lead}} else {`;
      if ((match = new RegExp(`^each\\s+(${identifier})\\s+in\\s+(.+)\\s*\\{$`).exec(line))) return `${lead}for (const ${match[1]} of ${expressionToJavaScript(match[2], { states, methods })}) {`;
      if ((match = /^show\s+(.+)$/.exec(line))) return `${lead}__show(${expressionToJavaScript(match[1], { states, methods })});`;
      if ((match = /^give\s+(.+)$/.exec(line))) return `${lead}return ${expressionToJavaScript(match[1], { states, methods })};`;
      if ((match = new RegExp(`^(${identifier})\\s*=\\s*(.+)$`).exec(line))) {
        const target = states.includes(match[1]) ? `this.${match[1]}` : match[1];
        return `${lead}${target} = ${expressionToJavaScript(match[2], { states, methods })};`;
      }
      if (/^[{}]+$/.test(line)) return `${lead}${line}`;
      return `${lead}${expressionToJavaScript(line, { states, methods })};`;
    });
    return `  ${method.name}(${method.params.join(", ")}) {\n${lines.join("\n")}\n  }`;
  }

  function project(ast, fileName = "main.joe") {
    const lines = ["\"use strict\";", `// JoeLang projection • ${fileName}`, "const __output = [];", "const __show = (value) => __output.push(String(value));", "const __objects = {};"];
    ast.types.forEach((descriptor) => {
      lines.push(`\nclass ${descriptor.name} {`, "  constructor() {");
      descriptor.states.forEach((state) => lines.push(`    this.${state.name} = ${expressionToJavaScript(state.expression)};`));
      lines.push("  }");
      descriptor.methods.forEach((method) => lines.push(translateMethod(method, descriptor)));
      lines.push("}");
    });
    lines.push("\n// Program entry");
    ast.statements.forEach((statement) => {
      if (statement.kind === "new") lines.push(`const ${statement.alias} = new ${statement.type}(); __objects.${statement.alias} = ${statement.alias};`);
      if (statement.kind === "show") lines.push(`__show(${expressionToJavaScript(statement.expression)});`);
      if (statement.kind === "call") lines.push(`${statement.receiver}.${statement.method}(${expressionToJavaScript(statement.args)});`);
    });
    lines.push("return { output: __output, objects: __objects };");
    return lines.join("\n");
  }

  function compile(source, options = {}) {
    const analysis = options.analysis || analyze(source);
    return { code: project(analysis.ast, options.fileName || "main.joe"), analysis, ast: analysis.ast, summary: analysis.summary };
  }

  function unwrapParentheses(expression) {
    const text = expression.trim();
    if (!text.startsWith("(") || !text.endsWith(")")) return text;
    let depth = 0;
    for (let index = 0; index < text.length; index += 1) {
      if (text[index] === "(") depth += 1;
      if (text[index] === ")") depth -= 1;
      if (depth === 0 && index < text.length - 1) return text;
    }
    return unwrapParentheses(text.slice(1, -1));
  }

  function topLevelOperator(text, candidates) {
    let round = 0, square = 0, quote = null;
    for (let index = 0; index < text.length; index += 1) {
      const char = text[index];
      if (quote) { if (char === quote && text[index - 1] !== "\\") quote = null; continue; }
      if (char === '"' || char === "'") { quote = char; continue; }
      if (char === "(") round += 1;
      else if (char === ")") round -= 1;
      else if (char === "[") square += 1;
      else if (char === "]") square -= 1;
      if (round || square) continue;
      for (const candidate of candidates) {
        if (text.slice(index, index + candidate.length) === candidate) return { index, operator: candidate };
      }
    }
    return null;
  }

  function evaluateExpression(expression, scope, invoke) {
    const text = unwrapParentheses(String(expression || "").trim().replace(/;$/, ""));
    if (!text) return undefined;
    if (text === "truejoe") return true;
    if (text === "falsejoe") return false;
    if (text === "nonejoe") return null;
    if (/^-?\d+(?:\.\d+)?$/.test(text)) return Number(text);
    const logical = topLevelOperator(text, [" or ", " and "]);
    if (logical) {
      const left = evaluateExpression(text.slice(0, logical.index), scope, invoke);
      return logical.operator.trim() === "or" ? (left || evaluateExpression(text.slice(logical.index + logical.operator.length), scope, invoke)) : (left && evaluateExpression(text.slice(logical.index + logical.operator.length), scope, invoke));
    }
    const comparison = topLevelOperator(text, ["==", "!=", ">=", "<=", ">", "<"]);
    if (comparison) {
      const left = evaluateExpression(text.slice(0, comparison.index), scope, invoke);
      const right = evaluateExpression(text.slice(comparison.index + comparison.operator.length), scope, invoke);
      return ({ "==": left === right, "!=": left !== right, ">": left > right, "<": left < right, ">=": left >= right, "<=": left <= right })[comparison.operator];
    }
    const plus = topLevelOperator(text, ["+"]);
    if (plus) return evaluateExpression(text.slice(0, plus.index), scope, invoke) + evaluateExpression(text.slice(plus.index + 1), scope, invoke);
    const minus = topLevelOperator(text.slice(1), ["-"]);
    if (minus) return evaluateExpression(text.slice(0, minus.index + 1), scope, invoke) - evaluateExpression(text.slice(minus.index + 2), scope, invoke);
    if (text.startsWith("not ")) return !evaluateExpression(text.slice(4), scope, invoke);
    if ((text.startsWith('"') && text.endsWith('"')) || (text.startsWith("'") && text.endsWith("'"))) {
      if (text[0] === '"') { try { return JSON.parse(text); } catch { /* fall through */ } }
      return text.slice(1, -1).replace(/\\(['"\\])/g, "$1");
    }
    if (text.startsWith("[") && text.endsWith("]")) return splitTopLevel(text.slice(1, -1)).map((item) => evaluateExpression(item, scope, invoke));
    const message = new RegExp(`^(${identifier})\\.(${identifier})\\s*\\((.*)\\)$`).exec(text);
    if (message) return invoke(message[1], message[2], splitTopLevel(message[3]).map((arg) => evaluateExpression(arg, scope, invoke)));
    const field = new RegExp(`^(${identifier})\\.(${identifier})$`).exec(text);
    if (field && own(scope, field[1]) && scope[field[1]] && own(scope[field[1]], field[2])) return scope[field[1]][field[2]];
    if (own(scope, text)) return scope[text];
    throw new Error(`I cannot evaluate “${text}”.`);
  }

  function statementsFrom(body) {
    const statements = [];
    let index = 0;
    while (index < body.length) {
      while (/\s|;/.test(body[index] || "")) index += 1;
      if (index >= body.length) break;
      if (body.startsWith("when ", index)) {
        const opening = body.indexOf("{", index);
        const closing = matchingBrace(body, opening);
        if (opening === -1 || closing === -1) throw new Error("A when statement needs a complete brace block.");
        const condition = body.slice(index + 5, opening).trim();
        let cursor = closing + 1;
        while (/\s/.test(body[cursor] || "")) cursor += 1;
        let alternate = "";
        if (body.startsWith("otherwise", cursor)) {
          const alternateOpening = body.indexOf("{", cursor);
          const alternateClosing = matchingBrace(body, alternateOpening);
          if (alternateOpening === -1 || alternateClosing === -1) throw new Error("Otherwise needs a complete brace block.");
          alternate = body.slice(alternateOpening + 1, alternateClosing);
          cursor = alternateClosing + 1;
        }
        statements.push({ kind: "when", condition, yes: body.slice(opening + 1, closing), no: alternate });
        index = cursor;
        continue;
      }
      if (body.startsWith("each ", index)) {
        const opening = body.indexOf("{", index);
        const closing = matchingBrace(body, opening);
        const header = body.slice(index + 5, opening).trim();
        const match = new RegExp(`^(${identifier})\\s+in\\s+(.+)$`).exec(header);
        if (!match || closing === -1) throw new Error("Use each item in collection { ... }.");
        statements.push({ kind: "each", item: match[1], collection: match[2], body: body.slice(opening + 1, closing) });
        index = closing + 1;
        continue;
      }
      const end = body.indexOf("\n", index);
      const line = commentlessLine(body.slice(index, end === -1 ? body.length : end)).trim().replace(/;$/, "");
      index = end === -1 ? body.length : end + 1;
      if (line) statements.push({ kind: "line", line });
    }
    return statements;
  }

  function run(source, options = {}) {
    const analysis = options.analysis || analyze(source);
    const error = analysis.diagnostics.find((item) => item.level === "error");
    if (error) { const fault = new Error(error.message); fault.diagnostic = error; throw fault; }
    const instances = {};
    const output = [];
    const events = [];
    const emit = (value) => { const text = String(value); output.push(text); events.push({ type: "output", value: text }); if (typeof options.onOutput === "function") options.onOutput(text); };
    const invoke = (receiver, methodName, args = []) => {
      const instance = instances[receiver];
      if (!instance) throw new Error(`Unknown receiver: ${receiver}. Create it with newjoe first.`);
      const method = instance.type.definition.methods.find((item) => item.name === methodName);
      if (!method) throw new Error(`${instance.type.name} does not observe a behavior named ${methodName}.`);
      if (method.params.length !== args.length) throw new Error(`${methodName} expects ${method.params.length} input${method.params.length === 1 ? "" : "s"}.`);
      const scope = { ...instance.state, ...instances };
      method.params.forEach((param, index) => { scope[param] = args[index]; });
      const execute = (body, localScope) => {
        for (const statement of statementsFrom(body)) {
          if (statement.kind === "when") {
            const result = execute(evaluateExpression(statement.condition, localScope, invoke) ? statement.yes : statement.no, localScope);
            if (result.returned) return result;
            continue;
          }
          if (statement.kind === "each") {
            const collection = evaluateExpression(statement.collection, localScope, invoke);
            if (!Array.isArray(collection)) throw new Error(`Each needs a list; ${statement.collection} is not a list.`);
            for (const item of collection) {
              const result = execute(statement.body, { ...localScope, [statement.item]: item });
              if (result.returned) return result;
            }
            continue;
          }
          if (statement.line.startsWith("show ")) { emit(evaluateExpression(statement.line.slice(5), localScope, invoke)); continue; }
          if (statement.line.startsWith("give ")) return { returned: true, value: evaluateExpression(statement.line.slice(5), localScope, invoke) };
          const update = new RegExp(`^(${identifier})\\s*=\\s*(.+)$`).exec(statement.line);
          if (update) {
            if (!own(instance.state, update[1])) throw new Error(`State ${update[1]} is not owned by ${instance.type.name}.`);
            instance.state[update[1]] = evaluateExpression(update[2], localScope, invoke);
            localScope[update[1]] = instance.state[update[1]];
            continue;
          }
          const message = new RegExp(`^(${identifier})\\.(${identifier})\\s*\\((.*)\\)$`).exec(statement.line);
          if (message) { invoke(message[1], message[2], splitTopLevel(message[3]).map((arg) => evaluateExpression(arg, localScope, invoke))); continue; }
          throw new Error(`I cannot yet run this line: ${statement.line}`);
        }
        return { returned: false, value: undefined };
      };
      return execute(method.body, scope).value;
    };
    for (const statement of analysis.ast.statements) {
      if (statement.kind === "new") {
        const definition = analysis.ast.typeMap.get(statement.type);
        const state = {};
        for (const descriptor of definition.states) state[descriptor.name] = evaluateExpression(descriptor.expression, { ...state, ...instances }, invoke);
        const methods = Object.fromEntries(definition.methods.map((method) => [method.name, { params: method.params.slice() }]));
        instances[statement.alias] = { type: { name: definition.name, methods, definition }, state };
        events.push({ type: "create", alias: statement.alias, joebject: definition.name });
      } else if (statement.kind === "show") {
        emit(evaluateExpression(statement.expression, instances, invoke));
      } else if (statement.kind === "call") {
        invoke(statement.receiver, statement.method, splitTopLevel(statement.args).map((arg) => evaluateExpression(arg, instances, invoke)));
      }
    }
    return { output, objects: instances, events, analysis };
  }

  function format(source) {
    let depth = 0;
    const lines = String(source || "").replace(/\r\n?/g, "\n").split("\n");
    const formatted = lines.map((raw) => {
      const clean = raw.trim();
      if (!clean) return "";
      if (/^}/.test(clean)) depth = Math.max(0, depth - 1);
      const line = `${INDENT.repeat(depth)}${clean.replace(/\s+$/g, "")}`;
      const opens = (commentlessLine(clean).match(/{/g) || []).length;
      const closes = (commentlessLine(clean).match(/}/g) || []).length;
      depth = Math.max(0, depth + opens - closes);
      return line;
    });
    return formatted.join("\n").replace(/\n{3,}/g, "\n\n").replace(/\s+$/, "") + "\n";
  }

  const templates = Object.freeze({
    basicFile({ typeName = "Main" } = {}) {
      const safe = String(typeName).replace(/[^A-Za-z0-9_]/g, "") || "Main";
      const alias = safe.charAt(0).toLowerCase() + safe.slice(1);
      return `joebject ${safe} {\n  has message = "Hello from ${safe}"\n\n  does run() {\n    show message\n  }\n}\n\nnewjoe ${safe} as ${alias}\n${alias}.run()\n`;
    }
  });

  const core = Object.freeze({
    apiVersion: API_VERSION,
    version: VERSION,
    language,
    completions: () => keywordCatalog.map((item) => ({ ...item })),
    parse,
    analyze,
    compile,
    run,
    format,
    templates,
    isCompatible: (apiVersion) => Number(apiVersion) === API_VERSION,
    assertCompatible(apiVersion) {
      if (!this.isCompatible(apiVersion)) throw new Error(`JoeLang core API ${API_VERSION} cannot satisfy API ${apiVersion}.`);
      return true;
    }
  });

  global.JoeLangCore = core;
})(globalThis);
