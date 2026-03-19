/// Platform abstraction layer.
///
/// On macOS: real implementations via AX API + AppleScript.
/// On Windows: shim re-exports keep the `platform::macos::*` namespace intact
///   so server.rs / routes can import unchanged on both platforms.
#[cfg(target_os = "macos")]
pub mod macos;

/// Windows shim — exposes `platform::macos::automation` and
/// `platform::macos::osascript` under the same paths so callers in server.rs
/// and routes/zalo_control.rs compile without modification.
#[cfg(target_os = "windows")]
pub mod macos {
    pub mod automation {
        pub fn send_message_zalo_desktop(to: &str, message: &str) -> Result<(), String> {
            crate::platform::windows::input::send_message_to_zalo(to, message)
        }
    }
    pub mod osascript {
        /// No-op on Windows — callers in zalo_control.rs that call run_osascript
        /// are also wrapped in cfg guards; this stub prevents compilation failure.
        pub fn run_osascript(_script: &str) -> Result<String, String> {
            Err("osascript not available on Windows".to_string())
        }
    }
    pub mod accessibility {
        pub fn read_zalo_messages(_reader_path: &str) -> Result<String, String> {
            // Windows uses uiautomation module directly; this stub satisfies
            // any residual imports of the macos accessibility path.
            Err("Use platform::windows::uiautomation on Windows".to_string())
        }
    }
}

#[cfg(target_os = "windows")]
pub mod windows;
