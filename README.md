<div align="center">

<img src="docs/media/aira.png" width="104" alt="Aira" style="border-radius: 22px; box-shadow: 0 10px 30px rgba(0,0,0,0.18);">

# AIRA

### The Ambient Intelligence Layer for Autonomous AI Coding Agents
**Keep agent diffs in your peripheral vision, approve tool permissions remotely with biometrics, dictate offline, and stay in deep flow.**

```
macOS (Native Swift 6) • Windows & Linux (Tauri 2 & Rust) • iOS (Dynamic Island) • Android (Punch-Hole Island)
```

<br>

[![Latest Release](https://img.shields.io/github/v/release/gilangggra/Aira?style=for-the-badge&color=0A84FF&label=Release)](https://github.com/gilangggra/Aira/releases)
[![License](https://img.shields.io/badge/License-MIT-34C759?style=for-the-badge)](LICENSE)
[![Stars](https://img.shields.io/github/stars/gilangggra/Aira?style=for-the-badge&color=F59E0B)](https://github.com/gilangggra/Aira)
[![Offline Speech](https://img.shields.io/badge/Speech-100%25%20Offline-blueviolet?style=for-the-badge)](docs/VOICE.md)
[![Privacy](https://img.shields.io/badge/Telemetry-Zero-black?style=for-the-badge)](docs/privacy.html)

<br>

<img src="docs/media/demo.gif" width="820" alt="Aira Ambient Island in Action" style="border-radius: 14px; box-shadow: 0 16px 40px rgba(0,0,0,0.28);">

<br><br>

> **⚡ Quick Install on macOS**: `brew tap gilangggra/aira https://github.com/gilangggra/Aira && brew install --cask aira`  
> **⚡ Quick Install on Windows**: [Download Aira-Windows-setup.exe](https://github.com/gilangggra/Aira/releases/download/windows-latest/Aira-Windows-setup.exe)

</div>

---

## 🧭 The 3 Pillars of Aira

Aira is designed to eliminate the friction of working alongside autonomous coding agents. Instead of switching windows or stalling execution, Aira brings intelligence into your peripheral vision:

```
                  ┌──────────────────────────────────────────────┐
                  │                 AIRA CORE                    │
                  └──────┬────────────────┬──────────────┬───────┘
                         │                │              │
           ┌─────────────▼────┐    ┌──────▼──────┐   ┌───▼──────────────┐
           │  1. DESKTOP      │    │  2. MOBILE  │   │  3. LOCAL VOICE  │
           │     ISLAND       │    │     RELAY   │   │     & FOCUS      │
           ├──────────────────┤    ├─────────────┤   ├──────────────────┤
           │ • Agent Diffs    │    │ • iPhone DI │   │ • Flow Dictation │
           │ • 1-Click Allow  │    │ • Android   │   │ • Whisper INT8   │
           │ • Terminal Jump  │    │ • Face ID   │   │ • Airadoro Timer │
           │ • Mochi Mascot   │    │ • P2P Sync  │   │ • Meeting Notes  │
           └──────────────────┘    └─────────────┘   └──────────────────┘
```

---

## ⚡ The Developer Workflow Experience

### 1. In the Zone: Desktop Ambient Island
When you run autonomous CLI agents like **Claude Code, Antigravity, Cursor, Codex, or Gemini CLI**, context-switching to your terminal breaks focus. Aira nests natively in your MacBook notch or hovers as an ultra-smooth floating dock on Windows and Linux.

<div align="center">
  <img src="docs/media/claude-code.png" width="740" alt="Claude Code live diff tracking" style="border-radius: 8px;">
</div>

- **Peripheral Execution Awareness**: Watch files being generated, modified, and saved in real-time with live `+added / -removed` line diff indicators.
- **Instant Decision Gates**: When an agent requests bash execution, file overwrites, or tool approvals, authorize with **Allow**, **Deny**, or **Always** with a single click.
- **Interactive Prompts**: Direct support for `AskUserQuestion` dialogs with single and multi-select choices without touching the terminal.
- **Focus Teleportation**: Click the terminal badge on any active pill to instantly bring the exact shell window and project directory to the front.

---

### 2. Deep Work: Airadoro Focus Companion
Coding with autonomous AI requires rhythm and recovery. **Airadoro** embeds an intelligent Pomodoro focus companion directly inside the island:

<div align="center">
  <img src="docs/media/hub-settings.png" width="700" alt="Airadoro and Island Controls" style="border-radius: 8px;">
</div>

- **Engineered for Deep Flow**: Configurable Work cycles (default 25 min) and Rest periods (default 5 min), fine-tunable from 1 to 60 minutes.
- **Zero-Drift Countdown**: Computes remaining time against hard timestamps (`Date.now() + durationMs`), maintaining exact second precision across system sleep and thread suspensions.
- **Glanceable Status**: Discreetly tracks countdown progress next to your active agent pill.
- **Automated Surfacing**: Gently reveals the island panel and plays an ambient alert chime when your sprint completes.

---

### 3. Stepping Away: Mobile Biometric Handoff
Never block a multi-file build or refactor because you stepped away to make coffee. Aira mirrors your desktop sessions directly to your mobile devices in real time:

<table>
  <tr>
    <td width="50%" align="center">
      <b>iPhone (iOS 18+)</b><br>
      <img src="docs/media/iphone-live-activity.jpg" width="320" alt="iPhone Live Activity" style="border-radius: 12px; margin-top: 8px;"><br>
      <sub>Dynamic Island • Live Activity • <b>Face ID Approval</b></sub>
    </td>
    <td width="50%" align="center">
      <b>Android (11+)</b><br>
      <img src="docs/media/android-island.jpg" width="320" alt="Android Floating Island" style="border-radius: 12px; margin-top: 8px;"><br>
      <sub>Punch-Hole Island • <b>Fingerprint Approval</b> • 9 Widgets</sub>
    </td>
  </tr>
</table>

- **Hardware-Level Biometric Authorizations**: Verify terminal commands using Apple Face ID / Touch ID or Android BiometricPrompt. Decisions are cryptographically validated by your computer.
- **Private Transport**:
  - **iOS**: End-to-end synchronized via your personal **iCloud CloudKit** container with zero intermediary servers.
  - **Android**: Direct local Wi-Fi peer-to-peer pairing using mutual TLS with pinned certificates, completely offline.

---

### 4. Capturing Thoughts: Flow Voice & Meeting Notetaker
Speak your mind faster than you can type. Aira includes a full offline speech recognition suite built natively into the app:

<div align="center">
  <img src="docs/media/hub-dictation.png" width="740" alt="Flow Voice Dictation Suite" style="border-radius: 8px;">
</div>

- **Global Push-to-Talk**: Hold your chosen shortcut anywhere on your operating system (`Right ⌥`, `fn`, or `Right Ctrl`), speak naturally, and release: your words are instantly pasted into your active app.
- **Dual Offline Speech Engines**:
  - 🧠 **OpenAI Whisper Large-v3 Turbo (INT8)**: High precision across 99+ languages, including native support for Bahasa Indonesia.
  - ⚡ **NVIDIA Parakeet TDT**: Sub-second, ultra-low latency transcription across 25 Western languages.
  - *All inference runs 100% locally on your device via `sherpa-onnx` and neural acceleration. Zero audio bytes ever touch the internet.*
- **Private Meeting Notetaker**: Record calls and conference meetings with clean microphone and system audio capture. Generates structured transcripts and action items locally without inviting awkward bot participants.
- **Context-Aware Polishing**: Automatically cleans up speech disfluencies and applies context-specific styling for code comments, email drafts, or markdown notes.

<table>
  <tr>
    <td width="33%"><img src="docs/media/hub-statistics.png" alt="Statistics" style="border-radius: 6px;"><br><sub><b>Local Analytics</b>: Words dictated, typing speed (WPM), time saved.</sub></td>
    <td width="33%"><img src="docs/media/hub-styles.png" alt="Styles" style="border-radius: 6px;"><br><sub><b>App Profiles</b>: Smart text clean-up custom to each active app.</sub></td>
    <td width="33%"><img src="docs/media/hub-settings.png" alt="Settings" style="border-radius: 6px;"><br><sub><b>Hardware Tuning</b>: Model selection, hotkeys, and mic gain.</sub></td>
  </tr>
</table>

---

### 5. Meet Mochi: The Living Squircle Companion
Resting at the center of the island is **Mochi**—a charming desktop mascot that makes coding with autonomous agents feel alive and responsive:

<div align="center">
  <table>
    <tr>
      <td align="center"><img src="docs/media/chat.png" width="340" alt="Chat with Mochi"><br><sub><b>Contextual Chat</b>: Ask questions about open diffs or dropped files</sub></td>
      <td align="center"><img src="docs/media/dizzy.png" width="340" alt="Expressive Emotes"><br><sub><b>Reactive Emotes</b>: Celebrates successes and alerts on blockers</sub></td>
    </tr>
  </table>
</div>

- **Interactive Physics**: Mochi's gaze tracks your mouse cursor across screens, blinks naturally, and squishes when tapped.
- **Agent State Awareness**: Celebrates when tasks complete, enters a thinking trance during long operations, and gets dizzy when clicked repeatedly.
- **Drag Anywhere**: Detach Mochi from the notch to float as an interactive desktop pet anywhere on your multi-monitor setup.
- **Drag-and-Drop Ingestion**: Drop files onto Mochi to summarize documents, inspect code, or attach context to your prompts.

---

## 🤖 Supported Agent Integrations

Aira hooks into your developer environment via ultra-lightweight, zero-overhead IPC relays:

| Agent / Tool | Integration Mode | Supported Platforms | Quick Setup |
|---|---|---|---|
| **Claude Code** | Native settings hook | macOS • Windows • Linux | Settings → Claude Code → **Install hooks** |
| **Antigravity (`agy`)** | Native configuration hook | macOS • Windows • Linux | Settings → Antigravity → **Install hooks** |
| **Cursor** | Automatic background hook | macOS • Windows • Linux | Automatically hooked on launch |
| **Codex CLI** | `--agent codex` parameter / hook | macOS • Windows • Linux | Settings → Codex → **Install hooks** |
| **Gemini CLI** | Native settings hook | macOS • Windows • Linux | Settings → Gemini CLI → **Install hooks** |
| **Copilot CLI** | `--agent copilot` flag | macOS • Windows • Linux | Pass flag or configure hook |
| **Custom Agents** | Generic JSON socket relay | macOS • Windows • Linux | See [docs/AGENTS.md](docs/AGENTS.md) |

> [!NOTE]
> **Subscription Tunneling**: Aira can tunnel chat prompts directly through your authenticated local CLI tools (Claude Code, Antigravity, Gemini CLI, etc.). Chat with state-of-the-art models using the subscriptions you already pay for—no separate API bills required.

---

## 📦 Installation & Setup

### macOS
Requirements: macOS 15.0+ (Apple Silicon or Intel)

```bash
# Install via Homebrew Cask
brew tap gilangggra/aira https://github.com/gilangggra/Aira
brew install --cask aira
```
*Or download `Aira.zip` directly from [GitHub Releases](https://github.com/gilangggra/Aira/releases).*

---

### Windows
Requirements: Windows 10 / 11 (64-bit)

Download the installer from [GitHub Releases](https://github.com/gilangggra/Aira/releases):
- **[Aira-Windows-setup.exe](https://github.com/gilangggra/Aira/releases/download/windows-latest/Aira-Windows-setup.exe)** — Portable / User-level installer (no administrator privileges needed).
- **[Aira-Windows.msi](https://github.com/gilangggra/Aira/releases/download/windows-latest/Aira-Windows.msi)** — System-wide installer.

---

### Linux
Requirements: X11 or Wayland (supporting `gtk-layer-shell` on KDE Plasma, Hyprland, Sway, COSMIC)

- **Universal AppImage**:
  ```bash
  chmod +x Aira-Linux-*.AppImage && ./Aira-Linux-*.AppImage
  ```
- **Debian / Ubuntu**: `sudo apt install ./Aira-Linux-*.deb`
- **Fedora / openSUSE**: `sudo dnf install ./Aira-Linux-*.rpm`
- **NixOS**: `nix run github:gilangggra/Aira`

---

### Mobile Companions
- **iPhone (iOS 18+)**: Join the public [Aira TestFlight Beta](https://testflight.apple.com/join/3GpeHv2b).
- **Android (11+)**: Download `Aira-Android.apk` from [GitHub Releases](https://github.com/gilangggra/Aira/releases).

---

## 🔒 Security, Privacy & Architecture

```
┌────────────────────────────────────────────────────────┐
│                   Aira Desktop App                     │
│  (macOS: Swift 6 / SwiftUI • Win/Linux: Tauri 2 / Rust)│
└───────────────────────────┬────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            ▼               ▼               ▼
┌───────────────────┐ ┌───────────┐ ┌───────────────────┐
│ Local Agent Pipes │ │ Flow Voice│ │  Mobile Sync      │
│  Unix Sockets /   │ │ Whisper / │ │ (iCloud CloudKit /│
│   Named Pipes     │ │  Parakeet │ │  TLS Local P2P)   │
└───────────────────┘ └───────────┘ └───────────────────┘
```

- **Local IPC Architecture**: All agent communications take place over local Named Pipes (`\\.\pipe\aira-*`) or Unix Domain Sockets (`/tmp/aira.sock`). No agent tokens or code diffs are ever broadcast over external networks.
- **Zero Cloud Middleman for Audio**: Flow dictation loads ONNX models directly into your computer's RAM. Audio transcription happens locally on your machine.
- **Hardware Credential Isolation**: API keys and secrets are securely anchored in your operating system's native vault: **Apple Keychain**, **Windows Credential Manager**, or **Linux Secret Service**.
- **No Telemetry**: No third-party tracking, analytics SDKs, or behavioral data collection.

---

## 🎮 Interactive Cheat Sheet

| Trigger / Action | Desktop Response |
|---|---|
| **Hover Notch / Top Edge** | Peeks out and waves 👋 |
| **Click Island** | Expands full session inspection panel |
| **Click Mochi** | Squishes with an expressive animated emote |
| **Triple-Click Mochi** | 😵‍💫 Becomes dizzy for a few seconds |
| **Drag & Drop Files onto Island** | Ingests file and opens AI query drawer |
| **Hold Dictation Hotkey** | Starts offline voice capture; pastes upon release |
| **Say « OK Aira »** | Activates multilingual voice assistant mode |

---

## 🤝 Contributing

Aira is proud to be community-driven and open source. We welcome bug reports, translations, character emotes, and new integrations:

- Review our [CONTRIBUTING.md](CONTRIBUTING.md) to explore the development setup.
- Contribute localizations in `NotchBuddy/Resources/Localizable.xcstrings` (macOS) and `windows/src/i18n/` (Windows & Linux).
- Share ideas or submit pull requests on [GitHub](https://github.com/gilangggra/Aira).

---

## 📄 License & Credits

- **Source Code**: [MIT License](LICENSE) © 2026 Gilang Ramadan.
- **Character, Mascot & Brand Assets**: © Gilang Ramadan, all rights reserved (see [LICENSE-ASSETS.md](LICENSE-ASSETS.md)).

<div align="center">

**If Aira brings joy and focus to your development workflow, please give us a ⭐ on GitHub!**

</div>
