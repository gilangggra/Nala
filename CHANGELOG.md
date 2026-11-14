# Changelog

All notable changes to **Aira** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Changed
- **Comprehensive Ecosystem Rebrand to Aira**:
  - Full brand transition from Nala to **Aira** across all platforms (macOS, Windows, Linux, Android, iOS).
  - Windows Rust Core, named pipes (`\\.\pipe\aira-*`), binaries (`aira.exe`, `aira-hook.exe`), and user data `%APPDATA%\Aira` (with zero-loss automatic migration from legacy `%APPDATA%\Nala`).
  - Android package namespace migrated to `com.aira.android` with application class `AiraApp` and widget provider metadata `aira_*_widget_info`.
  - Phone link subprotocol harmonized to `aira.v1`, join proofs `aira.join.*`, and mDNS discovery service type `_aira._tcp`.
  - Cloudflare Relay Server (`android/relay/`) updated to `aira-link-relay` with regenerated and validated cryptographic test vectors.
  - Voice AI engine wake word updated to « OK Aira » / « Hey Aira » / « Hai Aira » with phonetically tuned Whisper and Parakeet matcher.
  - Feature rebrand: Pomodoro timer renamed to **Airadoro** across 16 supported languages.

### Added
- Ongoing improvements for cross-device mobile synchronization and pairing.
- Extended agent integrations and custom hook presets.

---

## [1.0.0] - 2026-11-12

### Added
- **Dual Offline Speech Engines (Whisper Large-v3 Turbo & Parakeet)**:
  - Integrated **OpenAI Whisper Large-v3-Turbo (INT8)** via `sherpa-onnx` on Windows and Linux for high-accuracy **Bahasa Indonesia (`id`)** and 99+ language dictation and voice control alongside **NVIDIA Parakeet TDT**.
  - Dynamic dictation language resolution (`auto` mode automatically pins `"id"` when the Nala interface is in Bahasa Indonesia and uses Whisper native language identification for other UI languages, while supporting explicit selection across all supported languages).
  - Synchronized speech engine and language preferences across **Flow Dictation**, **Call Notetaker**, and **« OK Nala » Voice Control** to prevent model reload thrashing.
- **« OK Nala » Bahasa Indonesia & Accent Support**:
  - Added Indonesian speech synthesis (`id-ID`) language detection for spoken replies, wake word variants, and localized voice commands.
  - Added Brazilian Portuguese and Spanish accent variants for « OK Nala » wake spotting, plus single-press hands-free dictation mode.
- **Agent Hub & Session Workflow**:
  - Approve **Antigravity** and **OpenCode** execution requests directly from the island.
  - Start a new coding session directly from the notch/island by picking a project and typing or dictating a prompt.
  - Drag and drop files directly onto an active agent session card to insert the file into its prompt.

### Changed
- Aligned subscription toggle switches and action buttons in **Settings → Chat → Your AI subscriptions**.
- Updated privacy policy (`docs/privacy.html`), voice architecture docs (`docs/VOICE.md`), and version synchronization checks across all platforms.

---

## [0.5.9] - 2026-11-07

### Added

#### Core Island & Companion (Mochi)
- **Living Desktop Companion**: Physics-based squircle companion (Mochi) that reacts to cursor movements, blinks, breathes, and expresses agent states.
- **Cross-Platform Island**:
  - Native notch companion on macOS with fluid 60–120 fps animations (Swift 6, AppKit, SwiftUI).
  - Smooth floating Dynamic Island at the top of the screen on Windows and Linux (Tauri 2 + Rust).
- **Desktop Companion Mode**: Drag Mochi out of the notch or screen edge to let him float freely on your desktop as an interactive desktop pet.
- **Wardrobe & Customization**: Dynamic outfits and seasonal themes for Mochi.

#### Universal AI Coding Agent Hub
- **Multi-Agent Live Monitoring**: Real-time step tracking for Claude Code, Antigravity, Gemini CLI, Cursor, Codex, Copilot CLI, Muse Code, OpenCode, Amp, Hermes, and custom CLI agents.
- **One-Click Approvals**: Review and decide on execution permission requests (**Allow / Deny / Always**) directly in the notch or island.
- **Interactive Question Prompts**: Support for agent `AskUserQuestion` prompts with single and multi-select choices right from the island.
- **Live Diff Inspector**: Inspect file changes and additions/deletions counts (`+N / -M`) in real time.
- **Terminal Focus**: Jump directly to the active session's terminal window with a single click.
- **Remote Agent Support**: Monitor and approve sessions running on remote servers or SSH containers via SSH tunneling.

#### Flow: Voice Dictation & Call Notetaker
- **Global Push-to-Talk Dictation**: Hold a configurable shortcut key (Right ⌥, fn, Right Ctrl, etc.), speak naturally, and release to paste transcribed text into any active app.
- **Hands-Free & Clipboard Safe**: Double-tap for hands-free recording, Esc to cancel, and automatic restoration of original clipboard contents.
- **Offline Speech Recognition**: Speech recognition running 100% locally on device hardware via `sherpa-onnx` (Whisper Large-v3 Turbo & NVIDIA Parakeet) with zero cloud audio leakage.
- **Private Meeting Notetaker**: Record calls without external meeting bots; delivers speaker diarization, transcripts, summaries, and action items locally or via AI.
- **Vocabulary & Polishing**: Custom dictionary for technical jargon, snippet expansion, and per-app AI text polishing.
- **Local Analytics**: Track words dictated, time saved, typing speed (WPM), and daily streaks locally.

#### Multi-Model AI Chat & Subscription Tunneling
- **Multi-Provider AI Chat**: Chat seamlessly with Anthropic Claude, Google Gemini, OpenAI, DeepSeek, Qwen, Moonshot, and local models (Ollama, LM Studio).
- **Subscription Tunneling**: Connect directly to your existing paid CLI subscriptions (Claude Code, Antigravity, Gemini CLI, Codex, Cursor, Copilot) without requiring extra paid API keys.

#### Mobile Ecosystem
- **Nala on iPhone (iOS 18+)**:
  - Real-time Live Activities and Dynamic Island support on Lock Screen and home screen.
  - Secure biometric command approval using Face ID.
  - End-to-end encrypted synchronization via personal iCloud CloudKit database.
  - Interactive iOS Home Screen widgets, Control Center controls, and Siri integration.
- **Nala for Android (Android 11+)**:
  - Remote permission approvals with fingerprint or lock screen authentication.
  - Floating camera punch-hole island overlay and quick-settings tile.
  - 9 custom home screen widgets with status color indicators.
  - Direct local Wi-Fi pairing with pinned TLS certificates.

#### Productivity & Service Integrations
- **Developer & SaaS Pills**: Live tracking pills for GitHub (PRs, reviews, CI status), Stripe revenue, Vercel deployments, Resend emails, Notion, and Cal.com.
- **Integrated Mail Management**: Fast email reading, smart filtering, and AI sorting for Apple Mail (macOS) and classic Outlook (Windows).
- **WhatsApp Integration**: Read incoming conversations and send replies directly from the notch via local private gateway (GOWA).
- **Media Controls**: Spotify and Apple Music controllers with progress scrubbing and dancing Mochi animations.
- **Crypto Portfolio Pill**: Real-time balance and 7-day sparklines tracking across 16 blockchain networks and 5 crypto exchanges.

#### Privacy & Security
- **100% Local-First Architecture**: No accounts, no telemetry, no tracking servers.
- **OS Credential Storage**: All sensitive API keys and tokens are secured inside macOS Keychain, Windows Credential Manager, or Linux Secret Service.
