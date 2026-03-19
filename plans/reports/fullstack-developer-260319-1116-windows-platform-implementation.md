# Phase Implementation Report

## Executed Phase
- Phase: Phase 4 — Windows Support for Haviz Rust Agent
- Plan: none (direct task)
- Status: completed

## Files Modified

| File | Change | Lines |
|---|---|---|
| `agent/Cargo.toml` | Added Windows-only deps block | +9 |
| `agent/src/platform/mod.rs` | Added Windows macos-shim inline module + `pub mod windows` | ~37 |
| `agent/src/channels/mod.rs` | cfg-gated `zalo_desktop` (macos) and `zalo_desktop_windows` (windows) | +4 |
| `agent/src/app/webview.rs` | cfg-gated `with_data_store_identifier` + updated user-agent | ~86 |
| `agent/src/routes/zalo_control.rs` | cfg-gated `run_osascript` import + send handler macOS/Windows branches | +15 |
| `agent/src/routes/screenshot.rs` | Full rewrite: PowerShell screenshot path for Windows | ~90 |
| `agent/src/main.rs` | cfg-gated channel construction (macos vs windows) | ~80 |

## Files Created

| File | Purpose | Lines |
|---|---|---|
| `agent/src/platform/windows/mod.rs` | Declares `uiautomation` and `input` submodules | 2 |
| `agent/src/platform/windows/uiautomation.rs` | UI Automation COM API — walk Zalo AX tree, extract messages | ~130 |
| `agent/src/platform/windows/input.rs` | SendInput + Win32 clipboard — simulate keyboard to send messages | ~155 |
| `agent/src/channels/zalo_desktop_windows.rs` | `WindowsZaloDesktop` — ChannelReader + ChannelSender for Windows | ~50 |

## Tasks Completed

- [x] Step 1: Added `windows = 0.58` with 7 features to `Cargo.toml` (windows-only target)
- [x] Step 2: Created `platform/windows/mod.rs`, `uiautomation.rs`, `input.rs`
- [x] Step 3: Updated `platform/mod.rs` — real macos mod on macOS, inline shim on Windows
- [x] Step 4: Created `channels/zalo_desktop_windows.rs` with `WindowsZaloDesktop`
- [x] Step 5: Updated `channels/mod.rs` with cfg-gated module declarations
- [x] Step 6: Fixed `haviz_app.rs` webview cross-platform (`with_data_store_identifier` macOS-only)
- [x] Bonus: Fixed `routes/screenshot.rs` with PowerShell Windows path
- [x] Bonus: Fixed `routes/zalo_control.rs` `run_osascript` cfg guards
- [x] Bonus: Fixed `main.rs` channel construction with cfg branches

## Key Design Decisions

**Platform shim in `platform/mod.rs`**: Rather than touching P2-owned `server.rs` (which imports `platform::macos::automation`), the Windows cfg block defines an inline `pub mod macos` with stub submodules (`automation`, `osascript`, `accessibility`) that re-route to Windows implementations. This keeps `server.rs` unchanged and compilable on both platforms.

**API correctness** (verified against microsoft.github.io/windows-docs-rs):
- `FindWindowW` returns `Result<HWND>` — used `.map_err(|_| ...)?`
- `HWND(x as *mut std::ffi::c_void)` — inner field is `*mut c_void`
- `HGLOBAL.0` is `*mut c_void`; cast to `HANDLE` via `HANDLE(hmem.0)`
- `SetClipboardData` second param is `Option<HANDLE>`
- `GlobalUnlock` returns `Result<()>` — result ignored (lock-count semantics)
- `ShowWindow` / `SetForegroundWindow` return `BOOL` — not `Result`, wrapped in `let _ =`
- `CoInitializeEx` returns `HRESULT`; `is_err()` skips S_FALSE correctly
- `IUIAutomationElement::CurrentName()` returns `BSTR` directly (avoids VARIANT complexity)

## Tests Status
- Type check: not runnable (cargo not installed in this shell environment)
- Unit tests: not runnable
- Integration tests: not runnable

## Issues Encountered

1. **`cargo` not in bash PATH** — Rust installed via a different mechanism not accessible in this shell session. Static analysis used instead of live `cargo check`.
2. **P2-owned `server.rs`** — Uses `platform::macos::automation` unconditionally. Resolved without modifying the file via the inline Windows macos-shim in `platform/mod.rs`.

## Unresolved Questions

1. **Zalo Desktop window class name** — `"ZPMainWnd"` is the most commonly reported class for Zalo Desktop on Windows, but should be verified with `Spy++` or `winspy` against the actual installed version.
2. **UI Automation tree structure** — The heuristic grouping (sender→content→timestamp triples) is an educated guess. Real Zalo Desktop AX tree structure should be verified once `cargo check` passes and the binary runs.
3. **`CoInitializeEx` threading** — If `read_zalo_messages()` is called from a Tokio thread that already initialized COM with a different apartment type, the call will fail. Consider moving COM initialization to a dedicated OS thread.
4. **`cargo check` verification** — Needs to be run once Rust is accessible in the shell to confirm no remaining type errors in the `windows` crate bindings.
