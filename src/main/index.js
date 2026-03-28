const { app, BrowserWindow, ipcMain, dialog } = require("electron");
const path = require("path");
const fs = require("fs");
const Store = require("../shared/store.js");
const { parsePack, fetchDiscord, sleep } = require("./utils.js");

let mainWindow;
const store = new Store();

/**
 * Runtime deletion state — reset before every deletion run.
 * stopRequested is checked before each message; setting it to true
 * causes the loop to break cleanly without killing the process.
 */
let stopRequested = false;

/**
 * Creates the main application window
 * Configures frameless window
 */
function createWindow() {
  mainWindow = new BrowserWindow({
    width: 1100,
    height: 760,
    minWidth: 900,
    minHeight: 600,
    frame: false,
    titleBarStyle: "hidden",
    backgroundColor: "#0d0e11",
    show: false,
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      contextIsolation: true,
      nodeIntegration: false,
    },
    icon: path.join(__dirname, "../resources/icon.png"),
  });
  mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  mainWindow.on("ready-to-show", () => {
    mainWindow.show();
  });
}

// App Events
app.whenReady().then(() => {
  createWindow();
});
app.on("window-all-closed", () => {
  app.quit();
});
app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) createWindow();
});

// Window Control IPC

ipcMain.on("window-minimize", () => mainWindow.minimize());
ipcMain.on("window-maximize", () => {
  mainWindow.isMaximized() ? mainWindow.unmaximize() : mainWindow.maximize();
});
ipcMain.on("window-close", () => mainWindow.close());

// Settings IPC

/**
 * Retrieve all stored settings
 * @returns {Object} Settings object
 */
ipcMain.handle("settings-get", () => store.getAll());

/**
 * Save settings to persistent storage
 * @param {Event} _ - IPC event (unused)
 * @param {Object} data - Partial settings object to merge
 * @returns {boolean} Success status
 */
ipcMain.handle("settings-save", (_, data) => {
  store.setAll(data);
  return true;
});

// Internationalization (i18n) IPC

/**
 * Load translation strings for a specific locale
 * Falls back to English if requested locale doesn't exist
 * @param {Event} _ - IPC event (unused)
 * @param {string} lang - Locale code (e.g., "en", "pl")
 * @returns {Object} Translation key-value pairs
 */
ipcMain.handle("i18n-load", (_, lang) => {
  const localesDir = path.join(__dirname, "../shared/locales");
  const targetFile = path.join(localesDir, `${lang}.json`);
  const fallbackFile = path.join(localesDir, "en.json");
  for (const file of [targetFile, fallbackFile]) {
    if (fs.existsSync(file)) {
      try {
        return JSON.parse(fs.readFileSync(file, "utf-8"));
      } catch (_) {
        // Continue to next file on parse error
      }
    }
  }
  return {};
});

/**
 * List all available locales
 * @returns {Array<{code: string, name: string}>} Array of locale objects
 */
ipcMain.handle("i18n-list", () => {
  const localesDir = path.join(__dirname, "../shared/locales");
  if (!fs.existsSync(localesDir)) return [];
  return fs
    .readdirSync(localesDir)
    .filter((file) => file.endsWith(".json"))
    .map((file) => {
      const code = file.replace(".json", "");
      try {
        const data = JSON.parse(fs.readFileSync(path.join(localesDir, file), "utf-8"));
        return { code, name: data._meta?.name || code };
      } catch (_) {
        return { code, name: code };
      }
    });
});

// File System IPC

/**
 * Open native folder selection dialog
 * @returns {string|null} Selected folder path or null if cancelled
 */
ipcMain.handle("pick-folder", async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    properties: ["openDirectory"],
    title: "Select extracted Discord data pack folder",
  });
  return result.canceled ? null : result.filePaths[0];
});

// Discord Data Pack IPC

/**
 * Parse Discord GDPR data pack
 * @param {Event} _ - IPC event (unused)
 * @param {string} folderPath - Path to extracted pack folder
 * @returns {Object} Parsed pack data or error object
 */
ipcMain.handle("parse-pack", async (_, folderPath) => {
  try {
    return parsePack(folderPath);
  } catch (error) {
    return { error: error.message };
  }
});

// Discord API IPC

/**
 * Validate Discord authentication token
 * @param {Event} _ - IPC event (unused)
 * @param {string} token - Discord token to validate
 * @returns {Object} Validation result with user data or error
 */
ipcMain.handle("validate-token", async (_, token) => {
  try {
    const response = await fetchDiscord("https://discord.com/api/v10/users/@me", token, "GET");
    if (response.status === 401) {
      return { valid: false, error: "Invalid token" };
    }
    const userData = await response.json();
    return {
      valid: true,
      username: userData.username,
      id: userData.id,
      avatar: userData.avatar,
    };
  } catch (error) {
    return { valid: false, error: error.message };
  }
});

// Deletion stop IPC

/**
 * Signal the active deletion loop to stop after the current message.
 */
ipcMain.on("stop-deletion", () => {
  stopRequested = true;
});

/**
 * Start the message deletion process
 * Streams progress events back to renderer
 */
ipcMain.handle("start-deletion", async (event, { messages, token, delay, batchMode, deletedIds = [] }) => {
  stopRequested = false; // reset flag at the start of each run
  const delayMs = (delay || 4) * 1000;
  let deleted = 0;
  let errors = 0;
  const historySet = new Set(deletedIds);
  // Build list of batches if batchMode is enabled, otherwise one big batch
  let batches;
  if (batchMode && batchMode.enabled) {
    const minBatch = Math.max(1, batchMode.minBatch || 400);
    const maxBatch = Math.max(minBatch, batchMode.maxBatch || 600);
    const minBreak = Math.max(0, (batchMode.minBreakMin || 10) * 60 * 1000);
    const maxBreak = Math.max(minBreak, (batchMode.maxBreakMin || 120) * 60 * 1000);
    batches = [];
    let i = 0;
    while (i < messages.length) {
      const size = Math.floor(Math.random() * (maxBatch - minBatch + 1)) + minBatch;
      batches.push({
        msgs: messages.slice(i, i + size),
        breakMs: Math.floor(Math.random() * (maxBreak - minBreak + 1)) + minBreak,
      });
      i += size;
    }
  } else {
    batches = [{ msgs: messages, breakMs: 0 }];
  }

  outer: for (let batchIdx = 0; batchIdx < batches.length; batchIdx++) {
    const { msgs: batchMsgs, breakMs } = batches[batchIdx];
    if (batchMode && batchMode.enabled) {
      event.sender.send("deletion-progress", {
        type: "batch-start",
        batchIndex: batchIdx + 1,
        batchTotal: batches.length,
        batchSize: batchMsgs.length,
      });
    }

    for (const message of batchMsgs) {
      if (stopRequested) break outer;
      if (historySet.has(message.id)) {
        deleted++;
        event.sender.send("deletion-progress", {
          type: "success",
          msgId: message.id,
          deleted,
          errors,
          skipped: true,
        });
        continue;
      }
      let channelId = message.channelId;

      /* Handle DM channels (need to refresh channel ID)
       Discord may reassign DM channel IDs; closed DMs must be reopened */
      if (message.isDM && Array.isArray(message.recipientIds) && message.recipientIds.length > 0) {
        for (const recipientId of message.recipientIds) {
          try {
            const dmResponse = await fetchDiscord("https://discord.com/api/v10/users/@me/channels", token, "POST", { recipient_id: recipientId });
            if (dmResponse.ok) {
              const dmData = await dmResponse.json();
              if (dmData.id) {
                channelId = dmData.id;
                break;
              }
            }
          } catch (_) {
            // Continue to next recipient on error
          }
        }
      }

      // Attempt message deletion
      try {
        const response = await fetchDiscord(`https://discord.com/api/v10/channels/${channelId}/messages/${message.id}`, token, "DELETE");

        if (response.status === 204 || response.status === 200) {
          deleted++;
          event.sender.send("deletion-progress", {
            type: "success",
            msgId: message.id,
            deleted,
            errors,
          });
        } else if (response.status === 429) {
          // Rate limited — parse retry-after and retry once
          const rateLimitData = await response.json().catch(() => ({}));
          const retryAfter = Math.ceil((rateLimitData.retry_after || 5) * 1000);
          event.sender.send("deletion-progress", {
            type: "ratelimit",
            retryAfter,
          });
          await sleep(retryAfter + 500);

          // Check stop flag again after the sleep
          if (stopRequested) break outer;

          const retryResponse = await fetchDiscord(`https://discord.com/api/v10/channels/${channelId}/messages/${message.id}`, token, "DELETE");
          if (retryResponse.status === 204 || retryResponse.status === 200) {
            deleted++;
            event.sender.send("deletion-progress", {
              type: "success",
              msgId: message.id,
              deleted,
              errors,
            });
          } else {
            const errorData = await retryResponse.json().catch(() => ({}));
            errors++;
            event.sender.send("deletion-progress", {
              type: "error",
              msgId: message.id,
              deleted,
              errors,
              status: retryResponse.status,
              reason: errorData.message || JSON.stringify(errorData),
            });
          }
        } else {
          const errorData = await response.json().catch(() => ({}));
          errors++;
          event.sender.send("deletion-progress", {
            type: "error",
            msgId: message.id,
            deleted,
            errors,
            status: response.status,
            reason: errorData.message || JSON.stringify(errorData),
          });
        }
      } catch (error) {
        errors++;
        event.sender.send("deletion-progress", {
          type: "error",
          msgId: message.id,
          deleted,
          errors,
          status: "exception",
          reason: error.message,
        });
      }
      await sleep(delayMs);
    } // end message loop

    // After batch (but not after the last one), wait for the inter-batch break
    if (batchMode && batchMode.enabled && batchIdx < batches.length - 1) {
      event.sender.send("deletion-progress", {
        type: "batch-break",
        breakMs,
        batchIndex: batchIdx + 1,
        batchTotal: batches.length,
      });

      const CHUNK = 10000;
      let remaining = breakMs;
      while (remaining > 0) {
        // Check stop flag during long inter-batch breaks too
        if (stopRequested) break outer;
        const wait = Math.min(CHUNK, remaining);
        await sleep(wait);
        remaining -= wait;
        if (remaining > 0) {
          event.sender.send("deletion-progress", {
            type: "batch-break-tick",
            remainingMs: remaining,
          });
        }
      }
    }
  } // end batch loop

  // Distinguish normal completion from user-requested stop
  const finalType = stopRequested ? "stopped" : "done";
  event.sender.send("deletion-progress", {
    type: finalType,
    deleted,
    errors,
  });
  return { deleted, errors };
});

// Deletion History IPC

/**
 * Open a save-file dialog so the user can pick where to store the history JSON.
 * @returns {string|null} Chosen file path or null if cancelled
 */
ipcMain.handle("history-pick-path", async () => {
  const result = await dialog.showSaveDialog(mainWindow, {
    title: "Choose history file location",
    defaultPath: "disvoke-history.json",
    filters: [{ name: "JSON", extensions: ["json"] }],
  });
  return result.canceled ? null : result.filePath;
});

/**
 * Open an open-file dialog so the user can load an existing history file.
 * @returns {string|null} Chosen file path or null if cancelled
 */
ipcMain.handle("history-pick-existing", async () => {
  const result = await dialog.showOpenDialog(mainWindow, {
    title: "Open history file",
    filters: [{ name: "JSON", extensions: ["json"] }],
    properties: ["openFile"],
  });
  return result.canceled ? null : result.filePaths[0];
});

/**
 * Persist the deletion history object to a JSON file.
 * Called automatically every AUTO_SAVE_INTERVAL messages and on stop/done.
 * @param {Event} _ - IPC event (unused)
 * @param {string} filePath - Absolute path to write to
 * @param {Object} historyData - History payload to serialise
 * @returns {boolean} true on success, false on write error
 */
ipcMain.handle("history-save", (_, filePath, historyData) => {
  try {
    fs.writeFileSync(filePath, JSON.stringify(historyData, null, 2), "utf-8");
    return true;
  } catch (_err) {
    return false;
  }
});

/**
 * Read and parse a history JSON file from disk.
 * @param {Event} _ - IPC event (unused)
 * @param {string} filePath - Absolute path to read from
 * @returns {Object|null} Parsed history object, or null on error
 */
ipcMain.handle("history-load", (_, filePath) => {
  try {
    const raw = fs.readFileSync(filePath, "utf-8");
    return JSON.parse(raw);
  } catch (_err) {
    return null;
  }
});

module.exports = { fetchDiscord, sleep, parsePack };
