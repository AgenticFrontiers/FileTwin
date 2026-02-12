<<<<<<< HEAD
=======
<<<<<<< HEAD
# FileTwin
FileTwin is a peer-to-peer file synchronization system designed for macOS. It enables seamless, secure, and near real-time syncing between two Macs — whether over a local network or remotely — without relying on third-party cloud storage.  Built for speed, privacy, and simplicity.
=======
>>>>>>> 1320ae5 (for commiting the changes for file sharing)
# RemoteSync

A **Tauri** app that syncs clipboard and files between two computers over the same WiFi — **no IP address needed**. Devices find each other by name using mDNS (Bonjour). Works on **macOS**; can be adapted for Windows and Linux.

---

## Table of contents

- [Features](#features)
- [Quick start (TL;DR)](#quick-start-tldr)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
  - [macOS](#macos)
  - [Windows](#windows)
  - [Linux](#linux)
- [Running the app](#running-the-app)
- [Building for distribution](#building-for-distribution)
- [Sharing and installing on another Mac](#sharing-and-installing-on-another-mac)
- [Usage](#usage)
- [Troubleshooting](#troubleshooting)
- [Tech stack](#tech-stack)
- [License](#license)

---

## Features

| Feature | Description |
|--------|-------------|
| **Connect over WiFi** | Uses mDNS so the other machine appears by name (e.g. `RemoteSync-YourMac`). No typing IPs. |
| **Open on other device** | Bring the app window to the front on the connected machine. |
| **Transfer anything** | Send any file; the other side can Save or Open it. |
| **Real-time clipboard** | Copy on one machine, paste on the other. Optional “Sync clipboard in real time”. |

---

## Quick start (TL;DR)

If you already have **Node.js**, **Rust**, and (on macOS) **Xcode Command Line Tools** installed:

```bash
git clone <repo-url>
cd Remote
npm install
npm run tauri dev
```

For a full install from scratch, follow [Prerequisites](#prerequisites) and [Installation](#installation) for your OS.

---

## Prerequisites

RemoteSync is a [Tauri 2](https://tauri.app/) app, so you need:

| Requirement | Purpose |
|-------------|--------|
| **Node.js** (v18 or v20 LTS recommended) | Frontend (React + Vite) and npm scripts |
| **npm** (comes with Node) or **pnpm** / **yarn** | Install JS dependencies |
| **Rust** (latest stable) | Tauri backend (Rust) |
| **OS-specific build tools** | Compile native code (see per-OS steps below) |

Check what you have:

```bash
node -v    # e.g. v20.x.x
npm -v     # e.g. 10.x.x
rustc -v   # e.g. rustc 1.xx.x
```

---

## Installation

### macOS

#### Step 1: Xcode Command Line Tools (required for Rust/Tauri)

1. Open **Terminal** (Spotlight: `Cmd + Space` → type “Terminal”).
2. Run:
   ```bash
   xcode-select --install
   ```
3. In the dialog, click **Install** and wait for the download to finish.
4. Verify:
   ```bash
   xcode-select -p
   ```
   You should see a path like `/Library/Developer/CommandLineTools`.

#### Step 2: Install Node.js

**Option A – Official installer**

1. Go to [nodejs.org](https://nodejs.org/) and download the **LTS** version.
2. Run the installer and follow the steps.
3. Restart Terminal, then run:
   ```bash
   node -v
   npm -v
   ```

**Option B – Homebrew**

```bash
brew install node
node -v && npm -v
```

#### Step 3: Install Rust

1. In Terminal, run:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Choose **1) Proceed with installation (default)**.
3. Restart Terminal (or run `source "$HOME/.cargo/env"`).
4. Verify:
   ```bash
   rustc -v
   cargo -v
   ```

#### Step 4: Clone and install the project

```bash
cd /Users/user/Projects/Remote   # or ~/Projects/Remote — use your path
npm install
```

#### Step 5: Run the app

```bash
npm run tauri dev
```

The first run may take a few minutes while Rust compiles. Later runs are much faster.

---

### Windows

#### Step 1: Microsoft C++ Build Tools (required for Rust/Tauri)

1. Download **Visual Studio Build Tools**:  
   [https://visualstudio.microsoft.com/visual-cpp-build-tools/](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Run the installer and select the workload **“Desktop development with C++”**.
3. Install and restart if prompted.

#### Step 2: WebView2 (usually already on Windows 10/11)

Tauri uses WebView2. If needed, install from:  
[https://developer.microsoft.com/en-us/microsoft-edge/webview2/](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)

#### Step 3: Install Node.js

1. Go to [nodejs.org](https://nodejs.org/) and download the **LTS** version for Windows.
2. Run the installer (option “Add to PATH” is recommended).
3. Open a **new** Command Prompt or PowerShell:
   ```powershell
   node -v
   npm -v
   ```

#### Step 4: Install Rust

1. Go to [rustup.rs](https://rustup.rs/) and download `rustup-init.exe`.
2. Run it and choose **default installation**.
3. Restart the terminal, then:
   ```powershell
   rustc -v
   cargo -v
   ```

#### Step 5: Clone and install the project

```powershell
cd C:\Users\user\Projects\Remote   # use your path
npm install
```

#### Step 6: Run the app

```powershell
npm run tauri dev
```

> **Note:** This app is currently tuned for macOS (mDNS/Bonjour, Mac sharing). On Windows you may need to adjust discovery/network code for full functionality.

---

### Linux

#### Step 1: Install system dependencies (Rust/Tauri)

**Debian / Ubuntu:**

```bash
sudo apt update
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

**Fedora:**

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file libxdo-devel libappindicator-gtk3-devel librsvg2-devel
```

**Arch:**

```bash
sudo pacman -S webkit2gtk-4.1 base-devel curl wget file openssl appindicator3-gtk3 librsvg
```

#### Step 2: Install Node.js

**Option A – NodeSource (Debian/Ubuntu)**

```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
node -v && npm -v
```

**Option B – Package manager**

- **Fedora:** `sudo dnf install nodejs npm`
- **Arch:** `sudo pacman -S nodejs npm`

#### Step 3: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustc -v && cargo -v
```

#### Step 4: Clone and install the project

```bash
cd /home/user/Projects/Remote   # use your path
npm install
```

#### Step 5: Run the app

```bash
npm run tauri dev
```

> **Note:** Discovery and “Mac sharing” behaviour are designed for macOS; Linux support may require code changes.

---

## Running the app

| Command | Description |
|--------|-------------|
| `npm run tauri dev` | Start in **development** mode (hot reload, devtools). |
| `npm run tauri build` | Create **production** build and installers. |

After `npm run tauri dev`, the app window opens. Use it on two machines on the same network to sync clipboard and files.

---

## Building for distribution

From the project root:

```bash
npm install
npm run tauri build
```

Outputs are under `src-tauri/target/release/bundle/`:

- **macOS:**  
  - App: `bundle/macos/RemoteSync.app`  
  - DMG: `bundle/dmg/RemoteSync_0.1.0_aarch64.dmg` (Apple Silicon) or `..._x64.dmg` (Intel)
- **Windows:** `.msi` and `.exe` in `bundle/msi/` and `bundle/nsis/`
- **Linux:** `.deb`, `.AppImage`, etc. in the corresponding `bundle/` subfolders

---

## Sharing and installing on another Mac

After building on a Mac, you can copy the app to another Mac without building there.

1. **Build** (on a Mac that has the project):
   ```bash
   npm install
   npm run tauri build
   ```
2. **Locate the app:**
   - **App:** `src-tauri/target/release/bundle/macos/RemoteSync.app`
   - **DMG:** `src-tauri/target/release/bundle/dmg/RemoteSync_0.1.0_aarch64.dmg` (or `x64` for Intel)
3. **Share:** AirDrop, USB drive, or shared folder (copy `.app` or `.dmg`).
4. **On the other Mac:**  
   - If you copied the **.app:** drag it into **Applications** (or leave on Desktop).  
   - If you copied the **.dmg:** open it and drag **RemoteSync** into Applications.
5. **First launch – “RemoteSync is damaged”:**  
   macOS may quarantine the app. In Terminal on that Mac run:
   ```bash
   xattr -cr /Applications/RemoteSync.app
   ```
   Then open RemoteSync as usual. Alternatively: Right‑click the app → **Open** → **Open** in the dialog.

---

## Usage

1. **On machine A:** Click **Share this Mac (host)**. This machine is now discoverable.
2. **On machine B:** Click **Find other Macs**. When the other machine appears, click **Connect**.
3. After connection:
   - Toggle **Sync clipboard in real time** or use **Send my clipboard** / **Paste from remote**.
   - Use **Send a file** to send a file; the other side can **Save** or **Open** it.
   - Click **Open on other device** to bring the app window to the front on the other machine.

---

## Troubleshooting

| Issue | What to do |
|-------|------------|
| **“RemoteSync is damaged” on macOS** | Run `xattr -cr /Applications/RemoteSync.app` (or the path where the app is), then open again. Or Right‑click → Open → Open. |
| **`xcode-select: error` on macOS** | Install Xcode Command Line Tools: `xcode-select --install`. |
| **`rustc: command not found`** | Install Rust from [rustup.rs](https://rustup.rs) and restart the terminal, or run `source "$HOME/.cargo/env"`. |
| **`npm: command not found`** | Install Node.js from [nodejs.org](https://nodejs.org) and ensure it’s on your PATH. |
| **Tauri build fails on Windows** | Ensure “Desktop development with C++” is installed via Visual Studio Build Tools. |
| **Tauri build fails on Linux** | Install the [system dependencies](#linux) for your distro (e.g. `libwebkit2gtk-4.1-dev` on Debian/Ubuntu). |
| **Dev server port in use** | Change the dev port in `vite.config.ts` or `tauri.conf.json` if 1420 is already used. |

---

## Tech stack

| Layer | Technology |
|-------|------------|
| **Frontend** | React, TypeScript, Vite |
| **Backend** | Tauri 2 (Rust) |
| **Discovery** | mDNS (Bonjour) via `mdns-sd` |
| **Sync** | WebSocket (JSON: clipboard, base64 file chunks, “bring to front”) |
| **Plugins** | clipboard-manager, dialog (file open/save), opener |

---

## License

MIT
<<<<<<< HEAD
=======
>>>>>>> ef07763 (for commiting the changes for file sharing)
>>>>>>> 1320ae5 (for commiting the changes for file sharing)
