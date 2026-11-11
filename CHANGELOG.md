# Changelog

All notable changes to **Nala** will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0/).

## [Unreleased]

### Added
- Ongoing improvements for cross-device mobile synchronization and pairing.
- Extended agent integrations and custom hook presets.

---

## [1.0.0] - 2026-11-11

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
- **Offline Parakeet Engine**: Speech recognition running 100% locally on device hardware via NVIDIA Parakeet across 25 languages with zero cloud audio leakage.
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
