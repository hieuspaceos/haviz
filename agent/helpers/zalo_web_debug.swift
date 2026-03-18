// Debug: dump ALL text elements from Zalo Web browser window
// Shows depth + role + value for every text element found
import Cocoa
import ApplicationServices

let browserName = CommandLine.arguments.count > 1
    ? CommandLine.arguments[1...].joined(separator: " ")
    : "Google Chrome"

let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == browserName }
guard let app = apps.first else { fputs("browser not running: \(browserName)\n", stderr); exit(1) }

let appRef = AXUIElementCreateApplication(app.processIdentifier)
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)
guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else { fputs("no windows\n", stderr); exit(1) }

var zaloWindow: AXUIElement? = nil
for window in windows {
    var title: CFTypeRef?
    AXUIElementCopyAttributeValue(window, kAXTitleAttribute as CFString, &title)
    if ((title as? String) ?? "").lowercased().contains("zalo") {
        zaloWindow = window
        break
    }
}
guard let target = zaloWindow else { fputs("no zalo tab\n", stderr); exit(1) }

struct TextItem: Codable {
    let depth: Int
    let role: String
    let value: String
    let desc: String
}

var items: [TextItem] = []

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

    if !valueStr.isEmpty || (!descStr.isEmpty && descStr.count > 5) {
        items.append(TextItem(depth: depth, role: roleStr, value: valueStr, desc: String(descStr.prefix(200))))
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(300) {
            scan(kid, depth: depth + 1)
        }
    }
}

scan(target)

let encoder = JSONEncoder()
encoder.outputFormatting = .prettyPrinted
if let data = try? encoder.encode(items), let str = String(data: data, encoding: .utf8) {
    print(str)
}
