// Scan Zalo Web trong browser để đọc tin nhắn
import Cocoa
import ApplicationServices

// Tìm browser đang mở (Safari, Chrome, Arc, Edge...)
let browsers = ["Safari", "Google Chrome", "Arc", "Microsoft Edge", "Firefox", "Brave Browser"]
var foundApp: NSRunningApplication? = nil
var appName = ""

for browser in browsers {
    let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == browser }
    if let app = apps.first {
        foundApp = app
        appName = browser
        break
    }
}

guard let app = foundApp else {
    print("❌ Không tìm thấy browser nào đang mở")
    print("Đang chạy: \(NSWorkspace.shared.runningApplications.compactMap { $0.localizedName }.joined(separator: ", "))")
    exit(1)
}

print("✅ Tìm thấy \(appName) PID: \(app.processIdentifier)")

let appRef = AXUIElementCreateApplication(app.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
    print("❌ Không lấy được window — cần bật Accessibility cho \(appName)")
    exit(1)
}

// Tìm window có title chứa "Zalo"
var zaloWindow: AXUIElement? = nil
for window in windows {
    var title: CFTypeRef?
    AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
    let titleStr = (title as? String) ?? ""
    if titleStr.lowercased().contains("zalo") {
        zaloWindow = window
        print("✅ Tìm thấy Zalo Web window: \"\(titleStr)\"")
        break
    }
}

guard let targetWindow = zaloWindow else {
    // List all window titles
    print("❌ Không tìm thấy tab Zalo Web")
    print("Các window đang mở:")
    for window in windows {
        var title: CFTypeRef?
        AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
        print("  - \((title as? String) ?? "?")")
    }
    exit(1)
}

var totalElements = 0
var textElements = 0

func scanDeep(_ element: AXUIElement, depth: Int = 0, inChatArea: Bool = false) {
    if depth > 25 { return }
    totalElements += 1

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

    let hasTime = descStr.range(of: #"\d{1,2}:\d{2}"#, options: .regularExpression) != nil
    let isChatContent = hasTime && descStr.count > 15
    let shouldPrint = inChatArea || isChatContent

    if shouldPrint && (!descStr.isEmpty || !valueStr.isEmpty) {
        var parts = ["\(indent)[\(roleStr) d=\(depth)]"]
        if !descStr.isEmpty { parts.append("desc=\"\(descStr)\"") }
        if !valueStr.isEmpty { parts.append("val=\"\(valueStr)\"") }
        print(parts.joined(separator: " "))
        textElements += 1
    } else if !shouldPrint && roleStr == "AXStaticText" && !valueStr.isEmpty && valueStr.count > 3 {
        // Also print interesting static text outside chat area
        if depth >= 8 {
            print("\(indent)[\(roleStr) d=\(depth)] val=\"\(valueStr)\"")
            textElements += 1
        }
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(300) {
            scanDeep(kid, depth: depth + 1, inChatArea: shouldPrint || inChatArea)
        }
    }
}

print("\n=== SCANNING ZALO WEB ===\n")
scanDeep(targetWindow)
print("\n--- Total: \(totalElements) elements, \(textElements) text elements ---")
