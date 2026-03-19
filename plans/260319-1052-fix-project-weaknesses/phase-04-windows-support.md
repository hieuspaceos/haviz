---
phase: 4
title: "Windows Support"
priority: HIGH
status: pending
effort: 16h
depends_on: [3]
---

# Phase 4: Windows Support

## Context Links

- [agent/src/platform/mod.rs](../../agent/src/platform/mod.rs) — only `#[cfg(target_os = "macos")] pub mod macos;`
- [agent/src/platform/macos/accessibility.rs](../../agent/src/platform/macos/accessibility.rs) — AX API reader (Swift helper)
- [agent/src/platform/macos/automation.rs](../../agent/src/platform/macos/automation.rs) — AppleScript send_message
- [agent/src/channels/traits.rs](../../agent/src/channels/traits.rs) — ChannelReader + ChannelSender traits
- [agent/src/channels/zalo_desktop.rs](../../agent/src/channels/zalo_desktop.rs) — macOS Zalo desktop reader
- [ARCHITECTURE.md](../../ARCHITECTURE.md) — planned `windows/uiautomation.rs` + `windows/input.rs`
- [agent/src/bin/haviz_app.rs](../../agent/src/bin/haviz_app.rs) — wry WebViewBuilderExtDarwin import (macOS-only)

## Overview

Agent only works on macOS. Windows has 0 LOC but most Vietnamese salespeople use Windows. `platform/mod.rs` only declares `macos` module. The macOS implementation uses AX API (Swift helper binary) for reading and AppleScript for sending — neither exists on Windows.

Windows equivalent:
- **Reading**: Win32 UI Automation API to traverse Zalo Desktop's UI tree
- **Sending**: `SendInput` API for keystrokes + clipboard paste
- **Zalo Desktop on Windows**: Electron-based app with standard Win32 UI Automation support

## Key Insights

- Zalo Desktop on Windows is Electron — UI Automation API can read its accessibility tree
- Win32 UI Automation is analogous to macOS AX API — both traverse hierarchical UI elements
- `SendInput` API replaces AppleScript for keystrokes (Ctrl+F search, Ctrl+V paste, Enter)
- The `windows-rs` crate provides safe Rust bindings for Win32 APIs including UI Automation
- `wry` and `tao` already support Windows — webview code needs minimal changes
- `haviz_app.rs` uses `#[cfg(target_os = "macos")]` for Darwin-specific WebView extensions — need `#[cfg(target_os = "windows")]` branches
- Cross-compilation: can develop on Windows, CI builds both platforms

## Requirements

### Functional
- Read messages from Zalo Desktop (Windows) via UI Automation API
- Send messages via Zalo Desktop (Windows) using keyboard simulation
- Implement `ChannelReader` and `ChannelSender` traits for Windows
- Desktop app (wry/tao) runs on Windows with same layout
- Zalo Desktop search, open conversation, paste + send workflow

### Non-Functional
- Same polling interval (3s) as macOS
- Message parsing produces identical `ParsedMessage` structs
- No macOS-specific code compiled on Windows (proper `#[cfg]` gating)
- Memory usage <500MB
- Build time <5min on Windows

## Architecture

### Platform Abstraction

```
agent/src/platform/
├── mod.rs                         # cfg-gate: macos | windows
├── macos/
│   ├── mod.rs
│   ├── accessibility.rs           # AX API (existing)
│   ├── automation.rs              # AppleScript (existing)
│   └── osascript.rs              # Helper (Phase 3)
└── windows/
    ├── mod.rs                     # Re-exports
    ├── uiautomation.rs           # Win32 UI Automation API (~180 LOC)
    └── input.rs                   # SendInput keyboard simulation (~120 LOC)
```

### Channel Implementation

```
agent/src/channels/
├── mod.rs                         # cfg-gate channel selection
├── traits.rs                      # ChannelReader + ChannelSender (existing)
├── zalo_desktop.rs               # macOS impl (existing, rename to zalo_desktop_macos.rs)
├── zalo_desktop_windows.rs       # NEW: Windows impl
└── zalo_web.rs                    # Cross-platform (existing)
```

### UI Automation Tree (Zalo Desktop Windows)

```
Zalo.exe (Electron)
└── Window "Zalo"
    └── Pane (main container)
        ├── Pane (sidebar — conversation list)
        │   └── List
        │       └── ListItem[] (each conversation)
        │           ├── Text (contact name)
        │           ├── Text (last message preview)
        │           └── Text (timestamp)
        └── Pane (chat area)
            └── Group (message container)
                └── Group[] (each message)
                    ├── Text (sender name)
                    ├── Text (message content)
                    └── Text (timestamp)
```

## Related Code Files

### Create
- `agent/src/platform/windows/mod.rs` — re-exports
- `agent/src/platform/windows/uiautomation.rs` — UI Automation reader
- `agent/src/platform/windows/input.rs` — SendInput keyboard simulation
- `agent/src/channels/zalo_desktop_windows.rs` — Windows ChannelReader/Sender impl

### Modify
- `agent/src/platform/mod.rs` — add `#[cfg(target_os = "windows")] pub mod windows;`
- `agent/src/channels/mod.rs` — cfg-gate channel selection for macOS vs Windows
- `agent/src/channels/zalo_desktop.rs` — rename to `zalo_desktop_macos.rs` (clarity)
- `agent/src/bin/haviz_app.rs` — cfg-gate macOS-specific WebView extensions
- `agent/Cargo.toml` — add `windows-rs` dependency (target-gated)
- `agent/src/config.rs` — platform-specific defaults (e.g., db_path on Windows)

### Delete
- None

## Implementation Steps

### Step 1: Add Windows dependencies (30min)
1. Update `agent/Cargo.toml`:
   ```toml
   [target.'cfg(windows)'.dependencies]
   windows = { version = "0.58", features = [
     "Win32_UI_Accessibility",
     "Win32_UI_Input_KeyboardAndMouse",
     "Win32_System_Com",
     "Win32_Foundation",
     "Win32_UI_WindowsAndMessaging",
   ]}
   ```
2. Verify cross-compilation setup or native Windows build

### Step 2: Platform module cfg-gating (30min)
1. Update `agent/src/platform/mod.rs`:
   ```rust
   #[cfg(target_os = "macos")]
   pub mod macos;
   #[cfg(target_os = "windows")]
   pub mod windows;
   ```
2. Create `agent/src/platform/windows/mod.rs`:
   ```rust
   pub mod uiautomation;
   pub mod input;
   ```

### Step 3: Implement UI Automation reader (5h)
1. Create `agent/src/platform/windows/uiautomation.rs`:
   - Initialize COM with `CoInitializeEx`
   - Create `IUIAutomation` instance
   - Find Zalo window: `FindWindow` by class name or title "Zalo"
   - Get root element from window handle
   - Traverse UI tree to find message elements:
     - Walk children of chat pane
     - Extract: sender name (Text element), content (Text element), timestamp (Text element)
   - Return `Vec<RawUiElement>` with extracted text
   - Handle: Zalo not running → return error "zalo_not_running"
   - Handle: window minimized → still accessible via UI Automation
2. Key functions:
   - `pub fn find_zalo_window() -> Result<HWND, String>`
   - `pub fn read_chat_messages(hwnd: HWND) -> Result<Vec<RawUiElement>, String>`
   - `pub fn read_conversation_list(hwnd: HWND) -> Result<Vec<ConversationItem>, String>`

### Step 4: Implement SendInput keyboard simulation (3h)
1. Create `agent/src/platform/windows/input.rs`:
   - `pub fn send_keys(text: &str) -> Result<(), String>` — type text using SendInput
   - `pub fn send_hotkey(modifiers: &[u16], key: u16) -> Result<(), String>` — Ctrl+F, Ctrl+V, etc.
   - `pub fn set_clipboard(text: &str) -> Result<(), String>` — copy text to clipboard
   - `pub fn press_enter() -> Result<(), String>`
   - `pub fn focus_window(hwnd: HWND) -> Result<(), String>` — bring Zalo to foreground
2. Key workflows (mirroring macOS AppleScript):
   - **Search contact**: focus Zalo → Ctrl+F → type name → Enter → wait
   - **Send message**: focus chat input → set clipboard → Ctrl+V → Enter
   - **Random delays**: use same jitter constants from safety spec

### Step 5: Implement Windows ChannelReader (2h)
1. Rename `agent/src/channels/zalo_desktop.rs` → `zalo_desktop_macos.rs`
2. Create `agent/src/channels/zalo_desktop_windows.rs`:
   - `pub struct ZaloDesktopWindows { config: Config }`
   - Implement `ChannelReader`:
     - `read_messages()` — call `uiautomation::read_chat_messages()`
     - Parse into `ParsedMessage` using `message_parser::compute_hash` and `determine_direction`
   - Implement `ChannelSender`:
     - `send_message(to, message)` — call input functions: search contact → paste → send
3. Update `agent/src/channels/mod.rs`:
   ```rust
   #[cfg(target_os = "macos")]
   pub mod zalo_desktop_macos;
   #[cfg(target_os = "windows")]
   pub mod zalo_desktop_windows;

   // Factory function
   pub fn create_zalo_desktop_channel(config: &Config) -> Box<dyn ChannelReader + Send> {
       #[cfg(target_os = "macos")]
       { Box::new(zalo_desktop_macos::ZaloDesktopMacos::new(config)) }
       #[cfg(target_os = "windows")]
       { Box::new(zalo_desktop_windows::ZaloDesktopWindows::new(config)) }
   }
   ```

### Step 6: Platform-gate haviz_app.rs (1.5h)
1. In `haviz_app.rs`, gate macOS-specific code:
   ```rust
   #[cfg(target_os = "macos")]
   use wry::WebViewBuilderExtDarwin;
   ```
2. Gate `with_data_store_identifier()` (macOS-only):
   ```rust
   #[cfg(target_os = "macos")]
   let sidebar_builder = sidebar_builder.with_data_store_identifier(store_id);
   ```
3. In Zalo control handlers, use platform abstraction instead of direct AppleScript calls
4. Update `config.rs` for Windows-specific paths:
   - DB path: `%APPDATA%\Haviz\haviz.db` (via `dirs::data_dir()` — already works cross-platform)

### Step 7: Cross-platform build verification (1.5h)
1. On Windows: `cargo build` — verify compilation
2. On macOS: `cargo build` — verify no regressions
3. Test Windows-specific code paths:
   - Zalo Desktop not running → graceful error
   - Zalo Desktop running → read messages
   - Send message workflow
4. Verify webview works on Windows (wry supports it natively)

### Step 8: CI cross-compilation (1h)
1. Add Windows build target to planned CI pipeline (Phase 5)
2. Document build requirements:
   - Windows: MSVC toolchain, Visual Studio Build Tools
   - Cross-compile from Linux: `x86_64-pc-windows-msvc` target (or use Windows runner)

## Todo List

- [ ] Add windows-rs dependency to Cargo.toml (target-gated)
- [ ] Update platform/mod.rs with Windows cfg-gate
- [ ] Create platform/windows/mod.rs
- [ ] Implement uiautomation.rs — UI tree traversal for Zalo Desktop
- [ ] Implement input.rs — SendInput keyboard simulation
- [ ] Rename zalo_desktop.rs to zalo_desktop_macos.rs
- [ ] Create zalo_desktop_windows.rs implementing ChannelReader + ChannelSender
- [ ] Update channels/mod.rs with platform factory function
- [ ] Cfg-gate macOS-specific code in haviz_app.rs
- [ ] Verify compilation on Windows
- [ ] Verify no macOS regressions
- [ ] Smoke test: read messages from Zalo Desktop (Windows)
- [ ] Smoke test: send message via Zalo Desktop (Windows)

## Success Criteria

- `cargo build` succeeds on both macOS and Windows
- On Windows: Agent reads messages from Zalo Desktop via UI Automation
- On Windows: Agent sends messages via keyboard simulation
- On macOS: zero regressions — all existing functionality works
- Each platform file <200 LOC
- Platform selection is automatic via `#[cfg]` — no runtime flags needed

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Zalo Desktop UI tree differs from expected | High | High | Build exploration tool first to inspect actual tree structure; adapt selectors |
| UI Automation perf — traversal too slow | Medium | Medium | Cache window handle; traverse only chat pane subtree, not entire tree |
| SendInput blocked by anti-cheat/security | Low | High | SendInput is standard Win32 API; Zalo Desktop (Electron) accepts it |
| Zalo updates change UI structure | Medium | Medium | Use control type + depth heuristics, not brittle selectors |
| COM initialization threading issues | Medium | Medium | Use STA (single-threaded apartment) for UI Automation |

## Security Considerations

- `SendInput` simulates real keystrokes — user must be aware automation is happening
- Clipboard is used for message pasting — clear clipboard after use to prevent leaking
- UI Automation requires no elevated privileges (runs as current user)
- Windows Defender may flag automation behavior — document whitelist instructions
- Same safety engine (Phase 6) applies: rate limiting, working hours, duplicate detection

## Unresolved Questions

1. **Exact Zalo Desktop (Windows) UI tree structure** — need to inspect with Accessibility Insights or UI Spy tool before implementation. The tree structure described above is estimated based on typical Electron apps.
2. **Multi-monitor DPI handling** — wry handles this, but SendInput coordinates may need DPI awareness for click-based fallbacks.
3. **Zalo Desktop version compatibility** — which versions of Zalo Desktop for Windows are we targeting? Latest only, or also older versions?
