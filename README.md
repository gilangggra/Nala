<div align="center">

<img src="docs/media/aira.png" width="108" alt="Aira Icon" style="border-radius: 24px; box-shadow: 0 8px 24px rgba(0,0,0,0.15);">

# Aira

### The Intelligent Desktop Island & Autonomous AI Agent Companion
*Seamlessly monitor AI coding sessions, approve tool permissions, dictate with offline speech models, and stay focused with an ambient desktop companion.*

**Available on macOS, Windows, Linux, iOS, and Android**

<br>

[![Releases](https://img.shields.io/github/v/release/gilangggra/Aira?label=release&color=0A84FF&style=flat-square)](https://github.com/gilangggra/Aira/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-34C759?style=flat-square)](LICENSE)
![macOS](https://img.shields.io/badge/macOS-15%2B-000000?style=flat-square&logo=apple&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-10%2F11-0078D4?style=flat-square&logo=windows&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-AppImage%20%7C%20deb%20%7C%20rpm%20%7C%20nix-FCC624?style=flat-square&logo=linux&logoColor=black)
![iOS](https://img.shields.io/badge/iOS-18%2B-000000?style=flat-square&logo=apple&logoColor=white)
![Android](https://img.shields.io/badge/Android-11%2B-3DDC84?style=flat-square&logo=android&logoColor=white)
[![GitHub stars](https://img.shields.io/github/stars/gilangggra/Aira?style=social)](https://github.com/gilangggra/Aira)

<br>

<img src="docs/media/demo.gif" width="780" alt="Aira in Action" style="border-radius: 12px; box-shadow: 0 12px 32px rgba(0,0,0,0.25);">

<br><br>

[✨ Highlights](#-highlights-at-a-glance) •
[🚀 Quick Start](#-quick-start--installation) •
[🤖 Agent Hub](#-universal-ai-coding-agent-hub) •
[🍅 Airadoro](#-airadoro--smart-focus-companion) •
[🎙️ Flow Voice](#-flow-voice-dictation--call-notetaker) •
[📱 Mobile](#-mobile-companions) •
[📖 Documentation](#-setup--agent-hooks)

</div>

---

## 💡 Why Aira?

Autonomous AI coding agents like **Claude Code, Antigravity, Cursor, Codex, and Gemini CLI** are transforming software engineering. However, constantly switching windows to approve permissions, check terminal diffs, or verify command status fractures your creative flow.

**Aira solves this by bringing your agent's heartbeat directly into your peripheral vision.** 

- **At Your Desk**: Aira lives organically in your MacBook notch or floats as a sleek, non-intrusive desktop island on Windows and Linux. See code diffs live, approve commands with a single click, or chat with AI without leaving your active IDE.
- **Away from Your Desk**: When you step away to grab a coffee, Aira seamlessly transitions to your **iPhone** (Dynamic Island & Live Activities) or **Android** phone. Review diffs and authorize terminal actions remotely with Face ID or fingerprint biometrics.
- **Deep Work Focus**: Boost productivity with **Airadoro** (the integrated Pomodoro companion) and dictate code or notes using **Flow** (offline speech-to-text with zero audio leaving your machine).

100% open source under the MIT License. Zero telemetry, zero cloud tracking, private by design.

---

## ✨ Highlights at a Glance

<table>
  <tr>
    <td width="50%">
      <h3>🤖 Universal Agent Hub</h3>
      <p>Live peripheral monitoring for Claude Code, Antigravity, Cursor, Codex, Gemini CLI, Copilot, and custom agents with real-time <code>+N / -M</code> line diffs.</p>
    </td>
    <td width="50%">
      <h3>✅ Frictionless Approvals</h3>
      <p>Approve tool executions (<code>Allow / Deny / Always</code>) and answer interactive CLI questions directly from the island or via phone biometrics.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>🍅 Airadoro Focus Companion</h3>
      <p>Built-in Pomodoro timer with configurable Work & Rest intervals, zero-drift precision countdowns, and gentle ambient audio cues.</p>
    </td>
    <td width="50%">
      <h3>🎙️ Flow Offline Voice Suite</h3>
      <p>Hold a hotkey anywhere to dictate instantly. Powered by local <b>Whisper Large-v3 Turbo</b> and <b>Parakeet TDT</b> with private meeting recording.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>🧸 Mochi Desktop Companion</h3>
      <p>A delightful, living squircle living in your notch that reacts to your agent's state, celebrates wins, and can float freely across your desktop.</p>
    </td>
    <td width="50%">
      <h3>📱 Mobile Synchrony (iOS & Android)</h3>
      <p>Take your agents with you. iPhone Dynamic Island with Face ID approval, or Android camera island with 9 customizable Home Screen widgets.</p>
    </td>
  </tr>
  <tr>
    <td width="50%">
      <h3>💳 Subscription Tunneling</h3>
      <p>Chat directly with premium AI models using the CLI tool subscriptions you already own (Claude Code, Antigravity, etc.) without extra API key costs.</p>
    </td>
    <td width="50%">
      <h3>🔒 Private by Design</h3>
      <p>No account required, no remote telemetry. Credentials are saved in your OS Keychain / Credential Vault; network communication is local and encrypted.</p>
    </td>
  </tr>
</table>

---

## 🤖 Universal AI Coding Agent Hub

Aira connects natively to your favorite CLI and desktop agents using low-latency local IPC:

<div align="center">
  <img src="docs/media/claude-code.png" width="720" alt="Claude Code Session in Aira" style="border-radius: 8px;">
</div>

- **Step-by-Step Execution Tracking**: Watch files being created, modified, or deleted with real-time progress indicators and diff statistics.
- **One-Click Permission Gates**: Never let your agent stall while waiting for permissions. Grant single-command or permanent execution rights instantly.
- **Interactive Questions**: Respond to agent prompts (`AskUserQuestion`), choose single or multi-select options, and provide free-form text feedback.
- **Jump to Terminal**: Click the terminal icon on the island to instantly bring the exact workspace window running that agent session to the foreground.
- **Multi-Agent Concurrency**: Running multiple tasks across different workspaces? Easily switch between active agent pills in your island.

---

## 🍅 Airadoro — Smart Focus Companion

Engineered specifically for developers and knowledge workers, **Airadoro** integrates classic Pomodoro productivity directly into your desktop island:

<div align="center">
  <img src="docs/media/hub-settings.png" width="680" alt="Airadoro Settings" style="border-radius: 8px;">
</div>

- **Dual-Phase Cycles**: Seamlessly alternates between **Work** (default 25 min) and **Rest** (default 5 min), fully customizable from 1 to 60 minutes in Settings.
- **Zero-Drift Engine**: Uses timestamp-deadline tracking (`Date.now() + remainingMs`) ensuring perfect clock accuracy even if your machine sleeps or throttles background tasks.
- **Ambient Audio & Visual Cues**: Plays gentle chimes on phase transitions, and surfaces the island automatically when work finishes so you never miss a rest period.
- **Non-Intrusive Glanceability**: When collapsed, the remaining time is discreetly displayed next to your active agent pill.

---

## 🎙️ Flow: Voice Dictation & Call Notetaker

A complete local speech intelligence suite built right into your desktop:

<div align="center">
  <img src="docs/media/hub-dictation.png" width="760" alt="Flow Voice Dictation Suite" style="border-radius: 8px;">
</div>

- **Instant Flow Dictation**: Hold your preferred hotkey (e.g., `Right ⌥`, `fn`, or `Right Ctrl`), speak your thoughts naturally, and release. Your speech is transcribed and pasted into the active application instantly.
- **Dual Offline Speech Engines**:
  - **OpenAI Whisper Large-v3 Turbo (INT8)**: High-accuracy transcription supporting Bahasa Indonesia and 99+ global languages.
  - **NVIDIA Parakeet TDT**: Ultra-fast, low-latency transcription across 25 Western languages.
  - *Runs 100% locally via `sherpa-onnx` and hardware acceleration — zero audio data ever leaves your computer.*
- **Private Call Notetaker**: Record calls and meetings without uninvited third-party bots joining your meeting. Captures microphone and system audio cleanly, producing speaker-separated transcripts and actionable summaries.
- **Smart AI Clean-Up**: Automatically fixes punctuation, removes filler words ("um", "uh"), and adapts formatting according to the destination app (Code, Email, Casual).
- **Personal Wordbook & Analytics**: Train Aira on unique technical jargon, function names, and acronyms. Track your typing speed (WPM) and hours saved locally.

<table>
  <tr>
    <td width="33%"><img src="docs/media/hub-statistics.png" alt="Local Stats" style="border-radius: 6px;"><br><sub><b>Local Analytics</b>: Words dictated, time saved, typing speed.</sub></td>
    <td width="33%"><img src="docs/media/hub-styles.png" alt="App Styles" style="border-radius: 6px;"><br><sub><b>Style Profiles</b>: Context-aware AI text formatting.</sub></td>
    <td width="33%"><img src="docs/media/hub-settings.png" alt="Preferences" style="border-radius: 6px;"><br><sub><b>Engine Tuning</b>: Hotkeys, language selection, models.</sub></td>
  </tr>
</table>

---

## 🧸 Meet Mochi — Your Desktop Companion

**Mochi** is your desktop companion: a warm, expressive squircle with lively eyes that lives in your notch or island:

<div align="center">
  <table>
    <tr>
      <td align="center"><img src="docs/media/chat.png" width="340" alt="Chat with Mochi"><br><sub><b>Interactive Chat</b>: Query models with file context</sub></td>
      <td align="center"><img src="docs/media/dizzy.png" width="340" alt="Playful Reactions"><br><sub><b>Expressive Life</b>: Blinks, reacts, and celebrates</sub></td>
    </tr>
  </table>
</div>

- **Living Interaction**: Mochi follows your cursor across the display, waves hello when launched, blinks naturally, and squishes when poked.
- **Agent Awareness**: Celebrates with excitement when tasks finish, alerts you when your agent needs approval, and looks focused while tasks run.
- **Detachable Desktop Pet**: Drag Mochi out of the notch to place him anywhere on your multi-monitor workspace.
- **File Ingestion**: Simply drag and drop files onto Mochi to summarize them, ask questions, or attach them to your active prompts.

---

## 📱 Mobile Companions

Take your workflow with you. When you leave your computer, Aira hands off your active agent sessions to your mobile devices:

<table>
  <tr>
    <th width="50%"><h3>Aira on iPhone (iOS 18+)</h3></th>
    <th width="50%"><h3>Aira for Android (Android 11+)</h3></th>
  </tr>
  <tr>
    <td align="center">
      <img src="docs/media/iphone-live-activity.jpg" width="340" alt="iPhone Live Activity" style="border-radius: 12px;"><br><br>
      <img src="docs/media/iphone-approval.jpg" width="160" alt="Face ID Approval">
      <img src="docs/media/iphone-session.jpg" width="160" alt="iPhone Session">
    </td>
    <td align="center">
      <img src="docs/media/android-island.jpg" width="340" alt="Android Island" style="border-radius: 12px;"><br><br>
      <img src="docs/media/android-allow-deny.jpg" width="160" alt="Biometric Approval">
      <img src="docs/media/android-home.jpg" width="160" alt="Android Widgets">
    </td>
  </tr>
  <tr>
    <td>
      <ul>
        <li>🏝️ <b>Dynamic Island & Live Activities</b>: Follow real-time agent output right on your lock screen.</li>
        <li>🔐 <b>Face ID Approvals</b>: Authorize terminal commands with a biometric glance.</li>
        <li>☁️ <b>iCloud CloudKit Sync</b>: Private end-to-end encryption with zero third-party servers.</li>
        <li>🧩 <b>Home Screen Widgets & Siri</b>: Instant glanceability and voice shortcuts.</li>
      </ul>
      <p align="center"><i><a href="docs/IPHONE.md">Read iPhone Setup Guide →</a></i></p>
    </td>
    <td>
      <ul>
        <li>🏝️ <b>Punch-Hole Camera Island</b>: Dynamic floating overlay around your selfie camera.</li>
        <li>🔐 <b>Fingerprint Approvals</b>: Instant biometric confirmation over any app or lockscreen.</li>
        <li>📶 <b>Local Wi-Fi Pairing</b>: Direct peer-to-peer connection with TLS pinned certificates.</li>
        <li>🧩 <b>9 Custom Widgets</b>: Agent meters, plan usage gauges, and quick actions.</li>
      </ul>
      <p align="center"><i><a href="android/README.md">Read Android Setup Guide →</a></i></p>
    </td>
  </tr>
</table>

---

## 🌐 Platform Support Matrix

| Feature | macOS (15+) | Windows (10/11) | Linux (X11/Wayland) | iOS (18+) | Android (11+) |
|---|:---:|:---:|:---:|:---:|:---:|
| **Notch / Island Interface** | Native Notch | Floating Island | Layer-Shell / Island | Dynamic Island | Punch-Hole Island |
| **Agent Monitoring & Diffs** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **1-Click Approvals** | ✅ | ✅ | ✅ | ✅ *(Face ID)* | ✅ *(Fingerprint)* |
| **Airadoro Pomodoro Timer** | ✅ | ✅ | ✅ | ✅ *(Widget)* | ✅ *(Widget)* |
| **Flow Voice Dictation** | ✅ | ✅ | ✅ | — | — |
| **Offline Speech Recognition** | ✅ *(Whisper/Parakeet)* | ✅ *(Whisper/Parakeet)* | ✅ *(Whisper/Parakeet)* | — | — |
| **Private Call Notetaker** | ✅ | ✅ | ✅ | — | — |
| **Subscription Tunneling** | ✅ | ✅ | ✅ | — | — |
| **Mochi Companion** | Full Physics | Desktop Float | Desktop Float | Avatar | Avatar |
| **Productivity Integrations** | ✅ | ✅ | ✅ | Read-Only | Read-Only |

---

## 🚀 Quick Start & Installation

### macOS

#### Via Homebrew (Recommended)
```bash
brew tap gilangggra/aira https://github.com/gilangggra/Aira
brew install --cask aira
```

To update anytime:
```bash
brew upgrade --cask --greedy aira
```

#### Direct Download
1. Grab the latest `Aira.zip` from [GitHub Releases](https://github.com/gilangggra/Aira/releases).
2. Unzip and drag `Aira.app` into your `/Applications` folder.
3. Launch Aira and grant Accessibility & Microphone permissions when prompted for Flow dictation.

---

### Windows

Download the installer from [GitHub Releases](https://github.com/gilangggra/Aira/releases):

- **[Aira-Windows-setup.exe](https://github.com/gilangggra/Aira/releases/download/windows-latest/Aira-Windows-setup.exe)** — User-level installer (no administrator privileges required).
- **[Aira-Windows.msi](https://github.com/gilangggra/Aira/releases/download/windows-latest/Aira-Windows.msi)** — System-wide enterprise Windows installer.

> [!NOTE]
> **Windows SmartScreen**: If Windows displays an "Unknown publisher" notice during initial release, click **More info** → **Run anyway**. Aira is open source, clean, and pre-scanned by Microsoft Defender.

---

### Linux

Download universal binaries from [GitHub Releases](https://github.com/gilangggra/Aira/releases):

- **AppImage (Universal)**:
  ```bash
  chmod +x Aira-Linux-*.AppImage
  ./Aira-Linux-*.AppImage
  ```
- **Debian / Ubuntu (.deb)**:
  ```bash
  sudo apt install ./Aira-Linux-*.deb
  ```
- **Fedora / openSUSE (.rpm)**:
  ```bash
  sudo dnf install ./Aira-Linux-*.rpm
  ```
- **Arch Linux**: Build using the included [PKGBUILD](linux/arch/PKGBUILD).
- **NixOS**: Run directly via `nix run github:gilangggra/Aira` or import the provided flake.

---

### Mobile Apps

- **iOS / iPhone**: Join our [TestFlight Beta](https://testflight.apple.com/join/3GpeHv2b).
- **Android**: Download `Aira-Android.apk` from [GitHub Releases](https://github.com/gilangggra/Aira/releases) and pair via Settings → Android Phone.

---

## 🛠️ Setup & Agent Hooks

Configuring Aira with your developer tools takes less than a minute. Open **Settings** (via Dock/Menu bar on macOS, or System Tray on Windows/Linux):

| Tool | Capability | Setup Action |
|---|---|---|
| **Claude Code** | Real-time monitoring & approvals | Click **Install hooks** (backs up `~/.claude/settings.json`) |
| **Antigravity (agy)** | Notch alerts & session tracking | Settings → Antigravity → **Install hooks** |
| **Cursor** | Automatic background hook detection | Detected automatically when Cursor is running |
| **Gemini CLI** | Session progress & tool monitoring | Settings → Gemini CLI → **Install hooks** |
| **Codex CLI** | Execution monitoring & approvals | Settings → Codex → **Install hooks** |
| **Airadoro** | Pomodoro focus intervals | Settings → Active Pills → Airadoro |
| **Flow Dictation** | Offline voice input | Settings → Flow → Select hotkey & language |
| **External Integrations** | GitHub, Stripe, Vercel, Resend | Settings → Active Pills |

> [!TIP]
> Aira hooks are designed with zero performance overhead. If Aira is closed, all hook relays exit immediately without blocking your terminal sessions.

---

## 🛡️ Architecture & Privacy First

```
┌────────────────────────────────────────────────────────┐
│                      User Action                       │
│    (Notch Hover, Flow Hotkey, File Drop, Approvals)    │
└───────────────────────────┬────────────────────────────┘
                            │
            ┌───────────────┴───────────────┐
            ▼                               ▼
┌───────────────────────┐       ┌───────────────────────┐
│     macOS Engine      │       │ Windows/Linux Engine  │
│  (Swift 6 / AppKit /  │       │  (Tauri 2 / Rust /    │
│       SwiftUI)        │       │      TypeScript)      │
└───────────┬───────────┘       └───────────┬───────────┘
            │                               │
            ├───────────────┬───────────────┤
            ▼               ▼               ▼
┌───────────────────┐ ┌───────────┐ ┌───────────────────┐
│ Agent IPC Relays  │ │ Flow Voice│ │  Mobile Link      │
│ (Unix Sockets /   │ │ (Whisper /│ │ (iCloud CloudKit /│
│   Named Pipes)    │ │  Parakeet)│ │  Local TLS Wi-Fi) │
└───────────────────┘ └───────────┘ └───────────────────┘
```

- **Local-First Communication**: Desktop agents communicate with Aira via standard local Unix Domain Sockets (`/tmp/aira.sock`) or Windows Named Pipes (`\\.\pipe\aira-*`).
- **Encrypted Secret Storage**: All API tokens and credentials remain strictly on your machine, guarded by Apple Keychain, Windows Credential Manager, or Linux Secret Service.
- **Zero Cloud Processing for Audio**: Flow dictation transcribes speech locally on your GPU/NPU/CPU.
- **No Third-Party Analytics**: No telemetry, analytics SDKs, or background tracking pixels exist anywhere in the code.

---

## 🎮 Interactive Gestures

| Gesture / Trigger | Mochi's Response |
|---|---|
| **Hover Notch / Top Edge** | Peeks out and waves 👋 |
| **Click the Island** | Expands full overview panel |
| **Hover over Mochi** | Blinks curiously and pupils dilate |
| **Click Mochi** | Squishes with a playful emote |
| **Triple-Click Rapidly** | 😵‍💫 Becomes temporarily dizzy |
| **Drag & Drop File onto Island** | Transforms into an intake box and ingests file |
| **Hold Dictation Hotkey** | Starts real-time speech transcription |
| **Say « OK Aira »** | Activates voice command listener |

---

## 🤝 Contributing

We welcome contributions from developers, designers, and AI enthusiasts worldwide!

- Check out our [CONTRIBUTING.md](CONTRIBUTING.md) guide to get started.
- Help localize Aira in your language via `NotchBuddy/Resources/Localizable.xcstrings` (macOS) and `windows/src/i18n/` (Windows/Linux).
- Submit feature requests or bug reports via [GitHub Issues](https://github.com/gilangggra/Aira/issues).

---

## 👤 Author & Maintainer

Created with ❤️ by **[Gilang Ramadan](https://github.com/gilangggra)**.

---

## 📄 License

- **Source Code**: Released under the [MIT License](LICENSE) © 2026 Gilang Ramadan.
- **Character, Artwork & Audio Assets**: © Gilang Ramadan, all rights reserved (see [LICENSE-ASSETS.md](LICENSE-ASSETS.md)).

<div align="center">

**If Aira enhances your daily workflow, consider starring the repository ⭐!**

</div>
