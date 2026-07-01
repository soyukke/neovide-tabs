import AppKit
import MetalKit

final class TerminalMetalView: MTKView {
    override var acceptsFirstResponder: Bool { true }

    required init(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    init(frame frameRect: NSRect) {
        guard let device = MTLCreateSystemDefaultDevice() else {
            fatalError("Metal is not available")
        }

        super.init(frame: frameRect, device: device)
        colorPixelFormat = .bgra8Unorm
        clearColor = MTLClearColor(red: 0.078, green: 0.086, blue: 0.102, alpha: 1.0)
        enableSetNeedsDisplay = false
        isPaused = false
        preferredFramesPerSecond = 120
    }

    override func keyDown(with event: NSEvent) {
        // Future path: forward key events to the Rust terminal core through the
        // native command router.
    }

    override func rightMouseDown(with event: NSEvent) {
        let menu = NSMenu()
        menu.addItem(withTitle: "Rename Session", action: nil, keyEquivalent: "")
        menu.addItem(NSMenuItem.separator())
        menu.addItem(withTitle: "Graphite", action: nil, keyEquivalent: "")
        menu.addItem(withTitle: "Juniper", action: nil, keyEquivalent: "")
        NSMenu.popUpContextMenu(menu, with: event, for: self)
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?

    func applicationDidFinishLaunching(_ notification: Notification) {
        let contentRect = NSRect(x: 0, y: 0, width: 1100, height: 720)
        let window = NSWindow(
            contentRect: contentRect,
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )

        window.title = "neovide-tabs native shell spike"
        window.tabbingMode = .preferred
        window.contentView = TerminalMetalView(frame: contentRect)
        window.makeKeyAndOrderFront(nil)
        self.window = window

        NSApp.activate(ignoringOtherApps: true)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.regular)
app.run()
