use std::process::Command;
use std::thread;
use std::time::Duration;

/// Send a message to a contact via Zalo Desktop using AppleScript + clipboard paste.
/// This mimics human behavior: Cmd+F → search name → paste message → Enter.
pub fn send_message_zalo_desktop(to: &str, message: &str) -> Result<(), String> {
    // Step 1: Copy recipient name to clipboard, focus Zalo, search
    set_clipboard(to)?;
    run_osascript(
        r#"
        tell application "Zalo" to activate
        delay 0.5
        tell application "System Events"
            keystroke "f" using command down
            delay 0.5
            keystroke "a" using command down
            delay 0.1
            keystroke "v" using command down
            delay 1
            key code 36
            delay 0.5
        end tell
    "#,
    )?;

    // Random human-like delay between search and typing
    let delay_ms = 500 + (rand_u64() % 1000);
    thread::sleep(Duration::from_millis(delay_ms));

    // Step 2: Copy message to clipboard, paste, send
    set_clipboard(message)?;
    run_osascript(
        r#"
        tell application "System Events"
            keystroke "v" using command down
            delay 0.3
            key code 36
        end tell
    "#,
    )?;

    Ok(())
}

fn set_clipboard(text: &str) -> Result<(), String> {
    let escaped = escape_applescript(text);
    run_osascript(&format!(r#"set the clipboard to "{}""#, escaped))
}

fn run_osascript(script: &str) -> Result<(), String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| format!("osascript failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("osascript error: {}", stderr));
    }
    Ok(())
}

fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Simple pseudo-random using thread-based seed (no extra crate needed)
fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);
    hasher.finish()
}
