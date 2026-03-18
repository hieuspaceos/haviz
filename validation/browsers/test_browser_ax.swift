// Test AX API đọc Zalo Web trên nhiều browser
// Chạy: swift test_browser_ax.swift
// Yêu cầu: Accessibility permission cho Terminal

import Cocoa
import ApplicationServices

let browsers = [
    "Google Chrome",
    "Safari",
    "Arc",
    "Microsoft Edge",
    "Firefox",
    "Brave Browser",
    "Opera"
]

print("=== TEST ZALO WEB - MULTI BROWSER ===\n")

for browserName in browsers {
    let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == browserName }
    guard let app = apps.first else {
        print("⏭  \(browserName): không chạy")
        continue
    }

    print("🔍 \(browserName) (PID: \(app.processIdentifier))")

    let appRef = AXUIElementCreateApplication(app.processIdentifier)
    var windowsRef: CFTypeRef?
    AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

    guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
        print("   ❌ Không lấy được window — cần Accessibility permission")
        continue
    }

    // Tìm window có Zalo
    var zaloWindow: AXUIElement? = nil
    for window in windows {
        var title: CFTypeRef?
        AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
        let titleStr = (title as? String) ?? ""
        if titleStr.lowercased().contains("zalo") {
            zaloWindow = window
            print("   ✅ Tìm thấy tab Zalo: \"\(titleStr)\"")
            break
        }
    }

    guard let targetWindow = zaloWindow else {
        print("   ⚠️  Không tìm thấy tab Zalo Web")
        continue
    }

    // Scan elements
    var totalElements = 0
    var textElements = 0
    var chatTexts: [String] = []

    func scan(_ element: AXUIElement, depth: Int = 0) {
        if depth > 20 { return }
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

        if roleStr == "AXStaticText" && !valueStr.isEmpty && valueStr.count > 3 {
            textElements += 1
            if depth >= 8 {
                chatTexts.append(valueStr)
            }
        }

        // Check desc for chat content
        let hasTime = descStr.range(of: #"\d{1,2}:\d{2}"#, options: .regularExpression) != nil
        if hasTime && descStr.count > 15 {
            chatTexts.append("[DESC] \(descStr.prefix(100))")
        }

        var children: CFTypeRef?
        AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
        if let kids = children as? [AXUIElement] {
            for kid in kids.prefix(200) {
                scan(kid, depth: depth + 1)
            }
        }
    }

    scan(targetWindow)

    print("   📊 Total elements: \(totalElements)")
    print("   📝 Text elements: \(textElements)")

    if chatTexts.isEmpty {
        print("   ❌ Không đọc được tin nhắn")
    } else {
        print("   ✅ Đọc được \(chatTexts.count) text, ví dụ:")
        for text in chatTexts.suffix(3) {
            print("      → \"\(text.prefix(80))\"")
        }
    }

    print()
}

print("=== DONE ===")
