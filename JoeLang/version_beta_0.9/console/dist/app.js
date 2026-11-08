const starters = {
  greeter: `// Start with a Joebject. Then send it a message.
joebject Greeter {
  has name = "Joe Team"

  does welcome() {
    show "Welcome, " + name + "!"
  }
}

newjoe Greeter as guide
guide.welcome()`,
  lantern: `// State belongs to the Joebject that changes it.
joebject Lantern {
  has charge = 3
  has lit = falsejoe

  does light() {
    when charge > 0 {
      lit = truejoe
      charge = charge - 1
      show "The lantern glows."
    } otherwise {
      show "No charge remains."
    }
    give charge
  }
}

newjoe Lantern as field_lantern
show field_lantern.light()`,
  fieldlog: `// Lists are useful for observations, tasks, and records.
joebject FieldLog {
  has entries = ["Signal received", "Projection verified"]

  does review() {
    each entry in entries {
      show "- " + entry
    }
    give entries
  }
}

newjoe FieldLog as log
log.review()`,
  scoreboard: `joebject Scoreboard {
  has score = 0

  does add(points) {
    score = score + points
    show "Score is now " + score
    give score
  }
}

newjoe Scoreboard as board
show board.add(10)`
};

const sourceEditor = document.querySelector("#source-editor");
const lineNumbers = document.querySelector("#line-numbers");
const projectionOutput = document.querySelector("#projection-output");
const consoleOutput = document.querySelector("#console-output");
const inspectorOutput = document.querySelector("#inspector-output");
const sourceStat = document.querySelector("#source-stat");
const projectionStat = document.querySelector("#projection-stat");
const runStatus = document.querySelector("#run-status");
const starterSelect = document.querySelector("#starter-select");
const autocompleteMenu = document.querySelector("#autocomplete-menu");
const INDENT = "  ";
const core = globalThis.JoeLangCore;
const keywordCatalog = core?.isCompatible(1) ? core.completions() : [];
let completionItems = [];
let completionIndex = 0;

function setStatus(message, state = "ready") {
  runStatus.dataset.state = state;
  runStatus.querySelector("span:last-child").textContent = message;
}

function updateLineNumbers() {
  const count = sourceEditor.value.split("\n").length;
  lineNumbers.innerHTML = Array.from({ length: count }, (_, index) => `<span>${index + 1}</span>`).join("");
  sourceStat.textContent = `${count} ${count === 1 ? "line" : "lines"} · ${sourceEditor.value.length} chars`;
}

function pushConsole(message, isError = false) {
  const hint = consoleOutput.querySelector(".console-hint");
  if (hint) hint.remove();
  const line = document.createElement("p");
  line.className = `console-line${isError ? " console-error" : ""}`;
  line.textContent = message;
  consoleOutput.append(line);
  consoleOutput.scrollTop = consoleOutput.scrollHeight;
}

function clearConsole() {
  consoleOutput.innerHTML = '<p class="console-hint">Console cleared. Run the source to continue.</p>';
}

function currentWordContext() {
  const beforeCaret = sourceEditor.value.slice(0, sourceEditor.selectionStart);
  const match = beforeCaret.match(/[A-Za-z][A-Za-z0-9_]*$/);
  return { word: match ? match[0] : "", start: match ? sourceEditor.selectionStart - match[0].length : sourceEditor.selectionStart };
}

function hideAutocomplete() {
  completionItems = [];
  completionIndex = 0;
  autocompleteMenu.hidden = true;
  autocompleteMenu.innerHTML = "";
  sourceEditor.setAttribute("aria-expanded", "false");
  sourceEditor.removeAttribute("aria-activedescendant");
}

function positionAutocomplete() {
  const beforeCaret = sourceEditor.value.slice(0, sourceEditor.selectionStart);
  const lines = beforeCaret.split("\n");
  const style = window.getComputedStyle(sourceEditor);
  const fontSize = Number.parseFloat(style.fontSize) || 12;
  const lineHeight = Number.parseFloat(style.lineHeight) || fontSize * 1.65;
  const x = Math.max(48, 60 + lines.at(-1).length * fontSize * 0.61 - sourceEditor.scrollLeft);
  const y = Math.max(8, 16 + lines.length * lineHeight - sourceEditor.scrollTop);
  autocompleteMenu.style.left = `${Math.min(x, sourceEditor.clientWidth - 245)}px`;
  autocompleteMenu.style.top = `${Math.min(y, sourceEditor.clientHeight - 62)}px`;
}

function renderAutocomplete() {
  const context = currentWordContext();
  if (!context.word) return hideAutocomplete();
  completionItems = keywordCatalog.filter((item) => item.word.startsWith(context.word.toLowerCase()) && item.word !== context.word.toLowerCase()).slice(0, 7);
  if (!completionItems.length) return hideAutocomplete();
  completionIndex = Math.min(completionIndex, completionItems.length - 1);
  autocompleteMenu.innerHTML = completionItems.map((item, index) => `<button id="completion-${index}" class="autocomplete-option${index === completionIndex ? " is-active" : ""}" type="button" role="option" aria-selected="${index === completionIndex}"><b>${item.word}</b><small class="completion-key">${index === completionIndex ? "Tab" : ""}</small><span>${item.detail}</span></button>`).join("");
  autocompleteMenu.querySelectorAll(".autocomplete-option").forEach((button, index) => button.addEventListener("mousedown", (event) => { event.preventDefault(); applyCompletion(index); }));
  positionAutocomplete();
  autocompleteMenu.hidden = false;
  sourceEditor.setAttribute("aria-expanded", "true");
  sourceEditor.setAttribute("aria-activedescendant", `completion-${completionIndex}`);
}

function applyCompletion(index = completionIndex) {
  const item = completionItems[index];
  if (!item) return;
  const context = currentWordContext();
  sourceEditor.setRangeText(item.word, context.start, sourceEditor.selectionStart, "end");
  updateLineNumbers(); hideAutocomplete(); sourceEditor.focus();
}

function insertText(text, cursorOffset = text.length) {
  const start = sourceEditor.selectionStart;
  sourceEditor.setRangeText(text, start, sourceEditor.selectionEnd, "end");
  sourceEditor.setSelectionRange(start + cursorOffset, start + cursorOffset);
  updateLineNumbers();
}

function autoIndent() {
  const start = sourceEditor.selectionStart;
  const before = sourceEditor.value.slice(0, start);
  const after = sourceEditor.value.slice(sourceEditor.selectionEnd);
  const line = before.slice(before.lastIndexOf("\n") + 1);
  const leading = (line.match(/^\s*/) || [""])[0];
  const insideNewPair = line.trim().endsWith("{") && after.trimStart().startsWith("}");
  if (insideNewPair) insertText(`\n${leading}${INDENT}\n${leading}`, leading.length + INDENT.length + 1);
  else insertText(`\n${leading}${line.trim().endsWith("{") ? INDENT : ""}`);
  hideAutocomplete();
}

function smartClosingBrace() {
  const start = sourceEditor.selectionStart;
  const currentLine = sourceEditor.value.slice(0, start).split("\n").at(-1);
  if (!/^\s+$/.test(currentLine)) return false;
  const dedented = currentLine.slice(0, Math.max(0, currentLine.length - INDENT.length));
  sourceEditor.setRangeText(`${dedented}}`, start - currentLine.length, sourceEditor.selectionEnd, "end");
  updateLineNumbers(); hideAutocomplete(); return true;
}

function renderInspector(instances) {
  const entries = Object.entries(instances || {});
  if (!entries.length) { inspectorOutput.innerHTML = '<p class="empty-inspector">No instances were created.</p>'; return; }
  inspectorOutput.innerHTML = entries.map(([name, instance]) => {
    const stateRows = Object.entries(instance.state).map(([key, value]) => `<b>${key}</b><span>${Array.isArray(value) ? `[${value.join(", ")}]` : String(value)}</span>`).join("");
    const behaviorList = Object.keys(instance.type.methods).join(", ") || "none";
    return `<article class="inspector-card"><div class="inspector-type"><span>${name} : ${instance.type.name}</span><span>LIVE</span></div><div class="inspector-state">${stateRows || "<b>state</b><span>none</span>"}<b>does</b><span>${behaviorList}</span></div></article>`;
  }).join("");
}

function projectAndRun() {
  consoleOutput.innerHTML = "";
  if (!core?.isCompatible(1)) {
    pushConsole("JoeLang language pack is unavailable. Reload the page and try again.", true);
    setStatus("Language pack unavailable", "error");
    return;
  }
  const analysis = core.analyze(sourceEditor.value);
  const compiled = core.compile(sourceEditor.value, { fileName: "console.joe", analysis });
  projectionOutput.textContent = compiled.code;
  projectionStat.textContent = `${compiled.summary.joebjects} Joebject${compiled.summary.joebjects === 1 ? "" : "s"} projected`;
  const problem = analysis.diagnostics.find((item) => item.level === "error");
  if (problem) {
    projectionOutput.textContent = "// Projection paused.\n// Resolve the source finding and run again.\n\n// " + problem.code + ": " + problem.message;
    projectionStat.textContent = "Projection not available";
    inspectorOutput.innerHTML = '<p class="empty-inspector">The inspector appears after a successful projection.</p>';
    pushConsole(`${problem.code}: ${problem.message} (line ${problem.line})`, true);
    setStatus(`Projection finding: ${problem.message}`, "error");
    return;
  }
  try {
    const result = core.run(sourceEditor.value, { analysis });
    result.events.forEach((event) => {
      if (event.type === "create") pushConsole(`Observed ${event.alias} : ${event.joebject}`);
      if (event.type === "output") pushConsole(event.value);
    });
    renderInspector(result.objects);
    const count = Object.keys(result.objects).length;
    setStatus(`Projection successful — ${count} live instance${count === 1 ? "" : "s"} observed.`, "success");
  } catch (error) {
    inspectorOutput.innerHTML = '<p class="empty-inspector">The inspector appears after a successful projection.</p>';
    pushConsole(error.message, true);
    setStatus(`Runtime finding: ${error.message}`, "error");
  }
}

function formatSource() {
  if (!core?.isCompatible(1)) return;
  sourceEditor.value = core.format(sourceEditor.value);
  updateLineNumbers(); hideAutocomplete(); setStatus("Source formatted. Ready to project.", "ready");
}

sourceEditor.value = starters.greeter;
updateLineNumbers();
sourceEditor.addEventListener("input", () => { updateLineNumbers(); renderAutocomplete(); });
sourceEditor.addEventListener("scroll", () => { lineNumbers.scrollTop = sourceEditor.scrollTop; if (!autocompleteMenu.hidden) positionAutocomplete(); });
sourceEditor.addEventListener("click", renderAutocomplete);
sourceEditor.addEventListener("keyup", (event) => { if (!["ArrowUp", "ArrowDown", "Enter", "Tab", "Escape"].includes(event.key)) renderAutocomplete(); });
sourceEditor.addEventListener("blur", () => window.setTimeout(hideAutocomplete, 120));
sourceEditor.addEventListener("keydown", (event) => {
  if (!autocompleteMenu.hidden && event.key === "ArrowDown") { event.preventDefault(); completionIndex = (completionIndex + 1) % completionItems.length; renderAutocomplete(); return; }
  if (!autocompleteMenu.hidden && event.key === "ArrowUp") { event.preventDefault(); completionIndex = (completionIndex - 1 + completionItems.length) % completionItems.length; renderAutocomplete(); return; }
  if (!autocompleteMenu.hidden && event.key === "Tab") { event.preventDefault(); applyCompletion(); return; }
  if (!autocompleteMenu.hidden && event.key === "Escape") { event.preventDefault(); hideAutocomplete(); return; }
  if (!autocompleteMenu.hidden && event.key === "Enter") hideAutocomplete();
  if (event.key === "Enter") { event.preventDefault(); autoIndent(); return; }
  if (event.key === "Tab") { event.preventDefault(); insertText(INDENT); hideAutocomplete(); return; }
  if (event.key === "{" && sourceEditor.selectionStart === sourceEditor.selectionEnd) { event.preventDefault(); insertText("{}", 1); hideAutocomplete(); return; }
  if (event.key === "(" && sourceEditor.selectionStart === sourceEditor.selectionEnd) { event.preventDefault(); insertText("()", 1); hideAutocomplete(); return; }
  if (event.key === "}" && smartClosingBrace()) { event.preventDefault(); return; }
});
document.querySelector("#run-button").addEventListener("click", projectAndRun);
document.querySelector("#format-button").addEventListener("click", formatSource);
document.querySelector("#clear-button").addEventListener("click", clearConsole);
starterSelect.addEventListener("change", (event) => { sourceEditor.value = starters[event.target.value]; updateLineNumbers(); setStatus(`Loaded ${event.target.options[event.target.selectedIndex].text} starter.`, "ready"); sourceEditor.focus(); });
document.querySelectorAll("[data-insert]").forEach((button) => button.addEventListener("click", () => { insertText(button.dataset.insert); sourceEditor.focus(); }));
projectAndRun();
