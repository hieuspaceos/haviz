// Scan từng element trong vùng chat để tìm tin nhắn riêng lẻ
import Cocoa
import ApplicationServices

let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == "Zalo" }
guard let zalo = apps.first else { print("❌ Zalo chưa mở"); exit(1) }

let appRef = AXUIElementCreateApplication(zalo.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)
guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else { print("❌"); exit(1) }

// Tìm vùng chat (element có desc chứa timestamp) rồi scan children sâu
func scanDeep(_ element: AXUIElement, depth: Int = 0, inChatArea: Bool = false) {
    if depth > 25 { return }

    var desc: CFTypeRef?
    var value: CFTypeRef?
    var role: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute as CFString, &desc)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)

    let descStr = (desc as? String) ?? ""
    let valueStr = (value as? String) ?? ""
    let roleStr = (role as? String) ?? ""
    let indent = String(repeating: "  ", count: depth)

    // Detect chat area
    let isChatArea = descStr.range(of: #"\d{1,2}:\d{2}"#, options: .regularExpression) != nil && descStr.count > 15
    let shouldPrint = inChatArea || isChatArea

    if shouldPrint && (!descStr.isEmpty || !valueStr.isEmpty) {
        var parts = ["\(indent)[\(roleStr) d=\(depth)]"]
        if !descStr.isEmpty { parts.append("desc=\"\(descStr)\"") }
        if !valueStr.isEmpty { parts.append("val=\"\(valueStr)\"") }
        print(parts.joined(separator: " "))
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(300) {
            scanDeep(kid, depth: depth + 1, inChatArea: shouldPrint || inChatArea)
        }
    }
}

print("=== DEEP SCAN CHAT AREA ===\n")
scanDeep(windows[0])
