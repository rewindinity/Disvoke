const { contextBridge, ipcRenderer } = require("electron");

/**
 * Safe API surface exposed to the renderer process
 * All methods communicate with main process via IPC
 */
contextBridge.exposeInMainWorld("api", {
  // Window Controls
  /** Minimize the application window */
  minimize: () => ipcRenderer.send("window-minimize"),
  /** Maximize or unmaximize the application window */
  maximize: () => ipcRenderer.send("window-maximize"),
  /** Close the application window */
  close: () => ipcRenderer.send("window-close"),

  // Settings Management
  /** Retrieve all saved settings from persistent storage */
  getSettings: () => ipcRenderer.invoke("settings-get"),
  /** Save settings to persistent storage */
  saveSettings: (data) => ipcRenderer.invoke("settings-save", data),

  // Internationalization (i18n)
  /** Load translation strings for a specific locale */
  loadLocale: (lang) => ipcRenderer.invoke("i18n-load", lang),
  /** Get list of all available locales with display names */
  listLocales: () => ipcRenderer.invoke("i18n-list"),

  // File System Operations
  /** Open native folder picker dialog */
  pickFolder: () => ipcRenderer.invoke("pick-folder"),
  /** Parse Discord GDPR data pack from selected folder */
  parsePack: (folderPath) => ipcRenderer.invoke("parse-pack", folderPath),

  // Discord API Integration
  /** Validate Discord authentication token */
  validateToken: (token) => ipcRenderer.invoke("validate-token", token),
  /** Start message deletion process with progress streaming */
  startDeletion: (payload) => ipcRenderer.invoke("start-deletion", payload),
  /** Signal the main process to stop the active deletion loop */
  stopDeletion: () => ipcRenderer.send("stop-deletion"),
  /** Subscribe to deletion progress events; returns an unsubscribe function */
  onProgress: (callback) => {
    ipcRenderer.on("deletion-progress", (_, data) => callback(data));
    return () => ipcRenderer.removeAllListeners("deletion-progress");
  },

  // Deletion History
  /** Open a save-file dialog to choose history file location */
  historyPickPath: () => ipcRenderer.invoke("history-pick-path"),
  /** Open an open-file dialog to load an existing history file */
  historyPickExisting: () => ipcRenderer.invoke("history-pick-existing"),
  /** Write history JSON to a file on disk */
  historySave: (filePath, data) => ipcRenderer.invoke("history-save", filePath, data),
  /** Read and parse a history JSON file from disk */
  historyLoad: (filePath) => ipcRenderer.invoke("history-load", filePath),
});
