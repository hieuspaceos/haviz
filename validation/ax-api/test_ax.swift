// Test Accessibility API trên Zalo Desktop
// Chạy: swift test_ax.swift
// Yêu cầu: Zalo Desktop đang mở + Terminal có Accessibility permission

import Cocoa
import ApplicationServices

// Tìm Zalo process
let apps = NSWorkspace.shared.runningApplications.filter { $0.localizedName == "Zalo" }
guard let zalo = apps.first else {
    print("❌ Zalo Desktop chưa mở. Mở Zalo lên rồi chạy lại.")
    exit(1)
}

print("✅ Tìm thấy Zalo PID: \(zalo.processIdentifier)")

let appRef = AXUIElementCreateApplication(zalo.processIdentifier)

// Lấy tất cả windows
var windowsRef: CFTypeRef?
AXUIElementCopyAttributeValue(appRef, kAXWindowsAttribute as CFString, &windowsRef)

guard let windows = windowsRef as? [AXUIElement], !windows.isEmpty else {
    print("❌ Không lấy được window")
    print("👉 Vào System Settings → Privacy & Security → Accessibility")
    print("   Thêm Terminal (hoặc app đang chạy script) vào danh sách")
    exit(1)
}

print("✅ Số windows: \(windows.count)")

// Duyệt recursive để tìm text
var textCount = 0

func findTexts(_ element: AXUIElement, depth: Int = 0) {
    if depth > 10 { return }

    var role: CFTypeRef?
    var value: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXRoleAttribute as CFString, &role)
    AXUIElementCopyAttributeValue(element, kAXValueAttribute as CFString, &value)

    let indent = String(repeating: "  ", count: depth)
    let roleStr = (role as? String) ?? "?"
    let valueStr = (value as? String) ?? ""

    if !valueStr.isEmpty {
        print("\(indent)[\(roleStr)] \"\(valueStr)\"")
        textCount += 1
    }

    var children: CFTypeRef?
    AXUIElementCopyAttributeValue(element, kAXChildrenAttribute as CFString, &children)
    if let kids = children as? [AXUIElement] {
        for kid in kids.prefix(100) {
            findTexts(kid, depth: depth + 1)
        }
    }
}

print("\n--- Scanning Zalo window for text content ---\n")
findTexts(windows[0])

print("\n--- Kết quả ---")
if textCount > 0 {
    print("✅ Tìm thấy \(textCount) text elements — AX API HOẠT ĐỘNG!")
    print("👉 Có thể build Haviz với Accessibility API approach")
} else {
    print("❌ Không tìm thấy text nào")
    print("👉 Zalo có thể block AX API hoặc render bằng canvas/WebView")
    print("👉 Thử Accessibility Inspector để xem chi tiết hơn")
    print("👉 Backup plan: Browser extension cho Zalo Web")
}
