/// Windows UI Automation API for reading Zalo Desktop messages.
///
/// Uses IUIAutomation COM interface to walk the accessibility tree and extract
/// message text from Zalo Desktop's chat window.
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTreeWalker,
};
use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
use windows::core::PCWSTR;

/// Raw Zalo message extracted from the accessibility tree.
#[derive(Debug, Clone)]
pub struct ZaloMessage {
    pub sender: String,
    pub content: String,
    pub timestamp: String,
}

/// Find the Zalo Desktop top-level window by its registered class name.
/// Returns raw HWND pointer value as isize; error string on failure.
pub fn find_zalo_window() -> Result<isize, String> {
    // Zalo Desktop on Windows registers this window class
    let class_wide: Vec<u16> = "ZPMainWnd\0".encode_utf16().collect();

    let hwnd = unsafe {
        FindWindowW(PCWSTR(class_wide.as_ptr()), PCWSTR::null())
            .map_err(|_| "zalo_not_running".to_string())?
    };

    Ok(hwnd.0 as isize)
}

/// Walk an element subtree depth-first collecting each element's CurrentName.
/// Mirrors the macOS AX depth traversal but uses UI Automation tree walker.
fn collect_names(
    walker: &IUIAutomationTreeWalker,
    element: &IUIAutomationElement,
    depth: u32,
    out: &mut Vec<String>,
) {
    if depth == 0 {
        return;
    }

    // CurrentName() returns BSTR — cheaper than GetCurrentPropertyValue(UIA_NamePropertyId)
    if let Ok(bstr) = unsafe { element.CurrentName() } {
        let s = bstr.to_string();
        let trimmed = s.trim().to_string();
        if !trimmed.is_empty() {
            out.push(trimmed);
        }
    }

    // Recurse into first child then visit siblings
    if let Ok(child) = unsafe { walker.GetFirstChildElement(element) } {
        collect_names(walker, &child, depth - 1, out);

        let mut sibling = child;
        while let Ok(next) = unsafe { walker.GetNextSiblingElement(&sibling) } {
            collect_names(walker, &next, depth - 1, out);
            sibling = next;
        }
    }
}

/// Read Zalo Desktop messages via Windows UI Automation.
/// Initialises COM on this thread, walks the element tree, then uninitialises.
pub fn read_zalo_messages() -> Result<Vec<ZaloMessage>, String> {
    let hwnd_val = find_zalo_window()?;
    let hwnd = HWND(hwnd_val as *mut std::ffi::c_void);

    unsafe {
        // S_FALSE means COM already initialised on this thread — both are OK
        let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if hr.is_err() {
            // is_err() returns true only for actual failure HRESULTs (not S_FALSE)
            return Err(format!("CoInitializeEx failed: {:?}", hr));
        }
    }

    let result = read_messages_inner(hwnd);
    unsafe { CoUninitialize() };
    result
}

fn read_messages_inner(hwnd: HWND) -> Result<Vec<ZaloMessage>, String> {
    let automation: IUIAutomation = unsafe {
        CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("CoCreateInstance IUIAutomation failed: {}", e))?
    };

    let root: IUIAutomationElement = unsafe {
        automation
            .ElementFromHandle(hwnd)
            .map_err(|e| format!("ElementFromHandle failed: {}", e))?
    };

    // Content view walker skips purely decorative/hidden elements
    let walker: IUIAutomationTreeWalker = unsafe {
        automation
            .ContentViewWalker()
            .map_err(|e| format!("ContentViewWalker failed: {}", e))?
    };

    let mut names: Vec<String> = Vec::new();
    collect_names(&walker, &root, 12, &mut names);

    Ok(parse_names_into_messages(names))
}

/// Heuristic grouping: Zalo Desktop accessibility tree emits tokens in order
/// sender → content → timestamp per message bubble; bundle into triples.
fn parse_names_into_messages(names: Vec<String>) -> Vec<ZaloMessage> {
    let mut messages = Vec::new();
    let mut i = 0;

    while i + 2 < names.len() {
        let sender = names[i].clone();
        let content = names[i + 1].clone();
        let timestamp = names[i + 2].clone();

        // Skip degenerate groups
        if content.is_empty() || content == sender {
            i += 1;
            continue;
        }

        messages.push(ZaloMessage { sender, content, timestamp });
        i += 3;
    }

    messages
}
