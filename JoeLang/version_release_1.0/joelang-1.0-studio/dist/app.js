(() => {
  "use strict";

  const core = globalThis.JoeLangCore;
  if (!core || !core.isCompatible(2)) {
    document.body.innerHTML = '<main style="padding:28px;background:#171a21;color:#ff9ba4;font:14px ui-monospace,monospace">JoeLang Studio could not load its 1.0 language pack. Refresh the page or restore <code>pack/joelang-core.js</code>.</main>';
    return;
  }

  const $ = (selector) => document.querySelector(selector);
  const el = {
    studio: $(".studio"), workbench: $(".workbench"), tree: $("#file-tree"), tabs: $("#editor-tabs"), editor: $("#code-editor"),
    numbers: $("#line-numbers"), syntax: $("#syntax-highlight code"), syntaxPre: $("#syntax-highlight"), crumb: $("#crumb-path"), completion: $("#completion-menu"),
    compile: $("#compile-state"), problems: $("#problems-list"), problemCount: $("#problem-count"), problemsPanel: $(".problems-panel"), runtimeStatus: $("#runtime-status"),
    cursor: $("#cursor-status"), activeStatus: $("#active-file-status"), output: $("#program-output"), objects: $("#object-tree"), projection: $("#js-projection"), inspector: $("#inspector"),
    context: $("#context-menu"), workspaceInput: $("#workspace-input"), sourceInput: $("#source-input"), itemDialog: $("#item-dialog"), itemForm: $("#item-form"), itemTitle: $("#item-dialog-title"),
    itemKind: $("#item-kind"), itemName: $("#item-name"), itemParent: $("#item-parent"), templateLabel: $("#template-label"), templateChoice: $("#template-choice"), itemNote: $("#item-note"),
    renameDialog: $("#rename-dialog"), renameForm: $("#rename-form"), renameTitle: $("#rename-title"), renameName: $("#rename-name"), renameNote: $("#rename-note"),
    deleteDialog: $("#delete-dialog"), deleteForm: $("#delete-form"), deleteTitle: $("#delete-title"), deleteWarning: $("#delete-warning"), skipDelete: $("#skip-delete-warning"),
    inputDialog: $("#input-dialog"), inputForm: $("#input-form"), inputPrompt: $("#input-prompt"), inputType: $("#input-type"), inputValue: $("#input-value"), inputError: $("#input-error")
  };
  const STORAGE = "joelang-studio-1.0-workspace";
  const THEME = "joelang-studio-1.0-theme";
  const FILE_TYPES = new Set(["file", "folder", "package"]);
  const keywordWords = core.completions().map((item) => item.word);
  const keywordPattern = keywordWords.map((word) => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|");
  const editorFontLine = 20.54;
  let activeId = "main";
  let openIds = ["main"];
  let pendingDelete = null;
  let pendingRename = null;
  let completionItems = [];
  let completionIndex = 0;
  let inputResolve = null;
  let saveTimer = 0;

  const starter = `jinport "std/joemath" as math

joebject Greeter {
  has strjin title = "JoeLang 1.0"

  does greet(strjin person) {
    show title + " welcomes " + person
    give math.max(1, 3)
  }
}

newjoe Greeter as greeter
show greeter.greet("Joe")`;
  let files = [
    { id: "src", type: "folder", name: "src", parent: null, expanded: true },
    { id: "main", type: "file", name: "main.joe", parent: "src", content: starter, modified: false },
    { id: "examples", type: "folder", name: "examples", parent: null, expanded: true },
    { id: "input-demo", type: "file", name: "input_demo.joe", parent: "examples", content: `has strjin name = input("What is your name?")\nshow "Hello, " + name`, modified: false }
  ];

  const escapeHTML = (value) => String(value).replace(/[&<>"']/g, (character) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#039;" })[character]);
  const fileById = (id) => files.find((item) => item.id === id);
  const activeFile = () => fileById(activeId);
  const id = (prefix) => `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 7)}`;
  const isContainer = (item) => item && (item.type === "folder" || item.type === "package");
  const childrenOf = (parent) => files.filter((item) => (item.parent || null) === (parent || null)).sort((a, b) => Number(b.type === "file") - Number(a.type === "file") || a.name.localeCompare(b.name));
  const descendants = (itemId, result = new Set()) => { result.add(itemId); childrenOf(itemId).forEach((item) => descendants(item.id, result)); return result; };
  const pathFor = (item) => { const parts = [item.name]; let cursor = item; while (cursor.parent) { cursor = fileById(cursor.parent); if (!cursor) break; parts.unshift(cursor.name); } return parts.join(" › "); };

  function restoreWorkspace() {
    try {
      const saved = JSON.parse(localStorage.getItem(STORAGE));
      if (!saved || saved.version !== 1 || !Array.isArray(saved.files)) return;
      const valid = saved.files.filter((item) => item && typeof item.id === "string" && FILE_TYPES.has(item.type) && typeof item.name === "string").map((item) => ({ ...item, content: typeof item.content === "string" ? item.content : "", modified: false, expanded: item.expanded !== false }));
      if (!valid.some((item) => item.type === "file")) return;
      const knownIds = new Set(valid.map((item) => item.id));
      valid.forEach((item) => { if (item.parent && !knownIds.has(item.parent)) item.parent = null; });
      files = valid; activeId = valid.some((item) => item.id === saved.activeId && item.type === "file") ? saved.activeId : valid.find((item) => item.type === "file").id;
      openIds = (saved.openIds || []).filter((item) => valid.some((file) => file.id === item && file.type === "file")); if (!openIds.length) openIds = [activeId];
    } catch { /* A corrupt saved workspace never blocks the starter workspace. */ }
  }
  function persistSoon() { clearTimeout(saveTimer); saveTimer = setTimeout(() => { try { localStorage.setItem(STORAGE, JSON.stringify({ version: 1, activeId, openIds, files: files.map(({ modified, ...item }) => item) })); } catch { /* Browser privacy settings can disable local storage. */ } }, 200); }
  function setTheme(theme) { el.studio.dataset.theme = theme; $("#theme-button").textContent = theme === "dark" ? "☾" : "☀"; try { localStorage.setItem(THEME, theme); } catch {} }

  function renderTree() {
    const fragment = document.createDocumentFragment();
    const draw = (parent, depth) => childrenOf(parent).forEach((item) => {
      const row = document.createElement("div"); row.className = `tree-row${item.id === activeId ? " active" : ""}`; row.dataset.id = item.id; row.dataset.depth = String(Math.min(depth, 3)); row.setAttribute("role", "treeitem");
      const chevron = isContainer(item) ? (item.expanded === false ? "›" : "⌄") : "";
      const icon = item.type === "folder" ? "▰" : item.type === "package" ? "⬡" : "◇";
      row.innerHTML = `<span class="tree-chevron">${chevron}</span><span class="file-icon ${item.type}">${icon}</span><span class="tree-name">${escapeHTML(item.name)}</span>`;
      row.addEventListener("click", () => { hideContext(); if (item.type === "file") selectFile(item.id); else { item.expanded = item.expanded === false; renderTree(); persistSoon(); } });
      row.addEventListener("contextmenu", (event) => { event.preventDefault(); showContext(event.clientX, event.clientY, item.id); });
      fragment.append(row); if (isContainer(item) && item.expanded !== false) draw(item.id, depth + 1);
    });
    draw(null, 0);
    // A transferred workspace may have an older root marker. Surface its files instead of hiding a usable project.
    if (!fragment.childNodes.length && files.length) { files.forEach((item) => { item.parent = null; }); draw(null, 0); }
    el.tree.replaceChildren(fragment);
  }
  function renderTabs() {
    el.tabs.replaceChildren(...openIds.map((itemId) => {
      const item = fileById(itemId); if (!item) return null;
      const button = document.createElement("button"); button.type = "button"; button.className = `editor-tab${itemId === activeId ? " active" : ""}`;
      button.innerHTML = `<span>◇</span><span>${escapeHTML(item.name)}</span>${item.modified ? '<span class="dirty">●</span>' : ""}`;
      button.addEventListener("click", () => selectFile(itemId)); return button;
    }).filter(Boolean));
  }
  function selectFile(itemId) {
    const file = fileById(itemId); if (!file || file.type !== "file") return;
    activeId = itemId; if (!openIds.includes(itemId)) openIds.push(itemId);
    el.editor.value = file.content; el.editor.selectionStart = el.editor.selectionEnd = 0; el.editor.scrollTop = 0; el.editor.scrollLeft = 0;
    renderTree(); renderTabs(); renderEditor(); compileActive(); updateCursor(); persistSoon(); el.editor.focus();
  }
  function renderEditor() { const file = activeFile(); if (!file) return; el.crumb.textContent = pathFor(file); el.activeStatus.textContent = file.name; el.syntax.innerHTML = highlight(file.content); el.numbers.textContent = Array.from({ length: Math.max(1, file.content.split("\n").length) }, (_, index) => index + 1).join("\n"); syncScroll(); }
  function highlight(source) {
    const token = new RegExp(`\\/\\/.*$|"(?:\\\\.|[^"\\\\])*"|\\b(?:${keywordPattern})\\b|\\b\\d+(?:\\.\\d+)?J?\\b|\\b[A-Z][A-Za-z0-9_]*\\b|\\b[a-z_][A-Za-z0-9_]*(?=\\s*\\()|[{}()[\\],.=+\\-*/!<>]+`, "g");
    return String(source).split("\n").map((line) => { let cursor = 0; let html = ""; line.replace(token, (match, offset) => { html += escapeHTML(line.slice(cursor, offset)); let className = "syn-name"; if (match.startsWith("//")) className = "syn-comment"; else if (match.startsWith('"')) className = "syn-string"; else if (/^\d/.test(match)) className = "syn-number"; else if (/^(jint|joeble|joelean|jong|strjin)$/.test(match)) className = "syn-type"; else if (keywordWords.includes(match)) className = "syn-keyword"; else if (/^[{}()[\],.=+\-*/!<>]+$/.test(match)) className = "syn-operator"; else if (/^[a-z_]/.test(match)) className = "syn-function"; html += `<span class="${className}">${escapeHTML(match)}</span>`; cursor = offset + match.length; return match; }); return html + escapeHTML(line.slice(cursor)); }).join("\n");
  }
  function syncScroll() { el.syntaxPre.scrollTop = el.editor.scrollTop; el.syntaxPre.scrollLeft = el.editor.scrollLeft; el.numbers.scrollTop = el.editor.scrollTop; }
  function diagnosticsList(items) {
    el.problemCount.textContent = String(items.length);
    if (!items.length) { el.problems.innerHTML = '<span class="quiet">No problems in this file.</span>'; return; }
    el.problems.replaceChildren(...items.map((item) => { const row = document.createElement("div"); row.className = "problem"; row.innerHTML = `<code>${escapeHTML(item.code || "JOE")}</code><span>${escapeHTML(item.message)}</span><span class="problem-loc">Ln ${item.line || 1}, Col ${item.column || 1}</span>`; row.title = item.suggestion || ""; row.addEventListener("click", () => jumpTo(item.line, item.column)); return row; }));
  }
  function compileActive() {
    const file = activeFile(); if (!file) return; const result = core.compile(file.content, { fileName: file.name }); const errors = result.diagnostics.filter((item) => item.level === "error");
    el.compile.textContent = errors.length ? `${errors.length} error${errors.length === 1 ? "" : "s"}` : "Compiled"; el.compile.style.color = errors.length ? "var(--red)" : "var(--green)"; diagnosticsList(result.diagnostics); el.projection.textContent = result.code; return result;
  }
  function jumpTo(line, column) { const lines = el.editor.value.split("\n"); let position = 0; for (let index = 0; index < Math.max(0, line - 1); index += 1) position += lines[index].length + 1; position += Math.max(0, column - 1); el.editor.focus(); el.editor.selectionStart = el.editor.selectionEnd = position; updateCursor(); el.editor.scrollTop = Math.max(0, (line - 3) * editorFontLine); }
  function updateCursor() { const before = el.editor.value.slice(0, el.editor.selectionStart); const line = before.split("\n").length; const column = before.length - before.lastIndexOf("\n"); el.cursor.textContent = `Ln ${line}, Col ${column}`; }

  function currentWord() { const position = el.editor.selectionStart; if (position !== el.editor.selectionEnd) return null; const prior = el.editor.value.slice(0, position); const match = /[A-Za-z_][A-Za-z0-9_]*$/.exec(prior); return match ? { word: match[0], start: position - match[0].length, end: position } : null; }
  function hideCompletions() { completionItems = []; completionIndex = 0; el.completion.hidden = true; el.completion.replaceChildren(); }
  function updateCompletions() {
    if (document.activeElement !== el.editor) return hideCompletions(); const word = currentWord(); if (!word || !word.word) return hideCompletions(); completionItems = core.completions(word.word).filter((item) => item.word !== word.word); if (!completionItems.length) return hideCompletions(); completionIndex = Math.min(completionIndex, completionItems.length - 1);
    el.completion.replaceChildren(...completionItems.map((item, index) => { const button = document.createElement("button"); button.type = "button"; button.className = `completion-item${index === completionIndex ? " active" : ""}`; button.innerHTML = `<b>${item.word}</b><small>${item.detail}</small>`; button.addEventListener("mousedown", (event) => { event.preventDefault(); complete(item); }); return button; }));
    const prior = el.editor.value.slice(0, el.editor.selectionStart); const lines = prior.split("\n"); el.completion.style.top = `${Math.min(el.editor.clientHeight - 48, 14 + lines.length * editorFontLine - el.editor.scrollTop)}px`; el.completion.style.left = `${Math.min(el.editor.clientWidth - 278, 62 + (lines[lines.length - 1].length * 7.8) - el.editor.scrollLeft)}px`; el.completion.hidden = false;
  }
  function complete(item) { const word = currentWord(); if (!word) return hideCompletions(); replaceRange(word.start, word.end, item.word); hideCompletions(); }
  function replaceRange(start, end, text, caret = start + text.length) { el.editor.setRangeText(text, start, end, "end"); el.editor.selectionStart = el.editor.selectionEnd = caret; handleEditorChange(); }
  function handleEditorChange() { const file = activeFile(); if (!file) return; file.content = el.editor.value; file.modified = true; renderEditor(); renderTabs(); compileActive(); updateCursor(); updateCompletions(); persistSoon(); }
  function insertPair(open, close) { const start = el.editor.selectionStart, end = el.editor.selectionEnd, selected = el.editor.value.slice(start, end); replaceRange(start, end, open + selected + close, start + open.length + selected.length); }
  function handleEnter() {
    const start = el.editor.selectionStart, end = el.editor.selectionEnd, text = el.editor.value; const before = text.slice(0, start), after = text.slice(end); const lineStart = before.lastIndexOf("\n") + 1; const lineText = before.slice(lineStart); const indent = (/^\s*/.exec(lineText) || [""])[0];
    if (/\{\s*$/.test(before) && /^\s*}/.test(after)) replaceRange(start, end, `\n${indent}  \n${indent}`, start + indent.length + 3);
    else { const extra = /\{\s*$/.test(lineText) ? "  " : ""; replaceRange(start, end, `\n${indent}${extra}`, start + 1 + indent.length + extra.length); }
  }
  function indentSelection() { const start = el.editor.selectionStart, end = el.editor.selectionEnd; const selected = el.editor.value.slice(start, end); if (selected.includes("\n")) { const replacement = selected.split("\n").map((line) => `  ${line}`).join("\n"); replaceRange(start, end, replacement, start + replacement.length); } else replaceRange(start, end, "  "); }

  function parentOptions(selected) { const options = [{ id: "", label: "Workspace root" }]; files.filter(isContainer).forEach((item) => options.push({ id: item.id, label: `${item.type === "package" ? "Package" : "Folder"}: ${pathFor(item)}` })); el.itemParent.replaceChildren(...options.map((option) => new Option(option.label, option.id))); el.itemParent.value = selected || ""; }
  function templateFor(name) { const clean = name.replace(/\.joe$/i, "").replace(/[^A-Za-z0-9_]/g, ""); const type = clean ? clean[0].toUpperCase() + clean.slice(1) : "Main"; return `joebject ${type} {\n  has strjin message = "Hello from JoeLang 1.0"\n\n  does run() {\n    show message\n  }\n}\n\nnewjoe ${type} as app\napp.run()`; }
  function normalizeName(name, type) { const clean = String(name || "").trim().replace(/[\\/:*?"<>|]/g, ""); return type === "file" && clean && !/\.joe$/i.test(clean) ? `${clean}.joe` : clean; }
  function siblingTaken(name, parent, except) { return files.some((item) => item.id !== except && item.parent === parent && item.name.toLowerCase() === name.toLowerCase()); }
  function openItemDialog(parent) { const active = fileById(parent); const destination = isContainer(active) ? active.id : active && active.parent || null; el.itemKind.value = "file"; el.itemName.value = "main.joe"; el.templateChoice.checked = true; parentOptions(destination); updateItemDialog(); el.itemDialog.showModal(); setTimeout(() => el.itemName.select(), 0); }
  function updateItemDialog() { const kind = el.itemKind.value; const noun = kind === "file" ? "JoeLang file" : kind; el.itemTitle.textContent = `New ${noun}`; el.templateLabel.hidden = kind !== "file"; el.itemName.placeholder = kind === "file" ? "main.joe" : kind === "folder" ? "src" : "utilities"; el.itemNote.textContent = kind === "file" ? "A .joe file can start blank or with a runnable 1.0 template." : `Create a ${kind} in the selected location.`; }
  function createItem() { const type = el.itemKind.value; const parent = el.itemParent.value || null; const name = normalizeName(el.itemName.value, type); if (!name) { el.itemNote.textContent = "A name is required."; return; } if (siblingTaken(name, parent)) { el.itemNote.textContent = "That name is already used in this location."; return; } const item = { id: id(type), type, name, parent, expanded: true, modified: false }; if (type === "file") item.content = el.templateChoice.checked ? templateFor(name) : ""; files.push(item); if (parent) { const container = fileById(parent); if (container) container.expanded = true; } el.itemDialog.close(); renderTree(); if (type === "file") selectFile(item.id); else persistSoon(); }
  function requestDelete(itemId) { const item = fileById(itemId); if (!item) return; if (localStorage.getItem("joelang-studio-1.0-skip-delete") === "true") return deleteItem(itemId); pendingDelete = itemId; const count = descendants(itemId).size - 1; el.deleteTitle.textContent = `Delete ${item.name}?`; el.deleteWarning.textContent = count ? `This cannot be undone. ${count} nested item${count === 1 ? "" : "s"} will be permanently deleted as well.` : "This cannot be undone. The selected item will be permanently deleted."; el.skipDelete.checked = false; el.deleteDialog.showModal(); }
  function deleteItem(itemId) { const gone = descendants(itemId); files = files.filter((item) => !gone.has(item.id)); openIds = openIds.filter((entry) => !gone.has(entry)); if (gone.has(activeId)) { const next = files.find((item) => item.type === "file"); if (next) activeId = next.id; else { const replacement = { id: id("file"), type: "file", name: "untitled.joe", parent: null, content: "", modified: false }; files.push(replacement); activeId = replacement.id; } } if (!openIds.includes(activeId)) openIds.push(activeId); pendingDelete = null; selectFile(activeId); persistSoon(); }
  function requestRename(itemId) { const item = fileById(itemId); if (!item) return; pendingRename = itemId; el.renameTitle.textContent = `Rename ${item.name}`; el.renameName.value = item.name; el.renameNote.textContent = ""; el.renameDialog.showModal(); setTimeout(() => el.renameName.select(), 0); }
  function renameItem() { const item = fileById(pendingRename); if (!item) return; const name = normalizeName(el.renameName.value, item.type); if (!name) { el.renameNote.textContent = "A name is required."; return; } if (siblingTaken(name, item.parent, item.id)) { el.renameNote.textContent = "That name is already used in this location."; return; } item.name = name; pendingRename = null; el.renameDialog.close(); renderTree(); renderTabs(); renderEditor(); persistSoon(); }
  function duplicateFile(itemId) { const item = fileById(itemId); if (!item || item.type !== "file") return; const base = item.name.replace(/\.joe$/i, ""); let number = 2, name = `${base}_copy.joe`; while (siblingTaken(name, item.parent)) name = `${base}_copy_${number++}.joe`; const copy = { ...item, id: id("file"), name, content: item.content, modified: false }; files.push(copy); selectFile(copy.id); }
  function moveToRoot(itemId) { const item = fileById(itemId); if (!item || !item.parent) return; item.parent = null; renderTree(); renderTabs(); renderEditor(); persistSoon(); }
  function showContext(x, y, itemId) { const item = fileById(itemId); if (!item) return; const actions = []; if (isContainer(item)) actions.push(["New file here", () => openItemDialog(item.id)], ["New folder here", () => { openItemDialog(item.id); el.itemKind.value = "folder"; updateItemDialog(); }]); if (item.type === "file") actions.push(["Duplicate file", () => duplicateFile(item.id)]); actions.push(["Rename", () => requestRename(item.id)]); if (item.parent) actions.push(["Move to workspace root", () => moveToRoot(item.id)]); actions.push(["Delete…", () => requestDelete(item.id), "danger-option"]); el.context.replaceChildren(...actions.map(([label, action, className]) => { const button = document.createElement("button"); button.type = "button"; button.textContent = label; if (className) button.className = className; button.addEventListener("click", () => { hideContext(); action(); }); return button; })); el.context.style.left = `${Math.min(x, window.innerWidth - 190)}px`; el.context.style.top = `${Math.min(y, window.innerHeight - (actions.length * 34 + 8))}px`; el.context.hidden = false; }
  function hideContext() { el.context.hidden = true; }

  function download(name, text, type) { const blob = new Blob([text], { type: type || "text/plain;charset=utf-8" }); const link = document.createElement("a"); link.href = URL.createObjectURL(blob); link.download = name; document.body.append(link); link.click(); setTimeout(() => { URL.revokeObjectURL(link.href); link.remove(); }, 0); }
  function exportWorkspace() { const payload = { format: "JoeLang Studio workspace", version: 1, languageVersion: core.version, savedAt: new Date().toISOString(), activeId, openIds, files: files.map(({ modified, ...item }) => item) }; download("joelang-workspace.joeworkspace.json", JSON.stringify(payload, null, 2), "application/json"); }
  async function importWorkspace(file) { try { const parsed = JSON.parse(await file.text()); if (!parsed || parsed.version !== 1 || !Array.isArray(parsed.files)) throw new Error("This is not a JoeLang Studio 1.0 workspace."); const valid = parsed.files.filter((item) => item && FILE_TYPES.has(item.type) && typeof item.id === "string" && typeof item.name === "string").map((item) => ({ ...item, content: typeof item.content === "string" ? item.content : "", modified: false, expanded: item.expanded !== false })); if (!valid.some((item) => item.type === "file")) throw new Error("The workspace has no source files."); const knownIds = new Set(valid.map((item) => item.id)); valid.forEach((item) => { if (item.parent && !knownIds.has(item.parent)) item.parent = null; }); files = valid; activeId = valid.some((item) => item.id === parsed.activeId && item.type === "file") ? parsed.activeId : valid.find((item) => item.type === "file").id; openIds = (parsed.openIds || []).filter((item) => valid.some((fileItem) => item === fileItem.id && fileItem.type === "file")); if (!openIds.length) openIds = [activeId]; selectFile(activeId); } catch (error) { alert(`Could not open workspace: ${error.message}`); } }
  async function importSource(filesToImport) { for (const file of filesToImport) { const raw = await file.text(); let name = normalizeName(file.name || "imported.joe", "file"); let suffix = 2; const base = name.replace(/\.joe$/i, ""); while (siblingTaken(name, null)) name = `${base}_${suffix++}.joe`; files.push({ id: id("file"), type: "file", name, parent: null, content: raw, modified: false }); } const newest = files[files.length - 1]; renderTree(); if (newest && newest.type === "file") selectFile(newest.id); }
  function clearRuntime() { el.output.textContent = "Run a JoeLang file to see output."; el.objects.textContent = "No Joebjects created."; el.runtimeStatus.textContent = "Ready"; }
  async function runActive() {
    const file = activeFile(); if (!file) return; const compiled = compileActive(); if (!compiled.ok) { el.runtimeStatus.textContent = "Fix compile errors before running"; return; }
    el.output.textContent = ""; el.objects.textContent = ""; el.runtimeStatus.textContent = "Running…"; $("#run-button").disabled = true;
    const result = await core.run(file.content, { fileName: file.name, onOutput: ({ text }) => { el.output.textContent += `${text}\n`; }, onInputError: ({ message }) => { el.inputError.textContent = message; }, inputProvider: ({ prompt, expectedType }) => new Promise((resolve) => { inputResolve = resolve; el.inputPrompt.textContent = prompt; el.inputType.textContent = `Expected ${expectedType}. Enter a valid JoeLang value.`; el.inputValue.value = ""; el.inputError.textContent = ""; el.inputDialog.showModal(); setTimeout(() => el.inputValue.focus(), 0); }) });
    $("#run-button").disabled = false; if (el.inputDialog.open) el.inputDialog.close(); inputResolve = null;
    if (!result.ok) { el.runtimeStatus.textContent = "Stopped with runtime error"; diagnosticsList(result.diagnostics); if (!el.output.textContent) el.output.textContent = "Runtime stopped before producing output."; }
    else { el.runtimeStatus.textContent = "Completed"; if (!el.output.textContent) el.output.textContent = "Program completed with no output."; diagnosticsList([]); }
    el.objects.textContent = Object.keys(result.objects).length ? JSON.stringify(result.objects, null, 2) : "No Joebjects created.";
  }

  el.editor.addEventListener("input", handleEditorChange);
  el.editor.addEventListener("scroll", syncScroll);
  el.editor.addEventListener("click", () => { updateCursor(); updateCompletions(); });
  el.editor.addEventListener("keyup", (event) => { if (!["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight", "Escape", "Enter"].includes(event.key)) { updateCursor(); updateCompletions(); } });
  el.editor.addEventListener("blur", () => setTimeout(hideCompletions, 120));
  el.editor.addEventListener("keydown", (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") { event.preventDefault(); hideCompletions(); runActive(); return; }
    if ((event.ctrlKey || event.metaKey) && event.shiftKey && event.key.toLowerCase() === "f") { event.preventDefault(); formatActive(); return; }
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "s") { event.preventDefault(); exportWorkspace(); return; }
    if (!el.completion.hidden && event.key === "ArrowDown") { event.preventDefault(); completionIndex = (completionIndex + 1) % completionItems.length; updateCompletions(); return; }
    if (!el.completion.hidden && event.key === "ArrowUp") { event.preventDefault(); completionIndex = (completionIndex + completionItems.length - 1) % completionItems.length; updateCompletions(); return; }
    if (!el.completion.hidden && event.key === "Tab") { event.preventDefault(); complete(completionItems[completionIndex]); return; }
    if (event.key === "Escape") { hideCompletions(); return; }
    if (event.key === "Enter") { hideCompletions(); event.preventDefault(); handleEnter(); return; }
    if (event.key === "Tab") { event.preventDefault(); indentSelection(); return; }
    if (["(", "[", "{"].includes(event.key) && !event.ctrlKey && !event.metaKey) { event.preventDefault(); insertPair(event.key, ({ "(": ")", "[": "]", "{": "}" })[event.key]); return; }
    if (event.key === "}" && el.editor.value.slice(el.editor.selectionStart, el.editor.selectionStart + 1) === "}") { event.preventDefault(); el.editor.selectionStart += 1; el.editor.selectionEnd = el.editor.selectionStart; updateCursor(); return; }
    if (event.key === '"' && !event.ctrlKey && !event.metaKey && el.editor.value.slice(el.editor.selectionStart, el.editor.selectionStart + 1) !== '"') { event.preventDefault(); insertPair('"', '"'); return; }
    if (["ArrowLeft", "ArrowRight", "Home", "End", "PageUp", "PageDown"].includes(event.key)) hideCompletions();
  });
  function formatActive() { const file = activeFile(); if (!file) return; file.content = core.format(file.content); file.modified = true; el.editor.value = file.content; renderEditor(); renderTabs(); compileActive(); persistSoon(); }

  $("#run-button").addEventListener("click", runActive); $("#format-button").addEventListener("click", formatActive); $("#clear-output-button").addEventListener("click", clearRuntime);
  $("#new-item-button").addEventListener("click", () => openItemDialog(activeFile() && activeFile().parent)); $("#collapse-tree-button").addEventListener("click", () => { files.filter(isContainer).forEach((item) => { item.expanded = false; }); renderTree(); persistSoon(); });
  $("#theme-button").addEventListener("click", () => setTheme(el.studio.dataset.theme === "dark" ? "light" : "dark")); $("#right-sidebar-button").addEventListener("click", () => { el.inspector.classList.toggle("is-hidden"); el.workbench.classList.toggle("inspector-hidden"); });
  $("#hide-problems-button").addEventListener("click", () => { el.problemsPanel.classList.add("is-hidden"); });
  $("#open-workspace-button").addEventListener("click", () => el.workspaceInput.click()); $("#save-workspace-button").addEventListener("click", exportWorkspace); $("#import-file-button").addEventListener("click", () => el.sourceInput.click()); $("#download-file-button").addEventListener("click", () => { const file = activeFile(); if (file) download(file.name, file.content); });
  el.workspaceInput.addEventListener("change", () => { const file = el.workspaceInput.files[0]; if (file) importWorkspace(file); el.workspaceInput.value = ""; }); el.sourceInput.addEventListener("change", () => { if (el.sourceInput.files.length) importSource(el.sourceInput.files); el.sourceInput.value = ""; });
  el.itemKind.addEventListener("change", updateItemDialog); el.itemForm.addEventListener("submit", (event) => { event.preventDefault(); createItem(); }); el.renameForm.addEventListener("submit", (event) => { event.preventDefault(); renameItem(); }); el.deleteForm.addEventListener("submit", (event) => { event.preventDefault(); if (el.skipDelete.checked) localStorage.setItem("joelang-studio-1.0-skip-delete", "true"); if (pendingDelete) deleteItem(pendingDelete); el.deleteDialog.close(); });
  el.inputForm.addEventListener("submit", (event) => { event.preventDefault(); if (inputResolve) { const resolve = inputResolve; inputResolve = null; el.inputDialog.close(); resolve(el.inputValue.value); } }); $("#cancel-input-button").addEventListener("click", () => { if (inputResolve) { const resolve = inputResolve; inputResolve = null; el.inputDialog.close(); resolve(null); } });
  document.querySelectorAll("[data-close]").forEach((button) => button.addEventListener("click", () => $("#" + button.dataset.close).close())); document.addEventListener("click", (event) => { if (!el.context.hidden && !el.context.contains(event.target)) hideContext(); });
  document.querySelectorAll(".activity").forEach((button) => button.addEventListener("click", () => { document.querySelectorAll(".activity").forEach((item) => item.classList.toggle("active", item === button)); if (button.dataset.view === "docs") $("#docs-dialog").showModal(); }));

  restoreWorkspace(); setTheme((() => { try { return localStorage.getItem(THEME) || "dark"; } catch { return "dark"; } })()); selectFile(activeId);
})();
