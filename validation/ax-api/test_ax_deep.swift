// Deep scan — in toàn bộ AX hierarchy của Zalo
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

var totalElements = 0

func scanAll(_ element: AXUIElement, depth: Int = 0) {
    if depth > 15 { return }
    totalElements += 1

    var role: CFTypeRef?
    var value: CFTypeRef?
    var title: CFTypeRef?
    var desc: CFTypeRef?
    var roleDesc: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)
    AXUIElementCopyAttributeValue(element, kAXTitleAttribute as CFString, &title)
    AXUIElementCopyAttributeValue(element, kAXDescriptionAttribute as CFString, &desc)
    AXUIElementCopyAttributeValue(element, kAXRoleDescriptionAttribute as CFString, &roleDesc)

    let indent = String(repeating: "  ", count: depth)
    let roleStr = (role as? String) ?? "?"
    let valueStr = (value as? String) ?? ""
    let titleStr = (title as? String) ?? ""
    let descStr = (desc as? String) ?? ""

    // Print all elements that have any info
    let hasInfo = !valueStr.isEmpty || !titleStr.isEmpty || !descStr.isEmpty
    let isInteresting = roleStr.contains("Web") || roleStr.contains("HTML") || roleStr.contains("Group") || roleStr.contains("Scroll") || roleStr.contains("Text") || roleStr.contains("List") || roleStr.contains("Cell") || roleStr.contains("Image")

    if hasInfo || isInteresting || depth <= 4 {
        var parts = ["\(indent)[\(roleStr)]"]
        if !titleStr.isEmpty { parts.append("title=\"\(titleStr)\"") }
        if !valueStr.isEmpty { parts.append("value=\"\(valueStr)\"") }
        if !descStr.isEmpty { parts.append("desc=\"\(descStr)\"") }
        print(parts.joined(separator: " "))
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(200) {
            scanAll(kid, depth: depth + 1)
        }
    }
}

print("--- Deep scan Zalo hierarchy ---\n")
scanAll(windows[0])
print("\n--- Total elements: \(totalElements) ---")
