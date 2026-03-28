const fs = require("fs");
const path = require("path");

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Makes authenticated requests to Discord API
 * @param {string} url - Discord API endpoint URL
 * @param {string} token - Discord authentication token
 * @param {string} [method="GET"] - HTTP method (GET, POST, DELETE, etc.)
 * @param {Object|null} [body=null] - Request body for POST/PUT requests
 * @returns {Promise<Response>} Fetch Response object
 */
function fetchDiscord(url, token, method = "GET", body = null) {
  const headers = {
    Authorization: token,
    "Content-Type": "application/json",
    "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:145.0) Gecko/20100101 Firefox/145.0", // set "normal" useragent (like windows device) to bypass automation detection
  };
  const opts = { method, headers };
  if (body) opts.body = JSON.stringify(body);
  return fetch(url, opts);
}

/**
 * Safely parses Discord messages.json while preserving 64-bit snowflake IDs
 * @param {string} rawText - Raw JSON content from messages.json
 * @returns {Array} Parsed messages with IDs as strings
 */
function safeParseMessages(rawText) {
  const fixed = rawText.replace(/"ID"\s*:\s*(\d+)/g, '"ID": "$1"');
  return JSON.parse(fixed);
}

/**
 * Parses a Discord GDPR data pack directory structure
 * @param {string} folderPath - Absolute path to extracted Discord pack
 * @returns {Object} Structured pack data
 * @throws {Error} If Messages folder is not found
 */
function parsePack(folderPath) {
  const results = {
    servers: [], // Guild/server messages grouped by server
    dms: [], // Direct messages grouped by conversation
    totalMessages: 0, // Total message count across all channels
    userId: null, // Discord user ID from account info
    username: null, // Username from account info
  };
  // 1. Extract user account information
  for (const accountPath of [path.join(folderPath, "Account", "user.json"), path.join(folderPath, "account", "user.json")]) {
    if (fs.existsSync(accountPath)) {
      try {
        const accountData = JSON.parse(fs.readFileSync(accountPath, "utf-8"));
        results.userId = accountData.id || null;
        results.username = accountData.username || null;
      } catch (_) {
        // Silently ignore parsing errors - account info is optional
      }
      break;
    }
  }
  // 2. Locate Messages directory
  let messagesRoot = null;
  for (const candidate of ["Messages", "messages"]) {
    const potentialPath = path.join(folderPath, candidate);
    if (fs.existsSync(potentialPath) && fs.statSync(potentialPath).isDirectory()) {
      messagesRoot = potentialPath;
      break;
    }
  }
  if (!messagesRoot) {
    throw new Error('"Messages" folder not found in pack. ' + "Make sure you selected the extracted (not ZIP) Discord pack folder.");
  }
  // 3. Process each channel directory
  // Channel directories are named c<snowflake>, e.g., c12345678904567
  const channelDirs = fs.readdirSync(messagesRoot).filter((dir) => {
    const fullPath = path.join(messagesRoot, dir);
    return fs.statSync(fullPath).isDirectory() && /^c\d+$/.test(dir);
  });
  for (const channelDir of channelDirs) {
    const channelPath = path.join(messagesRoot, channelDir);
    const messagesPath = path.join(channelPath, "messages.json");
    const channelInfoPath = path.join(channelPath, "channel.json");
    // Skip channels without messages
    if (!fs.existsSync(messagesPath)) continue;
    let channelInfo = {
      id: channelDir.slice(1),
      type: "GUILD_TEXT",
      name: channelDir,
      recipients: [],
      guild: null,
    };
    if (fs.existsSync(channelInfoPath)) {
      try {
        channelInfo = {
          ...channelInfo,
          ...JSON.parse(fs.readFileSync(channelInfoPath, "utf-8")),
        };
      } catch (_) {
        // Use defaults if channel.json is malformed
      }
    }
    // Parse messages with snowflake preservation
    let rawMessages = [];
    try {
      const rawText = fs.readFileSync(messagesPath, "utf-8");
      rawMessages = safeParseMessages(rawText);
      if (!Array.isArray(rawMessages)) rawMessages = [];
    } catch (_) {
      continue;
    }
    // Determine channel type
    const typeString = String(channelInfo.type).toUpperCase();
    const isDM = typeString === "DM" || typeString === "1" || typeString === "GROUP_DM" || typeString === "3";
    const channelId = String(channelInfo.id || channelDir.slice(1));
    const recipientIds = Array.isArray(channelInfo.recipients) ? channelInfo.recipients.map((r) => String(r)).filter(Boolean) : [];
    // Transform raw messages to internal format
    const messages = rawMessages
      .map((msg) => ({
        id: String(msg.ID || msg.id || ""),
        timestamp: msg.Timestamp || msg.timestamp || "",
        content: msg.Contents ?? msg.content ?? "",
        attachments: msg.Attachments || msg.attachments || "",
        channelId,
        isDM,
        recipientIds,
        guildId: channelInfo.guild?.id || null,
        guildName: channelInfo.guild?.name || null,
        channelName: channelInfo.name || channelDir,
      }))
      .filter((msg) => msg.id); // Drop entries without valid IDs
    results.totalMessages += messages.length;
    // Sort into DM or server buckets
    if (isDM) {
      const displayName = channelInfo.name || recipientIds[0] || channelDir;
      results.dms.push({
        channelId,
        name: displayName,
        recipientIds,
        messages,
      });
    } else {
      const guildName = channelInfo.guild?.name || "Unknown server";
      const guildId = channelInfo.guild?.id || "__unknown__";
      // Find or create server group
      let server = results.servers.find((s) => s.guildId === guildId);
      if (!server) {
        server = { guildId, name: guildName, channels: [] };
        results.servers.push(server);
      }
      server.channels.push({
        channelId,
        name: channelInfo.name || channelDir,
        messages,
      });
    }
  }
  return results;
}

module.exports = { sleep, fetchDiscord, parsePack };
