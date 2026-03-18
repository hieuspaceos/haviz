// Haviz — Read Zalo Web conversations from browser via AX API
// Validated structure (Chrome, chat.zalo.me):
//   depth 23: conversation name, timestamp parts
//   depth 25: sender prefix ("Bạn:"), message preview
// Compile: swiftc zalo_web_reader.swift -o zalo_web_reader -framework Cocoa -framework ApplicationServices
// Usage: ./zalo_web_reader [browser_name]

import Cocoa
import ApplicationServices

let browserName = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1...].joined(separator: " ")
    : "Google Chrome"

// Find browser
let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == browserName }
guard let app = apps.first else { fputs("browser_not_running:\(browserName)\n", stderr); exit(1) }

let appRef = AXUIElementCreateApplication(app.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)
guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else { fputs("no_windows\n", stderr); exit(1) }

// Find Zalo tab
var zaloWindow: AXUIElement? = nil
var windowTitle = ""
for window in windows {
    var title: CFTypeRef?
    AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
    let t = (title as? String) ?? ""
    if t.lowercased().contains("zalo") { zaloWindow = window; windowTitle = t; break }
}
guard let target = zaloWindow else { fputs("zalo_tab_not_found\n", stderr); exit(1) }

// Collect all AXStaticText by depth
struct TextItem {
    let depth: Int
    let value: String
}

var textItems: [TextItem] = []

func scan(_ element: AXUIElement, depth: Int = 0) {
    if depth > 26 { return }

    var role: CFTypeRef?
    var value: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)

    let roleStr = (role as? String) ?? ""
    let valueStr = (value as? String) ?? ""

    if roleStr == "AXStaticText" {
        textItems.append(TextItem(depth: depth, value: valueStr))
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(500) {
            scan(kid, depth: depth + 1)
        }
    }
}

scan(target)

// Parse conversation list from depth-based structure
// Pattern: depth 23 = conv name | timestamp parts | empty strings between convs
//          depth 25 = sender prefix | message preview
struct Conversation {
    var name: String
    var timeRaw: String       // "22 phút", "03/03", "1 giờ"
    var lastSender: String
    var lastMessage: String
}

var conversations: [Conversation] = []
var currentConv: Conversation? = nil
var timeNumberBuffer: String? = nil // for "22" + " " + "phút" pattern

// UI labels to skip
let skipValues = Set(["", "Tìm kiếm", "Tất cả", "Chưa đọc", "Phân loại",
    "Tin nhắn", "Danh bạ", "Công cụ", "Zalo Cloud", "My Documents",
    "Tải ngay", "Đóng", "Tải Zalo PC để xem đầy đủ tin nhắn"])

let timeUnits = Set(["phút", "giờ", "ngày", "tuần", "tháng"])
let dateRegex = try! NSRegularExpression(pattern: "^\\d{1,2}/\\d{1,2}$")

func isDate(_ s: String) -> Bool {
    dateRegex.firstMatch(in: s, range: NSRange(s.startIndex..., in: s)) != nil
}

func isNumber(_ s: String) -> Bool {
    s.allSatisfy { $0.isNumber }
}

// State machine to parse depth-23 and depth-25 elements
var i = 0
while i < textItems.count {
    let item = textItems[i]

    // Skip non-conversation depths and UI chrome
    if item.depth < 20 || skipValues.contains(item.value) {
        // Skip known UI text at depth 14, 17, 19 etc
        if item.depth <= 19 || item.value.count > 80 { i += 1; continue }
    }

    if item.depth == 23 {
        let val = item.value

        // Empty string at depth 23 = separator between conversations, or group icon
        if val.isEmpty || val == " " {
            i += 1; continue
        }

        // Check if this is a time number: "22" followed by " " and "phút"
        if isNumber(val) && val.count <= 2 {
            // Look ahead for time unit
            if i + 2 < textItems.count
                && textItems[i+1].depth == 23 && textItems[i+1].value == " "
                && textItems[i+2].depth == 23 && timeUnits.contains(textItems[i+2].value) {
                // This is timestamp: "22 phút"
                if var conv = currentConv {
                    conv.timeRaw = "\(val) \(textItems[i+2].value)"
                    currentConv = conv
                }
                i += 3; continue
            }
        }

        // Check if date format "03/03"
        if isDate(val) {
            if var conv = currentConv {
                conv.timeRaw = val
                currentConv = conv
            }
            i += 1; continue
        }

        // Check for time unit alone (shouldn't happen but safety)
        if timeUnits.contains(val) { i += 1; continue }

        // Check for reaction emoji like "/-heart"
        if val.hasPrefix("/-") { i += 1; continue }

        // Otherwise: this is a conversation NAME
        // Save previous conversation
        if let conv = currentConv, !conv.name.isEmpty {
            conversations.append(conv)
        }
        currentConv = Conversation(name: val, timeRaw: "", lastSender: "", lastMessage: "")

    } else if item.depth == 25 && currentConv != nil {
        let val = item.value
        if val.isEmpty { i += 1; continue }

        // Sender prefix ends with ":" like "Bạn:", "IMA Dương Bích Ngọc:"
        if val.hasSuffix(":") {
            currentConv!.lastSender = String(val.dropLast()) // remove ":"
        } else if val == "Hình ảnh" || val == "Tin nhắn đã được thu hồi" || val == "Cuộc gọi thoại đến" {
            currentConv!.lastMessage = val
        } else if !val.isEmpty {
            // Actual message content
            if currentConv!.lastMessage.isEmpty {
                currentConv!.lastMessage = val
            } else {
                currentConv!.lastMessage += " " + val
            }
        }
    }

    i += 1
}

// Don't forget last conversation
if let conv = currentConv, !conv.name.isEmpty {
    conversations.append(conv)
}

// Output JSON
var jsonConvs: [[String: String]] = []
for conv in conversations {
    jsonConvs.append([
        "name": conv.name,
        "time": conv.timeRaw,
        "sender": conv.lastSender,
        "preview": conv.lastMessage,
    ])
}

let output: [String: Any] = [
    "browser": browserName,
    "window_title": windowTitle,
    "conversations": jsonConvs,
    "total_text_elements": textItems.count,
]

if let data = try? JSONSerialization.data(withJSONObject: output, options: [.prettyPrinted]),
   let str = String(data: data, encoding: .utf8) {
    print(str)
}
