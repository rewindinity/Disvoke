# Disvoke

![Disvoke Icon](src/resources/logo.ico)

**Disvoke** is an Electron-based desktop application designed to help you efficiently and safely delete your past messages using a Discord GDPR privacy data pack.

Whether you want to clean up your Direct Messages (DMs) or wipe your history from Discord servers, Disvoke automates the process while respecting Discord's API rate limits.

## ✨ Features

- **Comprehensive Deletion:** Delete both Direct Messages (DMs) and Server (Guild) messages parsed directly from your Discord data pack.
- **Smart Rate Limiting:** Built-in deletion delays and rate-limit handling to protect your account from being flagged for API abuse.
- **Multilanguage Support:** The interface supports multiple languages, making it accessible to a global user base.
- **Cross-Platform:** Available for Windows, macOS, and Linux.

---

## 🚀 Getting Started

There are two ways to get Disvoke running on your machine: downloading a pre-built release (recommended for most users) or building it yourself from the source code.

### Option 1: Download from Releases (Recommended)

You can download the ready-to-use installers and executables for your operating system directly from the [Releases](https://github.com/Rewindinity/Disvoke/releases) page.

- **Windows:** Download the `.exe` installer.
- **Linux:** Download the `.AppImage` file.

### Option 2: Build from Source

If you want to contribute to the project or compile the application yourself, follow the instructions below.

#### Prerequisites

Before you begin, ensure you have the following installed on your system:

- **Node.js**: Minimum version **v18** or higher (Tested and recommended on **v22**).
- **npm**: Usually comes bundled with Node.js.
- **OS-Specific Build Tools**: Depending on your operating system, `electron-builder` may require native compilation tools (e.g., `build-essential` on Linux, Visual Studio Build Tools on Windows, or Xcode Command Line Tools on macOS).

#### Clone and Install

First, clone the repository and install the required dependencies:

```bash
git clone https://github.com/Rewindinity/Disvoke.git
cd Disvoke
npm install
```

If you don't want to build the app, just fast startup, use: `npm run start`

#### Building the App

Disvoke provides a convenient bash script (`build.sh`) to easily target specific platforms, as well as the ability to build directly using `electron-builder`.

**Method A: Using `build.sh` (Recommended for Unix/Linux/macOS or Git Bash)**

Run the script with your desired target platform:

```bash
# Build Linux x64 AppImage
./build.sh linux

# Build Linux ARM64 AppImage
./build.sh linux-arm

# Build Windows x64 installer
./build.sh windows

# Build macOS x64 DMG (requires macOS)
./build.sh mac

# Build macOS ARM64 DMG (requires macOS)
./build.sh mac-arm

# Build for all available platforms at once
./build.sh all
```

**Method B: Using standard npm / npx commands**

If you cannot run bash scripts (e.g., standard Windows Command Prompt) or prefer using Node commands directly, you can trigger `electron-builder` via `npx`:

```bash
# Build for Linux
npx electron-builder --linux --x64 --publish never

# Build for Linux ARM
npx electron-builder --linux --arm64 --publish never

# Build for Windows
npx electron-builder --win --x64 --publish never

# Build for macOS - Intel CPUs (requires macOS)
npx electron-builder --mac --x64 --publish never

# Buiild for macOS - Apple ARM CPUs (requires macOS)
npx electron-builder --mac --arm64 --publish never

# Build for the current platform you are running on
npx electron-builder --publish never
```

_(Note: You can also add these commands to the `"scripts"` section of your `package.json` to run them via `npm run build:win`, etc.)_

All compiled executables and installers will be output to the `build/` (or default `electron-builder` output) directory.

---

## 📝 How to Use

1. **Request your Data:** Go to your Discord Settings -> Privacy & Safety -> "Request all of my data". Wait for Discord to email you the ZIP file.
2. **Extract the Pack:** Extract the downloaded ZIP file to a folder on your computer.
3. **Get your Token:** Obtain your Discord account authorization token. _(Note: Do not share your token with anyone)._
4. **Run Disvoke:** Open the app, input your token, configure your deletion delay (rate limit), and point the app to the extracted data pack folder.
5. **Review & Delete:** Filter the messages you want to remove, review the selection, and start the deletion process!

---

## ⚠️ Disclaimer

Automating user accounts (self-botting) is technically against Discord's Terms of Service. Disvoke includes rate-limiting to mitigate risks, but you use this software entirely at your own risk. The developers of Disvoke are not responsible for any account bans, suspensions, or lost data resulting from the use of this application.
