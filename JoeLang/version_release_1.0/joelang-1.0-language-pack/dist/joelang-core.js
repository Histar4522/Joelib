/* JoeLang Language Pack | browser-ready core | v1.0.0 */
(function attachJoeLangCore(global) {
  "use strict";

  const VERSION = "1.0.0";
  const API_VERSION = 2;
  const INDENT = "  ";
  const RESERVED = new Set([
    "joebject", "has", "does", "newjoe", "as", "show", "give", "when", "otherwise", "joeach", "jin", "jinport", "input",
    "jint", "joeble", "joelean", "jong", "strjin", "type", "joet", "joef", "gunmu", "JAND", "JOR", "JOT", "self"
  ]);
  const TYPE_WORDS = new Set(["jint", "joeble", "joelean", "jong", "strjin"]);
  const KEYWORDS = Object.freeze([
    ["joebject", "declaration", "Declare an object-facing type."], ["has", "declaration", "Declare a typed state or variable."],
    ["does", "behavior", "Declare a typed behavior."], ["newjoe", "creation", "Create a Joebject instance."], ["as", "creation", "Name a created instance."],
    ["show", "output", "Write a value to runtime output."], ["give", "control", "Return a behavior value."],
    ["when", "control", "Run a block when a condition is joet."], ["otherwise", "control", "Run the alternate condition block."],
    ["joeach", "loop", "Start a count or collection loop."], ["jin", "loop", "Name the iterable in a collection loop."],
    ["jinport", "package", "Activate a standard package."], ["input", "input", "Request typed input from the host."],
    ["jint", "type", "32-bit whole-number type."], ["joeble", "type", "Decimal-number type."], ["joelean", "type", "Logical type."],
    ["jong", "type", "64-bit whole-number type."], ["strjin", "type", "Immutable text type."],
    ["joet", "literal", "True logical value."], ["joef", "literal", "False logical value."], ["gunmu", "literal", "Intentional absence."],
    ["JAND", "operator", "Short-circuit logical AND."], ["JOR", "operator", "Short-circuit logical OR."], ["JOT", "operator", "Logical negation."],
    ["self", "receiver", "Current Joebject instance inside a behavior."]
  ].map(([word, category, detail]) => Object.freeze({ word, category, detail })));

  const language = Object.freeze({
    name: "JoeLang", version: VERSION, apiVersion: API_VERSION, keywords: KEYWORDS,
    featureFlags: Object.freeze({ typedDeclarations: true, typedParameters: true, input: true, countLoop: true, fixedArrays: true, dynamicLists: true, packages: true })
  });

  function diagnostic(level, message, token, code, suggestion) {
    const start = token && token.start ? token.start : { line: 1, column: 1 };
    const end = token && token.end ? token.end : start;
    return { level, code, message, suggestion: suggestion || "", line: start.line, column: start.column, endLine: end.line, endColumn: end.column };
  }

  function lex(source) {
    const text = String(source || "").replace(/\r\n?/g, "\n");
    const tokens = [];
    const diagnostics = [];
    let index = 0, line = 1, column = 1;
    const point = () => ({ line, column, offset: index });
    const take = () => { const char = text[index++]; if (char === "\n") { line += 1; column = 1; } else column += 1; return char; };
    const emit = (kind, value, start) => tokens.push({ kind, value, start, end: point() });
    while (index < text.length) {
      const char = text[index];
      if (/\s/.test(char)) { take(); continue; }
      const start = point();
      if (char === "/" && text[index + 1] === "/") { while (index < text.length && text[index] !== "\n") take(); continue; }
      if (char === '"') {
        take(); let value = ""; let closed = false;
        while (index < text.length) {
          const next = take();
          if (next === '"') { closed = true; break; }
          if (next === "\\") {
            const escape = take();
            const table = { n: "\n", r: "\r", t: "\t", '"': '"', "\\": "\\" };
            value += Object.prototype.hasOwnProperty.call(table, escape) ? table[escape] : escape;
          } else value += next;
        }
        if (!closed) diagnostics.push(diagnostic("error", "Unterminated strjin literal.", { start, end: point() }, "JOE001", "Close the string with a double quote."));
        emit("string", value, start); continue;
      }
      if (/\d/.test(char)) {
        let value = ""; while (index < text.length && /\d/.test(text[index])) value += take();
        if (text[index] === "." && /\d/.test(text[index + 1])) { value += take(); while (index < text.length && /\d/.test(text[index])) value += take(); }
        if (text[index] === "J") { value += take(); emit("jong", value, start); } else emit("number", value, start);
        continue;
      }
      if (/[A-Za-z_]/.test(char)) { let value = ""; while (index < text.length && /[A-Za-z0-9_]/.test(text[index])) value += take(); emit("word", value, start); continue; }
      const double = text.slice(index, index + 2);
      if (["==", "!=", "<=", ">=", "++"].includes(double)) { take(); take(); emit("operator", double, start); continue; }
      if ("{}()[];,.?=+-*/%<>".includes(char)) { take(); emit("operator", char, start); continue; }
      take(); diagnostics.push(diagnostic("error", `Unexpected character ${JSON.stringify(char)}.`, { start, end: point() }, "JOE002", "Remove it or use a supported JoeLang token."));
    }
    tokens.push({ kind: "eof", value: "", start: point(), end: point() });
    return { text, tokens, diagnostics };
  }

  class Parser {
    constructor(source) { const result = lex(source); this.source = result.text; this.tokens = result.tokens; this.index = 0; this.diagnostics = result.diagnostics; }
    current() { return this.tokens[this.index]; }
    previous() { return this.tokens[this.index - 1]; }
    at(value) { return this.current().value === value; }
    take() { const token = this.current(); if (token.kind !== "eof") this.index += 1; return token; }
    match(value) { if (this.at(value)) { this.take(); return true; } return false; }
    error(message, token, code = "JOE003", suggestion = "") { this.diagnostics.push(diagnostic("error", message, token || this.current(), code, suggestion)); }
    expect(value, message, code, suggestion) { if (this.match(value)) return this.previous(); this.error(message || `Expected ${value}.`, this.current(), code, suggestion); return null; }
    isName(token = this.current()) { return token.kind === "word" && !RESERVED.has(token.value); }
    takeName(label = "name") { const token = this.current(); if (this.isName(token)) return this.take(); this.error(`Expected ${label}; ${token.value || "end of file"} cannot be used here.`, token, "JOE004", "Use an identifier that is not a reserved word."); return null; }
    takeMemberName(label = "member name") { const token = this.current(); if (token.kind === "word") return this.take(); this.error(`Expected ${label}.`, token, "JOE004", "Use a member identifier after the dot."); return null; }
    sync(stop = [";", "}"]) { while (this.current().kind !== "eof" && !stop.includes(this.current().value)) this.take(); if (this.at(";")) this.take(); }
    optionalEnd() { this.match(";"); }
    parse() {
      const program = { kind: "Program", imports: [], types: [], statements: [], start: this.current().start, end: null };
      while (this.current().kind !== "eof") {
        if (this.at("jinport")) program.imports.push(this.parseImport());
        else if (this.at("joebject")) program.types.push(this.parseJoebject());
        else { const statement = this.parseStatement("top"); if (statement) program.statements.push(statement); else this.sync(); }
      }
      program.end = this.current().end; return program;
    }
    parseImport() {
      const start = this.take(); const path = this.current();
      if (path.kind !== "string") { this.error("Expected a package path after jinport.", path, "JOE100", "Write a quoted path such as \"std/joemath\"."); this.sync(); return { kind: "Import", path: "", alias: "", start, end: this.previous().end }; }
      this.take(); let alias = path.value.split("@")[0].split("/").pop();
      if (this.match("as")) { const aliasToken = this.takeName("package alias"); alias = aliasToken ? aliasToken.value : alias; }
      this.optionalEnd(); return { kind: "Import", path: path.value, alias, start: start.start, end: this.previous().end };
    }
    parseJoebject() {
      const start = this.take(); const nameToken = this.takeName("Joebject name"); const name = nameToken ? nameToken.value : "<missing>";
      this.expect("{", `Expected { after Joebject ${name}.`, "JOE101", "Open the Joebject body with a brace.");
      const states = [], methods = [], names = new Set();
      while (this.current().kind !== "eof" && !this.at("}")) {
        if (this.at("has")) { const state = this.parseDeclaration("state"); if (state) { if (names.has(state.name)) this.error(`Duplicate member ${state.name} on ${name}.`, state.token, "JOE102", "Use a unique state or behavior name."); names.add(state.name); states.push(state); } }
        else if (this.at("does")) { const method = this.parseMethod(); if (method) { if (names.has(method.name)) this.error(`Duplicate member ${method.name} on ${name}.`, method.token, "JOE103", "Use a unique state or behavior name."); names.add(method.name); methods.push(method); } }
        else { this.error(`Expected has or does in Joebject ${name}.`, this.current(), "JOE104", "Joebject bodies can contain states and behaviors."); this.sync([";", "}"]); }
      }
      this.expect("}", `Expected closing brace for Joebject ${name}.`, "JOE105", "Close the Joebject body.");
      return { kind: "Joebject", name, states, methods, start: start.start, end: this.previous().end, token: nameToken || start };
    }
    parseMethod() {
      const start = this.take(); const nameToken = this.takeName("behavior name"); const name = nameToken ? nameToken.value : "<missing>";
      this.expect("(", `Expected ( after behavior ${name}.`, "JOE106"); const params = []; const parameterNames = new Set();
      while (this.current().kind !== "eof" && !this.at(")")) {
        const type = this.parseType(); const paramToken = this.takeName("parameter name");
        if (paramToken) { if (parameterNames.has(paramToken.value)) this.error(`Duplicate parameter ${paramToken.value}.`, paramToken, "JOE107", "Use a unique parameter name."); parameterNames.add(paramToken.value); params.push({ name: paramToken.value, type, token: paramToken }); }
        if (!this.match(",")) break;
      }
      this.expect(")", `Expected ) after parameters for ${name}.`, "JOE108"); const body = this.parseBlock();
      return { kind: "Method", name, params, body, start: start.start, end: body.end, token: nameToken || start };
    }
    parseType() {
      const token = this.current();
      if (token.kind !== "word") { this.error("Expected a type.", token, "JOE109", "Use jint, joeble, joelean, jong, strjin, or a visible qualified type."); this.take(); return { name: "<missing>", nullable: false, generic: null, array: null, token }; }
      let name = this.take().value;
      while (this.match(".")) { const part = this.takeName("type member"); if (!part) break; name += `.${part.value}`; }
      let generic = null;
      if (this.match("<")) { generic = this.parseType(); this.expect(">", "Expected > after generic type.", "JOE110"); }
      let nullable = false; if (this.match("?")) nullable = true;
      let array = null;
      if (this.match("[")) { if (this.at("]")) array = { fixed: false, length: null }; else { const length = this.parseExpression(); array = { fixed: true, length }; } this.expect("]", "Expected ] after array type.", "JOE111"); }
      return { name, nullable, generic, array, token };
    }
    parseBlock() {
      const start = this.current(); this.expect("{", "Expected { to start a block.", "JOE112", "Open the block with a brace."); const statements = [];
      while (this.current().kind !== "eof" && !this.at("}")) { const statement = this.parseStatement("block"); if (statement) statements.push(statement); else this.sync(); }
      this.expect("}", "Expected } to close this block.", "JOE113", "Close the block with a brace."); return { kind: "Block", statements, start: start.start, end: this.previous().end };
    }
    parseStatement(scope) {
      if (this.at("has")) return this.parseDeclaration(scope);
      if (this.at("show")) { const start = this.take(); const expression = this.parseExpression(); this.optionalEnd(); return { kind: "Show", expression, start: start.start, end: this.previous().end }; }
      if (this.at("give")) { const start = this.take(); const expression = this.parseExpression(); this.optionalEnd(); return { kind: "Give", expression, start: start.start, end: this.previous().end }; }
      if (this.at("when")) return this.parseWhen();
      if (this.at("joeach")) return this.parseLoop();
      if (this.at("newjoe")) return this.parseNew();
      if (this.at("}")) return null;
      const start = this.current(); const target = this.parseExpression();
      if (this.match("=")) { const value = this.parseExpression(); this.optionalEnd(); return { kind: "Assign", target, value, start: start.start, end: this.previous().end }; }
      this.optionalEnd(); return { kind: "Expr", expression: target, start: start.start, end: this.previous().end };
    }
    parseDeclaration(scope) {
      const start = this.take(); const type = this.parseType(); const nameToken = this.takeName("variable name"); const name = nameToken ? nameToken.value : "<missing>"; let value = null;
      if (this.match("=")) value = this.parseExpression();
      this.optionalEnd(); return { kind: "Declare", scope, type, name, value, token: nameToken || start, start: start.start, end: this.previous().end };
    }
    parseWhen() {
      const start = this.take(); const condition = this.parseExpression(); const then = this.parseBlock(); let otherwise = null;
      if (this.match("otherwise")) otherwise = this.parseBlock();
      return { kind: "When", condition, then, otherwise, start: start.start, end: (otherwise || then).end };
    }
    parseLoop() {
      const start = this.take();
      if (this.match("(")) {
        const type = this.parseType(); const nameToken = this.takeName("loop counter"); const name = nameToken ? nameToken.value : "<missing>";
        this.expect("=", "Expected = in count-loop initialization.", "JOE114"); const initial = this.parseExpression();
        this.expect(",", "Expected , after count-loop initialization.", "JOE115"); const condition = this.parseExpression();
        this.expect(",", "Expected , after count-loop condition.", "JOE116"); const update = this.parseExpression();
        this.expect(")", "Expected ) after count-loop update.", "JOE117"); const body = this.parseBlock();
        return { kind: "CountLoop", type, name, initial, condition, update, body, token: nameToken || start, start: start.start, end: body.end };
      }
      const nameToken = this.takeName("loop item"); const name = nameToken ? nameToken.value : "<missing>"; this.expect("jin", "Expected jin in collection loop.", "JOE118"); const iterable = this.parseExpression(); const body = this.parseBlock();
      return { kind: "EachLoop", name, iterable, body, token: nameToken || start, start: start.start, end: body.end };
    }
    parseNew() {
      const start = this.take(); const typeToken = this.takeName("Joebject type"); this.expect("as", "Expected as after the Joebject type.", "JOE119"); const nameToken = this.takeName("instance name"); this.optionalEnd();
      return { kind: "New", type: typeToken ? typeToken.value : "<missing>", name: nameToken ? nameToken.value : "<missing>", token: typeToken || start, start: start.start, end: this.previous().end };
    }
    parseExpression(minimum = 0) {
      let left = this.parsePrefix();
      const precedence = { JOR: 1, JAND: 2, "==": 3, "!=": 3, "<": 4, "<=": 4, ">": 4, ">=": 4, "+": 5, "-": 5, "*": 6, "/": 6, "%": 6 };
      while (true) {
        const token = this.current(); const power = precedence[token.value] || 0; if (!power || power < minimum) break;
        this.take(); const right = this.parseExpression(power + 1); left = { kind: "Binary", operator: token.value, left, right, token, start: left.start, end: right.end };
      }
      return left;
    }
    parsePrefix() {
      const token = this.take(); let expression;
      if (token.kind === "string") expression = { kind: "Literal", value: token.value, valueType: "strjin", token, start: token.start, end: token.end };
      else if (token.kind === "number") expression = { kind: "Literal", value: Number(token.value), valueType: token.value.includes(".") ? "joeble" : "jint", token, start: token.start, end: token.end };
      else if (token.kind === "jong") expression = { kind: "Literal", value: BigInt(token.value.slice(0, -1)), valueType: "jong", token, start: token.start, end: token.end };
      else if (token.value === "joet" || token.value === "joef" || token.value === "gunmu") expression = { kind: "Literal", value: token.value === "joet" ? true : token.value === "joef" ? false : null, valueType: token.value === "gunmu" ? "gunmu" : "joelean", token, start: token.start, end: token.end };
      else if (token.value === "JOT" || token.value === "-") { const value = this.parseExpression(7); expression = { kind: "Unary", operator: token.value, value, token, start: token.start, end: value.end }; }
      else if (token.value === "(") { expression = this.parseExpression(); this.expect(")", "Expected ) after expression.", "JOE120"); }
      else if (token.value === "[") { const values = []; while (this.current().kind !== "eof" && !this.at("]")) { values.push(this.parseExpression()); if (!this.match(",")) break; } const close = this.expect("]", "Expected ] after array literal.", "JOE121"); expression = { kind: "Array", values, token, start: token.start, end: (close || this.previous()).end }; }
      else if (token.kind === "word") expression = { kind: "Name", name: token.value, token, start: token.start, end: token.end };
      else { this.error(`Expected an expression, found ${token.value || "end of file"}.`, token, "JOE122"); expression = { kind: "Literal", value: null, valueType: "gunmu", token, start: token.start, end: token.end }; }
      return this.parsePostfix(expression);
    }
    parsePostfix(expression) {
      while (true) {
        if (this.match(".")) { const name = this.takeMemberName("member name"); expression = { kind: "Member", object: expression, name: name ? name.value : "<missing>", token: name || this.previous(), start: expression.start, end: (name || this.previous()).end }; continue; }
        if (this.match("[")) { const index = this.parseExpression(); const close = this.expect("]", "Expected ] after index.", "JOE123"); expression = { kind: "Index", object: expression, index, token: this.previous(), start: expression.start, end: (close || this.previous()).end }; continue; }
        let generic = null;
        if (this.at("<") && expression.kind === "Member") { const before = this.index; this.take(); const candidate = this.parseType(); if (this.match(">") && this.at("(")) generic = candidate; else this.index = before; }
        if (this.match("(")) { const args = []; while (this.current().kind !== "eof" && !this.at(")")) { args.push(this.parseExpression()); if (!this.match(",")) break; } const close = this.expect(")", "Expected ) after arguments.", "JOE124"); expression = { kind: "Call", callee: expression, args, generic, token: this.previous(), start: expression.start, end: (close || this.previous()).end }; continue; }
        if (this.match("++")) { expression = { kind: "Update", target: expression, token: this.previous(), start: expression.start, end: this.previous().end }; continue; }
        break;
      }
      return expression;
    }
  }

  function typeToString(type) { if (!type) return "unknown"; let result = type.name + (type.generic ? `<${typeToString(type.generic)}>` : "") + (type.nullable ? "?" : ""); if (type.array) result += type.array.fixed ? "[length]" : "[]"; return result; }
  function collectSemantics(ast, diagnostics) {
    const types = new Map(); const imported = new Set();
    ast.imports.forEach((item) => { if (imported.has(item.alias)) diagnostics.push(diagnostic("error", `Package alias ${item.alias} is used more than once.`, { start: item.start, end: item.end }, "JOE200", "Give each imported package a unique alias.")); imported.add(item.alias); });
    ast.types.forEach((type) => { if (types.has(type.name)) diagnostics.push(diagnostic("error", `Joebject ${type.name} is declared more than once.`, type.token, "JOE201", "Use a unique Joebject name.")); types.set(type.name, type); });
    const checkBlock = (block, declared) => block.statements.forEach((statement) => {
      if (statement.kind === "Declare") { if (declared.has(statement.name)) diagnostics.push(diagnostic("error", `Name ${statement.name} is already active in this scope.`, statement.token, "JOE202", "Choose a distinct local, parameter, or state name.")); declared.add(statement.name); if (statement.value) checkInput(statement.value, true); }
      if (statement.kind === "Assign") checkInput(statement.value, true);
      if (statement.kind === "When") { checkBlock(statement.then, new Set(declared)); if (statement.otherwise) checkBlock(statement.otherwise, new Set(declared)); }
      if (statement.kind === "CountLoop" || statement.kind === "EachLoop") checkBlock(statement.body, new Set(declared));
    });
    const checkInput = (expression, allowed) => {
      if (!expression || typeof expression !== "object") return;
      if (expression.kind === "Call" && expression.callee.kind === "Name" && expression.callee.name === "input" && !allowed) diagnostics.push(diagnostic("error", "input(...) needs a typed declaration or assignment destination.", expression.token, "JOE203", "Assign input directly to a typed variable."));
      ["left", "right", "value", "object", "index", "callee"].forEach((key) => { if (expression[key]) checkInput(expression[key], false); }); (expression.args || []).forEach((item) => checkInput(item, false)); (expression.values || []).forEach((item) => checkInput(item, false));
    };
    ast.types.forEach((type) => type.methods.forEach((method) => checkBlock(method.body, new Set([...type.states.map((item) => item.name), ...method.params.map((item) => item.name)]))));
    checkBlock({ statements: ast.statements }, new Set());
    return types;
  }

  function parse(source) { const parser = new Parser(source); const ast = parser.parse(); const types = collectSemantics(ast, parser.diagnostics); return { ast, diagnostics: parser.diagnostics, types }; }
  function analyze(source) { const result = parse(source); return { ast: result.ast, diagnostics: result.diagnostics, language, summary: { joebjects: result.ast.types.length, statements: result.ast.statements.length, imports: result.ast.imports.length } }; }

  class RuntimeFault extends Error { constructor(code, message, node, stack) { super(message); this.code = code; this.node = node; this.joeStack = stack || []; } }
  function fault(code, message, node, stack) { throw new RuntimeFault(code, message, node, stack); }
  function runtimeDiagnostic(error) { const token = error.node && (error.node.token || error.node); const item = diagnostic("error", error.message || "Runtime failure.", token, error.code || "JOE699", "Correct the referenced program value and run again."); item.stack = error.joeStack || []; return item; }
  function defaultValue(type) {
    if (type.array) return { kind: "array", elementType: { ...type, array: null }, values: Array.from({ length: 0 }, () => null) };
    if (type.nullable) return null;
    if (type.name === "jint" || type.name === "joeble") return 0;
    if (type.name === "jong") return 0n;
    if (type.name === "joelean") return false;
    if (type.name === "strjin") return "";
    return null;
  }
  function isReference(value) { return value && typeof value === "object"; }
  function display(value) {
    if (value === null) return "gunmu";
    if (value === true) return "joet"; if (value === false) return "joef";
    if (typeof value === "bigint") return `${value}J`;
    if (typeof value === "string" || typeof value === "number") return String(value);
    if (value.kind === "array") return `[${value.values.map(display).join(", ")}]`;
    if (value.kind === "list") return `[${value.values.map(display).join(", ")}]`;
    if (value.kind === "instance") return `<${value.type.name}>`;
    if (value.kind === "time") return value.value.toISOString();
    if (value.kind === "duration") return `${value.milliseconds}ms`;
    return "<value>";
  }
  function same(a, b) { if (a === b) return true; if (!isReference(a) || !isReference(b) || a.kind !== b.kind) return false; if (a.kind === "array" || a.kind === "list") return a.values.length === b.values.length && a.values.every((value, index) => same(value, b.values[index])); return false; }
  function numeric(value, node) { if (typeof value !== "number" && typeof value !== "bigint") fault("JOE500", "This operator requires numeric values.", node); return value; }
  function rangeJint(value, node) { if (!Number.isInteger(value) || value < -2147483648 || value > 2147483647) fault("JOE501", "jint value is outside the signed 32-bit range.", node); return value; }
  function matchType(value, type, node) {
    if (value === null) { if (type.nullable) return value; fault("JOE502", `${typeToString(type)} cannot receive gunmu.`, node); }
    if (type.array) {
      if (value && value.kind === "array") { if (type.array.fixed && value.values.length !== type.array.length) fault("JOE503", `Expected an array of length ${type.array.length}.`, node); value.values.forEach((item) => matchType(item, { ...type, array: null }, node)); return value; }
      fault("JOE504", `Expected ${typeToString(type)}.`, node);
    }
    if (type.name === "jint") { if (typeof value !== "number") fault("JOE505", "Expected a jint value.", node); return rangeJint(value, node); }
    if (type.name === "joeble") { if (typeof value !== "number" || !Number.isFinite(value)) fault("JOE506", "Expected a finite joeble value.", node); return value; }
    if (type.name === "jong") { if (typeof value !== "bigint") fault("JOE507", "Expected a jong value.", node); if (value < -9223372036854775808n || value > 9223372036854775807n) fault("JOE508", "jong value is outside the signed 64-bit range.", node); return value; }
    if (type.name === "joelean") { if (typeof value !== "boolean") fault("JOE509", "Expected a joelean value.", node); return value; }
    if (type.name === "strjin") { if (typeof value !== "string") fault("JOE510", "Expected a strjin value.", node); return value; }
    if (type.name.endsWith("JoerayList")) { if (!value || value.kind !== "list") fault("JOE511", `Expected ${typeToString(type)}.`, node); return value; }
    if (!value || value.kind !== "instance" || value.type.name !== type.name) fault("JOE512", `Expected ${typeToString(type)}.`, node); return value;
  }
  function makeArray(values, type, length, node) { const array = { kind: "array", elementType: type, values: values.slice() }; if (length !== null && values.length !== length) fault("JOE513", `Array initializer has ${values.length} elements; expected ${length}.`, node); array.values.forEach((item) => matchType(item, type, node)); return array; }
  class Env {
    constructor(parent, context) { this.parent = parent || null; this.context = context || (parent && parent.context) || null; this.values = new Map(); }
    declare(name, type, value, node) { if (this.values.has(name)) fault("JOE514", `Name ${name} is already declared in this scope.`, node); this.values.set(name, { type, value: matchType(value, type, node) }); }
    find(name) { if (this.values.has(name)) return { env: this, cell: this.values.get(name) }; if (this.parent) return this.parent.find(name); if (this.context && this.context.states.has(name)) return { env: null, cell: this.context.states.get(name) }; return null; }
  }
  function indexValue(target, index, node, write) { if (!Number.isInteger(index)) fault("JOE515", "Array and list indexes must be jint values.", node); const values = target && (target.kind === "array" || target.kind === "list") ? target.values : null; if (!values) fault("JOE516", "Only arrays and JoerayLists can be indexed.", node); if (index < 0 || index >= values.length) fault("JOE517", `Index ${index} is outside the valid range 0 through ${Math.max(-1, values.length - 1)}.`, node); return { values, target, write: Boolean(write) };
  }
  function pathCallee(node) { const parts = []; let current = node; while (current && current.kind === "Member") { parts.unshift(current.name); current = current.object; } if (current && current.kind === "Name") parts.unshift(current.name); return parts; }
  function makePackages(runtime, imports) {
    const map = new Map();
    const allowed = new Map([["std/joemath", "math"], ["std/joeraylist", "lists"], ["std/joetime", "joetime"], ["std/joetext", "text"]]);
    imports.forEach((item) => {
      const base = item.path.split("@")[0]; if (!allowed.has(base)) fault("JOE520", `Package ${item.path} is unavailable in JoeLang 1.0.`, { token: { start: item.start, end: item.end } });
      map.set(item.alias, { path: base, alias: item.alias });
    });
    return map;
  }
  function packageCall(pkg, member, args, generic, node, runtime) {
    if (pkg.path === "std/joemath") {
      const numbers = args.map((item) => numeric(item, node)); const number = (value) => typeof value === "bigint" ? Number(value) : value;
      const simple = { abs: () => Math.abs(number(numbers[0])), min: () => Math.min(...numbers.map(number)), max: () => Math.max(...numbers.map(number)), floor: () => Math.floor(number(numbers[0])), ceil: () => Math.ceil(number(numbers[0])), round: () => Math.round(number(numbers[0])), sqrt: () => Math.sqrt(number(numbers[0])), pow: () => Math.pow(number(numbers[0]), number(numbers[1])), clamp: () => Math.min(Math.max(number(numbers[0]), number(numbers[1])), number(numbers[2])) };
      if (member === "sqrt" && number(numbers[0]) < 0) fault("JOE521", "math.sqrt does not accept a negative value.", node);
      if (member === "random") { const seed = Number(numbers[0]); const x = Math.sin(seed) * 10000; return x - Math.floor(x); }
      if (!simple[member]) fault("JOE522", `math.${member} is not part of std/joemath.`, node); return simple[member]();
    }
    if (pkg.path === "std/joeraylist") {
      if (member !== "create") fault("JOE523", `lists.${member} is not part of std/joeraylist.`, node);
      if (!generic) fault("JOE524", "lists.create requires a type argument, for example lists.create<jint>().", node);
      return { kind: "list", elementType: generic, values: [], iterating: 0 };
    }
    if (pkg.path === "std/joetime") {
      const now = () => new Date(runtime.clock ? runtime.clock() : Date.now());
      if (member === "now") return { kind: "time", value: now() };
      if (member === "today") { const value = now(); value.setUTCHours(0, 0, 0, 0); return { kind: "time", value }; }
      if (member === "parse") { const value = new Date(args[0]); if (Number.isNaN(value.getTime())) fault("JOE525", "joetime.parse requires ISO-8601 text.", node); return { kind: "time", value }; }
      if (member === "duration") return { kind: "duration", milliseconds: Number(args[0]) };
      if (member === "between") return { kind: "duration", milliseconds: args[1].value.getTime() - args[0].value.getTime() };
      if (member === "add") return { kind: "time", value: new Date(args[0].value.getTime() + args[1].milliseconds) };
      if (member === "format") return args[0].value.toISOString();
      fault("JOE526", `joetime.${member} is not part of std/joetime.`, node);
    }
    if (pkg.path === "std/joetext") {
      const value = String(args[0]); const index = Number(args[1]);
      const api = { length: () => value.length, charAt: () => { if (index < 0 || index >= value.length) fault("JOE527", "Text index is out of bounds.", node); return value.charAt(index); }, substring: () => { const end = Number(args[2]); if (index < 0 || end < index || end > value.length) fault("JOE528", "Text range is out of bounds.", node); return value.substring(index, end); }, contains: () => value.includes(String(args[1])), indexOf: () => value.indexOf(String(args[1])), toUpper: () => value.toUpperCase(), toLower: () => value.toLowerCase(), trim: () => value.trim(), split: () => makeArray(value.split(String(args[1])), { name: "strjin", nullable: false, generic: null, array: null }, null, node) };
      if (!api[member]) fault("JOE529", `text.${member} is not part of std/joetext.`, node); return api[member]();
    }
    fault("JOE530", "Unsupported package call.", node);
  }
  function convertType(member, value, node) {
    if (member === "jint") { if (typeof value !== "number" || !Number.isFinite(value)) fault("JOE531", "type.jint requires a finite numeric value.", node); return rangeJint(Math.trunc(value), node); }
    if (member === "joeble") { if (typeof value === "bigint") return Number(value); if (typeof value === "number" && Number.isFinite(value)) return value; fault("JOE532", "type.joeble requires a numeric value.", node); }
    if (member === "jong") { if (typeof value === "bigint") return matchType(value, { name: "jong", nullable: false }, node); if (typeof value === "number" && Number.isSafeInteger(value)) return matchType(BigInt(value), { name: "jong", nullable: false }, node); fault("JOE533", "type.jong requires an integral numeric value.", node); }
    if (member === "strjin") { if (value === null || (typeof value !== "string" && typeof value !== "number" && typeof value !== "bigint" && typeof value !== "boolean")) fault("JOE534", "type.strjin accepts scalar non-gunmu values only.", node); return display(value); }
    fault("JOE535", `type.${member} is not a JoeLang conversion.`, node);
  }

  async function requestInput(prompt, expectedType, node, runtime) {
    if (!expectedType) fault("JOE610", "input(...) needs a known destination type.", node);
    if (typeof runtime.inputProvider !== "function") fault("JOE610", "Input host unavailable.", node);
    let attempt = 0;
    while (true) {
      const answer = await runtime.inputProvider({ id: ++runtime.inputId, prompt, expectedType: typeToString(expectedType), attempt, line: node.start.line, column: node.start.column });
      if (answer === null || typeof answer === "undefined") { if (expectedType.nullable) return null; fault("JOE611", "Required input was cancelled.", node); }
      const text = String(answer).trim(); let value;
      if (expectedType.name === "strjin") value = String(answer);
      else if (expectedType.name === "jint" && /^-?\d+$/.test(text)) value = Number(text);
      else if (expectedType.name === "joeble" && /^-?(?:\d+\.?\d*|\.\d+)$/.test(text)) value = Number(text);
      else if (expectedType.name === "jong" && /^-?\d+J?$/.test(text)) value = BigInt(text.replace(/J$/, ""));
      else if (expectedType.name === "joelean" && (text === "joet" || text === "joef")) value = text === "joet";
      else { runtime.onInputError({ message: `Enter a valid ${typeToString(expectedType)} value.`, expectedType: typeToString(expectedType), node }); attempt += 1; continue; }
      try { return matchType(value, expectedType, node); } catch (error) { runtime.onInputError({ message: error.message, expectedType: typeToString(expectedType), node }); attempt += 1; }
    }
  }
  async function readTarget(target, env, runtime) {
    if (target.kind === "Name") { const found = env.find(target.name); if (!found) fault("JOE540", `Unknown name ${target.name}.`, target); return { type: found.cell.type, get: () => found.cell.value, set: (value) => { found.cell.value = matchType(value, found.cell.type, target); } }; }
    if (target.kind === "Index") { const object = await evaluate(target.object, env, runtime); const index = await evaluate(target.index, env, runtime); const slot = indexValue(object, index, target, true); return { type: object.elementType, get: () => slot.values[index], set: (value) => { slot.values[index] = matchType(value, object.elementType, target); } }; }
    fault("JOE541", "This expression cannot receive an assignment.", target);
  }
  async function evaluate(node, env, runtime, expectedType) {
    if (!node) return null;
    if (node.kind === "Literal") return node.value;
    if (node.kind === "Array") { const values = []; for (const item of node.values) values.push(await evaluate(item, env, runtime)); return { kind: "array", elementType: null, values }; }
    if (node.kind === "Name") {
      if (node.name === "input") return { kind: "input-function" };
      if (node.name === "type") return { kind: "type-namespace" };
      if (node.name === "self") { if (!env.context) fault("JOE542", "self is only available inside a behavior.", node); return env.context.instance; }
      const found = env.find(node.name); if (found) return found.cell.value;
      const packageValue = runtime.packages.get(node.name); if (packageValue) return { kind: "package", package: packageValue };
      fault("JOE543", `Unknown name ${node.name}.`, node);
    }
    if (node.kind === "Unary") { const value = await evaluate(node.value, env, runtime); if (node.operator === "JOT") { if (typeof value !== "boolean") fault("JOE544", "JOT requires a joelean value.", node); return !value; } const numericValue = numeric(value, node); return typeof numericValue === "bigint" ? -numericValue : -numericValue; }
    if (node.kind === "Binary") {
      const left = await evaluate(node.left, env, runtime);
      if (node.operator === "JAND") { if (typeof left !== "boolean") fault("JOE545", "JAND requires joelean values.", node); return left ? Boolean(await evaluate(node.right, env, runtime)) : false; }
      if (node.operator === "JOR") { if (typeof left !== "boolean") fault("JOE546", "JOR requires joelean values.", node); return left ? true : Boolean(await evaluate(node.right, env, runtime)); }
      const right = await evaluate(node.right, env, runtime);
      if (node.operator === "==") return same(left, right); if (node.operator === "!=") return !same(left, right);
      if (node.operator === "+" && (typeof left === "string" || typeof right === "string")) return display(left) + display(right);
      const a = numeric(left, node), b = numeric(right, node); const big = typeof a === "bigint" || typeof b === "bigint";
      if (big && (typeof a !== "bigint" || typeof b !== "bigint")) fault("JOE547", "jong arithmetic cannot mix with non-integral numeric values without conversion.", node);
      if (["<", "<=", ">", ">="].includes(node.operator)) return node.operator === "<" ? a < b : node.operator === "<=" ? a <= b : node.operator === ">" ? a > b : a >= b;
      if (node.operator === "+") return big ? a + b : a + b;
      if (node.operator === "-") return big ? a - b : a - b;
      if (node.operator === "*") return big ? a * b : a * b;
      if (node.operator === "/") { if (b === 0 || b === 0n) fault("JOE548", "Division by zero.", node); return big ? a / b : (Number.isInteger(a) && Number.isInteger(b) ? Math.trunc(a / b) : a / b); }
      if (node.operator === "%") { if (b === 0 || b === 0n) fault("JOE549", "Division by zero.", node); return big ? a % b : a % b; }
    }
    if (node.kind === "Member") {
      const object = await evaluate(node.object, env, runtime);
      if (object && object.kind === "array" && node.name === "length") return object.values.length;
      if (object && object.kind === "type-namespace") return { kind: "type-member", name: node.name };
      if (object && object.kind === "package") return { kind: "package-member", package: object.package, name: node.name };
      if (object && object.kind === "instance") return { kind: "method", instance: object, name: node.name };
      fault("JOE550", `Member ${node.name} is not available on this value.`, node);
    }
    if (node.kind === "Index") { const object = await evaluate(node.object, env, runtime); const index = await evaluate(node.index, env, runtime); return indexValue(object, index, node).values[index]; }
    if (node.kind === "Update") { const target = await readTarget(node.target, env, runtime); const before = target.get(); if (typeof before !== "number" && typeof before !== "bigint") fault("JOE551", "++ requires a jint or jong target.", node); target.set(typeof before === "bigint" ? before + 1n : before + 1); return before; }
    if (node.kind === "Call") {
      if (node.callee.kind === "Name" && node.callee.name === "input") { const values = []; for (const arg of node.args) values.push(await evaluate(arg, env, runtime)); if (values.length !== 1 || typeof values[0] !== "string") fault("JOE552", "input requires exactly one strjin prompt.", node); return requestInput(values[0], expectedType, node, runtime); }
      if (node.callee.kind === "Member") {
        const owner = await evaluate(node.callee.object, env, runtime);
        if (owner && owner.kind === "list") { const args = []; for (const arg of node.args) args.push(await evaluate(arg, env, runtime)); return listCall(owner, node.callee.name, args, node); }
      }
      const callee = await evaluate(node.callee, env, runtime); const args = []; for (const arg of node.args) args.push(await evaluate(arg, env, runtime));
      if (callee && callee.kind === "package-member") return packageCall(callee.package, callee.name, args, node.generic, node, runtime);
      if (callee && callee.kind === "type-member") { if (args.length !== 1) fault("JOE536", `type.${callee.name} requires exactly one value.`, node); return convertType(callee.name, args[0], node); }
      if (callee && callee.kind === "method") return invoke(callee.instance, callee.name, args, node, runtime);
      fault("JOE553", "This value is not callable.", node);
    }
    fault("JOE554", "Unsupported expression.", node);
  }
  function listCall(list, name, args, node) {
    if (name === "add") { if (list.iterating) fault("JOE555", "A JoerayList cannot change length during its own joeach iteration.", node); if (args.length !== 1) fault("JOE556", "list.add requires one value.", node); list.values.push(matchType(args[0], list.elementType, node)); return null; }
    if (name === "size") return list.values.length;
    if (name === "get" || name === "set" || name === "removeAt") { const slot = indexValue(list, args[0], node, name !== "get"); if (name === "get") return slot.values[args[0]]; if (name === "set") { if (args.length !== 2) fault("JOE557", "list.set requires an index and a value.", node); const old = slot.values[args[0]]; slot.values[args[0]] = matchType(args[1], list.elementType, node); return old; } if (list.iterating) fault("JOE558", "A JoerayList cannot change length during its own joeach iteration.", node); return list.values.splice(args[0], 1)[0]; }
    if (name === "contains") return list.values.some((item) => same(item, args[0]));
    if (name === "clear") { if (list.iterating) fault("JOE559", "A JoerayList cannot change length during its own joeach iteration.", node); list.values.length = 0; return null; }
    if (name === "toArray") return makeArray(list.values, list.elementType, null, node);
    fault("JOE560", `JoerayList has no ${name} behavior.`, node);
  }
  async function executeBlock(block, env, runtime) { for (const statement of block.statements) { const result = await execute(statement, env, runtime); if (result && result.returned) return result; } return null; }
  async function execute(statement, env, runtime) {
    if (statement.kind === "Declare") {
      let value;
      if (statement.value) value = await evaluate(statement.value, env, runtime, statement.type); else if (statement.type.array) {
        const elementType = { ...statement.type, array: null }; let length = 0;
        if (statement.type.array.fixed) { length = await evaluate(statement.type.array.length, env, runtime); if (!Number.isInteger(length) || length < 0) fault("JOE561", "Fixed array length must be a non-negative jint.", statement); }
        value = makeArray(Array.from({ length }, () => defaultValue(elementType)), elementType, length, statement);
      } else value = defaultValue(statement.type);
      if (statement.type.array && statement.value && value && value.kind === "array") { value.elementType = { ...statement.type, array: null }; value.values.forEach((item) => matchType(item, value.elementType, statement)); }
      env.declare(statement.name, statement.type, value, statement); return null;
    }
    if (statement.kind === "Assign") { const target = await readTarget(statement.target, env, runtime); const value = await evaluate(statement.value, env, runtime, target.type); target.set(value); return null; }
    if (statement.kind === "Show") { const value = await evaluate(statement.expression, env, runtime); runtime.output.push(display(value)); runtime.onOutput({ text: display(value), value, line: statement.start.line }); return null; }
    if (statement.kind === "Expr") { await evaluate(statement.expression, env, runtime); return null; }
    if (statement.kind === "Give") return { returned: true, value: await evaluate(statement.expression, env, runtime) };
    if (statement.kind === "When") { const condition = await evaluate(statement.condition, env, runtime); if (typeof condition !== "boolean") fault("JOE562", "when requires a joelean condition.", statement.condition); return condition ? executeBlock(statement.then, new Env(env), runtime) : statement.otherwise ? executeBlock(statement.otherwise, new Env(env), runtime) : null; }
    if (statement.kind === "New") { const type = runtime.types.get(statement.type); if (!type) fault("JOE563", `Unknown Joebject ${statement.type}.`, statement); const instance = await createInstance(type, runtime); env.declare(statement.name, { name: statement.type, nullable: false, generic: null, array: null }, instance, statement); runtime.objects[statement.name] = describeObject(instance); return null; }
    if (statement.kind === "CountLoop") { const loop = new Env(env); loop.declare(statement.name, statement.type, await evaluate(statement.initial, loop, runtime, statement.type), statement); let guard = 0; while (true) { if (guard++ > 100000) fault("JOE564", "Loop safety limit reached (100,000 iterations).", statement); const condition = await evaluate(statement.condition, loop, runtime); if (condition !== true) break; const result = await executeBlock(statement.body, new Env(loop), runtime); if (result && result.returned) return result; await evaluate(statement.update, loop, runtime); } return null; }
    if (statement.kind === "EachLoop") { const iterable = await evaluate(statement.iterable, env, runtime); if (!iterable || (iterable.kind !== "array" && iterable.kind !== "list")) fault("JOE565", "joeach requires a Joeray or JoerayList.", statement.iterable); if (iterable.kind === "list") iterable.iterating += 1; try { const snapshot = iterable.values.slice(); for (const value of snapshot) { const loop = new Env(env); loop.declare(statement.name, iterable.elementType || { name: "strjin" }, value, statement); const result = await executeBlock(statement.body, loop, runtime); if (result && result.returned) return result; } } finally { if (iterable.kind === "list") iterable.iterating -= 1; } return null; }
    fault("JOE566", `Unsupported statement ${statement.kind}.`, statement);
  }
  async function createInstance(type, runtime) { const instance = { kind: "instance", type, states: new Map() }; const context = { instance, states: instance.states }; const env = new Env(null, context); for (const state of type.states) { let value; if (state.value) value = await evaluate(state.value, env, runtime, state.type); else value = defaultValue(state.type); env.declare(state.name, state.type, value, state); } env.values.forEach((cell, name) => instance.states.set(name, cell)); return instance; }
  function describeObject(instance) { const states = {}; instance.states.forEach((cell, name) => { states[name] = display(cell.value); }); return { type: instance.type.name, states }; }
  async function invoke(instance, name, args, node, runtime) { const method = instance.type.methods.find((item) => item.name === name); if (!method) fault("JOE567", `${instance.type.name} has no ${name} behavior.`, node); if (args.length !== method.params.length) fault("JOE568", `${name} expects ${method.params.length} argument(s), received ${args.length}.`, node); const context = { instance, states: instance.states }; const env = new Env(null, context); method.params.forEach((param, index) => env.declare(param.name, param.type, args[index], node)); runtime.stack.push(`${instance.type.name}.${name}`); try { const result = await executeBlock(method.body, env, runtime); return result && result.returned ? result.value : null; } catch (error) { if (error instanceof RuntimeFault) error.joeStack = runtime.stack.slice(); throw error; } finally { runtime.stack.pop(); } }

  function projection(ast, fileName) {
    const lines = ["\"use strict\";", `// JoeLang 1.0 projection • ${fileName || "main.joe"}`, "// This source-shaped projection is for inspection; execution uses the language pack.", ""];
    ast.imports.forEach((item) => lines.push(`// jinport ${JSON.stringify(item.path)} as ${item.alias}`));
    ast.types.forEach((type) => { lines.push(`class ${type.name} {`, "  constructor() {"); type.states.forEach((state) => lines.push(`    this.${state.name} = /* ${typeToString(state.type)} */ undefined;`)); lines.push("  }"); type.methods.forEach((method) => lines.push(`  ${method.name}(${method.params.map((param) => param.name).join(", ")}) { /* JoeLang behavior */ }`)); lines.push("}", ""); });
    lines.push("async function main() {", "  // Top-level JoeLang statements are interpreted by JoeLangCore.run().", "}"); return lines.join("\n");
  }
  function compile(source, options) { const result = parse(source); return { ok: !result.diagnostics.some((item) => item.level === "error"), diagnostics: result.diagnostics, ast: result.ast, code: projection(result.ast, options && options.fileName) }; }
  async function run(source, options) {
    const compiled = compile(source, options || {}); if (!compiled.ok) return { ok: false, diagnostics: compiled.diagnostics, output: [], objects: {}, code: compiled.code };
    const runtime = { output: [], objects: {}, stack: [], types: new Map(compiled.ast.types.map((type) => [type.name, type])), inputId: 0, inputProvider: options && options.inputProvider, onOutput: options && options.onOutput || (() => {}), onInputError: options && options.onInputError || (() => {}), clock: options && options.clock, packages: null };
    try { runtime.packages = makePackages(runtime, compiled.ast.imports); const env = new Env(null, null); for (const statement of compiled.ast.statements) await execute(statement, env, runtime); Object.keys(runtime.objects).forEach((name) => { const value = env.find(name); if (value && value.cell.value && value.cell.value.kind === "instance") runtime.objects[name] = describeObject(value.cell.value); }); return { ok: true, diagnostics: [], output: runtime.output, objects: runtime.objects, code: compiled.code }; }
    catch (error) { const item = error instanceof RuntimeFault ? runtimeDiagnostic(error) : diagnostic("error", error && error.message ? error.message : "Host runtime failure.", null, "JOE699"); return { ok: false, diagnostics: [item], output: runtime.output, objects: runtime.objects, code: compiled.code }; }
  }
  function format(source) {
    const text = String(source || "").replace(/\r\n?/g, "\n"); let depth = 0; const lines = [];
    text.split("\n").forEach((raw) => { const trimmed = raw.trim(); if (!trimmed) { lines.push(""); return; } if (trimmed.startsWith("}")) depth = Math.max(0, depth - 1); lines.push(`${INDENT.repeat(depth)}${trimmed}`); if (trimmed.endsWith("{") && !trimmed.startsWith("//")) depth += 1; }); return lines.join("\n").replace(/\n{3,}/g, "\n\n");
  }
  function completions(prefix) { const needle = String(prefix || "").toLowerCase(); return KEYWORDS.filter((item) => item.word.toLowerCase().startsWith(needle)); }
  global.JoeLangCore = Object.freeze({ version: VERSION, apiVersion: API_VERSION, language, analyze, compile, run, runAsync: run, format, completions, isCompatible: (version) => Number(version) <= API_VERSION, display });
}(typeof globalThis !== "undefined" ? globalThis : window));
