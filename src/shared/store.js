const fs = require("fs");
const path = require("path");
const { app } = require("electron");
class Store {
  constructor() {
    const userDataPath = app.getPath("userData");
    this.filePath = path.join(userDataPath, "settings.json");
    this.data = this._load();
  }
  /**
   * Load settings from disk.
   * Falls back to safe defaults on missing file or parse error.
   * @returns {object}
   */
  _load() {
    try {
      if (fs.existsSync(this.filePath)) {
        return JSON.parse(fs.readFileSync(this.filePath, "utf-8"));
      }
    } catch (_) {}
    return { token: "", delay: 4, lang: "en" };
  }
  /** Return a shallow copy of all stored settings. */
  getAll() {
    return { ...this.data };
  }
  /**
   * Merge new values into the store and persist to disk.
   * @param {object} data - Partial settings object to merge
   */
  setAll(data) {
    this.data = { ...this.data, ...data };
    try {
      fs.writeFileSync(this.filePath, JSON.stringify(this.data, null, 2), "utf-8");
    } catch (_) {}
  }
}
module.exports = Store;
