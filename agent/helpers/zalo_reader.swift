// Haviz Zalo Reader — Reads messages from Zalo Desktop via AX API
// Outputs JSON to stdout for the Rust Agent to parse
// Compile: swiftc zalo_reader.swift -o zalo_reader -framework Cocoa -framework ApplicationServices

import Cocoa
import ApplicationServices

// Find Zalo process
let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == "Zalo" }
guard let zalo = apps.first else {
    fputs("Zalo chưa mở\n", stderr)
    exit(1)
}

let appRef = AXUIElementCreateApplication(zalo.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
    fputs("Không lấy được window\n", stderr)
    exit(1)
}

// Structures to collect messages
struct ChatMessage {
    var sender: String
    var content: String
    var timestamp: String
}

var collectedMessages: [ChatMessage] = []
var conversationName: String? = nil

// Track current message being built
var currentTimestamp: String? = nil
var currentSender: String? = nil
var currentContents: [String] = []

func flushMessage() {
    if let sender = currentSender, !currentContents.isEmpty, let ts = currentTimestamp {
        let content = currentContents.joined(separator: " ")
        collectedMessages.append(ChatMessage(sender: sender, content: content, timestamp: ts))
    }
    currentSender = nil
    currentContents = []
}

// Regex for timestamp detection
let timeRegex = try! NSRegularExpression(pattern: "^\\d{1,2}:\\d{2}$")

func isTimestamp(_ str: String) -> Bool {
    let range = NSRange(str.startIndex..., in: str)
    return timeRegex.firstMatch(in: str, range: range) != nil
}

// Recursive scan
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

    // Try to get conversation name from window title area (depth ~5-8)
    if depth >= 3 && depth <= 10 && roleStr == "AXStaticText" && !valueStr.isEmpty {
        if conversationName == nil && valueStr.count > 1 && valueStr.count < 50 {
            let uiLabels = ["Tìm kiếm", "Tất cả", "Chưa đọc", "Phân loại", "Zalo", "Online", "Offline",
                          "Truy cập", "vừa truy cập", "đang hoạt động", "Vừa", "phút", "giờ"]
            let isUI = uiLabels.contains(where: { valueStr.lowercased().contains($0.lowercased()) })
            if !isUI {
                // Likely the contact name at the top
            }
        }
    }

    // Message detection at deeper levels
    if roleStr == "AXStaticText" && !valueStr.isEmpty {
        // Depth ~18: timestamps
        if depth >= 16 && depth <= 20 && isTimestamp(valueStr) {
            // New timestamp = potentially new message group
            flushMessage()
            currentTimestamp = valueStr
        }
        // Depth ~21: sender names
        else if depth >= 19 && depth <= 23 && valueStr.count < 50 && valueStr.count > 1 {
            // Could be sender name
            if currentTimestamp != nil && currentSender == nil {
                currentSender = valueStr
            } else if currentTimestamp != nil && currentSender != nil && currentContents.isEmpty {
                // This might be the actual sender, previous was something else
                // Heuristic: shorter strings without spaces are more likely names
            }
        }
        // Depth ~22: message content
        else if depth >= 20 && depth <= 25 {
            if currentTimestamp != nil {
                if currentSender == nil {
                    currentSender = "Unknown"
                }
                currentContents.append(valueStr)
            }
        }
    }

    // Also check description for grouped chat content
    if !descStr.isEmpty && descStr.count > 20 {
        let hasTime = descStr.range(of: "\\d{1,2}:\\d{2}", options: .regularExpression) != nil
        if hasTime && depth >= 10 {
            // This is a large block containing multiple messages
            // Parse it: "Sender1 message1 15:57 Sender2 message2 15:58"
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

func parseDescBlock(_ desc: String) {
    // Description blocks often have format:
    // "date Sender1 msg1 HH:MM Sender2 msg2 HH:MM"
    // This is a fallback parser for when individual elements aren't available
    let parts = desc.components(separatedBy: "\n").flatMap { $0.components(separatedBy: "\r") }
    for part in parts {
        let trimmed = part.trimmingCharacters(in: .whitespaces)
        if !trimmed.isEmpty && trimmed.count > 3 {
            // Try to extract time pattern at the end
            if let range = trimmed.range(of: "\\d{1,2}:\\d{2}$", options: .regularExpression) {
                let time = String(trimmed[range])
                let content = trimmed[..<range.lowerBound].trimmingCharacters(in: .whitespaces)
                if !content.isEmpty {
                    collectedMessages.append(ChatMessage(sender: "Unknown", content: content, timestamp: time))
                }
            }
        }
    }
}

// Run scan
scan(windows[0])
flushMessage()

// Output JSON
var jsonMessages: [[String: String]] = []
for msg in collectedMessages {
    jsonMessages.append([
        "sender": msg.sender,
        "content": msg.content,
        "timestamp": msg.timestamp,
    ])
}

let output: [String: Any] = [
    "conversation_name": conversationName ?? NSNull(),
    "messages": jsonMessages,
]

if let data = try? JSONSerialization.data(withJSONObject: output, options: []),
   let str = String(data: data, encoding: .utf8) {
    print(str)
} else {
    // Fallback: empty result
    print("{\"conversation_name\":null,\"messages\":[]}")
}
