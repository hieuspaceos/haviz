use crate::channels::traits::{ChannelReader, ChannelSender};
use crate::message_parser::{self, ParsedMessage};
use std::process::Command;
use std::time::Duration;

/// Zalo Web channel — reads/sends via browser (Chrome, Safari, Arc, Edge, Firefox)
/// Uses the SAME AX API approach as Zalo Desktop, but targets the browser window.
/// Validated in: validation/browsers/test_browser_ax.swift
///              validation/agent/agent-prototype.js (sendToZaloWeb)
pub struct ZaloWebChannel {
    reader_path: String,
    browser_name: String,
    my_name: String,
}

impl ZaloWebChannel {
    pub fn new(reader_path: String, browser_name: String, my_name: String) -> Self {
        Self {
            reader_path,
            browser_name,
            my_name,
        }
    }

    /// Send message via browser — uses AppleScript coordinate-based click + paste
    /// Ported from validation/agent/agent-prototype.js sendToZaloWeb()
    fn send_via_browser(&self, to: &str, message: &str) -> Result<(), String> {
        let browser = &self.browser_name;
        let process_name = if browser == "Safari" { "Safari" } else { browser };

        // Step 1: Copy recipient name → focus browser → click search → paste → Enter
        set_clipboard(to)?;
        let search_script = format!(r#"
            tell application "{browser}" to activate
            delay 0.5
            tell application "System Events"
                tell process "{process_name}"
                    set frontmost to true
                    set winPos to position of window 1
                    set winSize to size of window 1
                end tell
                -- Click search area (top-left, below avatar)
                set xSearch to (item 1 of winPos) + 200
                set ySearch to (item 2 of winPos) + 180
                click at {{xSearch, ySearch}}
                delay 0.3
                -- Clear + paste name
                keystroke "a" using command down
                delay 0.1
                keystroke "v" using command down
                delay 1.5
                -- Enter to select first result
                key code 36
                delay 0.8
                -- Click message input (bottom of window)
                set xMsg to (item 1 of winPos) + (item 1 of winSize) / 2
                set yMsg to (item 2 of winPos) + (item 2 of winSize) - 30
                click at {{xMsg, yMsg}}
                delay 0.3
            end tell
        "#, browser = browser, process_name = process_name);
        run_osascript(&search_script)?;

        // Random human-like delay
        let delay_ms = 500 + (rand_simple() % 1000);
        std::thread::sleep(Duration::from_millis(delay_ms));

        // Step 2: Copy message → paste → Enter
        set_clipboard(message)?;
        run_osascript(r#"
            tell application "System Events"
                keystroke "v" using command down
                delay 0.3
                key code 36
            end tell
        "#)?;

        Ok(())
    }
}

impl ChannelReader for ZaloWebChannel {
    fn read_messages(&self) -> Result<Vec<ParsedMessage>, String> {
        let output = Command::new(&self.reader_path)
            .arg(&self.browser_name)
            .output()
            .map_err(|e| format!("Failed to run zalo_web_reader: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.contains("browser_not_running") || stderr.contains("zalo_tab_not_found") {
                return Err("zalo_web_not_available".to_string());
            }
            return Err(format!("zalo_web_reader failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        message_parser::parse_snapshot(&stdout, &self.my_name)
    }
}

impl ChannelSender for ZaloWebChannel {
    fn send_message(&self, to: &str, message: &str) -> Result<(), String> {
        self.send_via_browser(to, message)
    }
}

fn set_clipboard(text: &str) -> Result<(), String> {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
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

fn rand_simple() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    hasher.finish()
}
