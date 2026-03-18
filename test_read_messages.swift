// Đọc tin nhắn mới nhất từ Zalo Desktop
import Cocoa
import ApplicationServices

let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == "Zalo" }
guard let zalo = apps.first else {
    print("❌ Zalo chưa mở")
    exit(1)
}

let appRef = AXUIElementCreateApplication(zalo.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
    print("❌ Không lấy được window")
    exit(1)
}

var messages: [String] = []

func findMessages(_ element: AXUIElement, depth: Int = 0) {
    if depth > 20 { return }

    var desc: CFTypeRef?
    var value: CFTypeRef?
    var role: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute as CFString, &desc)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)

    let descStr = (desc as? String) ?? ""
    let valueStr = (value as? String) ?? ""
    let roleStr = (role as? String) ?? ""

    // Tìm desc chứa nội dung chat (có timestamp pattern như HH:MM)
    if !descStr.isEmpty && descStr.count > 20 {
        // Check if it looks like chat content (contains time patterns)
        let hasTime = descStr.range(of: #"\d{1,2}:\d{2}"#, options: .regularExpression) != nil
        if hasTime {
            messages.append(descStr)
        }
    }

    // Cũng check AXStaticText value
    if roleStr == "AXStaticText" && !valueStr.isEmpty {
        // Skip UI labels
        let uiLabels = ["Tìm kiếm", "Tất cả", "Chưa đọc", "Phân loại", "Zalo -", ""]
        let isUI = uiLabels.contains(where: { valueStr.hasPrefix($0) })
        if !isUI && valueStr.count > 0 {
            // Could be message content at individual level
        }
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(200) {
            findMessages(kid, depth: depth + 1)
        }
    }
}

findMessages(windows[0])

print("=== TIN NHẮN TỪ ZALO ===\n")

for (i, msg) in messages.enumerated() {
    print("--- Block \(i + 1) ---")
    // Parse: tách theo pattern "tên người + nội dung + thời gian"
    // Desc format: "CN 15/03/2026 Phan Trung Kiên E chào chị 15:57 Nhà mẹ Tiên Cô chào con! 15:57"
    print(msg)
    print()
}

if messages.isEmpty {
    print("Không tìm thấy tin nhắn. Hãy mở 1 cuộc chat trong Zalo.")
}
