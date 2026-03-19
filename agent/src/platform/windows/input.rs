/// Windows SendInput API for sending messages to Zalo Desktop.
///
/// Simulates human keyboard input: brings Zalo to foreground, opens search
/// (Ctrl+F), pastes contact name, confirms, pastes message, sends with Enter.
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData,
};
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_RETURN,
};
use windows::Win32::UI::WindowsAndMessaging::{SetForegroundWindow, ShowWindow, SW_RESTORE};

use super::uiautomation::find_zalo_window;

// VK_F (0x46) and VK_V (0x56) are not named constants in the windows crate
const VK_F_CODE: u16 = 0x46;
const VK_V_CODE: u16 = 0x56;

// CF_UNICODETEXT clipboard format
const CF_UNICODETEXT: u32 = 13;

/// Send a message to a Zalo contact by simulating human keyboard interaction.
pub fn send_message_to_zalo(contact_name: &str, message: &str) -> Result<(), String> {
    let hwnd_val = find_zalo_window()?;
    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);

    bring_to_foreground(hwnd);
    sleep_random(400, 800);

    // Ctrl+F — open contact/conversation search bar
    send_key_combo(VK_CONTROL.0, VK_F_CODE)?;
    sleep_random(800, 2000);

    // Paste contact name into search (set clipboard then Ctrl+V)
    set_clipboard(contact_name)?;
    send_key_combo(VK_CONTROL.0, VK_V_CODE)?;
    sleep_random(500, 1200);

    // Enter — select first matching contact
    send_single_key(VK_RETURN.0)?;
    sleep_random(800, 1500);

    // Paste message and send
    set_clipboard(message)?;
    send_key_combo(VK_CONTROL.0, VK_V_CODE)?;
    sleep_random(300, 800);

    send_single_key(VK_RETURN.0)?;

    Ok(())
}

// ---------------------------------------------------------------------------
// Clipboard
// ---------------------------------------------------------------------------

/// Write UTF-8 text to the Windows clipboard as CF_UNICODETEXT.
pub(super) fn set_clipboard(text: &str) -> Result<(), String> {
    // Encode as UTF-16 with null terminator
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0u16)).collect();
    let byte_len = wide.len() * 2;

    unsafe {
        // Allocate moveable global memory required by the clipboard API
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len)
            .map_err(|e| format!("GlobalAlloc failed: {}", e))?;

        let ptr = GlobalLock(hmem) as *mut u16;
        if ptr.is_null() {
            return Err("GlobalLock returned null".to_string());
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
        // GlobalUnlock returns Result<()> in windows crate; ignore lock-count info
        let _ = GlobalUnlock(hmem);

        OpenClipboard(HWND(std::ptr::null_mut()))
            .map_err(|e| format!("OpenClipboard failed: {}", e))?;
        EmptyClipboard().map_err(|e| format!("EmptyClipboard failed: {}", e))?;

        // SetClipboardData takes ownership of hmem on success — do not free it
        // Pass HANDLE wrapping the HGLOBAL pointer directly (not Option)
        SetClipboardData(CF_UNICODETEXT, HANDLE(hmem.0))
            .map_err(|e| format!("SetClipboardData failed: {}", e))?;

        CloseClipboard().map_err(|e| format!("CloseClipboard failed: {}", e))?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// SendInput helpers
// ---------------------------------------------------------------------------

fn make_key_input(vk: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn send_single_key(vk: u16) -> Result<(), String> {
    let inputs = [
        make_key_input(vk, KEYBD_EVENT_FLAGS(0)),
        make_key_input(vk, KEYEVENTF_KEYUP),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(format!("SendInput sent {} of {}", sent, inputs.len()));
    }
    Ok(())
}

fn send_key_combo(modifier_vk: u16, key_vk: u16) -> Result<(), String> {
    let inputs = [
        make_key_input(modifier_vk, KEYBD_EVENT_FLAGS(0)),
        make_key_input(key_vk, KEYBD_EVENT_FLAGS(0)),
        make_key_input(key_vk, KEYEVENTF_KEYUP),
        make_key_input(modifier_vk, KEYEVENTF_KEYUP),
    ];
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(format!("SendInput sent {} of {}", sent, inputs.len()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Window management
// ---------------------------------------------------------------------------

fn bring_to_foreground(hwnd: HWND) {
    unsafe {
        // Both return BOOL; failure is non-fatal (focus may still work)
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let _ = SetForegroundWindow(hwnd);
    }
}

// ---------------------------------------------------------------------------
// Pseudo-random delay — same technique as macOS automation.rs (no extra crate)
// ---------------------------------------------------------------------------

fn sleep_random(min_ms: u64, max_ms: u64) {
    let jitter = rand_u64() % (max_ms - min_ms + 1);
    thread::sleep(Duration::from_millis(min_ms + jitter));
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    std::time::SystemTime::now().hash(&mut hasher);
    thread::current().id().hash(&mut hasher);
    hasher.finish()
}
