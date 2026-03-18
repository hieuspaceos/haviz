// Haviz — Read Zalo Web messages from any browser via AX API
// Same approach as Zalo Desktop reader but targets browser window
// Compile: swiftc zalo_web_reader.swift -o zalo_web_reader -framework Cocoa -framework ApplicationServices
// Usage: ./zalo_web_reader [browser_name]
//   browser_name: "Google Chrome" (default), "Safari", "Arc", "Microsoft Edge", "Firefox"

import Cocoa
import ApplicationServices

// Get browser name from args or default to Chrome
let browserName = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1...].joined(separator: " ")
    : "Google Chrome"

// Find browser process
let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == browserName }
guard let app = apps.first else {
    fputs("browser_not_running:\(browserName)\n", stderr)
    exit(1)
}

let appRef = AXUIElementCreateApplication(app.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
    fputs("no_windows\n", stderr)
    exit(1)
}

// Find window with Zalo tab
var zaloWindow: AXUIElement? = nil
var windowTitle = ""
for window in windows {
    var title: CFTypeRef?
    AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
    let titleStr = (title as? String) ?? ""
    if titleStr.lowercased().contains("zalo") {
        zaloWindow = window
        windowTitle = titleStr
        break
    }
}

guard let targetWindow = zaloWindow else {
    fputs("zalo_tab_not_found\n", stderr)
    exit(1)
}

// Collect messages — same depth-based approach as Zalo Desktop
struct ChatMessage {
    var sender: String
    var content: String
    var timestamp: String
}

var messages: [ChatMessage] = []
var conversationName: String? = nil
var contactNames: [String] = []

// Track texts by depth for message reconstruction
var depthTexts: [(depth: Int, role: String, value: String)] = []

func scan(_ element: AXUIElement, depth: Int = 0) {
    if depth > 25 { return }

    var role: CFTypeRef?
    var value: CFTypeRef?
    var desc: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)
    AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute as CFString, &desc)

    let roleStr = (role as? String) ?? ""
    let valueStr = (value as? String) ?? ""
    let descStr = (desc as? String) ?? ""

    // Collect static text elements at various depths
    if roleStr == "AXStaticText" && !valueStr.isEmpty && valueStr.count > 0 {
        // Filter out UI labels
        let uiLabels = ["Tìm kiếm", "Tất cả", "Chưa đọc", "Phân loại",
                       "Tin nhắn", "Danh bạ", "Công cụ", "Zalo Cloud",
                       "My Documents", "Online", "Offline", "Đóng"]
        let isUI = uiLabels.contains(where: { valueStr == $0 })

        if !isUI {
            depthTexts.append((depth: depth, role: roleStr, value: valueStr))
        }
    }

    // Check description blocks (contain grouped messages with timestamps)
    if !descStr.isEmpty && descStr.count > 20 {
        let hasTime = descStr.range(of: "\\d{1,2}:\\d{2}", options: .regularExpression) != nil
        if hasTime {
            // Parse desc block for messages
            parseDescBlock(descStr)
        }
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(300) {
            scan(kid, depth: depth + 1)
        }
    }
}

let timeRegex = try! NSRegularExpression(pattern: "^\\d{1,2}:\\d{2}$")

func isTimestamp(_ str: String) -> Bool {
    let range = NSRange(str.startIndex..., in: str)
    return timeRegex.firstMatch(in: str, range: range) != nil
}

func parseDescBlock(_ desc: String) {
    // Description blocks in browser AX API often contain all messages in one string
    // Format varies but timestamps are markers between messages
    let lines = desc.components(separatedBy: CharacterSet.newlines)
    var currentSender = "Unknown"

    for line in lines {
        let trimmed = line.trimmingCharacters(in: .whitespaces)
        if trimmed.isEmpty { continue }

        // Try to find timestamp at end
        if let range = trimmed.range(of: "\\d{1,2}:\\d{2}$", options: .regularExpression) {
            let time = String(trimmed[range])
            let content = trimmed[..<range.lowerBound].trimmingCharacters(in: .whitespaces)
            if !content.isEmpty {
                messages.append(ChatMessage(sender: currentSender, content: content, timestamp: time))
            }
        } else if trimmed.count < 40 && trimmed.count > 1 {
            // Short text without timestamp — likely a sender name
            currentSender = trimmed
        }
    }
}

// Run scan
scan(targetWindow)

// Also extract contact names from depthTexts
// In browser Zalo Web, contact names in sidebar are at moderate depths (8-15)
// Messages are deeper (15+)
for item in depthTexts {
    if item.depth >= 6 && item.depth <= 14 && item.value.count < 40 && item.value.count > 1 {
        if !isTimestamp(item.value) {
            contactNames.append(item.value)
        }
    }
}

// If no messages from desc blocks, try to reconstruct from depth-based texts
if messages.isEmpty {
    var currentTimestamp: String? = nil
    var currentSender: String? = nil

    for item in depthTexts {
        if isTimestamp(item.value) {
            currentTimestamp = item.value
            currentSender = nil
        } else if currentTimestamp != nil && item.value.count < 40 && currentSender == nil {
            currentSender = item.value
        } else if currentTimestamp != nil && currentSender != nil {
            messages.append(ChatMessage(
                sender: currentSender!,
                content: item.value,
                timestamp: currentTimestamp!
            ))
            currentTimestamp = nil
            currentSender = nil
        }
    }
}

// Deduplicate contact names
let uniqueContacts = Array(Set(contactNames))

// Output JSON
var jsonMessages: [[String: String]] = []
for msg in messages {
    jsonMessages.append([
        "sender": msg.sender,
        "content": msg.content,
        "timestamp": msg.timestamp,
    ])
}

let output: [String: Any] = [
    "browser": browserName,
    "window_title": windowTitle,
    "conversation_name": conversationName ?? NSNull(),
    "messages": jsonMessages,
    "contacts": uniqueContacts,
    "total_text_elements": depthTexts.count,
]

if let data = try? JSONSerialization.data(withJSONObject: output, options: []),
   let str = String(data: data, encoding: .utf8) {
    print(str)
} else {
    print("{\"browser\":\"\(browserName)\",\"messages\":[],\"contacts\":[],\"total_text_elements\":0}")
}
