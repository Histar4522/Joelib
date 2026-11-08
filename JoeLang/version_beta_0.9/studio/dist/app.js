(() => {
  "use strict";

  const core = globalThis.JoeLangCore;
  if (!core || !core.isCompatible(1)) {
    document.body.insertAdjacentHTML("afterbegin", '<p style="margin:16px;color:#ff8e8e;font:13px ui-monospace,monospace">JoeLang language pack failed to load. Reload the page to restore the editor.</p>');
    return;
  }
  const KEYWORDS = core.completions().map(({ word, category, detail }) => [word, category || detail]);
  const $ = (selector) => document.querySelector(selector);
  const el = {
    shell: $(".studio-shell"), workspace: $(".workspace"), explorer: $("#explorer"), tree: $("#file-tree"), tabs: $("#editor-tabs"),
    editor: $("#code-editor"), highlighter: $("#syntax-highlight code"), highlightPre: $("#syntax-highlight"),
    lineNumbers: $("#line-numbers"), crumb: $("#crumb-path"), saveDot: $("#save-dot"),
    completions: $("#completion-menu"), problems: $("#problems-list"), problemCount: $("#problem-count"),
    diagnosticStatus: $("#diagnostic-status"), cursor: $("#cursor-status"), run: $("#run-button"),
    objects: $("#object-tree"), js: $("#js-transcript code"), output: $("#program-output code"),
    runState: $("#run-state"), context: $("#context-menu"), createDialog: $("#create-dialog"), templateChoice: $("#template-choice"),
    createForm: $("#create-form"), createName: $("#create-name"), createTitle: $("#create-title"),
    createKicker: $("#create-kicker"), createNote: $("#create-note"), operationDialog: $("#operation-dialog"), operationForm: $("#operation-form"),
    operationKicker: $("#operation-kicker"), operationTitle: $("#operation-title"), operationName: $("#operation-name"), operationNameGroup: $("#operation-name-group"),
    operationDestination: $("#operation-destination"), operationDestinationGroup: $("#operation-destination-group"), operationNote: $("#operation-note"),
    deleteDialog: $("#delete-dialog"), deleteForm: $("#delete-form"), deleteTitle: $("#delete-title"), deleteWarning: $("#delete-warning"),
    manualDialog: $("#manual-dialog"), manualReader: $("#manual-reader"), shortcutDialog: $("#shortcut-dialog")
  };

  const mainSource = `// A compact JoeLang program. Press Ctrl+Enter to run.
joebject Lantern {
  has charge = 3
  has label = "Blue lantern"

  does glow() {
    when charge > 0 {
      show label + " is glowing."
      charge = charge - 1
    } otherwise {
      show "The lantern needs charge."
    }
    give charge
  }
}

newjoe Lantern as lantern
show lantern.glow()`;

  const files = [
    { id: "src", type: "folder", name: "src", parent: null, expanded: true },
    { id: "main", type: "file", name: "main.joe", parent: "src", content: mainSource, modified: false },
    { id: "lantern", type: "file", name: "lantern.joe", parent: "src", content: `joebject SignalLamp {\n  has color = "amber"\n\n  does announce() {\n    show "Signal: " + color\n  }\n}\n\nnewjoe SignalLamp as lamp\nlamp.announce()`, modified: false },
    { id: "examples", type: "folder", name: "examples", parent: null, expanded: true },
    { id: "roll-call", type: "file", name: "roll_call.joe", parent: "examples", content: `joebject RollCall {\n  has names = ["Ari", "Bo", "Cy"]\n\n  does greet_everyone() {\n    each person in names {\n      show "Hello, " + person\n    }\n  }\n}\n\nnewjoe RollCall as call\ncall.greet_everyone()`, modified: false }
  ];
  let activeId = "main";
  let openFiles = ["main", "lantern"];
  let createKind = "file";
  let createParentId = null;
  let contextTargetId = null;
  let pendingOperation = null;
  let pendingDeleteId = null;
  let completionIndex = 0;
  let currentDiagnostics = [];
  let runRecord = "";

  const fileById = (id) => files.find((file) => file.id === id);
  const activeFile = () => fileById(activeId);
  const escapeHTML = (value) => String(value).replace(/[&<>"']/g, (char) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" })[char]);
  const escapeRegExp = (value) => value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const indentFor = (line) => (line.match(/^\s*/) || [""])[0];

  function pathFor(node) {
    const parts = [node.name];
    let current = node;
    while (current.parent) {
      current = fileById(current.parent);
      if (!current) break;
      parts.unshift(current.name);
    }
    return parts.join(" › ");
  }

  function renderTree() {
    const fragment = document.createDocumentFragment();
    const renderChildren = (parent, depth) => {
      files.filter((node) => node.parent === parent).forEach((node) => {
        const row = document.createElement("div");
        row.className = `tree-row depth-${Math.min(depth, 3)}${node.id === activeId ? " active" : ""}`;
        row.dataset.id = node.id;
        const canExpand = node.type !== "file";
        const arrow = canExpand ? (node.expanded === false ? "›" : "⌄") : "";
        const icon = node.type === "file" ? "◇" : node.type === "package" ? "⬡" : "▰";
        row.innerHTML = `<span class="tree-chevron">${arrow}</span><span class="tree-icon ${node.type}">${icon}</span><span class="tree-name">${escapeHTML(node.name)}</span><button class="tree-delete" type="button" aria-label="Delete ${escapeHTML(node.name)}" title="Delete ${escapeHTML(node.name)}">×</button>`;
        row.addEventListener("click", () => {
          if (node.type === "file") selectFile(node.id);
          else { node.expanded = node.expanded === false; renderTree(); }
        });
        row.querySelector(".tree-delete").addEventListener("click", (event) => { event.stopPropagation(); requestDelete(node.id); });
        fragment.append(row);
        if (canExpand && node.expanded !== false) renderChildren(node.id, depth + 1);
      });
    };
    renderChildren(null, 0);
    el.tree.replaceChildren(fragment);
  }

  function renderTabs() {
    el.tabs.replaceChildren(...openFiles.map((id) => {
      const file = fileById(id);
      const button = document.createElement("button");
      button.type = "button";
      button.className = `editor-tab${id === activeId ? " active" : ""}`;
      button.innerHTML = `<span aria-hidden="true">◇</span><span>${escapeHTML(file.name)}</span>${file.modified ? '<span class="modified-mark">●</span>' : ""}`;
      button.addEventListener("click", () => selectFile(id));
      return button;
    }));
  }

  function selectFile(id) {
    const file = fileById(id);
    if (!file || file.type !== "file") return;
    activeId = id;
    if (!openFiles.includes(id)) openFiles.push(id);
    el.editor.value = file.content;
    el.editor.selectionStart = el.editor.selectionEnd = 0;
    renderTree(); renderTabs(); renderEditor(); updateCursor(); compileActive();
    el.editor.focus();
  }

  function descendantIds(id, ids = new Set()) {
    ids.add(id);
    files.filter((node) => node.parent === id).forEach((node) => descendantIds(node.id, ids));
    return ids;
  }

  function deleteItem(id) {
    const target = fileById(id);
    if (!target) return;
    const ids = descendantIds(id);
    const removedActive = ids.has(activeId);
    for (let index = files.length - 1; index >= 0; index -= 1) if (ids.has(files[index].id)) files.splice(index, 1);
    openFiles = openFiles.filter((fileId) => !ids.has(fileId));
    contextTargetId = null;
    let nextFile = files.find((node) => node.type === "file");
    if (!nextFile) {
      nextFile = { id: `untitled-${Date.now()}`, type: "file", name: "untitled.joe", parent: null, content: "", modified: false };
      files.push(nextFile);
    }
    if (removedActive) selectFile(nextFile.id);
    else { renderTree(); renderTabs(); }
  }

  function deleteWarningIsDisabled() {
    try { return localStorage.getItem("joelang-studio.skip-delete-warning") === "true"; } catch { return false; }
  }

  function requestDelete(id) {
    const target = fileById(id);
    if (!target) return;
    if (deleteWarningIsDisabled()) { deleteItem(id); return; }
    pendingDeleteId = id;
    const childCount = descendantIds(id).size - 1;
    const kind = target.type === "file" ? "file" : target.type;
    el.deleteTitle.textContent = `Delete ${target.name}?`;
    const warning = childCount
      ? `This cannot be undone. ${childCount} item${childCount === 1 ? "" : "s"} inside this ${kind} will also be deleted.`
      : `This cannot be undone. ${target.name} will be removed from the workspace.`;
    el.deleteWarning.textContent = `${warning} “Don't show again” deletes this item and disables future deletion warnings.`;
    el.deleteDialog.showModal();
  }

  function normalizeName(name, type) {
    const clean = name.trim().replace(/[\\/:*?"<>|]/g, "");
    return type === "file" && clean && !/\.joe$/i.test(clean) ? `${clean}.joe` : clean;
  }

  function nameTaken(name, exceptId = null) {
    return files.some((node) => node.id !== exceptId && node.name.toLowerCase() === name.toLowerCase());
  }

  function suggestedCopyName(name) {
    const base = name.replace(/\.joe$/i, ""); let attempt = `${base}_copy.joe`; let count = 2;
    while (nameTaken(attempt)) { attempt = `${base}_copy_${count}.joe`; count += 1; }
    return attempt;
  }

  function availableDestinations(item) {
    const excluded = descendantIds(item.id);
    return files.filter((node) => (node.type === "folder" || node.type === "package") && !excluded.has(node.id));
  }

  function openFileOperation(kind, id) {
    const item = fileById(id);
    if (!item) return;
    pendingOperation = { kind, id };
    const labels = {
      rename: [`RENAME ${item.type.toUpperCase()}`, `Rename ${item.name}`, "Choose a clear, unique name for this workspace item.", "Rename"],
      duplicate: ["DUPLICATE FILE", `Duplicate ${item.name}`, "A separate copy will be created beside the original.", "Create copy"],
      move: ["MOVE FILE", `Move ${item.name}`, "Choose the folder that should contain this file.", "Move file"]
    };
    const [kicker, title, note, confirm] = labels[kind];
    el.operationKicker.textContent = kicker; el.operationTitle.textContent = title; el.operationNote.textContent = note;
    $("#operation-confirm").textContent = confirm;
    const needsName = kind !== "move"; el.operationNameGroup.hidden = !needsName; el.operationName.required = needsName;
    el.operationDestinationGroup.hidden = kind !== "move";
    if (kind === "rename") el.operationName.value = item.name;
    if (kind === "duplicate") el.operationName.value = suggestedCopyName(item.name);
    if (kind === "move") {
      const destinations = availableDestinations(item);
      el.operationDestination.replaceChildren(...[
        new Option("Workspace root", ""),
        ...destinations.map((folder) => new Option(`${folder.type === "package" ? "Package" : "Folder"}: ${pathFor(folder)}`, folder.id))
      ]);
      el.operationDestination.value = item.parent || "";
    }
    el.operationDialog.showModal();
    setTimeout(() => (needsName ? el.operationName : el.operationDestination).focus(), 0);
  }

  function performFileOperation() {
    const operation = pendingOperation; const item = operation && fileById(operation.id);
    if (!operation || !item) return;
    if (operation.kind === "move") {
      item.parent = el.operationDestination.value || null;
    } else {
      const name = normalizeName(el.operationName.value, item.type);
      if (!name) { el.operationNote.textContent = "A name is required."; return; }
      if (nameTaken(name, operation.kind === "rename" ? item.id : null)) { el.operationNote.textContent = "That name is already in this workspace."; return; }
      if (operation.kind === "rename") item.name = name;
      if (operation.kind === "duplicate") {
        const copy = { ...item, id: `file-${Date.now()}`, name, content: item.content, modified: false };
        files.push(copy); selectFile(copy.id);
      }
    }
    pendingOperation = null; el.operationDialog.close();
    renderTree(); renderTabs();
    const active = activeFile(); if (active) el.crumb.textContent = pathFor(active);
  }

  function syntaxLine(line) {
    const keywordPattern = KEYWORDS.map(([word]) => escapeRegExp(word)).join("|");
    const token = new RegExp(`(\\/\\/.*$)|("(?:\\\\.|[^"\\\\])*")|\\b(${keywordPattern})\\b|\\b(\\d+(?:\\.\\d+)?)\\b|\\b([A-Z][A-Za-z0-9_]*)\\b|\\b([a-z_][A-Za-z0-9_]*)(?=\\s*\\()|([{}()[\\],.=+\\-*/!<>]+)|\\b([a-z_][A-Za-z0-9_]*)\\b`, "g");
    let output = ""; let cursor = 0; let match;
    while ((match = token.exec(line))) {
      output += escapeHTML(line.slice(cursor, match.index));
      const value = escapeHTML(match[0]);
      const kind = match[1] ? "comment" : match[2] ? "string" : match[3] ? "keyword" : match[4] ? "number" : match[5] ? "type" : match[6] ? "function" : match[7] ? "operator" : "name";
      output += `<span class="token-${kind}">${value}</span>`;
      cursor = match.index + match[0].length;
    }
    return output + escapeHTML(line.slice(cursor));
  }

  function renderEditor() {
    const source = el.editor.value;
    el.highlighter.innerHTML = source.split("\n").map(syntaxLine).join("\n") + "\n";
    const errorLines = new Set(currentDiagnostics.filter((item) => item.level === "error").map((item) => item.line));
    el.lineNumbers.innerHTML = source.split("\n").map((_, index) => `<span class="${errorLines.has(index + 1) ? "error-line" : ""}">${index + 1}</span>`).join("\n");
    el.highlightPre.scrollTop = el.editor.scrollTop;
    el.highlightPre.scrollLeft = el.editor.scrollLeft;
    el.lineNumbers.scrollTop = el.editor.scrollTop;
  }

  function updateCursor() {
    const before = el.editor.value.slice(0, el.editor.selectionStart);
    const row = before.split("\n").length;
    const col = before.length - before.lastIndexOf("\n");
    el.cursor.textContent = `Ln ${row}, Col ${col}`;
  }

  function compileActive() {
    const analysis = core.analyze(el.editor.value);
    currentDiagnostics = analysis.diagnostics;
    renderProblems(); renderEditor();
    const compiled = core.compile(el.editor.value, { fileName: activeFile().name, analysis });
    el.js.textContent = compiled.code;
    return { analysis, compiled };
  }

  function renderProblems() {
    const errors = currentDiagnostics.filter((item) => item.level === "error");
    el.problemCount.textContent = errors.length;
    el.diagnosticStatus.textContent = errors.length ? `${errors.length} error${errors.length === 1 ? "" : "s"}` : "Ready";
    if (!currentDiagnostics.length) { el.problems.innerHTML = '<span class="quiet">No problems found in the active file.</span>'; return; }
    el.problems.replaceChildren(...currentDiagnostics.map((item) => {
      const row = document.createElement("button"); row.type = "button"; row.className = `problem-row ${item.level}`;
      row.innerHTML = `<span class="problem-symbol">⊗</span><span class="problem-message">${escapeHTML(item.message)}</span><span class="problem-location">${activeFile().name}:${item.line} · ${item.code}</span>`;
      row.addEventListener("click", () => goToLine(item.line)); return row;
    }));
  }

  function goToLine(line) {
    const lines = el.editor.value.split("\n"); const position = lines.slice(0, Math.max(0, line - 1)).join("\n").length + (line > 1 ? 1 : 0);
    el.editor.focus(); el.editor.selectionStart = el.editor.selectionEnd = position; el.editor.scrollTop = Math.max(0, (line - 3) * 21); updateCursor();
  }

  function renderObjects(objects) {
    const entries = Object.entries(objects || {});
    if (!entries.length) { el.objects.innerHTML = '<div class="empty-state">No Joebject instances are available.</div>'; return; }
    el.objects.replaceChildren(...entries.flatMap(([name, object]) => {
      const rows = []; const root = document.createElement("div"); root.className = "object-row object-root"; root.innerHTML = `<span>⌄</span><span>${escapeHTML(name)}</span><span>:</span><span>${escapeHTML(object.type.name)}</span>`; rows.push(root);
      Object.keys(object.state).forEach((key) => { const row = document.createElement("div"); row.className = "object-row object-field"; row.innerHTML = `<span>├</span><span class="object-key">${escapeHTML(key)}</span><span>:</span><span class="object-value">${escapeHTML(JSON.stringify(object.state[key]))}</span>`; rows.push(row); });
      return rows;
    }));
  }

  function runActive() {
    const { analysis } = compileActive();
    el.runState.className = "run-state";
    if (analysis.diagnostics.some((item) => item.level === "error")) {
      el.output.textContent = "Run stopped: fix the compile-time errors listed below.";
      el.runState.textContent = "Blocked"; el.runState.classList.add("bad"); renderObjects({}); return;
    }
    try {
      const result = core.run(el.editor.value, { analysis });
      runRecord = result.output.join("\n") || "Program completed with no output.";
      el.output.textContent = runRecord; renderObjects(result.objects);
      el.runState.textContent = "Ran"; el.runState.classList.add("good"); el.diagnosticStatus.textContent = "Run complete";
    } catch (error) {
      currentDiagnostics = [error.diagnostic || { level: "error", message: `Runtime projection error: ${error.message}`, line: 1, code: "JOE500" }]; renderProblems(); renderEditor();
      el.output.textContent = `Runtime error: ${error.message}`; el.runState.textContent = "Failed"; el.runState.classList.add("bad"); renderObjects({});
    }
  }

  function getWordBeforeCaret() {
    const before = el.editor.value.slice(0, el.editor.selectionStart);
    const match = /([A-Za-z_][A-Za-z0-9_]*)$/.exec(before);
    return match ? match[1] : "";
  }

  function completionCandidates(force = false) {
    const prefix = getWordBeforeCaret().toLowerCase();
    if (!force && prefix.length < 1) return [];
    return KEYWORDS.filter(([word]) => !prefix || word.startsWith(prefix));
  }

  function caretMenuPosition() {
    const before = el.editor.value.slice(0, el.editor.selectionStart); const lines = before.split("\n");
    return { left: Math.max(10, 18 + lines[lines.length - 1].length * 7.8 - el.editor.scrollLeft), top: Math.max(8, 16 + (lines.length - 1) * 21 - el.editor.scrollTop) };
  }

  function showCompletions(force = false) {
    const options = completionCandidates(force);
    if (!options.length) { hideCompletions(); return; }
    completionIndex = Math.min(completionIndex, options.length - 1); const position = caretMenuPosition();
    el.completions.style.left = `${position.left}px`; el.completions.style.top = `${position.top}px`;
    el.completions.innerHTML = options.map(([word, kind], index) => `<button class="completion-option${index === completionIndex ? " selected" : ""}" type="button" role="option" data-word="${word}"><span class="completion-kind">◆</span><span>${word}</span><span class="completion-detail">${kind}</span></button>`).join("");
    el.completions.hidden = false;
    el.completions.querySelectorAll("button").forEach((button) => button.addEventListener("mousedown", (event) => { event.preventDefault(); acceptCompletion(button.dataset.word); }));
  }

  function hideCompletions() { el.completions.hidden = true; el.completions.replaceChildren(); }
  function acceptCompletion(word) {
    const prefix = getWordBeforeCaret(); const start = el.editor.selectionStart - prefix.length; const end = el.editor.selectionEnd;
    el.editor.setRangeText(word, start, end, "end"); persistEditor(); hideCompletions();
  }

  function persistEditor() {
    const file = activeFile(); file.content = el.editor.value; file.modified = true; el.saveDot.classList.add("modified"); renderTabs(); compileActive(); updateCursor();
  }

  function insertText(text, selection = "end") {
    el.editor.setRangeText(text, el.editor.selectionStart, el.editor.selectionEnd, selection); persistEditor();
  }

  function insertPaired(open, close) {
    const start = el.editor.selectionStart; const end = el.editor.selectionEnd;
    const selected = el.editor.value.slice(start, end);
    if (!selected && el.editor.value[start] === close) {
      el.editor.selectionStart = el.editor.selectionEnd = start + 1; hideCompletions(); updateCursor(); return;
    }
    el.editor.setRangeText(`${open}${selected}${close}`, start, end, "end");
    el.editor.selectionStart = start + 1;
    el.editor.selectionEnd = start + 1 + selected.length;
    hideCompletions(); persistEditor();
  }

  function insertIndentedLine() {
    const start = el.editor.selectionStart; const before = el.editor.value.slice(0, start); const after = el.editor.value.slice(start);
    const currentLine = before.slice(before.lastIndexOf("\n") + 1); const outerIndent = indentFor(currentLine);
    const opensBlock = currentLine.trim().endsWith("{");
    if (opensBlock && after.startsWith("}")) {
      const innerIndent = `${outerIndent}  `;
      el.editor.setRangeText(`\n${innerIndent}\n${outerIndent}`, start, el.editor.selectionEnd, "end");
      const inside = start + 1 + innerIndent.length;
      el.editor.selectionStart = el.editor.selectionEnd = inside;
    } else {
      el.editor.setRangeText(`\n${outerIndent}${opensBlock ? "  " : ""}`, start, el.editor.selectionEnd, "end");
    }
    hideCompletions(); persistEditor();
  }

  function handleEditorKeydown(event) {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") { event.preventDefault(); runActive(); return; }
    if ((event.ctrlKey || event.metaKey) && event.key === " ") { event.preventDefault(); showCompletions(true); return; }
    if (!el.completions.hidden) {
      const options = completionCandidates(true);
      if (event.key === "ArrowDown" || event.key === "ArrowUp") { event.preventDefault(); completionIndex = (completionIndex + (event.key === "ArrowDown" ? 1 : options.length - 1)) % options.length; showCompletions(true); return; }
      if (event.key === "Tab") { event.preventDefault(); if (options[completionIndex]) acceptCompletion(options[completionIndex][0]); return; }
      if (event.key === "Escape") { event.preventDefault(); hideCompletions(); return; }
      if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) hideCompletions();
      if (event.key === "Enter") hideCompletions();
    }
    if (event.key === "Tab") { event.preventDefault(); insertText("  "); return; }
    if (event.key === "Enter") { event.preventDefault(); insertIndentedLine(); return; }
    if (event.key === "}" && !event.ctrlKey && !event.metaKey && !event.altKey) {
      if (el.editor.value[el.editor.selectionStart] === "}") { event.preventDefault(); el.editor.selectionStart = el.editor.selectionEnd = el.editor.selectionStart + 1; hideCompletions(); updateCursor(); return; }
      const before = el.editor.value.slice(0, el.editor.selectionStart); const currentLine = before.slice(before.lastIndexOf("\n") + 1);
      if (/^\s+$/.test(currentLine) && currentLine.length >= 2) { event.preventDefault(); el.editor.setRangeText("}", el.editor.selectionStart - 2, el.editor.selectionStart, "end"); hideCompletions(); persistEditor(); return; }
    }
    const pairs = { "{": "}", "(": ")", "[": "]", '"': '"' };
    if (pairs[event.key] && !event.ctrlKey && !event.metaKey && !event.altKey) { event.preventDefault(); insertPaired(event.key, pairs[event.key]); }
  }

  function openCreateDialog(kind, parentId = null) {
    createParentId = parentId;
    createKind = kind; const labels = { file: ["NEW FILE", "Create a Joe file", "A .joe extension will be added if it is not included."], folder: ["NEW FOLDER", "Create a folder", "Folders keep Joe files together."], package: ["NEW PACKAGE", "Create a package", "Packages are shown as named project groups."] };
    const [kicker, title, note] = labels[kind]; el.createKicker.textContent = kicker; el.createTitle.textContent = title; el.createNote.textContent = note; el.createName.value = "";
    el.templateChoice.hidden = kind !== "file"; document.querySelector('input[name="file-template"][value="template"]').checked = true;
    el.createDialog.showModal(); setTimeout(() => el.createName.focus(), 0);
  }

  function createItem() {
    const name = normalizeName(el.createName.value, createKind); if (!name) return;
    if (files.some((file) => file.name.toLowerCase() === name.toLowerCase())) { el.createNote.textContent = "That name is already in this workspace."; return; }
    const selected = activeFile(); const parent = createParentId ?? (selected?.parent || null); const id = `${createKind}-${Date.now()}`;
    const node = { id, type: createKind, name, parent, expanded: true };
    if (createKind === "file") {
      const typeName = name.replace(/\.joe$/i, "").replace(/[^A-Za-z0-9]+/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase()).replace(/\s/g, "") || "NewJoe";
      const template = document.querySelector('input[name="file-template"]:checked').value;
      node.content = template === "blank" ? "" : core.templates.basicFile({ typeName }); node.modified = false;
    }
    files.push(node); createParentId = null; el.createDialog.close(); renderTree(); if (node.type === "file") selectFile(node.id);
  }

  function showContextMenu(event) {
    event.preventDefault();
    const row = event.target.closest(".tree-row"); contextTargetId = row?.dataset.id || null;
    const target = fileById(contextTargetId); const deleteItemButton = $("#delete-context-item");
    $("#rename-context-item").hidden = !target; $("#rename-context-item").textContent = target ? `Rename ${target.name}` : "Rename";
    $("#duplicate-context-item").hidden = target?.type !== "file"; $("#move-context-item").hidden = target?.type !== "file";
    deleteItemButton.hidden = !target; deleteItemButton.textContent = target ? `Delete ${target.name}` : "Delete";
    el.context.style.left = `${Math.min(event.clientX, window.innerWidth - 175)}px`; el.context.style.top = `${Math.min(event.clientY, window.innerHeight - 135)}px`; el.context.hidden = false;
  }
  function hideContextMenu() { el.context.hidden = true; }

  const manuals = {
    disclosure: `<h3>Technical disclosure · JoeLang</h3><p>JoeLang is object-facing: a program begins with a named <code>joebject</code>, its state, behaviors, and received messages. The browser projects that source into inspectable JavaScript.</p><ul><li>State belongs to its owning Joebject.</li><li>Messages resolve receiver first, then behavior.</li><li>Structural and unresolved-name findings are reported before execution.</li><li>The JavaScript pane remains visible so projection is inspectable.</li></ul>`,
    manual: `<h3>User manual · quick reference</h3><p>Declare a <code>joebject</code>; use <code>has</code> for owned values and <code>does</code> for behaviors. Create an instance with <code>newjoe Type as name</code>, then send it a message.</p><ul><li><code>show</code> writes output; <code>give</code> returns a value.</li><li><code>when</code> and <code>otherwise</code> choose a path.</li><li><code>each item in items</code> repeats over a list.</li><li><code>truejoe</code>, <code>falsejoe</code>, and <code>nonejoe</code> are values.</li></ul>`
  };

  $("#new-file-button").addEventListener("click", () => openCreateDialog("file"));
  $("#new-folder-button").addEventListener("click", () => openCreateDialog("folder"));
  $("#new-package-button").addEventListener("click", () => openCreateDialog("package"));
  el.explorer.addEventListener("contextmenu", showContextMenu);
  el.context.querySelectorAll("[data-create]").forEach((button) => button.addEventListener("click", () => {
    const target = fileById(contextTargetId); const parentId = target ? (target.type === "file" ? target.parent : target.id) : null;
    hideContextMenu(); openCreateDialog(button.dataset.create, parentId);
  }));
  $("#rename-context-item").addEventListener("click", () => { const targetId = contextTargetId; hideContextMenu(); if (targetId) openFileOperation("rename", targetId); });
  $("#duplicate-context-item").addEventListener("click", () => { const targetId = contextTargetId; hideContextMenu(); if (targetId) openFileOperation("duplicate", targetId); });
  $("#move-context-item").addEventListener("click", () => { const targetId = contextTargetId; hideContextMenu(); if (targetId) openFileOperation("move", targetId); });
  $("#delete-context-item").addEventListener("click", () => { const targetId = contextTargetId; hideContextMenu(); if (targetId) requestDelete(targetId); });
  document.addEventListener("click", (event) => { if (!el.context.contains(event.target)) hideContextMenu(); });
  el.createForm.addEventListener("submit", (event) => { event.preventDefault(); if (event.submitter?.value === "confirm") createItem(); else el.createDialog.close(); });
  el.operationForm.addEventListener("submit", (event) => { event.preventDefault(); if (event.submitter?.value === "confirm") performFileOperation(); else { pendingOperation = null; el.operationDialog.close(); } });
  el.deleteForm.addEventListener("submit", (event) => {
    event.preventDefault(); const choice = event.submitter?.value; const id = pendingDeleteId; pendingDeleteId = null;
    if (choice === "never") { try { localStorage.setItem("joelang-studio.skip-delete-warning", "true"); } catch {} }
    el.deleteDialog.close(); if ((choice === "confirm" || choice === "never") && id) deleteItem(id);
  });
  el.editor.addEventListener("input", () => { persistEditor(); showCompletions(false); });
  el.editor.addEventListener("keydown", handleEditorKeydown);
  el.editor.addEventListener("click", () => { hideCompletions(); updateCursor(); });
  el.editor.addEventListener("keyup", (event) => { updateCursor(); if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) hideCompletions(); });
  el.editor.addEventListener("scroll", () => { el.highlightPre.scrollTop = el.editor.scrollTop; el.highlightPre.scrollLeft = el.editor.scrollLeft; el.lineNumbers.scrollTop = el.editor.scrollTop; hideCompletions(); });
  el.run.addEventListener("click", runActive);
  $("#clear-output").addEventListener("click", () => { runRecord = ""; el.output.textContent = "Waiting for a run…"; el.runState.textContent = "Idle"; el.runState.className = "run-state"; });
  const setProblemsHidden = (hidden) => { el.workspace.classList.toggle("problems-hidden", hidden); $("#toggle-problems").setAttribute("aria-label", hidden ? "Show problems" : "Hide problems"); $("#toggle-problems").title = hidden ? "Show problems" : "Hide problems"; };
  $("#toggle-problems").addEventListener("click", () => setProblemsHidden(!el.workspace.classList.contains("problems-hidden")));
  $("#hide-problems").addEventListener("click", () => setProblemsHidden(true));
  const setInspector = (hidden) => { el.shell.classList.toggle("inspector-hidden", hidden); $("#toggle-inspector").setAttribute("aria-label", hidden ? "Show right sidebar" : "Hide right sidebar"); $("#toggle-inspector").title = hidden ? "Show right sidebar" : "Hide right sidebar"; };
  $("#toggle-inspector").addEventListener("click", () => setInspector(!el.shell.classList.contains("inspector-hidden")));
  $("#sidebar-close").addEventListener("click", () => setInspector(true));
  $("#theme-toggle").addEventListener("click", () => { const light = document.body.dataset.theme === "light"; document.body.dataset.theme = light ? "dark" : "light"; $("#theme-toggle").setAttribute("aria-label", light ? "Switch to light theme" : "Switch to dark theme"); document.querySelector('meta[name="theme-color"]').content = light ? "#1e1e1e" : "#ffffff"; });
  $("#manuals-rail").addEventListener("click", () => { el.manualReader.hidden = true; el.manualDialog.showModal(); });
  $("#manual-close").addEventListener("click", () => el.manualDialog.close());
  document.querySelectorAll("[data-manual]").forEach((button) => button.addEventListener("click", () => { el.manualReader.innerHTML = manuals[button.dataset.manual]; el.manualReader.hidden = false; }));
  $("#shortcuts-button").addEventListener("click", () => el.shortcutDialog.showModal()); $("#shortcut-close").addEventListener("click", () => el.shortcutDialog.close());
  $("#copy-js").addEventListener("click", async () => { try { await navigator.clipboard.writeText(el.js.textContent); $("#copy-js").textContent = "✓"; setTimeout(() => { $("#copy-js").textContent = "⧉"; }, 1000); } catch { $("#copy-js").textContent = "!"; } });

  renderTree(); renderTabs(); el.editor.value = activeFile().content; renderEditor(); updateCursor(); compileActive();
})();
