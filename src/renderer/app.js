const state = {
  /** Active locale strings loaded from a JSON file. */
  t: {},
  /** Persisted user settings (token, delay, lang, historyPath). */
  settings: {
    token: "",
    delay: 4,
    lang: "en",
    historyPath: "",
    batchMode: {
      enabled: false,
      minBatch: 400,
      maxBatch: 600,
      minBreakMin: 10,
      maxBreakMin: 120,
    },
  },
  /** Parsed Discord data pack returned by main.js parsePack(). */
  packData: null,
  /** Flat list of messages after applying the current filter. */
  filteredMessages: [],
  /** IDs of messages the user has checked for deletion. */
  selectedIds: new Set(),
  /**
   * Deletion history — persisted to JSON so sessions can be resumed.
   * deletedIds: Set of message IDs already successfully deleted.
   * historyPath: absolute path to the active history file (mirrors settings.historyPath).
   */
  history: {
    deletedIds: new Set(),
    historyPath: "",
  },
  /** Counter used to trigger autosave every AUTO_SAVE_INTERVAL messages. */
  _autoSaveCounter: 0,
};

/** Autosave frequency — write history file every N successful deletions. */
const AUTO_SAVE_INTERVAL = 20;

// Bootstrap
document.addEventListener("DOMContentLoaded", async () => {
  await loadSettings();
  await applyLocale(state.settings.lang || "en");
  await populateLangSelector();
  setupNav();
  setupTitleBar();
  setupPackPage();
  setupSettingsPage();
  setupReviewPage();
  setupProgressPage();
  setupDashboard();
});

/**
 * Load translation strings for the given locale code and re-render all i18n nodes.
 * @param {string} lang - Locale code, e.g. "en", "pl"
 */
async function applyLocale(lang) {
  state.t = await window.api.loadLocale(lang);
  renderI18n();
}

/** Walk the DOM and replace text / attributes for every element that carries a data-i18n* attribute. */
function renderI18n() {
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const v = t(el.dataset.i18n);
    if (v) el.textContent = v;
  });
  document.querySelectorAll("[data-i18n-html]").forEach((el) => {
    const v = t(el.dataset.i18nHtml);
    if (v) el.innerHTML = v;
  });
  document.querySelectorAll("[data-i18n-placeholder]").forEach((el) => {
    const v = t(el.dataset.i18nPlaceholder);
    if (v) el.placeholder = v;
  });
  document.querySelectorAll("[data-i18n-title]").forEach((el) => {
    const v = t(el.dataset.i18nTitle);
    if (v) el.title = v;
  });
}

/**
 * Resolve a dot-separated translation key against the active locale strings.
 * @param   {string} key - e.g. "progress.logSuccess"
 * @returns {string}
 */
function t(key) {
  const parts = key.split(".");
  let val = state.t;
  for (const p of parts) {
    if (val == null) return "";
    val = val[p];
  }
  return typeof val === "string" ? val : "";
}

/**
 * Interpolate named placeholders in a translation string.
 * @param   {string} key    - Translation key
 * @param   {object} params - Map of {placeholder: value}
 * @returns {string}
 */
function fmt(key, params = {}) {
  let str = t(key);
  for (const [k, v] of Object.entries(params)) {
    str = str.replaceAll(`{${k}}`, v);
  }
  return str;
}

/**
 * Populate the language <select> with all available locales.
 * Marks the currently active locale as selected.
 */
async function populateLangSelector() {
  const locales = await window.api.listLocales();
  const sel = document.getElementById("set-lang");
  sel.innerHTML = "";
  for (const loc of locales) {
    const opt = document.createElement("option");
    opt.value = loc.code;
    opt.textContent = loc.name || loc.code;
    if (loc.code === state.settings.lang) opt.selected = true;
    sel.appendChild(opt);
  }
}

// Navigation

/** Wire up sidebar buttons to page switching. */
function setupNav() {
  document.querySelectorAll(".nav-item").forEach((btn) => {
    btn.addEventListener("click", () => navigateTo(btn.dataset.page));
  });
}

/**
 * Switch to the named page by toggling .active on nav items and page divs.
 * @param {string} page - Page identifier matching data-page / id="page-<page>"
 */
function navigateTo(page) {
  document.querySelectorAll(".nav-item").forEach((b) => b.classList.remove("active"));
  document.querySelectorAll(".page").forEach((p) => p.classList.remove("active"));
  document.querySelector(`.nav-item[data-page="${page}"]`)?.classList.add("active");
  document.getElementById(`page-${page}`)?.classList.add("active");
}

// Title bar

/** Bind the frameless-window control buttons to their IPC calls. */
function setupTitleBar() {
  document.getElementById("btn-min").addEventListener("click", () => window.api.minimize());
  document.getElementById("btn-max").addEventListener("click", () => window.api.maximize());
  document.getElementById("btn-close").addEventListener("click", () => window.api.close());
}

// Settings

/** Load stored settings from disk and reflect them in the UI. */
async function loadSettings() {
  const s = await window.api.getSettings();
  state.settings = { ...state.settings, ...s };
  document.getElementById("set-token").value = s.token || "";
  document.getElementById("set-delay").value = s.delay || 4;

  // Restore history file path if previously set
  if (s.historyPath) {
    state.history.historyPath = s.historyPath;
    document.getElementById("set-history-path").value = s.historyPath;
  }

  // Batch mode fields
  const bm = s.batchMode || {};
  document.getElementById("set-batch-enabled").checked = !!bm.enabled;
  document.getElementById("set-batch-min").value = bm.minBatch ?? 400;
  document.getElementById("set-batch-max").value = bm.maxBatch ?? 600;
  document.getElementById("set-break-min").value = bm.minBreakMin ?? 10;
  document.getElementById("set-break-max").value = bm.maxBreakMin ?? 120;
  updateBatchFieldsVisibility();
}

/** Wire up all interactive elements on the Settings page. */
function setupSettingsPage() {
  const tokenInput = document.getElementById("set-token");

  // Toggle token visibility
  document.getElementById("btn-toggle-token").addEventListener("click", function () {
    const show = tokenInput.type === "password";
    tokenInput.type = show ? "text" : "password";
    this.textContent = t(show ? "settings.hideToken" : "settings.showToken");
  });

  // Validate the token against the Discord API
  document.getElementById("btn-validate-token").addEventListener("click", async () => {
    const token = tokenInput.value.trim();
    if (!token) return showResult("token-result", t("settings.noToken"), false);
    showResult("token-result", t("settings.tokenChecking"), null);
    const res = await window.api.validateToken(token);
    if (res.valid) {
      showResult("token-result", fmt("settings.tokenOk", { username: res.username, id: res.id }), true);
      state.settings.token = token;
      updateAccountCard(res);
    } else {
      showResult("token-result", fmt("settings.tokenErr", { error: res.error }), false);
    }
  });

  // Open file-picker to choose history save location
  document.getElementById("btn-pick-history-path").addEventListener("click", async () => {
    const picked = await window.api.historyPickPath();
    if (!picked) return;
    state.history.historyPath = picked;
    state.settings.historyPath = picked;
    document.getElementById("set-history-path").value = picked;
  });

  // Load an existing history file and apply it to the current state
  document.getElementById("btn-load-history").addEventListener("click", async () => {
    const picked = await window.api.historyPickExisting();
    if (!picked) return;
    const data = await window.api.historyLoad(picked);
    if (!data || !Array.isArray(data.deletedIds)) {
      return showResult("history-result", t("settings.historyLoadErr") || "Invalid history file.", false);
    }
    state.history.deletedIds = new Set(data.deletedIds);
    state.history.historyPath = picked;
    state.settings.historyPath = picked;
    document.getElementById("set-history-path").value = picked;
    showResult("history-result", (t("settings.historyLoaded") || "Loaded {count} deleted IDs.").replace("{count}", data.deletedIds.length), true);
    // Re-render review tree so already-deleted messages appear unchecked/grayed
    if (state.filteredMessages.length) {
      filterOutDeleted();
      renderReviewTree();
      updateReviewCount();
    }
    updateHistoryStats();
  });

  // Clear the in-memory history (does NOT delete the file)
  document.getElementById("btn-clear-history").addEventListener("click", () => {
    state.history.deletedIds = new Set();
    state._autoSaveCounter = 0;
    showResult("history-result", t("settings.historyCleared") || "History cleared from memory.", true);
    updateHistoryStats();
  });

  // Batch mode toggle
  document.getElementById("set-batch-enabled").addEventListener("change", updateBatchFieldsVisibility);

  // Save all settings and re-apply locale if language changed
  document.getElementById("btn-save-settings").addEventListener("click", async () => {
    const token = document.getElementById("set-token").value.trim();
    const delay = parseInt(document.getElementById("set-delay").value) || 4;
    const lang = document.getElementById("set-lang").value;
    const historyPath = document.getElementById("set-history-path").value.trim();
    const langChanged = lang !== state.settings.lang;
    const batchMode = {
      enabled: document.getElementById("set-batch-enabled").checked,
      minBatch: parseInt(document.getElementById("set-batch-min").value) || 400,
      maxBatch: parseInt(document.getElementById("set-batch-max").value) || 600,
      minBreakMin: parseInt(document.getElementById("set-break-min").value) || 10,
      maxBreakMin: parseInt(document.getElementById("set-break-max").value) || 120,
    };
    state.settings = {
      ...state.settings,
      token,
      delay,
      lang,
      historyPath,
      batchMode,
    };
    state.history.historyPath = historyPath;
    await window.api.saveSettings({
      token,
      delay,
      lang,
      historyPath,
      batchMode,
    });
    if (langChanged) {
      await applyLocale(lang);
      await populateLangSelector();
    }
    showResult("save-result", t("settings.saved"), true);
    setTimeout(() => {
      document.getElementById("save-result").textContent = "";
    }, 2000);
  });
}

/** Show/hide batch sub-settings based on the enabled checkbox. */
function updateBatchFieldsVisibility() {
  const enabled = document.getElementById("set-batch-enabled").checked;
  document.getElementById("batch-sub-fields").style.display = enabled ? "" : "none";
}

/**
 * Display a result message below a form element.
 * @param {string}       id  - Element ID of the result container
 * @param {string}       msg - Text to display
 * @param {boolean|null} ok  - true = success style, false = error, null = neutral
 */
function showResult(id, msg, ok) {
  const el = document.getElementById(id);
  el.textContent = msg;
  el.className =
    (id.includes("token") || id.includes("history") ? "validation-result" : "save-result") + (ok === true ? " ok" : ok === false ? " err" : "");
}

/**
 * Update the Dashboard account card with the validated user's details.
 * Attempts to load the user's avatar from the Discord CDN.
 * @param {{ username: string, id: string, avatar: string|null }} user
 */
function updateAccountCard(user) {
  document.getElementById("acc-name").textContent = user.username || "—";
  document.getElementById("acc-sub").textContent = `ID: ${user.id || "—"}`;
  document.getElementById("acc-badge").textContent = "✓ Active";
  if (user.avatar && user.id) {
    const avatarEl = document.getElementById("acc-avatar");
    const img = document.createElement("img");
    img.src = `https://cdn.discordapp.com/avatars/${user.id}/${user.avatar}.png?size=64`;
    img.onerror = () => {
      avatarEl.textContent = user.username?.[0]?.toUpperCase() || "?";
    };
    avatarEl.innerHTML = "";
    avatarEl.appendChild(img);
  }
}

// Dashboard

/** Wire up Dashboard quick-action buttons. */
function setupDashboard() {
  document.getElementById("btn-goto-pack").addEventListener("click", () => navigateTo("pack"));
  document.getElementById("btn-goto-settings").addEventListener("click", () => navigateTo("settings"));
}

/** Refresh the four stat counters on the Dashboard. */
function updateDashboardStats() {
  if (!state.packData) return;
  const { servers, dms, totalMessages } = state.packData;
  document.getElementById("stat-total").textContent = totalMessages.toLocaleString();
  document.getElementById("stat-servers").textContent = servers.length;
  document.getElementById("stat-dms").textContent = dms.length;
  document.getElementById("stat-selected").textContent = state.selectedIds.size.toLocaleString();
}

/** Update the history stats line in the Settings panel. */
function updateHistoryStats() {
  const el = document.getElementById("history-stats");
  if (!el) return;
  el.textContent = `${state.history.deletedIds.size.toLocaleString()} IDs in memory`;
}

// Pack page

/** Wire up the folder picker and filter form on the Data Pack page. */
function setupPackPage() {
  document.getElementById("btn-pick-folder").addEventListener("click", pickFolder);
  document.getElementById("folder-drop").addEventListener("click", (e) => {
    if (e.target !== document.getElementById("btn-pick-folder")) pickFolder();
  });
  document.getElementById("btn-apply-filter").addEventListener("click", applyFilterAndReview);
}

/** Open a folder picker, parse the chosen pack, and update the UI. */
async function pickFolder() {
  const folder = await window.api.pickFolder();
  if (!folder) return;
  const drop = document.getElementById("folder-drop");
  const pathText = document.getElementById("folder-path-text");
  pathText.textContent = t("pack.loading");
  drop.classList.add("loaded");
  const result = await window.api.parsePack(folder);
  if (result.error) {
    pathText.textContent = `${t("pack.errorGeneric")}: ${result.error}`;
    drop.classList.remove("loaded");
    document.getElementById("pack-filter").classList.add("hidden");
    return;
  }
  state.packData = result;
  pathText.textContent = `✓ ${folder}  —  ${result.totalMessages.toLocaleString()} msg.`;
  document.getElementById("pack-filter").classList.remove("hidden");
  updateDashboardStats();
}

/**
 * Read the current filter controls, build a flat message list, and navigate
 * to the Review page. Messages already present in the deletion history are
 * excluded from the selection (but still shown as unchecked so the user can
 * see what was already handled).
 */
function applyFilterAndReview() {
  if (!state.packData) return;
  const includeServers = document.getElementById("filter-servers").checked;
  const includeDMs = document.getElementById("filter-dms").checked;
  const searchStr = document.getElementById("filter-search").value.trim().toLowerCase();
  const all = [];

  if (includeServers) {
    for (const srv of state.packData.servers) {
      for (const ch of srv.channels) {
        for (const msg of ch.messages) {
          if (!searchStr || msg.content.toLowerCase().includes(searchStr)) {
            all.push({ ...msg, groupName: srv.name, groupType: "server" });
          }
        }
      }
    }
  }
  if (includeDMs) {
    for (const dm of state.packData.dms) {
      for (const msg of dm.messages) {
        if (!searchStr || msg.content.toLowerCase().includes(searchStr)) {
          all.push({ ...msg, groupName: dm.name, groupType: "dm" });
        }
      }
    }
  }

  state.filteredMessages = all;

  // Pre-select only messages that were NOT already deleted in a previous session
  state.selectedIds = new Set(all.filter((m) => !state.history.deletedIds.has(m.id)).map((m) => m.id));

  renderReviewTree();
  updateReviewCount();
  updateDashboardStats();
  navigateTo("review");

  // Inform the user if some messages were auto-excluded thanks to history
  const skipped = all.length - state.selectedIds.size;
  if (skipped > 0) {
    addLog(`History: ${skipped.toLocaleString()} message(s) already deleted in a previous session — deselected automatically.`, "info");
  }
}

/**
 * Remove already-deleted IDs from the current selection.
 * Called after loading a history file while a pack is already open.
 */
function filterOutDeleted() {
  for (const id of state.history.deletedIds) {
    state.selectedIds.delete(id);
  }
}

// Review page

/** Wire up select-all, deselect-all and the delete trigger on the Review page. */
function setupReviewPage() {
  document.getElementById("btn-select-all").addEventListener("click", () => {
    // Select all except those already in history
    state.filteredMessages.forEach((m) => {
      if (!state.history.deletedIds.has(m.id)) state.selectedIds.add(m.id);
    });
    document.querySelectorAll(".tree-msg input[type=checkbox]").forEach((cb) => {
      // Keep already-deleted messages unchecked even on "select all"
      if (!state.history.deletedIds.has(cb.dataset.id)) cb.checked = true;
    });
    updateReviewCount();
    updateDashboardStats();
  });

  document.getElementById("btn-deselect-all").addEventListener("click", () => {
    state.selectedIds.clear();
    document.querySelectorAll(".tree-msg input[type=checkbox]").forEach((cb) => (cb.checked = false));
    updateReviewCount();
    updateDashboardStats();
  });

  document.getElementById("btn-start-delete").addEventListener("click", confirmAndStartDeletion);
}

function renderReviewTree() {
  const tree = document.getElementById("review-tree");
  if (!state.filteredMessages.length) {
    tree.innerHTML = `<div class="empty-state">${t("review.noMessages")}</div>`;
    return;
  }

  // Bucket messages by (groupType, groupName, channelName)
  const channelMap = {};
  for (const msg of state.filteredMessages) {
    const key = msg.groupType === "server" ? `server::${msg.groupName}::${msg.channelName}` : `dm::${msg.groupName}`;
    if (!channelMap[key]) channelMap[key] = { ...msg, messages: [] };
    channelMap[key].messages.push(msg);
  }

  // Further bucket by top-level group (server name or "DM")
  const serverMap = {};
  const dmChannels = [];
  for (const group of Object.values(channelMap)) {
    if (group.groupType === "server") {
      (serverMap[group.groupName] ??= []).push(group);
    } else {
      dmChannels.push(group);
    }
  }

  tree.innerHTML = "";

  // Render server groups
  for (const [srvName, channels] of Object.entries(serverMap)) {
    const total = channels.reduce((s, c) => s + c.messages.length, 0);
    const grpEl = createGroupEl(`${t("review.serverGroup")} ${srvName}`, total);
    const body = grpEl.querySelector(".tree-group-body");
    for (const ch of channels) {
      body.appendChild(createChannelEl(`# ${ch.channelName}`, ch.messages));
    }
    tree.appendChild(grpEl);
  }

  // Render DM group
  if (dmChannels.length) {
    const total = dmChannels.reduce((s, d) => s + d.messages.length, 0);
    const grpEl = createGroupEl(t("review.dmGroup"), total);
    const body = grpEl.querySelector(".tree-group-body");
    for (const dm of dmChannels) {
      body.appendChild(createChannelEl(`@ ${dm.groupName}`, dm.messages));
    }
    tree.appendChild(grpEl);
  }
}

/**
 * Create a collapsible top-level group element (server or DM).
 * @param {string} label - Display label
 * @param {number} count - Total message count shown as a badge
 * @returns {HTMLElement}
 */
function createGroupEl(label, count) {
  const grp = document.createElement("div");
  grp.className = "tree-group";
  const header = document.createElement("div");
  header.className = "tree-group-header";
  header.innerHTML = `
    <span>${label}</span>
    <span style="color:var(--text-2);font-size:11px;margin-left:8px">${count} ${t("review.messages")}</span>
    <span class="chevron">▾</span>`;
  const body = document.createElement("div");
  body.className = "tree-group-body";
  header.addEventListener("click", () => {
    header.classList.toggle("collapsed");
    body.style.display = header.classList.contains("collapsed") ? "none" : "";
  });
  grp.appendChild(header);
  grp.appendChild(body);
  return grp;
}

/**
 * Create a collapsible channel / conversation element with individual message rows inside.
 * @param {string} label    - Display label (e.g. "# general")
 * @param {Array}  messages - Message objects for this channel
 * @returns {HTMLElement}
 */
function createChannelEl(label, messages) {
  const ch = document.createElement("div");
  ch.className = "tree-channel";
  const header = document.createElement("div");
  header.className = "tree-channel-header";
  header.innerHTML = `<span>${label}</span><span class="ch-badge">${messages.length}</span>`;
  const body = document.createElement("div");
  body.className = "tree-messages";
  body.style.display = "none";
  header.addEventListener("click", () => {
    body.style.display = body.style.display === "none" ? "" : "none";
  });
  for (const msg of messages) {
    body.appendChild(createMessageRow(msg));
  }
  ch.appendChild(header);
  ch.appendChild(body);
  return ch;
}

/**
 * Create a single message row with a checkbox, timestamp and content preview.
 * Rows for messages already in the deletion history are styled as dimmed.
 * @param {object} msg - Message object from state.filteredMessages
 * @returns {HTMLElement}
 */
function createMessageRow(msg) {
  const row = document.createElement("div");
  row.className = "tree-msg";
  const alreadyDeleted = state.history.deletedIds.has(msg.id);
  const checked = state.selectedIds.has(msg.id);
  const date = new Date(msg.timestamp).toLocaleString();
  const preview = escHtml((msg.content || t("review.noContent")).slice(0, 80));
  // Dim rows that were already processed in a previous session
  if (alreadyDeleted) row.style.opacity = "0.4";
  row.innerHTML = `
    <input type="checkbox" data-id="${msg.id}" ${checked ? "checked" : ""} ${alreadyDeleted ? "disabled title='Already deleted'" : ""}/>
    <span class="msg-meta">${date}</span>
    <span class="msg-content">${preview}</span>`;
  if (!alreadyDeleted) {
    row.querySelector("input").addEventListener("change", (e) => {
      if (e.target.checked) state.selectedIds.add(msg.id);
      else state.selectedIds.delete(msg.id);
      updateReviewCount();
      updateDashboardStats();
    });
    row.addEventListener("click", (e) => {
      if (e.target.tagName !== "INPUT") {
        const cb = row.querySelector("input");
        cb.checked = !cb.checked;
        cb.dispatchEvent(new Event("change"));
      }
    });
  }
  return row;
}

/** Update the "N selected" counter and enable/disable the delete button. */
function updateReviewCount() {
  const count = state.selectedIds.size;
  document.getElementById("review-count").innerHTML = `${count.toLocaleString()} <span data-i18n="review.selected">${t("review.selected")}</span>`;
  document.getElementById("btn-start-delete").disabled = count === 0;
}

/** Escape HTML special characters to prevent XSS in message previews. */
function escHtml(str) {
  return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

// Deletion History helpers

/**
 * Serialise the current history state to a plain object ready for JSON.stringify.
 * @returns {Object}
 */
function buildHistoryPayload() {
  return {
    version: 1,
    savedAt: new Date().toISOString(),
    deletedIds: Array.from(state.history.deletedIds),
  };
}

/**
 * Write the history file if a path is configured.
 * Silently skips when no path is set — history is only held in memory then.
 */
async function persistHistory() {
  if (!state.history.historyPath) return;
  await window.api.historySave(state.history.historyPath, buildHistoryPayload());
}

// Deletion flow

/**
 * Show a confirmation dialog, then start the deletion process if confirmed.
 * Guards against missing token before proceeding.
 */
async function confirmAndStartDeletion() {
  const count = state.selectedIds.size;
  if (!count) return;
  if (!state.settings.token) {
    alert(t("settings.noTokenAlert"));
    navigateTo("settings");
    return;
  }
  const ok = confirm(fmt("confirm.message", { count }));
  if (!ok) return;
  const messagesToDelete = state.filteredMessages.filter((m) => state.selectedIds.has(m.id));
  startDeletion(messagesToDelete);
}

/** Wire up the Stop and New Session buttons on the Progress page. */
function setupProgressPage() {
  document.getElementById("btn-stop-delete").addEventListener("click", () => {
    // Tell the main process to flip the stopRequested flag — the loop will
    // exit cleanly after the message currently being processed finishes.
    window.api.stopDeletion();
    addLog(t("progress.logStopping"), "warn");
    document.getElementById("btn-stop-delete").disabled = true;
  });
  document.getElementById("btn-new-session").addEventListener("click", () => navigateTo("dashboard"));
}

/**
 * Navigate to the Progress page and begin streaming deletion events from the main process.
 * Handles autosave every AUTO_SAVE_INTERVAL successful deletions.
 * @param {Array} messages - Filtered + selected message objects to delete
 */
async function startDeletion(messages) {
  navigateTo("progress");
  const total = messages.length;
  state._autoSaveCounter = 0;

  // Reset progress UI
  document.getElementById("log-box").innerHTML = "";
  document.getElementById("btn-stop-delete").disabled = false;
  document.getElementById("btn-new-session").style.display = "none";
  document.getElementById("progress-bar").style.width = "0%";
  document.getElementById("prog-deleted").textContent = "0";
  document.getElementById("prog-errors").textContent = "0";
  document.getElementById("prog-remaining").textContent = total;
  document.getElementById("progress-status").textContent = fmt("progress.deleting", { current: 0, total });

  addLog(fmt("progress.logStart", { count: total, delay: state.settings.delay }), "info");

  const bm = state.settings.batchMode;
  if (bm && bm.enabled) {
    addLog(
      fmt("progress.logBatchMode", {
        min: bm.minBatch,
        max: bm.maxBatch,
        breakMin: bm.minBreakMin,
        breakMax: bm.maxBreakMin,
      }),
      "info",
    );
  }

  if (state.history.historyPath) {
    addLog(`History file: ${state.history.historyPath}  (autosave every ${AUTO_SAVE_INTERVAL} messages)`, "info");
  } else {
    addLog("No history file configured — deleted IDs stored in memory only.", "warn");
  }

  // Subscribe to progress events; the returned function unsubscribes on done/stopped
  const cleanup = window.api.onProgress(async (data) => {
    if (data.type === "success") {
      // Record the deleted ID in the in-memory history
      state.history.deletedIds.add(data.msgId);
      state._autoSaveCounter++;

      document.getElementById("prog-deleted").textContent = data.deleted;
      document.getElementById("prog-remaining").textContent = total - data.deleted - data.errors;
      const pct = Math.round(((data.deleted + data.errors) / total) * 100);
      document.getElementById("progress-bar").style.width = pct + "%";
      document.getElementById("progress-status").textContent = fmt("progress.deleting", {
        current: data.deleted + data.errors,
        total,
      });
      addLog(fmt("progress.logSuccess", { id: data.msgId, count: data.deleted }), "success");

      // Autosave every AUTO_SAVE_INTERVAL successful deletions
      if (state._autoSaveCounter % AUTO_SAVE_INTERVAL === 0) {
        await persistHistory();
        addLog(`History autosaved (${state.history.deletedIds.size} IDs total).`, "info");
        updateHistoryStats();
      }
    } else if (data.type === "error") {
      document.getElementById("prog-errors").textContent = data.errors;
      const detail = data.status ? ` [HTTP ${data.status}${data.reason ? ": " + data.reason : ""}]` : "";
      addLog(fmt("progress.logError", { id: data.msgId, detail }), "error");
    } else if (data.type === "ratelimit") {
      addLog(
        fmt("progress.logRateLimit", {
          seconds: (data.retryAfter / 1000).toFixed(1),
        }),
        "warn",
      );
    } else if (data.type === "batch-start") {
      addLog(
        fmt("progress.logBatchStart", {
          index: data.batchIndex,
          total: data.batchTotal,
          size: data.batchSize,
        }),
        "info",
      );
    } else if (data.type === "batch-break") {
      const mins = Math.round(data.breakMs / 60000);
      addLog(
        fmt("progress.logBatchBreak", {
          index: data.batchIndex,
          total: data.batchTotal,
          mins,
        }),
        "warn",
      );
      document.getElementById("progress-status").textContent = fmt("progress.batchBreakStatus", { mins });
    } else if (data.type === "batch-break-tick") {
      const mins = Math.ceil(data.remainingMs / 60000);
      document.getElementById("progress-status").textContent = fmt("progress.batchBreakStatus", { mins });
    } else if (data.type === "stopped") {
      // User clicked Stop — save history then finalise UI
      cleanup();
      await persistHistory();
      updateHistoryStats();
      document.getElementById("btn-stop-delete").disabled = true;
      document.getElementById("btn-new-session").style.display = "";
      document.getElementById("progress-status").textContent =
        fmt("progress.stopped", {
          deleted: data.deleted,
          errors: data.errors,
        }) || `Stopped. Deleted: ${data.deleted}, Errors: ${data.errors}`;
      addLog(`Stopped by user. Deleted: ${data.deleted}, Errors: ${data.errors}. History saved.`, "warn");
    } else if (data.type === "done") {
      // Normal completion — save history then finalise UI
      cleanup();
      await persistHistory();
      updateHistoryStats();
      document.getElementById("btn-stop-delete").disabled = true;
      document.getElementById("btn-new-session").style.display = "";
      document.getElementById("progress-bar").style.width = "100%";
      document.getElementById("progress-status").textContent = fmt("progress.done", {
        deleted: data.deleted,
        errors: data.errors,
      });
      addLog(fmt("progress.logDone", { deleted: data.deleted, errors: data.errors }), "success");
    }
  });

  await window.api.startDeletion({
    messages,
    token: state.settings.token,
    delay: state.settings.delay,
    batchMode: bm && bm.enabled ? bm : null,
    deletedIds: Array.from(state.history.deletedIds),
  });
}

/**
 * Append a timestamped entry to the deletion log box and auto-scroll to bottom.
 * @param {string} msg  - Log message text
 * @param {string} type - CSS class for colouring: "info" | "success" | "error" | "warn"
 */
function addLog(msg, type = "info") {
  const box = document.getElementById("log-box");
  const entry = document.createElement("div");
  entry.className = `log-entry ${type}`;
  const time = new Date().toLocaleTimeString();
  entry.textContent = `[${time}] ${msg}`;
  box.appendChild(entry);
  box.scrollTop = box.scrollHeight;
}
