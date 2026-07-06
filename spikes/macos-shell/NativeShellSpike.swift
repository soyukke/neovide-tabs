import AppKit
import Foundation
import MetalKit

@_silgen_name("nvterm_core_create")
func nvterm_core_create() -> UnsafeMutableRawPointer?

@_silgen_name("nvterm_core_destroy")
func nvterm_core_destroy(_ handle: UnsafeMutableRawPointer?)

@_silgen_name("nvterm_core_new_tab")
func nvterm_core_new_tab(_ handle: UnsafeMutableRawPointer?) -> Int

@_silgen_name("nvterm_core_split_active")
func nvterm_core_split_active(_ handle: UnsafeMutableRawPointer?, _ axis: UInt32) -> Int

@_silgen_name("nvterm_core_select_tab")
func nvterm_core_select_tab(_ handle: UnsafeMutableRawPointer?, _ index: Int) -> UInt8

@_silgen_name("nvterm_core_rename_tab")
func nvterm_core_rename_tab(
    _ handle: UnsafeMutableRawPointer?,
    _ index: Int,
    _ title: UnsafePointer<CChar>?
) -> UInt8

@_silgen_name("nvterm_core_set_tab_theme")
func nvterm_core_set_tab_theme(
    _ handle: UnsafeMutableRawPointer?,
    _ index: Int,
    _ theme: UnsafePointer<CChar>?
) -> UInt8

@_silgen_name("nvterm_core_snapshot_json")
func nvterm_core_snapshot_json(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_renderer_contract_json")
func nvterm_renderer_contract_json() -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_runtime_create")
func nvterm_runtime_create(
    _ rows: UInt16,
    _ cols: UInt16,
    _ pixelWidth: UInt16,
    _ pixelHeight: UInt16
) -> UnsafeMutableRawPointer?

@_silgen_name("nvterm_runtime_destroy")
func nvterm_runtime_destroy(_ handle: UnsafeMutableRawPointer?)

@_silgen_name("nvterm_runtime_resize")
func nvterm_runtime_resize(
    _ handle: UnsafeMutableRawPointer?,
    _ rows: UInt16,
    _ cols: UInt16,
    _ pixelWidth: UInt16,
    _ pixelHeight: UInt16
) -> UInt8

@_silgen_name("nvterm_runtime_write")
func nvterm_runtime_write(
    _ handle: UnsafeMutableRawPointer?,
    _ bytes: UnsafePointer<UInt8>?,
    _ len: Int
) -> UInt8

@_silgen_name("nvterm_runtime_drain")
func nvterm_runtime_drain(_ handle: UnsafeMutableRawPointer?) -> UInt8

@_silgen_name("nvterm_runtime_scroll")
func nvterm_runtime_scroll(_ handle: UnsafeMutableRawPointer?, _ requestedRows: Int) -> Int

@_silgen_name("nvterm_runtime_frame_json")
func nvterm_runtime_frame_json(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_runtime_renderer_scroll_position")
func nvterm_runtime_renderer_scroll_position(_ handle: UnsafeMutableRawPointer?) -> Float

@_silgen_name("nvterm_runtime_cwd")
func nvterm_runtime_cwd(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_nvim_create")
func nvterm_nvim_create(
    _ rows: UInt16,
    _ cols: UInt16,
    _ pixelWidth: UInt16,
    _ pixelHeight: UInt16
) -> UnsafeMutableRawPointer?

@_silgen_name("nvterm_nvim_create_in_cwd")
func nvterm_nvim_create_in_cwd(
    _ rows: UInt16,
    _ cols: UInt16,
    _ pixelWidth: UInt16,
    _ pixelHeight: UInt16,
    _ cwd: UnsafePointer<CChar>?
) -> UnsafeMutableRawPointer?

@_silgen_name("nvterm_nvim_destroy")
func nvterm_nvim_destroy(_ handle: UnsafeMutableRawPointer?)

@_silgen_name("nvterm_nvim_resize")
func nvterm_nvim_resize(
    _ handle: UnsafeMutableRawPointer?,
    _ rows: UInt16,
    _ cols: UInt16,
    _ pixelWidth: UInt16,
    _ pixelHeight: UInt16
) -> UInt8

@_silgen_name("nvterm_nvim_input")
func nvterm_nvim_input(
    _ handle: UnsafeMutableRawPointer?,
    _ bytes: UnsafePointer<UInt8>?,
    _ len: Int
) -> UInt8

@_silgen_name("nvterm_nvim_command")
func nvterm_nvim_command(
    _ handle: UnsafeMutableRawPointer?,
    _ command: UnsafePointer<CChar>?
) -> UInt8

@_silgen_name("nvterm_nvim_drain")
func nvterm_nvim_drain(_ handle: UnsafeMutableRawPointer?) -> UInt8

@_silgen_name("nvterm_nvim_exited")
func nvterm_nvim_exited(_ handle: UnsafeMutableRawPointer?) -> UInt8

@_silgen_name("nvterm_nvim_frame_json")
func nvterm_nvim_frame_json(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_nvim_renderer_model_json")
func nvterm_nvim_renderer_model_json(_ handle: UnsafeMutableRawPointer?) -> UnsafeMutablePointer<CChar>?

@_silgen_name("nvterm_string_free")
func nvterm_string_free(_ value: UnsafeMutablePointer<CChar>?)

@_silgen_name("nvterm_skia_metal_create")
func nvterm_skia_metal_create(
    _ device: UnsafeMutableRawPointer?,
    _ commandQueue: UnsafeMutableRawPointer?
) -> UnsafeMutableRawPointer?

@_silgen_name("nvterm_skia_metal_destroy")
func nvterm_skia_metal_destroy(_ handle: UnsafeMutableRawPointer?)

@_silgen_name("nvterm_skia_metal_render_nvim")
func nvterm_skia_metal_render_nvim(
    _ renderer: UnsafeMutableRawPointer?,
    _ nvim: UnsafeMutableRawPointer?,
    _ texture: UnsafeMutableRawPointer?,
    _ width: Int32,
    _ height: Int32,
    _ originX: Float,
    _ originY: Float,
    _ cellWidth: Float,
    _ cellHeight: Float
) -> UInt8

@_silgen_name("nvterm_skia_metal_render_terminal")
func nvterm_skia_metal_render_terminal(
    _ renderer: UnsafeMutableRawPointer?,
    _ runtime: UnsafeMutableRawPointer?,
    _ texture: UnsafeMutableRawPointer?,
    _ width: Int32,
    _ height: Int32,
    _ originX: Float,
    _ originY: Float,
    _ cellWidth: Float,
    _ cellHeight: Float
) -> UInt8

@_silgen_name("nvterm_skia_metal_needs_animation_frame")
func nvterm_skia_metal_needs_animation_frame(_ renderer: UnsafeMutableRawPointer?) -> UInt8

@_silgen_name("nvterm_skia_metal_next_frame_delay_ms")
func nvterm_skia_metal_next_frame_delay_ms(_ renderer: UnsafeMutableRawPointer?) -> UInt64

private let ffiSplitVertical: UInt32 = 0
private let ffiSplitHorizontal: UInt32 = 1
private let themes = ["Graphite", "Juniper", "Harbor", "Rose", "Paper"]
private let terminalBackground = NSColor(
    calibratedRed: 20.0 / 255.0,
    green: 22.0 / 255.0,
    blue: 26.0 / 255.0,
    alpha: 1.0
)
private let terminalHorizontalInset: CGFloat = 12
private let terminalTextTop: CGFloat = 38
private let terminalTextBottomInset: CGFloat = 10
private let defaultTerminalFontSize: CGFloat = 15
private let minTerminalFontSize: CGFloat = 9
private let maxTerminalFontSize: CGFloat = 32
private let cursorTrailAlpha: CGFloat = 145.0 / 255.0
private let maxOutputScrollAnimationRows = 12
private let maxFullFrameScrollAnimationRows = 24
private let maxScrollRegionDetectionRows = 80
private let minFullFrameScrollMatchRows = 4
private let minTerminalVimScrollSmokePosition = 3.0
private let maxTerminalBottomInputSmokePosition = 0.1
private let minJumpAnimationContentRows = 8
private let minScrollRegionContentRows = 2
private let outputScrollAnimationFarLines = 1
private let themeAccentColors: [String: NSColor] = [
    "Graphite": NSColor(calibratedRed: 0.54, green: 0.56, blue: 0.62, alpha: 1.0),
    "Juniper": NSColor(calibratedRed: 0.18, green: 0.62, blue: 0.43, alpha: 1.0),
    "Harbor": NSColor(deviceRed: 0.10, green: 0.50, blue: 0.82, alpha: 1.0),
    "Rose": NSColor(calibratedRed: 0.78, green: 0.32, blue: 0.48, alpha: 1.0),
    "Paper": NSColor(calibratedRed: 0.78, green: 0.57, blue: 0.26, alpha: 1.0),
]

struct TerminalCoreSnapshot: Decodable {
    let active_tab: Int
    let tabs: [TerminalCoreTabSnapshot]
}

struct TerminalCoreTabSnapshot: Decodable {
    let index: Int
    let title: String
    let active_pane: Int
    let theme: String
    let panes: [TerminalCorePaneSnapshot]
}

struct TerminalCorePaneSnapshot: Decodable {
    let id: Int
    let cwd: String?
}

struct RendererContract: Decodable {
    let backend: String
    let surface: RendererSurfaceContract
    let cursor: CursorAnimationContract
    let scroll: ScrollAnimationContract
    let images: ImageProtocolContract
}

struct RendererSurfaceContract: Decodable {
    let view: String
    let pixel_format: String
    let preferred_frames_per_second: Int
}

struct CursorAnimationContract: Decodable {
    let neovide_like_trail: Bool
    let duration_seconds: Double
}

struct ScrollAnimationContract: Decodable {
    let smooth_history_scroll: Bool
    let output_shift_animation: Bool
}

struct ImageProtocolContract: Decodable {
    let kitty_graphics_protocol: Bool
    let iterm2_inline_images: Bool
}

struct TerminalFrameSnapshot: Decodable {
    let rows: [[TerminalCellSnapshot]]
    let row_updates: [TerminalRowUpdate]?
    let full_refresh: Bool?
    let commands: [TerminalDrawCommand]?
    let background: TerminalColorSnapshot
    let cursor_color: TerminalColorSnapshot
    let cursor: TerminalCursorSnapshot?
    let scrollbar: ScrollbarSnapshot
    let semantic_scroll: Bool?
    let scroll_hint: FrameScrollHint?

    func replacingRows(_ rows: [[TerminalCellSnapshot]]) -> TerminalFrameSnapshot {
        TerminalFrameSnapshot(
            rows: rows,
            row_updates: nil,
            full_refresh: true,
            commands: commands,
            background: background,
            cursor_color: cursor_color,
            cursor: cursor,
            scrollbar: scrollbar,
            semantic_scroll: semantic_scroll,
            scroll_hint: scroll_hint
        )
    }
}

struct TerminalRowUpdate: Decodable {
    let row: Int
    let cells: [TerminalCellSnapshot]
}

struct TerminalDrawCommand: Decodable {
    let grid_id: Int
    let kind: String
}

struct NeovideRendererModelSnapshot: Decodable {
    let schema_version: Int
    let background: TerminalColorSnapshot
    let cursor_color: TerminalColorSnapshot
    let cursor: TerminalCursorSnapshot?
    let scroll_hint: FrameScrollHint?
    let windows: [NeovideRenderedWindowSnapshot]
}

struct NeovideRenderedWindowSnapshot: Decodable {
    let grid_id: Int
    let top: Int
    let left: Int
    let width: Int
    let height: Int
    let window_kind: String
    let zindex: Int
    let compindex: Int
    let hidden: Bool
    let scroll_position: Double
    let lines: [NeovideLineSnapshot?]
}

struct NeovideLineSnapshot: Decodable {
    let text: String
    let cells: [TerminalCellSnapshot]
}

struct TerminalCellSnapshot: Decodable, Equatable {
    let text: String
    let fg: TerminalColorSnapshot
    let bg: TerminalColorSnapshot?
    let blend: UInt8
    let style: TerminalCellStyleSnapshot
}

struct TerminalCellStyleSnapshot: Decodable, Equatable {
    let bold: Bool
    let italic: Bool
    let underline: Bool
    let strikethrough: Bool

    static let plain = TerminalCellStyleSnapshot(
        bold: false,
        italic: false,
        underline: false,
        strikethrough: false
    )
}

struct TerminalColorSnapshot: Decodable, Equatable {
    let r: UInt8
    let g: UInt8
    let b: UInt8
}

struct TerminalCursorSnapshot: Decodable, Equatable {
    let x: UInt16
    let y: UInt16
    let style: String
    let cell_percentage: UInt8
    let blinkwait_ms: UInt64
    let blinkon_ms: UInt64
    let blinkoff_ms: UInt64
}

struct ScrollbarSnapshot: Decodable {
    let top: UInt64
    let visible: UInt64
    let total: UInt64
}

struct FrameScrollHint: Decodable {
    let start_row: Int
    let end_row: Int
    let start_col: Int?
    let end_col: Int?
    let rows: Int

    var outputShift: OutputScrollShift {
        OutputScrollShift(
            startRow: start_row,
            endRow: end_row,
            rows: rows,
            startCol: start_col,
            endCol: end_col
        )
    }
}

struct RowScrollAnimation {
    let startRow: Int
    let endRow: Int
    let startCol: Int?
    let endCol: Int?
    let rows: Int
    let previousRows: [[TerminalCellSnapshot]]
    var visualOffsetRows: CGFloat
    var velocityRows: CGFloat

    func contains(row: Int) -> Bool {
        row >= startRow && row <= endRow
    }

    func contains(row: Int, col: Int) -> Bool {
        guard contains(row: row) else {
            return false
        }
        if let startCol, col < startCol {
            return false
        }
        if let endCol, col > endCol {
            return false
        }
        return true
    }

    var isColumnBounded: Bool {
        startCol != nil || endCol != nil
    }
}

struct OutputScrollShift {
    let startRow: Int
    let endRow: Int
    let startCol: Int?
    let endCol: Int?
    let rows: Int

    init(startRow: Int, endRow: Int, rows: Int, startCol: Int? = nil, endCol: Int? = nil) {
        self.startRow = startRow
        self.endRow = endRow
        self.startCol = startCol
        self.endCol = endCol
        self.rows = rows
    }
}

struct ScrollShiftCandidate {
    let shift: OutputScrollShift
    let score: Int
    let contentRows: Int
}

struct SkiaRenderGeometry {
    let originX: Float
    let originY: Float
    let cellWidth: Float
    let cellHeight: Float
}

final class RustCore {
    private let handle: UnsafeMutableRawPointer

    init?() {
        guard let handle = nvterm_core_create() else {
            return nil
        }
        self.handle = handle
    }

    deinit {
        nvterm_core_destroy(handle)
    }

    func snapshot() -> TerminalCoreSnapshot? {
        decode(nvterm_core_snapshot_json(handle), as: TerminalCoreSnapshot.self)
    }

    func rendererContract() -> RendererContract? {
        decode(nvterm_renderer_contract_json(), as: RendererContract.self)
    }

    func newTab() {
        _ = nvterm_core_new_tab(handle)
    }

    func splitActive(axis: UInt32) {
        _ = nvterm_core_split_active(handle, axis)
    }

    func selectTab(_ index: Int) -> Bool {
        nvterm_core_select_tab(handle, index) != 0
    }

    func renameTab(_ index: Int, title: String) {
        title.withCString { value in
            _ = nvterm_core_rename_tab(handle, index, value)
        }
    }

    func setTheme(_ theme: String, tab index: Int) {
        theme.withCString { value in
            _ = nvterm_core_set_tab_theme(handle, index, value)
        }
    }

    private func decode<T: Decodable>(_ pointer: UnsafeMutablePointer<CChar>?, as type: T.Type) -> T? {
        guard let pointer else {
            return nil
        }
        defer {
            nvterm_string_free(pointer)
        }

        let json = String(cString: pointer)
        return try? JSONDecoder().decode(T.self, from: Data(json.utf8))
    }
}

protocol NativePane: AnyObject {
    var kind: NativePaneMode { get }

    func resize(grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int))
    func write(_ data: Data)
    func runCommand(_ command: String) -> Bool
    func drain() -> Bool
    func isExited() -> Bool
    func currentWorkingDirectory() -> String?
    func scroll(rows: Int) -> Int
    func frame() -> TerminalFrameSnapshot?
    func rendererModel() -> NeovideRendererModelSnapshot?
    func rendererScrollPosition() -> Double
    func renderHandle() -> UnsafeMutableRawPointer?
}

enum NativePaneMode {
    case terminal
    case neovim

    static func current() -> Self {
        ProcessInfo.processInfo.environment["NVTERM_NATIVE_PANE"] == "nvim" ? .neovim : .terminal
    }
}

final class RustTerminalPane: NativePane {
    let kind = NativePaneMode.terminal
    private let handle: UnsafeMutableRawPointer

    init?(grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int)) {
        guard let handle = nvterm_runtime_create(
            clampedUInt16(grid.rows),
            clampedUInt16(grid.cols),
            clampedUInt16(grid.widthPixels),
            clampedUInt16(grid.heightPixels)
        ) else {
            return nil
        }
        self.handle = handle
    }

    deinit {
        nvterm_runtime_destroy(handle)
    }

    func resize(grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int)) {
        _ = nvterm_runtime_resize(
            handle,
            clampedUInt16(grid.rows),
            clampedUInt16(grid.cols),
            clampedUInt16(grid.widthPixels),
            clampedUInt16(grid.heightPixels)
        )
    }

    func write(_ data: Data) {
        data.withUnsafeBytes { buffer in
            guard let base = buffer.bindMemory(to: UInt8.self).baseAddress else {
                return
            }
            _ = nvterm_runtime_write(handle, base, buffer.count)
        }
    }

    func runCommand(_ command: String) -> Bool {
        false
    }

    @discardableResult
    func drain() -> Bool {
        nvterm_runtime_drain(handle) != 0
    }

    func isExited() -> Bool {
        false
    }

    func currentWorkingDirectory() -> String? {
        guard let pointer = nvterm_runtime_cwd(handle) else {
            return nil
        }
        defer {
            nvterm_string_free(pointer)
        }

        let value = String(cString: pointer)
        return value.isEmpty ? nil : value
    }

    func scroll(rows: Int) -> Int {
        nvterm_runtime_scroll(handle, rows)
    }

    func frame() -> TerminalFrameSnapshot? {
        decode(nvterm_runtime_frame_json(handle), as: TerminalFrameSnapshot.self)
    }

    func rendererModel() -> NeovideRendererModelSnapshot? {
        nil
    }

    func rendererScrollPosition() -> Double {
        Double(nvterm_runtime_renderer_scroll_position(handle))
    }

    func renderHandle() -> UnsafeMutableRawPointer? {
        handle
    }

    private func decode<T: Decodable>(_ pointer: UnsafeMutablePointer<CChar>?, as type: T.Type) -> T? {
        guard let pointer else {
            return nil
        }
        defer {
            nvterm_string_free(pointer)
        }

        let json = String(cString: pointer)
        return try? JSONDecoder().decode(T.self, from: Data(json.utf8))
    }
}

final class RustNeovimPane: NativePane {
    let kind = NativePaneMode.neovim
    private let handle: UnsafeMutableRawPointer

    init?(grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int), cwd: String? = nil) {
        let handle = cwd.flatMap { directory in
            directory.withCString { value in
                nvterm_nvim_create_in_cwd(
                    clampedUInt16(grid.rows),
                    clampedUInt16(grid.cols),
                    clampedUInt16(grid.widthPixels),
                    clampedUInt16(grid.heightPixels),
                    value
                )
            }
        } ?? nvterm_nvim_create(
            clampedUInt16(grid.rows),
            clampedUInt16(grid.cols),
            clampedUInt16(grid.widthPixels),
            clampedUInt16(grid.heightPixels)
        )
        guard let handle = handle else {
            return nil
        }
        self.handle = handle
    }

    deinit {
        nvterm_nvim_destroy(handle)
    }

    func resize(grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int)) {
        _ = nvterm_nvim_resize(
            handle,
            clampedUInt16(grid.rows),
            clampedUInt16(grid.cols),
            clampedUInt16(grid.widthPixels),
            clampedUInt16(grid.heightPixels)
        )
    }

    func write(_ data: Data) {
        data.withUnsafeBytes { buffer in
            guard let base = buffer.bindMemory(to: UInt8.self).baseAddress else {
                return
            }
            _ = nvterm_nvim_input(handle, base, buffer.count)
        }
    }

    func runCommand(_ command: String) -> Bool {
        command.withCString { value in
            nvterm_nvim_command(handle, value) != 0
        }
    }

    @discardableResult
    func drain() -> Bool {
        nvterm_nvim_drain(handle) != 0
    }

    func isExited() -> Bool {
        nvterm_nvim_exited(handle) != 0
    }

    func currentWorkingDirectory() -> String? {
        nil
    }

    func scroll(rows: Int) -> Int {
        0
    }

    func frame() -> TerminalFrameSnapshot? {
        decode(nvterm_nvim_frame_json(handle), as: TerminalFrameSnapshot.self)
    }

    func rendererModel() -> NeovideRendererModelSnapshot? {
        decode(nvterm_nvim_renderer_model_json(handle), as: NeovideRendererModelSnapshot.self)
    }

    func rendererScrollPosition() -> Double {
        0
    }

    func renderHandle() -> UnsafeMutableRawPointer? {
        handle
    }

    private func decode<T: Decodable>(_ pointer: UnsafeMutablePointer<CChar>?, as type: T.Type) -> T? {
        guard let pointer else {
            return nil
        }
        defer {
            nvterm_string_free(pointer)
        }

        let json = String(cString: pointer)
        return try? JSONDecoder().decode(T.self, from: Data(json.utf8))
    }
}

struct NeovimLaunchRequest {
    let file: String?
}

struct TerminalInputCommandBuffer {
    private(set) var command = ""

    mutating func observe(_ data: Data) -> NeovimLaunchRequest? {
        var request: NeovimLaunchRequest?
        for byte in data {
            if let next = observe(byte) {
                request = next
            }
        }
        return request
    }

    private mutating func observe(_ byte: UInt8) -> NeovimLaunchRequest? {
        switch byte {
        case 3, 21:
            command.removeAll()
            return nil
        case 8, 127:
            if !command.isEmpty {
                command.removeLast()
            }
            return nil
        case 10, 13:
            defer {
                command.removeAll()
            }
            return parseNeovimLaunch(command)
        case 32...126:
            command.append(Character(UnicodeScalar(byte)))
            return nil
        default:
            return nil
        }
    }
}

private func parseNeovimLaunch(_ command: String) -> NeovimLaunchRequest? {
    let tokens = shellLikeTokens(command.trimmingCharacters(in: .whitespacesAndNewlines))
    guard let executable = tokens.first,
          neovimExecutableNames.contains((executable as NSString).lastPathComponent)
    else {
        return nil
    }

    return NeovimLaunchRequest(file: tokens.dropFirst().first(where: neovimFileArgument))
}

private let neovimExecutableNames = ["nvim", "vim"]

private func neovimFileArgument(_ token: String) -> Bool {
    !token.isEmpty && !token.hasPrefix("-") && !token.hasPrefix("+")
}

private func shellLikeTokens(_ value: String) -> [String] {
    var tokens: [String] = []
    var current = ""
    var quote: Character?
    var escaping = false

    for character in value {
        if escaping {
            current.append(character)
            escaping = false
            continue
        }
        if character == "\\" {
            escaping = true
            continue
        }
        if let activeQuote = quote {
            if character == activeQuote {
                quote = nil
            } else {
                current.append(character)
            }
            continue
        }
        if character == "'" || character == "\"" {
            quote = character
            continue
        }
        if character.isWhitespace {
            if !current.isEmpty {
                tokens.append(current)
                current.removeAll()
            }
            continue
        }
        current.append(character)
    }

    if !current.isEmpty {
        tokens.append(current)
    }
    return tokens
}

private func shellQuote(_ value: String) -> String {
    "'\(value.replacingOccurrences(of: "'", with: "'\\''"))'"
}

private func vimSingleQuote(_ value: String) -> String {
    value.replacingOccurrences(of: "'", with: "''")
}

final class RenameTextField: NSTextField, NSTextFieldDelegate {
    var onCommit: (() -> Void)?

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        delegate = self
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
        delegate = self
    }

    func control(_ control: NSControl, textView: NSTextView, doCommandBy commandSelector: Selector) -> Bool {
        if commandSelector == #selector(NSResponder.insertNewline(_:)) {
            onCommit?()
            return true
        }
        return false
    }
}

final class TerminalTextView: NSView {
    var onInput: ((Data) -> Void)?
    var onScroll: ((CGFloat) -> Void)?
    var onTabSelected: ((Int) -> Void)?
    var onContextMenuRequested: ((Int?, NSEvent, NSView) -> Void)?
    var onGeometryChanged: (() -> Void)?
    var onZoomIn: (() -> Void)?
    var onZoomOut: (() -> Void)?
    var onResetZoom: (() -> Void)?

    required init?(coder: NSCoder) {
        nil
    }

    private var frameSnapshot: TerminalFrameSnapshot?
    private var rendererModelSnapshot: NeovideRendererModelSnapshot?
    private var rendererModelFrameCount = 0
    private var externalRendererEnabled = false
    private var tabTitles: [String] = []
    private var tabThemes: [String] = []
    private var activeTab = 0
    private var cursorTrailStart: NSPoint?
    private var cursorTrailTarget: NSPoint?
    private var cursorTrailStartedAt: TimeInterval?
    private var scrollVisualOffsetRows: CGFloat = 0
    private var scrollVelocityRows: CGFloat = 0
    private var rowScrollAnimation: RowScrollAnimation?
    private var lastScrollRegionShift: OutputScrollShift?
    private var suppressNextOutputShiftAnimation = false
    private var lastAnimationFrameAt = CACurrentMediaTime()
    private var terminalFontSize = defaultTerminalFontSize
    private var terminalFont = NSFont.monospacedSystemFont(ofSize: defaultTerminalFontSize, weight: .regular)
    private let tabFont = NSFont.systemFont(ofSize: 12, weight: .semibold)
    private let terminalTextColor = NSColor(calibratedRed: 0.90, green: 0.88, blue: 0.85, alpha: 1.0)

    override init(frame frameRect: NSRect) {
        super.init(frame: frameRect)
        configure()
    }

    private func configure() {
        wantsLayer = true
        layer?.backgroundColor = terminalBackground.cgColor
    }

    override var acceptsFirstResponder: Bool {
        true
    }

    override var isFlipped: Bool {
        true
    }

    override var isOpaque: Bool {
        !externalRendererEnabled
    }

    override func draw(_ dirtyRect: NSRect) {
        if !externalRendererEnabled {
            terminalBackground.setFill()
            dirtyRect.fill()
        }
        drawTabStrip()
        guard !externalRendererEnabled else {
            return
        }
        drawTerminalFrame()
    }

    override func setFrameSize(_ newSize: NSSize) {
        let changed = frame.size != newSize
        super.setFrameSize(newSize)
        if changed {
            onGeometryChanged?()
        }
    }

    override func keyDown(with event: NSEvent) {
        if event.modifierFlags.contains(.command) {
            if handleCommandKey(event) {
                return
            }
            super.keyDown(with: event)
            return
        }
        guard let data = terminalInputData(for: event) else {
            return
        }
        onInput?(data)
    }

    override func mouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        if let index = tabIndex(at: point) {
            onTabSelected?(index)
            return
        }

        window?.makeFirstResponder(self)
    }

    override func rightMouseDown(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        onContextMenuRequested?(tabIndex(at: point), event, self)
    }

    override func scrollWheel(with event: NSEvent) {
        let point = convert(event.locationInWindow, from: nil)
        guard terminalTextRect().contains(point) else {
            super.scrollWheel(with: event)
            return
        }

        let rows = scrollRows(for: event)
        guard rows != 0 else {
            return
        }
        onScroll?(rows)
    }

    func setFrame(_ frame: TerminalFrameSnapshot?) {
        let nextFrame = materializedFrame(frame)
        animateOutputShiftIfNeeded(nextFrame: nextFrame)
        updateCursorTrail(to: nextFrame?.cursor)
        frameSnapshot = nextFrame
        rendererModelSnapshot = nil
        needsDisplay = true
    }

    func setRendererModel(_ model: NeovideRendererModelSnapshot?) {
        if externalRendererEnabled {
            resetCursorTrail()
        } else {
            updateCursorTrail(to: model?.cursor)
        }
        frameSnapshot = nil
        rendererModelSnapshot = model
        if model != nil {
            rendererModelFrameCount += 1
        }
        needsDisplay = true
    }

    private func materializedFrame(_ frame: TerminalFrameSnapshot?) -> TerminalFrameSnapshot? {
        guard let frame else {
            return nil
        }
        guard frame.full_refresh == false,
              frame.rows.isEmpty,
              let current = frameSnapshot
        else {
            return frame
        }

        var rows = current.rows
        for update in frame.row_updates ?? [] where rows.indices.contains(update.row) {
            rows[update.row] = update.cells
        }
        return frame.replacingRows(rows)
    }

    func advanceAnimation() {
        advanceScrollAnimation()
        guard let startedAt = cursorTrailStartedAt else {
            return
        }
        if CACurrentMediaTime() - startedAt >= 0.16 {
            cursorTrailStartedAt = nil
        }
        needsDisplay = true
    }

    func animateScrollRows(_ rows: Int, resetVelocity: Bool = false) {
        guard rows != 0 else {
            return
        }
        rowScrollAnimation = nil
        scrollVisualOffsetRows += CGFloat(rows)
        if resetVelocity {
            scrollVelocityRows = 0
        }
        needsDisplay = true
    }

    func animateScrollRegion(_ shift: OutputScrollShift, previousRows: [[TerminalCellSnapshot]]) {
        guard shift.rows != 0, shift.startRow <= shift.endRow else {
            return
        }

        lastScrollRegionShift = shift
        if let current = rowScrollAnimation,
           current.startRow == shift.startRow,
           current.endRow == shift.endRow,
           current.startCol == shift.startCol,
           current.endCol == shift.endCol {
            rowScrollAnimation = RowScrollAnimation(
                startRow: shift.startRow,
                endRow: shift.endRow,
                startCol: shift.startCol,
                endCol: shift.endCol,
                rows: shift.rows,
                previousRows: previousRows,
                visualOffsetRows: current.visualOffsetRows + CGFloat(shift.rows),
                velocityRows: 0
            )
        } else {
            rowScrollAnimation = RowScrollAnimation(
                startRow: shift.startRow,
                endRow: shift.endRow,
                startCol: shift.startCol,
                endCol: shift.endCol,
                rows: shift.rows,
                previousRows: previousRows,
                visualOffsetRows: CGFloat(shift.rows),
                velocityRows: 0
            )
        }
        needsDisplay = true
    }

    func clearLastScrollRegionShift() {
        lastScrollRegionShift = nil
    }

    func consumeLastScrollRegionShift() -> OutputScrollShift? {
        defer {
            lastScrollRegionShift = nil
        }
        return lastScrollRegionShift
    }

    func peekLastScrollRegionShift() -> OutputScrollShift? {
        lastScrollRegionShift
    }

    func hasRendererModelFrames() -> Bool {
        rendererModelFrameCount > 0
    }

    func rendererContentRowCount() -> Int {
        if let rendererModelSnapshot {
            return contentRowCount(rendererModelRows(rendererModelSnapshot))
        }
        return frameSnapshot.map { contentRowCount($0.rows) } ?? 0
    }

    func rendererModelContainsTexts(_ needles: [String]) -> Bool {
        rendererModelMissingTexts(needles).isEmpty
    }

    func rendererModelMissingTexts(_ needles: [String]) -> [String] {
        guard let rendererModelSnapshot else {
            return needles
        }
        let rows = rendererModelRows(rendererModelSnapshot)
        let cellTexts = rows.flatMap { row in row.map(\.text) }
        let rowText = rows
            .map { row in row.map(\.text).joined() }
            .joined(separator: "\n")
        return needles.filter { needle in
            !cellTexts.contains(needle) && !rowText.contains(needle)
        }
    }

    func rendererModelCellSummary(_ labels: [(label: String, text: String)]) -> String {
        let cells = labels.compactMap { label, text in
            rendererModelCellPosition(text).map { row, col in
                "\(label):\(row):\(col)"
            }
        }
        return cells.isEmpty ? "none" : cells.joined(separator: ",")
    }

    func rendererModelTextStartSummary(label: String, text: String) -> String {
        guard let position = rendererModelTextStartPosition(text) else {
            return "none"
        }
        return "\(label):\(position.row):\(position.col)"
    }

    func skiaGeometrySummary() -> String {
        let geometry = skiaRenderGeometry()
        return [
            Int(geometry.originX.rounded()),
            Int(geometry.originY.rounded()),
            Int(geometry.cellWidth.rounded()),
            Int(geometry.cellHeight.rounded()),
        ]
        .map(String.init)
        .joined(separator: ":")
    }

    private func rendererModelCellPosition(_ needle: String) -> (row: Int, col: Int)? {
        guard let rendererModelSnapshot else {
            return nil
        }
        let rows = rendererModelRows(rendererModelSnapshot)
        for (rowIndex, row) in rows.enumerated() {
            for (colIndex, cell) in row.enumerated()
            where cell.text == needle || cell.text.contains(needle) {
                return (rowIndex, colIndex)
            }
        }
        return nil
    }

    private func rendererModelTextStartPosition(_ needle: String) -> (row: Int, col: Int)? {
        guard let rendererModelSnapshot else {
            return nil
        }
        let rows = rendererModelRows(rendererModelSnapshot)
        for (rowIndex, row) in rows.enumerated() {
            let text = rowPlainText(row)
            guard let range = text.range(of: needle) else {
                continue
            }
            return (rowIndex, text.distance(from: text.startIndex, to: range.lowerBound))
        }
        return nil
    }

    func rendererModelTextOccurrences(_ needle: String) -> Int {
        guard let rendererModelSnapshot else {
            return 0
        }
        return rendererModelRows(rendererModelSnapshot).reduce(0) { count, row in
            count + rowPlainText(row).components(separatedBy: needle).count - 1
        }
    }

    func rendererModelBlendCellCount(minBlend: UInt8) -> Int {
        guard let rendererModelSnapshot else {
            return 0
        }
        return rendererModelRows(rendererModelSnapshot).reduce(0) { count, row in
            count + row.filter { $0.bg != nil && $0.blend >= minBlend }.count
        }
    }

    func rendererModelBlendCellSummary(minBlend: UInt8) -> String {
        guard let rendererModelSnapshot else {
            return "none"
        }
        let rows = rendererModelRows(rendererModelSnapshot)
        for (rowIndex, row) in rows.enumerated() {
            for (colIndex, cell) in row.enumerated()
            where cell.bg != nil && cell.blend >= minBlend && cell.text.trimmingCharacters(in: .whitespaces).isEmpty {
                guard let bg = cell.bg else {
                    continue
                }
                return [
                    rowIndex,
                    colIndex,
                    Int(cell.blend),
                    Int(bg.r),
                    Int(bg.g),
                    Int(bg.b),
                    Int(rendererModelSnapshot.background.r),
                    Int(rendererModelSnapshot.background.g),
                    Int(rendererModelSnapshot.background.b),
                ]
                .map(String.init)
                .joined(separator: ":")
            }
        }
        return "none"
    }

    func rendererModelWindowKindCounts() -> [String: Int] {
        guard let rendererModelSnapshot else {
            return [:]
        }
        return rendererModelSnapshot.windows.reduce(into: [:]) { counts, window in
            guard !window.hidden else {
                return
            }
            counts[window.window_kind, default: 0] += 1
        }
    }

    func rendererModelCursorSummary() -> String {
        guard let cursor = rendererModelSnapshot?.cursor else {
            return "none"
        }
        return "\(cursor.y):\(cursor.x)"
    }

    func rendererModelCursorDetailSummary() -> String {
        guard let cursor = rendererModelSnapshot?.cursor else {
            return "none"
        }
        return [
            String(cursor.y),
            String(cursor.x),
            cursor.style,
            String(cursor.cell_percentage),
            String(cursor.blinkwait_ms),
            String(cursor.blinkon_ms),
            String(cursor.blinkoff_ms),
        ]
        .joined(separator: ":")
    }

    func setExternalRendererEnabled(_ enabled: Bool) {
        externalRendererEnabled = enabled
        layer?.backgroundColor = enabled ? NSColor.clear.cgColor : terminalBackground.cgColor
        if enabled {
            resetCursorTrail()
        }
        needsDisplay = true
    }

    func resetScrollAnimation() {
        scrollVisualOffsetRows = 0
        scrollVelocityRows = 0
        rowScrollAnimation = nil
        lastScrollRegionShift = nil
    }

    func suppressNextOutputShift() {
        suppressNextOutputShiftAnimation = true
    }

    func updateTabs(_ titles: [String], themes: [String], active: Int) {
        tabTitles = titles
        tabThemes = themes
        activeTab = active
        needsDisplay = true
    }

    @discardableResult
    func setTerminalFontSize(_ size: CGFloat) -> Bool {
        let clampedSize = min(max(size, minTerminalFontSize), maxTerminalFontSize)
        guard abs(clampedSize - terminalFontSize) > 0.01 else {
            return false
        }

        terminalFontSize = clampedSize
        terminalFont = NSFont.monospacedSystemFont(ofSize: clampedSize, weight: .regular)
        needsDisplay = true
        onGeometryChanged?()
        return true
    }

    func zoomIn() -> Bool {
        setTerminalFontSize(terminalFontSize + 1)
    }

    func zoomOut() -> Bool {
        setTerminalFontSize(terminalFontSize - 1)
    }

    func resetZoom() -> Bool {
        setTerminalFontSize(defaultTerminalFontSize)
    }

    func terminalGridSize() -> (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int) {
        let textRect = terminalTextRect()
        let cellSize = terminalCellSize()
        let cols = max(1, Int(textRect.width / cellSize.width))
        let rows = max(1, Int(textRect.height / cellSize.height))
        let scale = window?.backingScaleFactor ?? NSScreen.main?.backingScaleFactor ?? 1
        let widthPixels = max(1, Int(textRect.width * scale))
        let heightPixels = max(1, Int(textRect.height * scale))
        return (rows, cols, widthPixels, heightPixels)
    }

    func skiaRenderGeometry() -> SkiaRenderGeometry {
        let textRect = terminalTextRect()
        let cellSize = terminalCellSize()
        let scale = window?.backingScaleFactor ?? NSScreen.main?.backingScaleFactor ?? 1
        return SkiaRenderGeometry(
            originX: Float(textRect.minX * scale),
            originY: Float(textRect.minY * scale),
            cellWidth: Float(cellSize.width * scale),
            cellHeight: Float(cellSize.height * scale)
        )
    }

    private func drawTabStrip() {
        NSColor(calibratedRed: 34.0 / 255.0, green: 36.0 / 255.0, blue: 46.0 / 255.0, alpha: 1.0)
            .setFill()
        NSRect(x: 0, y: 0, width: bounds.width, height: 30).fill()

        for (idx, rect) in tabRects().enumerated() {
            let theme = idx < tabThemes.count ? tabThemes[idx] : nil
            drawTab(title: tabTitles[idx], rect: rect, active: idx == activeTab, theme: theme)
        }
    }

    private func drawTab(title: String, rect: NSRect, active: Bool, theme: String?) {
        tabColor(theme: theme, active: active).setFill()
        NSBezierPath(roundedRect: rect, xRadius: 7, yRadius: 7).fill()

        let attributes: [NSAttributedString.Key: Any] = [
            .font: tabFont,
            .foregroundColor: NSColor.white,
        ]
        let size = (title as NSString).size(withAttributes: attributes)
        let textRect = NSRect(
            x: rect.midX - size.width * 0.5,
            y: rect.midY - size.height * 0.5,
            width: size.width,
            height: size.height
        )
        (title as NSString).draw(in: textRect, withAttributes: attributes)
    }

    private func drawTerminalFrame() {
        if let rendererModelSnapshot {
            drawNeovideRendererModel(rendererModelSnapshot)
            return
        }

        guard let frameSnapshot else {
            drawEmptyTerminal()
            return
        }

        let textRect = terminalTextRect()
        frameSnapshot.background.nsColor.setFill()
        textRect.fill()

        let cellSize = terminalCellSize()
        NSGraphicsContext.saveGraphicsState()
        textRect.clip()
        drawTerminalRows(frameSnapshot.rows, textRect: textRect, cellSize: cellSize)
        drawCursor(frameSnapshot.cursor, color: frameSnapshot.cursor_color, textRect: textRect, cellSize: cellSize)
        NSGraphicsContext.restoreGraphicsState()
        drawScrollbar(frameSnapshot.scrollbar, in: textRect)
    }

    private func drawNeovideRendererModel(_ model: NeovideRendererModelSnapshot) {
        let textRect = terminalTextRect()
        model.background.nsColor.setFill()
        textRect.fill()

        let cellSize = terminalCellSize()
        let rows = rendererModelRows(model)
        NSGraphicsContext.saveGraphicsState()
        textRect.clip()
        drawTerminalRows(rows, textRect: textRect, cellSize: cellSize)
        drawCursor(model.cursor, color: model.cursor_color, textRect: textRect, cellSize: cellSize)
        NSGraphicsContext.restoreGraphicsState()

        if let scrollbar = frameSnapshot?.scrollbar {
            drawScrollbar(scrollbar, in: textRect)
        }
    }

    private func rendererModelRows(_ model: NeovideRendererModelSnapshot) -> [[TerminalCellSnapshot]] {
        let mainWindow = model.windows.first { $0.grid_id == 1 }
        let grid = terminalGridSize()
        let rowCount = max(1, mainWindow?.height ?? grid.rows)
        let colCount = max(1, mainWindow?.width ?? grid.cols)
        var rows = Array(
            repeating: rendererModelBlankRow(cols: colCount, foreground: model.cursor_color),
            count: rowCount
        )

        for window in sortedRendererWindows(model.windows) where !window.hidden {
            overlayRendererWindow(window, into: &rows, foreground: model.cursor_color)
        }
        return rows
    }

    private func rendererModelBlankRow(
        cols: Int,
        foreground: TerminalColorSnapshot
    ) -> [TerminalCellSnapshot] {
        Array(
            repeating: TerminalCellSnapshot(
                text: " ",
                fg: foreground,
                bg: nil,
                blend: 0,
                style: .plain
            ),
            count: cols
        )
    }

    private func sortedRendererWindows(
        _ windows: [NeovideRenderedWindowSnapshot]
    ) -> [NeovideRenderedWindowSnapshot] {
        windows.sorted {
            ($0.zindex, $0.compindex, $0.grid_id) < ($1.zindex, $1.compindex, $1.grid_id)
        }
    }

    private func overlayRendererWindow(
        _ window: NeovideRenderedWindowSnapshot,
        into rows: inout [[TerminalCellSnapshot]],
        foreground: TerminalColorSnapshot
    ) {
        let maxRow = min(window.height, window.lines.count)
        for sourceRow in 0..<maxRow {
            let targetRow = window.top + sourceRow
            guard rows.indices.contains(targetRow),
                  let line = window.lines[sourceRow]
            else {
                continue
            }
            overlayRendererLine(
                line,
                targetRow: targetRow,
                left: window.left,
                width: window.width,
                rows: &rows,
                foreground: foreground
            )
        }
    }

    private func overlayRendererLine(
        _ line: NeovideLineSnapshot,
        targetRow: Int,
        left: Int,
        width: Int,
        rows: inout [[TerminalCellSnapshot]],
        foreground: TerminalColorSnapshot
    ) {
        let targetWidth = min(width, line.cells.count)
        guard targetWidth > 0 else {
            return
        }

        var row = rows[targetRow]
        if row.count < left + targetWidth {
            row.append(contentsOf: rendererModelBlankRow(
                cols: left + targetWidth - row.count,
                foreground: foreground
            ))
        }
        for sourceCol in 0..<targetWidth {
            row[left + sourceCol] = line.cells[sourceCol]
        }
        rows[targetRow] = row
    }

    private func drawTerminalRows(
        _ rows: [[TerminalCellSnapshot]],
        textRect: NSRect,
        cellSize: NSSize
    ) {
        guard let animation = rowScrollAnimation else {
            let shiftedTextRect = textRect.offsetBy(
                dx: 0,
                dy: scrollVisualOffsetRows * cellSize.height
            )
            for (rowIndex, row) in rows.enumerated() {
                drawTerminalRow(row, rowIndex: rowIndex, textRect: shiftedTextRect, cellSize: cellSize)
            }
            return
        }

        let baseTextRect = textRect.offsetBy(
            dx: 0,
            dy: scrollVisualOffsetRows * cellSize.height
        )
        for (rowIndex, row) in rows.enumerated() {
            if animation.contains(row: rowIndex) {
                drawTerminalRowOutsideScrollRegion(
                    row,
                    rowIndex: rowIndex,
                    animation: animation,
                    textRect: baseTextRect,
                    cellSize: cellSize
                )
                continue
            }
            drawTerminalRow(row, rowIndex: rowIndex, textRect: baseTextRect, cellSize: cellSize)
        }

        NSGraphicsContext.saveGraphicsState()
        frameSnapshot?.background.nsColor.setFill()
        let regionRect = scrollRegionRect(animation, textRect: textRect, cellSize: cellSize)
        regionRect.fill()
        regionRect.clip()
        let shiftedTextRect = baseTextRect.offsetBy(
            dx: 0,
            dy: animation.visualOffsetRows * cellSize.height
        )
        drawTerminalRows(
            rows,
            from: animation.startRow,
            through: animation.endRow,
            textRect: shiftedTextRect,
            cellSize: cellSize
        )
        drawScrollAnimationGap(
            animation,
            regionRect: regionRect,
            textRect: baseTextRect,
            cellSize: cellSize
        )
        NSGraphicsContext.restoreGraphicsState()
    }

    private func drawScrollAnimationGap(
        _ animation: RowScrollAnimation,
        regionRect: NSRect,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        let offsetRows = animation.visualOffsetRows
        guard abs(offsetRows) > 0.01 else {
            return
        }

        let gapHeight = min(abs(offsetRows) * cellSize.height, regionRect.height)
        let gapRect: NSRect
        if offsetRows > 0 {
            gapRect = NSRect(
                x: regionRect.minX,
                y: regionRect.minY,
                width: regionRect.width,
                height: gapHeight
            )
        } else {
            gapRect = NSRect(
                x: regionRect.minX,
                y: regionRect.maxY - gapHeight,
                width: regionRect.width,
                height: gapHeight
            )
        }

        let previousTextRect = textRect.offsetBy(
            dx: 0,
            dy: (offsetRows - CGFloat(animation.rows)) * cellSize.height
        )
        NSGraphicsContext.saveGraphicsState()
        gapRect.clip()
        drawTerminalRows(
            animation.previousRows,
            from: animation.startRow,
            through: animation.endRow,
            textRect: previousTextRect,
            cellSize: cellSize
        )
        NSGraphicsContext.restoreGraphicsState()
    }

    private func drawTerminalRows(
        _ rows: [[TerminalCellSnapshot]],
        from startRow: Int,
        through endRow: Int,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        let lastRow = min(endRow, rows.count - 1)
        guard startRow <= lastRow else {
            return
        }
        for rowIndex in startRow...lastRow {
            drawTerminalRow(rows[rowIndex], rowIndex: rowIndex, textRect: textRect, cellSize: cellSize)
        }
    }

    private func drawTerminalRowOutsideScrollRegion(
        _ row: [TerminalCellSnapshot],
        rowIndex: Int,
        animation: RowScrollAnimation,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        guard animation.isColumnBounded else {
            return
        }

        let startCol = max(animation.startCol ?? 0, 0)
        let endCol = min(animation.endCol ?? row.count - 1, row.count - 1)
        if startCol > 0 {
            drawTerminalRow(
                row,
                rowIndex: rowIndex,
                textRect: textRect,
                cellSize: cellSize,
                fromCol: 0,
                throughCol: startCol - 1
            )
        }
        if endCol + 1 < row.count {
            drawTerminalRow(
                row,
                rowIndex: rowIndex,
                textRect: textRect,
                cellSize: cellSize,
                fromCol: endCol + 1,
                throughCol: row.count - 1
            )
        }
    }

    private func drawTerminalRow(
        _ row: [TerminalCellSnapshot],
        rowIndex: Int,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        drawTerminalRow(
            row,
            rowIndex: rowIndex,
            textRect: textRect,
            cellSize: cellSize,
            fromCol: 0,
            throughCol: row.count - 1
        )
    }

    private func drawTerminalRow(
        _ row: [TerminalCellSnapshot],
        rowIndex: Int,
        textRect: NSRect,
        cellSize: NSSize,
        fromCol: Int,
        throughCol: Int
    ) {
        let y = textRect.minY + CGFloat(rowIndex) * cellSize.height
        guard y < textRect.maxY, fromCol <= throughCol else {
            return
        }

        let startCol = max(fromCol, 0)
        let endCol = min(throughCol, row.count - 1)
        guard startCol <= endCol else {
            return
        }

        drawTerminalBackgroundRuns(
            row,
            rowIndex: rowIndex,
            textRect: textRect,
            cellSize: cellSize,
            fromCol: startCol,
            throughCol: endCol
        )
        drawTerminalTextRuns(
            row,
            rowIndex: rowIndex,
            textRect: textRect,
            cellSize: cellSize,
            fromCol: startCol,
            throughCol: endCol
        )
    }

    private func drawTerminalBackgroundRuns(
        _ row: [TerminalCellSnapshot],
        rowIndex: Int,
        textRect: NSRect,
        cellSize: NSSize,
        fromCol: Int,
        throughCol: Int
    ) {
        var colIndex = fromCol
        while colIndex <= throughCol {
            guard let bg = row[colIndex].bg else {
                colIndex += 1
                continue
            }

            var endCol = colIndex
            while endCol + 1 <= throughCol,
                  row[endCol + 1].bg == bg,
                  row[endCol + 1].blend == row[colIndex].blend {
                endCol += 1
            }

            bg.nsColor.withAlphaComponent(backgroundAlpha(forBlend: row[colIndex].blend)).setFill()
            let rect = NSRect(
                x: textRect.minX + CGFloat(colIndex) * cellSize.width,
                y: textRect.minY + CGFloat(rowIndex) * cellSize.height,
                width: CGFloat(endCol - colIndex + 1) * cellSize.width,
                height: cellSize.height
            )
            rect.fill()
            colIndex = endCol + 1
        }
    }

    private func drawTerminalTextRuns(
        _ row: [TerminalCellSnapshot],
        rowIndex: Int,
        textRect: NSRect,
        cellSize: NSSize,
        fromCol: Int,
        throughCol: Int
    ) {
        var colIndex = fromCol
        while colIndex <= throughCol {
            let cell = row[colIndex]
            guard !cell.text.isEmpty else {
                colIndex += 1
                continue
            }

            var text = cell.text
            var endCol = colIndex
            while endCol + 1 <= throughCol,
                  row[endCol + 1].fg == cell.fg,
                  row[endCol + 1].style == cell.style,
                  !row[endCol + 1].text.isEmpty {
                endCol += 1
                text += row[endCol].text
            }

            drawTerminalTextRun(
                text,
                fg: cell.fg,
                style: cell.style,
                rowIndex: rowIndex,
                colIndex: colIndex,
                textRect: textRect,
                cellSize: cellSize
            )
            colIndex = endCol + 1
        }
    }

    private func drawTerminalTextRun(
        _ text: String,
        fg: TerminalColorSnapshot,
        style: TerminalCellStyleSnapshot,
        rowIndex: Int,
        colIndex: Int,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        let attributes: [NSAttributedString.Key: Any] = [
            .font: terminalFont(for: style),
            .foregroundColor: fg.nsColor,
            .underlineStyle: style.underline ? NSUnderlineStyle.single.rawValue : 0,
            .strikethroughStyle: style.strikethrough ? NSUnderlineStyle.single.rawValue : 0,
        ]
        let textY = textRect.minY
            + CGFloat(rowIndex) * cellSize.height
            + max(0, (cellSize.height - terminalFont.pointSize) * 0.5 - 1)
        (text as NSString).draw(
            at: NSPoint(x: textRect.minX + CGFloat(colIndex) * cellSize.width, y: textY),
            withAttributes: attributes
        )
    }

    private func terminalFont(for style: TerminalCellStyleSnapshot) -> NSFont {
        var traits: NSFontTraitMask = []
        if style.bold {
            traits.insert(.boldFontMask)
        }
        if style.italic {
            traits.insert(.italicFontMask)
        }
        return NSFontManager.shared.convert(terminalFont, toHaveTrait: traits)
    }

    private func backgroundAlpha(forBlend blend: UInt8) -> CGFloat {
        CGFloat(100 - min(blend, 100)) / 100.0
    }

    private func drawEmptyTerminal() {
        let attributes: [NSAttributedString.Key: Any] = [
            .font: terminalFont,
            .foregroundColor: terminalTextColor,
        ]
        "".draw(in: terminalTextRect(), withAttributes: attributes)
    }

    private func drawCursor(
        _ cursor: TerminalCursorSnapshot?,
        color: TerminalColorSnapshot,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        guard let cursor else {
            return
        }

        let cursorRow = Int(cursor.y)
        let cursorCol = Int(cursor.x)
        let cursorTextRect = textRect.offsetBy(
            dx: 0,
            dy: rowVisualOffsetRows(cursorRow, col: cursorCol) * cellSize.height
        )
        if let clipRect = cursorClipRect(cursorRow, col: cursorCol, textRect: textRect, cellSize: cellSize) {
            NSGraphicsContext.saveGraphicsState()
            clipRect.clip()
            drawCursorBody(cursor, color: color, textRect: cursorTextRect, cellSize: cellSize)
            NSGraphicsContext.restoreGraphicsState()
        } else {
            drawCursorBody(cursor, color: color, textRect: cursorTextRect, cellSize: cellSize)
        }
    }

    private func drawCursorBody(
        _ cursor: TerminalCursorSnapshot,
        color: TerminalColorSnapshot,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        let rect = cursorRect(cursor, textRect: textRect, cellSize: cellSize)
        drawCursorTrail(to: rect, color: color, textRect: textRect, cellSize: cellSize)
        color.nsColor.withAlphaComponent(0.78).setFill()
        switch cursor.style {
        case "bar":
            NSRect(
                x: rect.minX,
                y: rect.minY,
                width: cursorThickness(cursor.cell_percentage, size: rect.width),
                height: rect.height
            ).fill()
        case "underline":
            let height = cursorThickness(cursor.cell_percentage, size: rect.height)
            NSRect(x: rect.minX, y: rect.maxY - height, width: rect.width, height: height).fill()
        default:
            rect.fill()
        }
    }

    private func cursorThickness(_ percentage: UInt8, size: CGFloat) -> CGFloat {
        let fraction = CGFloat(max(1, min(percentage, 100))) / 100.0
        return min(size, max(1, size * fraction))
    }

    private func drawCursorTrail(
        to rect: NSRect,
        color: TerminalColorSnapshot,
        textRect: NSRect,
        cellSize: NSSize
    ) {
        guard let tail = animatedCursorTail() else {
            return
        }

        let tailRect = cursorRect(atGridPoint: tail, textRect: textRect, cellSize: cellSize)
        let distance = hypot(rect.minX - tailRect.minX, rect.minY - tailRect.minY)
        guard distance >= 0.75 else {
            return
        }

        color.nsColor.withAlphaComponent(cursorTrailAlpha).setFill()
        if let trail = cursorTrailRect(tail: tailRect, target: rect, cellSize: cellSize) {
            trail.fill()
            return
        }

        color.nsColor.withAlphaComponent(cursorTrailAlpha).setStroke()
        let path = NSBezierPath()
        path.lineCapStyle = .round
        path.lineWidth = max(cellSize.width, cellSize.height) * 0.72
        path.move(to: NSPoint(x: tailRect.midX, y: tailRect.midY))
        path.line(to: NSPoint(x: rect.midX, y: rect.midY))
        path.stroke()
    }

    private func cursorTrailRect(
        tail: NSRect,
        target: NSRect,
        cellSize: NSSize
    ) -> NSRect? {
        let dx = abs(target.minX - tail.minX)
        let dy = abs(target.minY - tail.minY)
        if dx < 0.75 && dy < 0.75 {
            return nil
        }
        if dy <= cellSize.height * 0.25 {
            return NSRect(
                x: min(tail.minX, target.minX),
                y: target.minY,
                width: dx + cellSize.width,
                height: cellSize.height
            )
        }
        if dx <= cellSize.width * 0.25 {
            return NSRect(
                x: target.minX,
                y: min(tail.minY, target.minY),
                width: cellSize.width,
                height: dy + cellSize.height
            )
        }
        return nil
    }

    private func drawScrollbar(_ scrollbar: ScrollbarSnapshot, in textRect: NSRect) {
        guard scrollbar.total > scrollbar.visible, scrollbar.total > 0 else {
            return
        }

        let ratio = CGFloat(scrollbar.visible) / CGFloat(scrollbar.total)
        let thumbHeight = max(24, textRect.height * ratio)
        let maxTop = max(1, CGFloat(scrollbar.total - scrollbar.visible))
        let progress = CGFloat(scrollbar.top) / maxTop
        let thumbY = textRect.minY + (textRect.height - thumbHeight) * progress
        let rect = NSRect(
            x: textRect.maxX - 4,
            y: thumbY,
            width: 3,
            height: thumbHeight
        )
        NSColor.white.withAlphaComponent(0.22).setFill()
        NSBezierPath(roundedRect: rect, xRadius: 1.5, yRadius: 1.5).fill()
    }

    private func animatedCursorTail() -> NSPoint? {
        guard let start = cursorTrailStart,
              let target = cursorTrailTarget,
              let startedAt = cursorTrailStartedAt
        else {
            return nil
        }

        let progress = min(1, (CACurrentMediaTime() - startedAt) / 0.16)
        let eased = 1 - pow(1 - progress, 3)
        return NSPoint(
            x: start.x + (target.x - start.x) * eased,
            y: start.y + (target.y - start.y) * eased
        )
    }

    private func tabIndex(at point: NSPoint) -> Int? {
        for (idx, rect) in tabRects().enumerated() {
            if rect.contains(point) {
                return idx
            }
        }
        return nil
    }

    private func tabRects() -> [NSRect] {
        var rects: [NSRect] = []
        var x: CGFloat = 12
        for title in tabTitles {
            let width = tabWidth(title)
            rects.append(NSRect(x: x, y: 5, width: width, height: 20))
            x += width + 6
        }
        return rects
    }

    private func tabWidth(_ title: String) -> CGFloat {
        let size = (title as NSString).size(withAttributes: [.font: tabFont])
        return min(max(size.width + 24, 92), 170)
    }

    private func handleCommandKey(_ event: NSEvent) -> Bool {
        guard let key = event.charactersIgnoringModifiers ?? event.characters else {
            return false
        }

        switch key {
        case "=", "+":
            onZoomIn?()
            return true
        case "-":
            onZoomOut?()
            return true
        case "0":
            onResetZoom?()
            return true
        default:
            return false
        }
    }

    private func terminalTextRect() -> NSRect {
        NSRect(
            x: terminalHorizontalInset,
            y: terminalTextTop,
            width: max(1, bounds.width - terminalHorizontalInset * 2),
            height: max(1, bounds.height - terminalTextTop - terminalTextBottomInset)
        )
    }

    private func terminalCellSize() -> NSSize {
        let measured = ("M" as NSString).size(withAttributes: [.font: terminalFont])
        let lineHeight = terminalFont.ascender - terminalFont.descender + terminalFont.leading
        return NSSize(width: max(1, measured.width), height: max(1, lineHeight))
    }

    private func rowVisualOffsetRows(_ row: Int, col: Int) -> CGFloat {
        guard let animation = rowScrollAnimation,
              animation.contains(row: row, col: col)
        else {
            return scrollVisualOffsetRows
        }
        return scrollVisualOffsetRows + animation.visualOffsetRows
    }

    private func cursorClipRect(
        _ row: Int,
        col: Int,
        textRect: NSRect,
        cellSize: NSSize
    ) -> NSRect? {
        guard let animation = rowScrollAnimation,
              animation.contains(row: row, col: col)
        else {
            return nil
        }
        return scrollRegionRect(animation, textRect: textRect, cellSize: cellSize)
    }

    private func scrollRegionRect(
        _ animation: RowScrollAnimation,
        textRect: NSRect,
        cellSize: NSSize
    ) -> NSRect {
        let startCol = max(animation.startCol ?? 0, 0)
        let endCol = animation.endCol.map { max($0, startCol) }
        let x = textRect.minX + CGFloat(startCol) * cellSize.width
        let width = endCol.map { CGFloat($0 - startCol + 1) * cellSize.width } ?? textRect.width
        return NSRect(
            x: x,
            y: textRect.minY + CGFloat(animation.startRow) * cellSize.height,
            width: min(width, textRect.maxX - x),
            height: CGFloat(animation.endRow - animation.startRow + 1) * cellSize.height
        )
    }

    private func scrollRows(for event: NSEvent) -> CGFloat {
        if event.hasPreciseScrollingDeltas {
            return -event.scrollingDeltaY / terminalCellSize().height
        }
        return -event.scrollingDeltaY * 3.0
    }

    private func advanceScrollAnimation() {
        let now = CACurrentMediaTime()
        let dt = CGFloat(min(max(now - lastAnimationFrameAt, 0), 1.0 / 15.0))
        lastAnimationFrameAt = now
        var changed = false
        changed = advanceSpring(offset: &scrollVisualOffsetRows, velocity: &scrollVelocityRows, dt: dt)

        if var animation = rowScrollAnimation {
            let rowChanged = advanceSpring(
                offset: &animation.visualOffsetRows,
                velocity: &animation.velocityRows,
                dt: dt
            )
            rowScrollAnimation = rowChanged ? animation : nil
            changed = changed || rowChanged
        }

        if changed {
            needsDisplay = true
        }
    }

    private func advanceSpring(
        offset: inout CGFloat,
        velocity: inout CGFloat,
        dt: CGFloat
    ) -> Bool {
        guard offset != 0 || velocity != 0 else {
            return false
        }

        let acceleration = -180.0 * offset - 27.0 * velocity
        velocity += acceleration * dt
        offset += velocity * dt
        if abs(offset) < 0.002 && abs(velocity) < 0.002 {
            offset = 0
            velocity = 0
            return false
        }
        return true
    }

    private func animateOutputShiftIfNeeded(nextFrame: TerminalFrameSnapshot?) {
        if suppressNextOutputShiftAnimation {
            suppressNextOutputShiftAnimation = false
            return
        }

        guard let previousFrame = frameSnapshot,
              let nextFrame
        else {
            return
        }

        if nextFrame.semantic_scroll == true {
            animateSemanticScroll(nextFrame: nextFrame, previousFrame: previousFrame)
            return
        }

        animateTerminalOutputShift(previousFrame: previousFrame, nextFrame: nextFrame)
    }

    private func animateSemanticScroll(
        nextFrame: TerminalFrameSnapshot,
        previousFrame: TerminalFrameSnapshot
    ) {
        rowScrollAnimation = nil
        lastScrollRegionShift = nil
        if let scrollHint = nextFrame.scroll_hint {
            animateScrollRegion(scrollHint.outputShift, previousRows: previousFrame.rows)
        }
    }

    private func animateTerminalOutputShift(
        previousFrame: TerminalFrameSnapshot,
        nextFrame: TerminalFrameSnapshot
    ) {
        if let shift = terminalOutputShift(from: previousFrame, to: nextFrame) {
            animateScrollRegion(shift, previousRows: previousFrame.rows)
        }
    }

    private func terminalOutputShift(
        from previousFrame: TerminalFrameSnapshot,
        to nextFrame: TerminalFrameSnapshot
    ) -> OutputScrollShift? {
        guard scrollbarIsAtBottom(nextFrame.scrollbar) else {
            return nil
        }

        if let frameShift = detectTerminalFullFrameShift(
            previous: previousFrame.rows,
            current: nextFrame.rows
        ) {
            return frameShift
        }

        if let rowShift = detectTerminalScrollRegionShift(
            previous: previousFrame.rows,
            current: nextFrame.rows
        ) {
            return rowShift
        }

        if let jumpShift = detectTerminalJumpReplacementShift(from: previousFrame, to: nextFrame) {
            return jumpShift
        }

        let totalDelta = nextFrame.scrollbar.total.saturatingSub(previousFrame.scrollbar.total)
        let cappedRows = capOutputScrollRows(totalDelta, visibleRows: nextFrame.scrollbar.visible)
        guard cappedRows != 0, !nextFrame.rows.isEmpty else {
            return nil
        }
        return OutputScrollShift(startRow: 0, endRow: nextFrame.rows.count - 1, rows: cappedRows)
    }

    private func detectTerminalFullFrameShift(
        previous: [[TerminalCellSnapshot]],
        current: [[TerminalCellSnapshot]]
    ) -> OutputScrollShift? {
        guard previous.count == current.count,
              previous.count >= 2,
              previous != current
        else {
            return nil
        }

        let maxShift = min(previous.count - 1, maxScrollRegionDetectionRows)
        var best: ScrollShiftCandidate?
        for shift in 1...maxShift {
            updateBestFullFrameShift(
                previous: previous,
                current: current,
                shiftedRows: shift,
                direction: 1,
                best: &best
            )
            updateBestFullFrameShift(
                previous: previous,
                current: current,
                shiftedRows: shift,
                direction: -1,
                best: &best
            )
        }

        let requiredRows = max(minFullFrameScrollMatchRows, previous.count / 4)
        guard let best, best.contentRows >= requiredRows else {
            return nil
        }
        return clampShiftToScrollableRows(best.shift, rows: current)
    }

    private func updateBestFullFrameShift(
        previous: [[TerminalCellSnapshot]],
        current: [[TerminalCellSnapshot]],
        shiftedRows: Int,
        direction: Int,
        best: inout ScrollShiftCandidate?
    ) {
        var matchedRows = 0
        var contentRows = 0
        var firstMatchedRow: Int?
        var lastMatchedRow: Int?
        let comparableRows = previous.count - shiftedRows
        for row in 0..<comparableRows {
            let previousRow = direction > 0 ? previous[row + shiftedRows] : previous[row]
            let currentRow = direction > 0 ? current[row] : current[row + shiftedRows]
            guard rowsMatchForScroll(previousRow, currentRow) else {
                continue
            }

            firstMatchedRow = min(firstMatchedRow ?? row, row)
            lastMatchedRow = max(lastMatchedRow ?? row, row)
            matchedRows += 1
            if rowHasScrollContent(previousRow) || rowHasScrollContent(currentRow) {
                contentRows += 1
            }
        }

        let score = contentRows * 100 + matchedRows
        guard let firstMatchedRow,
              let lastMatchedRow,
              contentRows >= minFullFrameScrollMatchRows,
              score > (best?.score ?? 0)
        else {
            return
        }

        best = ScrollShiftCandidate(
            shift: OutputScrollShift(
                startRow: firstMatchedRow,
                endRow: lastMatchedRow + shiftedRows,
                rows: direction * shiftedRows
            ),
            score: score,
            contentRows: contentRows
        )
    }

    private func detectTerminalJumpReplacementShift(
        from previousFrame: TerminalFrameSnapshot,
        to nextFrame: TerminalFrameSnapshot
    ) -> OutputScrollShift? {
        guard previousFrame.rows.count == nextFrame.rows.count,
              previousFrame.rows != nextFrame.rows,
              let previousCursor = previousFrame.cursor,
              let nextCursor = nextFrame.cursor
        else {
            return nil
        }

        let previousContentRows = contentRowCount(previousFrame.rows)
        let nextContentRows = contentRowCount(nextFrame.rows)
        let cursorDelta = Int(nextCursor.y) - Int(previousCursor.y)
        guard previousContentRows >= minJumpAnimationContentRows,
              nextContentRows >= minJumpAnimationContentRows,
              abs(cursorDelta) >= minJumpAnimationContentRows / 2
        else {
            return nil
        }

        let direction = cursorDelta > 0 ? 1 : -1
        let rows = direction * jumpAnimationRows(visibleRows: nextFrame.rows.count)
        return fullFrameShift(
            rows: rows,
            startRow: 0,
            endRow: scrollableEndRow(in: nextFrame.rows)
        )
    }

    private func detectTerminalScrollRegionShift(
        previous: [[TerminalCellSnapshot]],
        current: [[TerminalCellSnapshot]]
    ) -> OutputScrollShift? {
        guard previous.count == current.count,
              previous.count >= 2,
              previous != current
        else {
            return nil
        }

        let maxShift = min(previous.count - 1, maxScrollRegionDetectionRows)
        var best: OutputScrollShift?
        var bestScore = 0
        for shift in 1...maxShift {
            updateBestScrollRegion(
                previous: previous,
                current: current,
                shiftedRows: shift,
                direction: 1,
                best: &best,
                bestScore: &bestScore
            )
            updateBestScrollRegion(
                previous: previous,
                current: current,
                shiftedRows: shift,
                direction: -1,
                best: &best,
                bestScore: &bestScore
            )
        }
        guard let best else {
            return nil
        }
        return clampShiftToScrollableRows(best, rows: current)
    }

    private func updateBestScrollRegion(
        previous: [[TerminalCellSnapshot]],
        current: [[TerminalCellSnapshot]],
        shiftedRows: Int,
        direction: Int,
        best: inout OutputScrollShift?,
        bestScore: inout Int
    ) {
        var runStart: Int?
        var runContentRows = 0
        let comparableRows = previous.count - shiftedRows
        for row in 0..<comparableRows {
            let previousRow = direction > 0 ? previous[row + shiftedRows] : previous[row]
            let currentRow = direction > 0 ? current[row] : current[row + shiftedRows]
            let matches = rowsMatchForScroll(previousRow, currentRow)

            if matches {
                runStart = runStart ?? row
                if rowHasScrollContent(previousRow) || rowHasScrollContent(currentRow) {
                    runContentRows += 1
                }
                continue
            }
            finishScrollRegionRun(
                runStart: &runStart,
                runContentRows: &runContentRows,
                runEnd: row - 1,
                shiftedRows: shiftedRows,
                direction: direction,
                best: &best,
                bestScore: &bestScore
            )
        }
        finishScrollRegionRun(
            runStart: &runStart,
            runContentRows: &runContentRows,
            runEnd: comparableRows - 1,
            shiftedRows: shiftedRows,
            direction: direction,
            best: &best,
            bestScore: &bestScore
        )
    }

    private func finishScrollRegionRun(
        runStart: inout Int?,
        runContentRows: inout Int,
        runEnd: Int,
        shiftedRows: Int,
        direction: Int,
        best: inout OutputScrollShift?,
        bestScore: inout Int
    ) {
        guard let start = runStart else {
            return
        }
        defer {
            runStart = nil
            runContentRows = 0
        }

        let matchedRows = runEnd - start + 1
        let score = runContentRows * 100 + matchedRows
        guard matchedRows >= 2,
              runContentRows >= minScrollRegionContentRows,
              score > bestScore
        else {
            return
        }

        bestScore = score
        if direction > 0 {
            best = OutputScrollShift(
                startRow: start,
                endRow: runEnd + shiftedRows,
                rows: shiftedRows
            )
        } else {
            best = OutputScrollShift(
                startRow: start,
                endRow: runEnd + shiftedRows,
                rows: -shiftedRows
            )
        }
    }

    private func rowsMatchForScroll(
        _ previous: [TerminalCellSnapshot],
        _ current: [TerminalCellSnapshot]
    ) -> Bool {
        if previous == current {
            return true
        }

        let previousText = rowPlainText(previous)
        let currentText = rowPlainText(current)
        guard previousText.count > 8,
              currentText.count > 8
        else {
            return false
        }

        let previousBody = scrollRowBody(previousText)
        let currentBody = scrollRowBody(currentText)
        return previousBody == currentBody && !previousBody.trimmingCharacters(in: .whitespaces).isEmpty
    }

    private func rowHasScrollContent(_ row: [TerminalCellSnapshot]) -> Bool {
        let text = rowPlainText(row)
        let body = scrollRowBody(text).trimmingCharacters(in: .whitespaces)
        return !body.isEmpty || (text.count <= 8 && !text.trimmingCharacters(in: .whitespaces).isEmpty)
    }

    private func contentRowCount(_ rows: [[TerminalCellSnapshot]]) -> Int {
        rows.reduce(0) { count, row in
            rowHasScrollContent(row) ? count + 1 : count
        }
    }

    private func clampShiftToScrollableRows(
        _ shift: OutputScrollShift,
        rows: [[TerminalCellSnapshot]]
    ) -> OutputScrollShift? {
        let endRow = min(shift.endRow, scrollableEndRow(in: rows))
        return fullFrameShift(rows: shift.rows, startRow: shift.startRow, endRow: endRow)
    }

    private func scrollableEndRow(in rows: [[TerminalCellSnapshot]]) -> Int {
        guard let fixedTailStart = fixedTailStart(in: rows) else {
            return rows.count - 1
        }
        return max(0, fixedTailStart - 1)
    }

    private func fixedTailStart(in rows: [[TerminalCellSnapshot]]) -> Int? {
        guard rows.count >= 4 else {
            return nil
        }

        let firstCandidate = max(0, rows.count - 8)
        for rowIndex in firstCandidate..<rows.count where rowLooksFixed(row: rows[rowIndex]) {
            return rowIndex
        }
        return nil
    }

    private func rowLooksFixed(row: [TerminalCellSnapshot]) -> Bool {
        let coloredCells = row.reduce(0) { count, cell in
            cell.bg == nil ? count : count + 1
        }
        return coloredCells >= max(8, row.count / 4)
    }

    private func fullFrameShift(
        rows: Int,
        startRow: Int,
        endRow: Int
    ) -> OutputScrollShift? {
        guard rows != 0, startRow <= endRow else {
            return nil
        }

        return OutputScrollShift(
            startRow: startRow,
            endRow: endRow,
            rows: cappedAnimationRows(rows, regionHeight: endRow - startRow + 1)
        )
    }

    private func cappedAnimationRows(_ rows: Int, regionHeight: Int) -> Int {
        let sign = rows < 0 ? -1 : 1
        let maxRows = max(1, min(regionHeight - 1, maxFullFrameScrollAnimationRows))
        return sign * min(abs(rows), maxRows)
    }

    private func jumpAnimationRows(visibleRows: Int) -> Int {
        max(1, min(maxFullFrameScrollAnimationRows, max(1, visibleRows / 2)))
    }

    private func scrollRowBody(_ text: String) -> String {
        guard text.count > 8 else {
            return text
        }
        return String(text.dropFirst(8))
    }

    private func rowPlainText(_ row: [TerminalCellSnapshot]) -> String {
        row.map(\.text).joined()
    }

    private func scrollbarIsAtBottom(_ scrollbar: ScrollbarSnapshot) -> Bool {
        scrollbar.total <= scrollbar.visible ||
            scrollbar.top.saturatingAdd(scrollbar.visible) >= scrollbar.total
    }

    private func capOutputScrollRows(_ rows: UInt64, visibleRows: UInt64) -> Int {
        let cappedRows: UInt64
        if visibleRows > 0 && rows > visibleRows {
            cappedRows = UInt64(outputScrollAnimationFarLines)
        } else {
            cappedRows = min(rows, UInt64(maxOutputScrollAnimationRows))
        }
        return Int(min(cappedRows, UInt64(Int.max)))
    }

    private func updateCursorTrail(to cursor: TerminalCursorSnapshot?) {
        guard let cursor else {
            cursorTrailStart = nil
            cursorTrailTarget = nil
            cursorTrailStartedAt = nil
            return
        }
        let next = NSPoint(x: CGFloat(cursor.x), y: CGFloat(cursor.y))
        if cursorTrailTarget == nil {
            cursorTrailTarget = next
            return
        }
        guard cursorTrailTarget != next else {
            return
        }

        cursorTrailStart = animatedCursorTail() ?? cursorTrailTarget
        cursorTrailTarget = next
        cursorTrailStartedAt = CACurrentMediaTime()
    }

    private func resetCursorTrail() {
        cursorTrailStart = nil
        cursorTrailTarget = nil
        cursorTrailStartedAt = nil
    }

    private func cursorRect(
        _ cursor: TerminalCursorSnapshot,
        textRect: NSRect,
        cellSize: NSSize
    ) -> NSRect {
        cursorRect(
            atGridPoint: NSPoint(x: CGFloat(cursor.x), y: CGFloat(cursor.y)),
            textRect: textRect,
            cellSize: cellSize
        )
    }

    private func cursorRect(
        atGridPoint point: NSPoint,
        textRect: NSRect,
        cellSize: NSSize
    ) -> NSRect {
        NSRect(
            x: textRect.minX + point.x * cellSize.width,
            y: textRect.minY + point.y * cellSize.height,
            width: cellSize.width,
            height: cellSize.height
        )
    }
}

extension TerminalColorSnapshot {
    var nsColor: NSColor {
        NSColor(
            deviceRed: CGFloat(r) / 255,
            green: CGFloat(g) / 255,
            blue: CGFloat(b) / 255,
            alpha: 1
        )
    }
}

extension UInt64 {
    func saturatingAdd(_ value: UInt64) -> UInt64 {
        let (result, overflow) = addingReportingOverflow(value)
        return overflow ? UInt64.max : result
    }

    func saturatingSub(_ value: UInt64) -> UInt64 {
        self >= value ? self - value : 0
    }
}

func terminalInputData(for event: NSEvent) -> Data? {
    switch event.keyCode {
    case 36, 76:
        return Data([13])
    case 48:
        return Data([9])
    case 51:
        return Data([127])
    case 53:
        return Data([27])
    case 123:
        return Data("\u{1B}[D".utf8)
    case 124:
        return Data("\u{1B}[C".utf8)
    case 125:
        return Data("\u{1B}[B".utf8)
    case 126:
        return Data("\u{1B}[A".utf8)
    case 115:
        return Data("\u{1B}[H".utf8)
    case 119:
        return Data("\u{1B}[F".utf8)
    case 116:
        return Data("\u{1B}[5~".utf8)
    case 121:
        return Data("\u{1B}[6~".utf8)
    case 117:
        return Data("\u{1B}[3~".utf8)
    default:
        return textInputData(for: event)
    }
}

func textInputData(for event: NSEvent) -> Data? {
    if event.modifierFlags.contains(.control),
       let byte = controlByte(for: event) {
        return Data([byte])
    }
    guard let characters = event.characters, !characters.isEmpty else {
        return nil
    }
    return Data(characters.utf8)
}

func controlByte(for event: NSEvent) -> UInt8? {
    guard let scalar = event.charactersIgnoringModifiers?.unicodeScalars.first else {
        return nil
    }
    let value = scalar.value
    if (65...90).contains(value) {
        return UInt8(value - 64)
    }
    if (97...122).contains(value) {
        return UInt8(value - 96)
    }
    return nil
}

func clampedUInt16(_ value: Int) -> UInt16 {
    UInt16(min(max(value, 1), Int(UInt16.max)))
}

func themeAccentColor(_ theme: String?) -> NSColor {
    guard let theme else {
        return themeAccentColors["Graphite"] ?? NSColor.controlAccentColor
    }
    return themeAccentColors[theme] ?? NSColor.controlAccentColor
}

func tabColor(theme: String?, active: Bool) -> NSColor {
    let accent = themeAccentColor(theme)
    if active {
        return accent
    }
    return accent.withAlphaComponent(0.45)
}

func colorSwatchImage(_ color: NSColor) -> NSImage {
    let image = NSImage(size: NSSize(width: 14, height: 14))
    image.lockFocus()
    color.setFill()
    NSBezierPath(roundedRect: NSRect(x: 1, y: 1, width: 12, height: 12), xRadius: 3, yRadius: 3).fill()
    image.unlockFocus()
    return image
}

func metalObjectPointer(_ object: AnyObject) -> UnsafeMutableRawPointer {
    Unmanaged.passUnretained(object).toOpaque()
}

func sanitizedTerminalOutput(_ data: Data) -> String {
    var text = String(decoding: data, as: UTF8.self)
    text = stripAnsiSequences(text)
    text = text.replacingOccurrences(of: "\r\n", with: "\n")
    text = text.replacingOccurrences(of: "\r", with: "\n")
    return text
}

func stripAnsiSequences(_ text: String) -> String {
    var stripped = text.replacingOccurrences(
        of: "\u{1B}\\[[0-?]*[ -/]*[@-~]",
        with: "",
        options: .regularExpression
    )
    stripped = stripped.replacingOccurrences(
        of: "\u{1B}\\][^\u{7}]*(\u{7}|\u{1B}\\\\)",
        with: "",
        options: .regularExpression
    )
    return stripped
}

protocol TerminalContextMenuProvider: AnyObject {
    func terminalContextMenu(tabIndex: Int?) -> NSMenu
}

final class TerminalMetalView: MTKView, MTKViewDelegate {
    weak var contextMenuProvider: TerminalContextMenuProvider?
    var renderProvider: ((MTLTexture, UnsafeMutableRawPointer?) -> Bool)?

    private let commandQueue: MTLCommandQueue
    private var skiaRenderer: UnsafeMutableRawPointer?
    private var skiaFrameCount = 0

    required init(coder: NSCoder) {
        fatalError("init(coder:) is not supported")
    }

    init(frame frameRect: NSRect, contract: RendererContract?) {
        guard let device = MTLCreateSystemDefaultDevice(),
              let commandQueue = device.makeCommandQueue()
        else {
            fatalError("Metal is not available")
        }

        self.commandQueue = commandQueue
        super.init(frame: frameRect, device: device)
        self.skiaRenderer = nvterm_skia_metal_create(
            metalObjectPointer(device),
            metalObjectPointer(commandQueue)
        )
        colorPixelFormat = .bgra8Unorm
        clearColor = MTLClearColor(red: 0.078, green: 0.086, blue: 0.102, alpha: 1.0)
        enableSetNeedsDisplay = true
        isPaused = true
        preferredFramesPerSecond = contract?.surface.preferred_frames_per_second ?? 120
        delegate = self
        needsDisplay = true
    }

    deinit {
        nvterm_skia_metal_destroy(skiaRenderer)
    }

    override var acceptsFirstResponder: Bool {
        true
    }

    override func rightMouseDown(with event: NSEvent) {
        guard let menu = contextMenuProvider?.terminalContextMenu(tabIndex: nil) else {
            return
        }
        NSMenu.popUpContextMenu(menu, with: event, for: self)
    }

    func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {}

    func draw(in view: MTKView) {
        guard let drawable = currentDrawable,
              let commandBuffer = commandQueue.makeCommandBuffer()
        else {
            return
        }

        if renderProvider?(drawable.texture, skiaRenderer) == true {
            skiaFrameCount += 1
            requestNextSkiaFrameIfNeeded(commandBuffer)
            commandBuffer.present(drawable)
            commandBuffer.commit()
            return
        }

        guard let descriptor = currentRenderPassDescriptor else {
            return
        }
        let encoder = commandBuffer.makeRenderCommandEncoder(descriptor: descriptor)
        encoder?.endEncoding()
        commandBuffer.present(drawable)
        commandBuffer.commit()
    }

    func hasSkiaFrames() -> Bool {
        skiaFrameCount > 0
    }

    func skiaFrames() -> Int {
        skiaFrameCount
    }

    func resetSkiaFrameCount() {
        skiaFrameCount = 0
    }

    private func requestNextSkiaFrameIfNeeded(_ commandBuffer: MTLCommandBuffer) {
        let delayMs = nvterm_skia_metal_next_frame_delay_ms(skiaRenderer)
        guard delayMs != UInt64.max else {
            return
        }
        commandBuffer.addCompletedHandler { [weak self] _ in
            let deadline = DispatchTime.now() + .milliseconds(Int(min(delayMs, UInt64(Int.max))))
            DispatchQueue.main.asyncAfter(deadline: deadline) {
                self?.needsDisplay = true
            }
        }
    }
}

final class TerminalShellViewController: NSViewController, NSTabViewDelegate, TerminalContextMenuProvider {
    private let core: RustCore
    private let tabControl = NSSegmentedControl(frame: .zero)
    private let metalView: TerminalMetalView
    private let terminalTextView = TerminalTextView(frame: .zero)
    private let defaultPaneMode = NativePaneMode.current()
    private var terminalPanes: [Int: NativePane] = [:]
    private var commandBuffers: [Int: TerminalInputCommandBuffer] = [:]
    private var scrollRemainders: [Int: CGFloat] = [:]
    private var activePaneId: Int?
    private var lastSnapshot: TerminalCoreSnapshot?
    private var lastNvimModelScrollShift: OutputScrollShift?
    private var frameTimer: Timer?
    private var syncingTabs = false

    init(core: RustCore) {
        self.core = core
        self.metalView = TerminalMetalView(frame: .zero, contract: core.rendererContract())
        super.init(nibName: nil, bundle: nil)
        self.metalView.contextMenuProvider = self
        self.terminalTextView.onTabSelected = { [weak self] index in
            self?.selectTab(index)
        }
        self.terminalTextView.onContextMenuRequested = { [weak self] tabIndex, event, view in
            guard let menu = self?.terminalContextMenu(tabIndex: tabIndex) else {
                return
            }
            NSMenu.popUpContextMenu(menu, with: event, for: view)
        }
        self.terminalTextView.onGeometryChanged = { [weak self] in
            self?.resizeTerminalPanesToGrid()
        }
        self.terminalTextView.onInput = { [weak self] data in
            self?.writeToActivePane(data)
        }
        self.terminalTextView.onScroll = { [weak self] rows in
            self?.scrollActivePane(deltaRows: rows)
        }
        self.terminalTextView.onZoomIn = { [weak self] in
            self?.zoomIn(nil)
        }
        self.terminalTextView.onZoomOut = { [weak self] in
            self?.zoomOut(nil)
        }
        self.terminalTextView.onResetZoom = { [weak self] in
            self?.resetZoom(nil)
        }
        self.metalView.renderProvider = { [weak self] texture, renderer in
            self?.renderActiveMetalFrame(texture: texture, renderer: renderer) ?? false
        }
    }

    required init?(coder: NSCoder) {
        nil
    }

    deinit {
        frameTimer?.invalidate()
    }

    override func loadView() {
        view = NSView()
        configureTabControl()
        metalView.translatesAutoresizingMaskIntoConstraints = false
        configureTerminalTextView()
        terminalTextView.setExternalRendererEnabled(true)
        view.addSubview(metalView)
        metalView.addSubview(terminalTextView)

        NSLayoutConstraint.activate([
            metalView.topAnchor.constraint(equalTo: view.topAnchor),
            metalView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            metalView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            metalView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
            terminalTextView.topAnchor.constraint(equalTo: metalView.topAnchor),
            terminalTextView.leadingAnchor.constraint(equalTo: metalView.leadingAnchor),
            terminalTextView.trailingAnchor.constraint(equalTo: metalView.trailingAnchor),
            terminalTextView.bottomAnchor.constraint(equalTo: metalView.bottomAnchor),
        ])
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        syncFromCore()
        startFrameTimer()
    }

    func focusTerminal() {
        view.window?.makeFirstResponder(terminalTextView)
    }

    @objc func tabControlChanged(_ sender: NSSegmentedControl) {
        guard !syncingTabs, sender.selectedSegment >= 0 else {
            return
        }

        selectTab(sender.selectedSegment)
    }

    @objc func newTab(_ sender: Any?) {
        core.newTab()
        syncFromCore()
    }

    @objc func splitVertical(_ sender: Any?) {
        core.splitActive(axis: ffiSplitVertical)
        syncFromCore()
    }

    @objc func splitHorizontal(_ sender: Any?) {
        core.splitActive(axis: ffiSplitHorizontal)
        syncFromCore()
    }

    @objc func renameActiveTab(_ sender: Any?) {
        guard let snapshot = lastSnapshot,
              let tab = snapshot.tabs.first(where: { $0.index == snapshot.active_tab })
        else {
            return
        }

        let input = RenameTextField(frame: NSRect(x: 0, y: 0, width: 320, height: 24))
        input.stringValue = tab.title
        input.isEditable = true
        input.isSelectable = true
        let alert = NSAlert()
        alert.messageText = "Rename Session"
        alert.accessoryView = input
        let renameButton = alert.addButton(withTitle: "Rename")
        alert.addButton(withTitle: "Cancel")
        alert.layout()
        alert.window.initialFirstResponder = input
        alert.window.makeFirstResponder(input)
        input.selectText(nil)
        input.onCommit = { [weak renameButton] in
            renameButton?.performClick(nil)
        }
        DispatchQueue.main.async { [weak alert, weak input] in
            guard let alert, let input else {
                return
            }
            alert.window.makeFirstResponder(input)
            input.selectText(nil)
        }

        guard alert.runModal() == .alertFirstButtonReturn else {
            return
        }
        core.renameTab(snapshot.active_tab, title: input.stringValue)
        syncFromCore()
    }

    @objc func zoomIn(_ sender: Any?) {
        _ = terminalTextView.zoomIn()
        focusTerminal()
    }

    @objc func zoomOut(_ sender: Any?) {
        _ = terminalTextView.zoomOut()
        focusTerminal()
    }

    @objc func resetZoom(_ sender: Any?) {
        _ = terminalTextView.resetZoom()
        focusTerminal()
    }

    func selectTabFromShortcut(_ shortcutNumber: Int) {
        guard let snapshot = lastSnapshot,
              !snapshot.tabs.isEmpty,
              (1...9).contains(shortcutNumber)
        else {
            return
        }

        let index = shortcutNumber == 9 ? snapshot.tabs.count - 1 : shortcutNumber - 1
        guard index >= 0, index < snapshot.tabs.count else {
            return
        }
        selectTab(index)
    }

    @objc func setThemeFromMenu(_ sender: NSMenuItem) {
        guard let snapshot = lastSnapshot,
              let theme = sender.representedObject as? String
        else {
            return
        }
        core.setTheme(theme, tab: snapshot.active_tab)
        syncFromCore()
    }

    func terminalContextMenu(tabIndex: Int?) -> NSMenu {
        if let tabIndex {
            selectTabForContextMenu(tabIndex)
        }

        let menu = NSMenu()
        menu.addItem(menuItem("Rename Session", #selector(renameActiveTab(_:))))
        menu.addItem(NSMenuItem.separator())
        menu.addItem(themeMenuItem())
        return menu
    }

    func applySmokeScenario(resultPath: String?) {
        core.newTab()
        core.renameTab(1, title: "native smoke")
        core.setTheme("Harbor", tab: 1)
        core.splitActive(axis: ffiSplitVertical)
        syncFromCore()
        writeToActivePane(Data("printf 'native pty view ready\\n'\r".utf8))
        guard let resultPath, !resultPath.isEmpty else {
            return
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.writeNativeSmokeResult(resultPath, retries: 12)
        }
    }

    func applyTerminalVimScrollSmokeScenario(resultPath: String) {
        let command = [
            "tmp=/tmp/neovide-tabs-terminal-vim-scroll-smoke.txt",
            "awk 'BEGIN { for (i = 1; i <= 240; i++) " +
                "printf \"%08d terminal vim scroll line %03d token %06d\\n\", i, i, i * 17 }' > $tmp",
            "nvim -Nu NONE -n $tmp",
        ].joined(separator: "; ")
        writeToActivePane(Data("\(command)\r".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.0) { [weak self] in
            guard let self else {
                return
            }
            metalView.resetSkiaFrameCount()
            writeToActivePane(Data([0x04]))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.05) { [weak self] in
            self?.writeTerminalVimScrollSmokeResult(
                resultPath,
                retries: 24,
                maxScrollPosition: 0
            )
        }
    }

    func applyTerminalBottomInputSmokeScenario(resultPath: String) {
        let command = [
            "i=0",
            "while [ $i -lt 80 ]; do printf '\\n'; i=$((i + 1)); done",
        ].joined(separator: "; ")
        writeToActivePane(Data("\(command)\r".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.2) { [weak self] in
            self?.waitForTerminalBottomInputIdleThenType(resultPath, retries: 24)
        }
    }

    func applyTerminalNvimHandoffSmokeScenario(resultPath: String) {
        writeToActivePane(Data("nvim\r".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.runNvimCommandOrWrite(
                "enew | call setline(1, 'HANDOFFNVIM') | call cursor(1, 1)",
                fallback: Data()
            )
            self?.metalView.resetSkiaFrameCount()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.4) { [weak self] in
            self?.writeTerminalNvimHandoffSmokeResult(resultPath, retries: 16)
        }
    }

    func applyTerminalNvimCwdSmokeScenario(resultPath: String) {
        let cwd = ProcessInfo.processInfo.environment["NVTERM_NATIVE_CWD_EXPECTED"]
            ?? "/tmp/neovide-tabs-terminal-nvim-cwd"
        let cwdFile = ProcessInfo.processInfo.environment["NVTERM_NATIVE_CWD_ACTUAL"]
            ?? "/tmp/neovide-tabs-terminal-nvim-cwd.actual"
        try? FileManager.default.createDirectory(
            atPath: cwd,
            withIntermediateDirectories: true
        )
        try? FileManager.default.removeItem(atPath: cwdFile)

        let cwdCommand = [
            "cd \(shellQuote(cwd))",
            "printf '\\u{1b}]7;file://localhost\(cwd)\\u{07}'",
        ].joined(separator: "; ")
        writeToActivePane(Data("\(cwdCommand)\r".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) { [weak self] in
            self?.writeToActivePane(Data("nvim\r".utf8))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.6) { [weak self] in
            self?.runNvimCommandOrWrite(
                "call writefile([getcwd()], '\(vimSingleQuote(cwdFile))')",
                fallback: Data()
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 2.0) { [weak self] in
            self?.writeTerminalNvimCwdSmokeResult(
                resultPath,
                expected: cwd,
                actualFile: cwdFile,
                retries: 16
            )
        }
    }

    func applyTerminalNvimQuitSmokeScenario(resultPath: String) {
        writeToActivePane(Data("nvim\r".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.runNvimCommandOrWrite("qa!", fallback: Data())
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.4) { [weak self] in
            self?.waitForTerminalAfterNvimQuit(resultPath, retries: 20)
        }
    }

    func applyNvimScrollSmokeScenario(resultPath: String) {
        openNvimSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-scroll-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.5) { [weak self] in
            guard let self else {
                return
            }
            clearSmokeScrollShift()
            metalView.resetSkiaFrameCount()
            writeToActivePane(Data([0x04]))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 5.75) { [weak self] in
            self?.writeNvimAnimationSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimJumpSmokeScenario(resultPath: String) {
        openNvimSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-jump-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.waitForNvimSmokeContentThenJump(resultPath, retries: 24)
        }
    }

    func applyNvimSidePaneSmokeScenario(resultPath: String) {
        openNvimSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-side-pane-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
            guard let self else {
                return
            }
            runNvimCommandOrWrite(
                "topleft vertical 24new",
                fallback: Data("\u{1b}:topleft vertical 24new\r".utf8)
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.0) { [weak self] in
            guard let self else {
                return
            }
            runNvimCommandOrWrite(
                "wincmd l",
                fallback: Data(":wincmd l\r".utf8)
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.6) { [weak self] in
            guard let self else {
                return
            }
            clearSmokeScrollShift()
            metalView.resetSkiaFrameCount()
            writeToActivePane(Data([0x04]))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 5.8) { [weak self] in
            self?.writeNvimSidePaneSmokeResult(resultPath, retries: 8)
        }
    }

    func applyNvimCommandLineSmokeScenario(resultPath: String) {
        openNvimSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-commandline-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.5) { [weak self] in
            guard let self else {
                return
            }
            clearSmokeScrollShift()
            writeToActivePane(Data("\u{1b}:qa".utf8))
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.5) { [weak self] in
            self?.writeNvimNoScrollSmokeResult(resultPath, label: "commandline")
        }
    }

    func applyNvimShapedTextSmokeScenario(resultPath: String) {
        openNvimShapedTextSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-shaped-text-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.5) { [weak self] in
            self?.writeNvimShapedTextSmokeResult(resultPath, retries: 16)
        }
    }

    func applyNvimSkiaSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimSkiaSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 2.8) { [weak self] in
            self?.writeNvimSkiaSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimUiSurfacesSmokeScenario(resultPath: String) {
        openNvimSmokeBuffer(
            path: "/tmp/neovide-tabs-nvim-ui-surfaces-smoke.txt",
            terminalCommand: "nvim -Nu NONE -n $tmp"
        )

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
            guard let self else {
                return
            }
            runNvimCommandOrWrite("vnew", fallback: Data("\u{1b}:vnew\r".utf8))
            runNvimCommandOrWrite(
                "call setline(1, 'RIGHTSPLIT')",
                fallback: Data(":call setline(1, 'RIGHTSPLIT')\r".utf8)
            )
            runNvimCommandOrWrite(
                "set laststatus=2 statusline=STATUSLINE",
                fallback: Data(":set laststatus=2 statusline=STATUSLINE\r".utf8)
            )
            runNvimCommandOrWrite(
                nvimFloatCommand(),
                fallback: Data(":echo 'FLOATBOX'\r".utf8)
            )
            runNvimCommandOrWrite(
                "echo 'MSGBOX'",
                fallback: Data(":echo 'MSGBOX'\r".utf8)
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.5) { [weak self] in
            self?.writeNvimUiSurfacesSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimPopupmenuSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimPopupmenuSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.2) { [weak self] in
            self?.writeNvimPopupmenuSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimCursorSwitchSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimCursorSmokeTab(
                marker: "OLDTAB",
                cursorRow: 17,
                cursorCol: 59
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
            guard let self else {
                return
            }
            core.newTab()
            syncFromCore()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 4.5) { [weak self] in
            self?.configureNvimCursorSmokeTab(
                marker: "NEWTAB",
                cursorRow: 5,
                cursorCol: 11
            )
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 5.8) { [weak self] in
            self?.writeNvimCursorSwitchSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimCursorShapeSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimCursorShapeSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.2) { [weak self] in
            self?.writeNvimCursorShapeSmokeResult(resultPath, retries: 12)
        }
    }

    func applyNvimCursorNormalShapeSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimCursorNormalShapeSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
            self?.writeNvimCursorDetailSmokeResult(
                resultPath,
                label: "cursor-normal-shape",
                expected: "5:11:block:0:0:0:0",
                retries: 12
            )
        }
    }

    func applyNvimCursorReplaceShapeSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimCursorReplaceShapeSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 3.0) { [weak self] in
            self?.writeNvimCursorDetailSmokeResult(
                resultPath,
                label: "cursor-replace-shape",
                expected: "5:11:underline:20:0:0:0",
                retries: 12
            )
        }
    }

    func applyNvimCursorBlinkSmokeScenario(resultPath: String) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) { [weak self] in
            self?.configureNvimCursorBlinkSmoke()
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 2.2) { [weak self] in
            self?.writeNvimCursorBlinkSmokeResult(resultPath, retries: 8)
        }
    }

    private func moveNvimSmokeToBottomThenJump(_ resultPath: String, attempts: Int) {
        clearSmokeScrollShift()
        metalView.resetSkiaFrameCount()
        writeToActivePane(Data("\u{1b}G".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.finishNvimSmokeBottomMove(resultPath, attempts: attempts)
        }
    }

    private func waitForNvimSmokeContentThenJump(_ resultPath: String, retries: Int) {
        guard terminalTextView.rendererContentRowCount() >= minJumpAnimationContentRows else {
            if retries > 0 {
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                    self?.waitForNvimSmokeContentThenJump(resultPath, retries: retries - 1)
                }
                return
            }
            writeNvimAnimationSmokeResult(resultPath, retries: 0)
            return
        }

        moveNvimSmokeToBottomThenJump(resultPath, attempts: 4)
    }

    private func finishNvimSmokeBottomMove(_ resultPath: String, attempts: Int) {
        let moved = consumeSmokeScrollShift()
            .map { abs($0.rows) > maxOutputScrollAnimationRows } ?? false
        if !moved, attempts > 0 {
            moveNvimSmokeToBottomThenJump(resultPath, attempts: attempts - 1)
            return
        }

        clearSmokeScrollShift()
        metalView.resetSkiaFrameCount()
        writeToActivePane(Data("\u{1b}gg".utf8))

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) { [weak self] in
            self?.writeNvimAnimationSmokeResult(resultPath, retries: 8)
        }
    }

    private func writeNvimAnimationSmokeResult(_ resultPath: String, retries: Int) {
        let shift = peekSmokeScrollShift()
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let ok = hasModelFrames && skiaFrames >= 2 &&
            (shift.map { abs($0.rows) > maxOutputScrollAnimationRows } ?? false)
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimAnimationSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = nvimAnimationSmokeSummary(
            shift,
            hasModelFrames: hasModelFrames,
            skiaFrames: skiaFrames
        )
        clearSmokeScrollShift()
        let result = ok ? "ok \(summary)\n" : "failed \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeNativeSmokeResult(_ resultPath: String, retries: Int) {
        let skiaFrames = metalView.skiaFrames()
        let ok = skiaFrames > 0
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.15) { [weak self] in
                self?.writeNativeSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let result = ok
            ? "ok native-smoke skia-frames=yes count=\(skiaFrames)\n"
            : "failed native-smoke skia-frames=no count=\(skiaFrames)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
    }

    private func writeTerminalVimScrollSmokeResult(
        _ resultPath: String,
        retries: Int,
        maxScrollPosition: Double
    ) {
        let skiaFrames = metalView.skiaFrames()
        let scrollPosition = abs(activePaneRendererScrollPosition())
        let observedScrollPosition = max(maxScrollPosition, scrollPosition)
        let ok = skiaFrames >= 2 &&
            observedScrollPosition >= minTerminalVimScrollSmokePosition
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
                self?.writeTerminalVimScrollSmokeResult(
                    resultPath,
                    retries: retries - 1,
                    maxScrollPosition: observedScrollPosition
                )
            }
            return
        }

        let frameLabel = skiaFrames >= 2 ? "yes" : "no"
        let formattedPosition = String(format: "%.2f", observedScrollPosition)
        let result = ok
            ? "ok terminal-vim-scroll skia-frames=\(frameLabel) " +
                "count=\(skiaFrames) scroll-position=\(formattedPosition)\n"
            : "failed terminal-vim-scroll skia-frames=\(frameLabel) " +
                "count=\(skiaFrames) scroll-position=\(formattedPosition)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func waitForTerminalAfterNvimQuit(_ resultPath: String, retries: Int) {
        drainTerminalPanes()
        if activePaneMode() != .terminal, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
                self?.waitForTerminalAfterNvimQuit(resultPath, retries: retries - 1)
            }
            return
        }

        metalView.resetSkiaFrameCount()
        writeToActivePane(Data("printf 'AFTERQA\\n'\r".utf8))
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.6) { [weak self] in
            self?.writeTerminalNvimQuitSmokeResult(resultPath, retries: 8)
        }
    }

    private func writeTerminalNvimQuitSmokeResult(_ resultPath: String, retries: Int) {
        let modeOk = activePaneMode() == .terminal
        let skiaFrames = metalView.skiaFrames()
        let ok = modeOk && skiaFrames > 0
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
                self?.writeTerminalNvimQuitSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let result = ok
            ? "ok terminal-nvim-quit mode=terminal skia-frames=\(skiaFrames)\n"
            : "failed terminal-nvim-quit mode=\(activePaneMode()) skia-frames=\(skiaFrames)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeTerminalNvimHandoffSmokeResult(_ resultPath: String, retries: Int) {
        let modeOk = activePaneMode() == .neovim
        let modelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let textOk = terminalTextView.rendererModelContainsTexts(["HANDOFFNVIM"])
        let ok = modeOk && modelFrames && skiaFrames > 0 && textOk
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
                self?.writeTerminalNvimHandoffSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let result = ok
            ? "ok terminal-nvim-handoff mode=neovim model-frames=yes " +
                "skia-frames=\(skiaFrames) text=yes\n"
            : "failed terminal-nvim-handoff mode=\(activePaneMode()) " +
                "model-frames=\(modelFrames ? "yes" : "no") " +
                "skia-frames=\(skiaFrames) text=\(textOk ? "yes" : "no")\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeTerminalNvimCwdSmokeResult(
        _ resultPath: String,
        expected: String,
        actualFile: String,
        retries: Int
    ) {
        let actual = (try? String(contentsOfFile: actualFile, encoding: .utf8))?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let ok = activePaneMode() == .neovim && actual == expected
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
                self?.writeTerminalNvimCwdSmokeResult(
                    resultPath,
                    expected: expected,
                    actualFile: actualFile,
                    retries: retries - 1
                )
            }
            return
        }

        let result = ok
            ? "ok terminal-nvim-cwd cwd=\(expected)\n"
            : "failed terminal-nvim-cwd expected=\(expected) actual=\(actual ?? "nil") " +
                "mode=\(activePaneMode())\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func waitForTerminalBottomInputIdleThenType(_ resultPath: String, retries: Int) {
        let scrollPosition = abs(activePaneRendererScrollPosition())
        if scrollPosition > maxTerminalBottomInputSmokePosition, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) { [weak self] in
                self?.waitForTerminalBottomInputIdleThenType(resultPath, retries: retries - 1)
            }
            return
        }

        metalView.resetSkiaFrameCount()
        writeToActivePane(Data("abc".utf8))
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
            self?.writeTerminalBottomInputSmokeResult(
                resultPath,
                retries: 12,
                maxScrollPosition: 0
            )
        }
    }

    private func writeTerminalBottomInputSmokeResult(
        _ resultPath: String,
        retries: Int,
        maxScrollPosition: Double
    ) {
        let skiaFrames = metalView.skiaFrames()
        let scrollPosition = abs(activePaneRendererScrollPosition())
        let observedScrollPosition = max(maxScrollPosition, scrollPosition)
        let ok = skiaFrames > 0 &&
            observedScrollPosition <= maxTerminalBottomInputSmokePosition
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
                self?.writeTerminalBottomInputSmokeResult(
                    resultPath,
                    retries: retries - 1,
                    maxScrollPosition: observedScrollPosition
                )
            }
            return
        }

        let formattedPosition = String(format: "%.2f", observedScrollPosition)
        let result = ok
            ? "ok terminal-bottom-input no-scroll skia-frames=\(skiaFrames) " +
                "scroll-position=\(formattedPosition)\n"
            : "failed terminal-bottom-input unexpected-scroll skia-frames=\(skiaFrames) " +
                "scroll-position=\(formattedPosition)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeNvimSidePaneSmokeResult(_ resultPath: String, retries: Int) {
        let shift = peekSmokeScrollShift()
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let ok = hasModelFrames && skiaFrames >= 2 && (shift.map { shift in
            abs(shift.rows) > maxOutputScrollAnimationRows && (shift.startCol ?? 0) > 0
        } ?? false)
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimSidePaneSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = nvimAnimationSmokeSummary(
            shift,
            hasModelFrames: hasModelFrames,
            skiaFrames: skiaFrames
        )
        clearSmokeScrollShift()
        let result = ok ? "ok \(summary)\n" : "failed \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeNvimNoScrollSmokeResult(_ resultPath: String, label: String) {
        let shift = consumeSmokeScrollShift()
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let hasSkiaFrames = skiaFrames > 0
        let commandLineCount = terminalTextView.rendererModelTextOccurrences(":qa")
        let commandLineOk = label != "commandline" || commandLineCount == 1
        let summary = nvimAnimationSmokeSummary(
            shift,
            hasModelFrames: hasModelFrames,
            skiaFrames: skiaFrames
        )
        let commandSummary = label == "commandline" ? " cmdline=\(commandLineCount)" : ""
        let result = shift == nil && hasModelFrames && hasSkiaFrames && commandLineOk
            ? "ok \(label) no-scroll model-frames=yes skia-frames=yes\(commandSummary)\n"
            : "failed \(label) \(summary)\(commandSummary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        NSApp.terminate(nil)
    }

    private func writeNvimShapedTextSmokeResult(_ resultPath: String, retries: Int) {
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let expected = shapedTextSmokeLabels()
        let expectedText = expected.map(\.text)
        let missingText = terminalTextView.rendererModelMissingTexts(expectedText)
        let hasText = missingText.isEmpty
        let ok = hasModelFrames && skiaFrames > 0 && hasText
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimShapedTextSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let rendererSummary = "model-frames=\(hasModelFrames ? "yes" : "no") " +
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no") count=\(skiaFrames)"
        let missingSummary = missingText.isEmpty ? "none" : missingText.joined(separator: ",")
        let textSummary = "text=\(hasText ? "yes" : "no") missing=\(missingSummary)"
        let geometrySummary = "geometry=\(terminalTextView.skiaGeometrySummary())"
        let cellSummary = "cells=\(terminalTextView.rendererModelCellSummary(expected))"
        let result = ok
            ? "ok shaped-text \(textSummary) \(rendererSummary) \(geometrySummary) \(cellSummary)\n"
            : "failed shaped-text \(textSummary) \(rendererSummary) \(geometrySummary) \(cellSummary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func shapedTextSmokeLabels() -> [(label: String, text: String)] {
        [
            ("jp1", "日"),
            ("jp2", "本"),
            ("jp3", "語"),
            ("nerd", "\u{e0b0}"),
            ("combining", "e\u{301}"),
            ("ambiguous", "Ω"),
        ]
    }

    private func writeNvimSkiaSmokeResult(_ resultPath: String, retries: Int) {
        let marker = "SKIASMOKE"
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let markerCount = terminalTextView.rendererModelTextOccurrences(marker)
        let markerCell = terminalTextView.rendererModelCellSummary([("marker", "S")])
        let ok = hasModelFrames && skiaFrames > 0 && markerCount == 1 && markerCell != "none"
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimSkiaSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = [
            "model-frames=\(hasModelFrames ? "yes" : "no")",
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no")",
            "count=\(skiaFrames)",
            "text=\(markerCount)",
            "geometry=\(terminalTextView.skiaGeometrySummary())",
            "marker-cell=\(markerCell)",
        ].joined(separator: " ")
        let result = ok ? "ok nvim-skia \(summary)\n" : "failed nvim-skia \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func writeNvimUiSurfacesSmokeResult(_ resultPath: String, retries: Int) {
        let counts = terminalTextView.rendererModelWindowKindCounts()
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let rightSplitCount = terminalTextView.rendererModelTextOccurrences("RIGHTSPLIT")
        let floatCount = terminalTextView.rendererModelTextOccurrences("FLOATBOX")
        let statusCount = terminalTextView.rendererModelTextOccurrences("STATUSLINE")
        let messageCount = terminalTextView.rendererModelTextOccurrences("MSGBOX")
        let blendCellCount = terminalTextView.rendererModelBlendCellCount(minBlend: 35)
        let blendCellSummary = terminalTextView.rendererModelBlendCellSummary(minBlend: 35)
        let hasSplit = (counts["normal"] ?? 0) >= 2 && rightSplitCount == 1
        let hasFloat = (counts["float"] ?? 0) >= 1 && floatCount == 1
        let hasFixedSurfaces = statusCount == 1 && messageCount == 1
        let hasBlend = blendCellCount > 0 && blendCellSummary != "none"
        let ok = hasModelFrames && skiaFrames > 0 &&
            hasSplit && hasFloat && hasFixedSurfaces && hasBlend
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimUiSurfacesSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = [
            "model-frames=\(hasModelFrames ? "yes" : "no")",
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no")",
            "count=\(skiaFrames)",
            "normal=\(counts["normal"] ?? 0)",
            "float=\(counts["float"] ?? 0)",
            "right=\(rightSplitCount)",
            "float-text=\(floatCount)",
            "status=\(statusCount)",
            "message=\(messageCount)",
            "blend-cells=\(blendCellCount)",
            "geometry=\(terminalTextView.skiaGeometrySummary())",
            "blend-cell=\(blendCellSummary)",
        ].joined(separator: " ")
        let result = ok ? "ok ui-surfaces \(summary)\n" : "failed ui-surfaces \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func writeNvimPopupmenuSmokeResult(_ resultPath: String, retries: Int) {
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let popupCount = terminalTextView.rendererModelTextOccurrences("POPUPONE")
        let popupCellSummary = terminalTextView.rendererModelTextStartSummary(
            label: "popup",
            text: "POPUPONE"
        )
        let ok = hasModelFrames && skiaFrames > 0 && popupCount == 1 && popupCellSummary != "none"
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimPopupmenuSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = [
            "model-frames=\(hasModelFrames ? "yes" : "no")",
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no")",
            "count=\(skiaFrames)",
            "popup=\(popupCount)",
            "geometry=\(terminalTextView.skiaGeometrySummary())",
            "popup-cell=\(popupCellSummary)",
        ].joined(separator: " ")
        let result = ok ? "ok popupmenu \(summary)\n" : "failed popupmenu \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func writeNvimCursorSwitchSmokeResult(_ resultPath: String, retries: Int) {
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let newTextCount = terminalTextView.rendererModelTextOccurrences("NEWTAB")
        let oldTextCount = terminalTextView.rendererModelTextOccurrences("OLDTAB")
        let cursor = terminalTextView.rendererModelCursorSummary()
        let ok = hasModelFrames && skiaFrames > 0 &&
            newTextCount == 1 && oldTextCount == 0 && cursor == "5:11"
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.25) { [weak self] in
                self?.writeNvimCursorSwitchSmokeResult(resultPath, retries: retries - 1)
            }
            return
        }

        let summary = [
            "model-frames=\(hasModelFrames ? "yes" : "no")",
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no")",
            "count=\(skiaFrames)",
            "geometry=\(terminalTextView.skiaGeometrySummary())",
            "old=17:59",
            "new=\(cursor)",
            "new-text=\(newTextCount)",
            "old-text=\(oldTextCount)",
        ].joined(separator: " ")
        let result = ok ? "ok cursor-switch \(summary)\n" : "failed cursor-switch \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func writeNvimCursorShapeSmokeResult(_ resultPath: String, retries: Int) {
        writeNvimCursorDetailSmokeResult(
            resultPath,
            label: "cursor-shape",
            expected: "5:11:bar:25:300:200:150",
            retries: retries
        )
    }

    private func writeNvimCursorBlinkSmokeResult(_ resultPath: String, retries: Int) {
        writeNvimCursorDetailSmokeResult(
            resultPath,
            label: "cursor-blink",
            expected: "5:11:bar:25:100:100:2000",
            retries: retries,
            retryDelay: 0.15
        )
    }

    private func writeNvimCursorDetailSmokeResult(
        _ resultPath: String,
        label: String,
        expected: String,
        retries: Int,
        retryDelay: TimeInterval = 0.25
    ) {
        let hasModelFrames = terminalTextView.hasRendererModelFrames()
        let skiaFrames = metalView.skiaFrames()
        let cursor = terminalTextView.rendererModelCursorDetailSummary()
        let ok = hasModelFrames && skiaFrames > 0 && cursor == expected
        if !ok, retries > 0 {
            DispatchQueue.main.asyncAfter(deadline: .now() + retryDelay) { [weak self] in
                self?.writeNvimCursorDetailSmokeResult(
                    resultPath,
                    label: label,
                    expected: expected,
                    retries: retries - 1,
                    retryDelay: retryDelay
                )
            }
            return
        }

        let summary = [
            "model-frames=\(hasModelFrames ? "yes" : "no")",
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no")",
            "count=\(skiaFrames)",
            "geometry=\(terminalTextView.skiaGeometrySummary())",
            "cursor=\(cursor)",
        ].joined(separator: " ")
        let result = ok ? "ok \(label) \(summary)\n" : "failed \(label) \(summary)\n"
        try? result.write(toFile: resultPath, atomically: true, encoding: .utf8)
        if ProcessInfo.processInfo.environment["NVTERM_NATIVE_SMOKE_KEEP_OPEN"] != "1" {
            NSApp.terminate(nil)
        }
    }

    private func nvimFloatCommand() -> String {
        "lua vim.api.nvim_set_hl(0,'NormalFloat',{bg='#506070',blend=35}); " +
            "local b=vim.api.nvim_create_buf(false,true); " +
            "vim.api.nvim_buf_set_lines(b,0,-1,false,{'FLOATBOX'}); " +
            "local w=vim.api.nvim_open_win(b,true,{relative='editor',row=3,col=20,width=16,height=3,style='minimal'}); " +
            "vim.wo[w].winblend=35; vim.wo[w].winhl='Normal:NormalFloat'"
    }

    private func nvimPopupmenuSetupCommand() -> String {
        [
            "set wildmenu wildmode=full",
            "execute \"function! NvtermPopupComplete(A,L,P) abort\\nreturn [''POPUPONE'', ''POPUPTWO'']\\nendfunction\"",
            "command! -nargs=1 -complete=customlist,NvtermPopupComplete NvtermPopupDummy echo <q-args>",
        ].joined(separator: " | ")
    }

    private func configureNvimCursorSmokeTab(marker: String, cursorRow: Int, cursorCol: Int) {
        let rowNumber = cursorRow + 1
        let colNumber = cursorCol + 1
        let command = [
            "setlocal norelativenumber nonumber laststatus=0 noruler noshowmode virtualedit=all",
            "call setline(1, ['\(marker)'] + repeat([repeat(' ', 100)], 40))",
            "normal! \(rowNumber)G\(colNumber)|",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data(":enew\r".utf8))
    }

    private func configureNvimCursorShapeSmoke() {
        let command = [
            "set guicursor=i:ver25-blinkwait300-blinkon200-blinkoff150",
            "setlocal norelativenumber nonumber laststatus=0 noruler noshowmode virtualedit=all",
            "call setline(1, ['CURSORSHAPE'] + repeat([repeat(' ', 100)], 20))",
            "call cursor(6, 12)",
            "startinsert",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data(":startinsert\r".utf8))
    }

    private func configureNvimCursorNormalShapeSmoke() {
        let command = [
            "set guicursor=n:block-blinkon0",
            "setlocal norelativenumber nonumber laststatus=0 noruler noshowmode virtualedit=all",
            "call setline(1, ['CURSORNORMAL'] + repeat([repeat(' ', 100)], 20))",
            "call cursor(6, 12)",
            "stopinsert",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data("\u{1b}".utf8))
    }

    private func configureNvimCursorReplaceShapeSmoke() {
        let command = [
            "set guicursor=r:hor20-blinkon0",
            "setlocal norelativenumber nonumber laststatus=0 noruler noshowmode virtualedit=all",
            "call setline(1, ['CURSORREPLACE'] + repeat([repeat(' ', 100)], 20))",
            "call cursor(6, 12)",
            "startreplace",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data(":startreplace\r".utf8))
    }

    private func configureNvimCursorBlinkSmoke() {
        let command = [
            "set guicursor=i:ver25-blinkwait100-blinkon100-blinkoff2000",
            "setlocal norelativenumber nonumber laststatus=0 noruler noshowmode virtualedit=all",
            "call setline(1, ['CURSORBLINK'] + repeat([repeat(' ', 100)], 20))",
            "call cursor(6, 12)",
            "startinsert",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data(":startinsert\r".utf8))
    }

    private func configureNvimPopupmenuSmoke() {
        runNvimCommandOrWrite(
            nvimPopupmenuSetupCommand(),
            fallback: Data(":echo 'POPUPONE'\r".utf8)
        )
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.4) { [weak self] in
            self?.writeToActivePane(Data(":NvtermPopupDummy P\t".utf8))
        }
    }

    private func configureNvimSkiaSmoke() {
        let command = [
            "set laststatus=0 noruler noshowmode",
            "setlocal norelativenumber nonumber signcolumn=no foldcolumn=0 virtualedit=all",
            "call setline(1, ['SKIASMOKE', 'renderer basic smoke'])",
            "call cursor(2, 1)",
        ].joined(separator: " | ")
        runNvimCommandOrWrite(command, fallback: Data(":enew\r".utf8))
    }

    private func openNvimSmokeBuffer(path: String, terminalCommand: String) {
        writeSmokeLines(path: path)
        switch activePaneMode() {
        case .terminal:
            let command = [
                "tmp=\(path)",
                "seq 1 300 > $tmp",
                terminalCommand,
            ].joined(separator: "; ")
            writeToActivePane(Data("\(command)\r".utf8))
        case .neovim:
            runNvimCommandOrWrite(
                "edit \(path)",
                fallback: Data(":edit \(path)\r".utf8)
            )
        }
    }

    private func openNvimShapedTextSmokeBuffer(path: String, terminalCommand: String) {
        writeShapedTextSmokeLines(path: path)
        switch activePaneMode() {
        case .terminal:
            let command = [
                "tmp=\(path)",
                terminalCommand,
            ].joined(separator: "; ")
            writeToActivePane(Data("\(command)\r".utf8))
        case .neovim:
            runNvimCommandOrWrite(
                "edit \(path)",
                fallback: Data(":edit \(path)\r".utf8)
            )
        }
    }

    private func writeSmokeLines(path: String) {
        let text = (1...300).map(String.init).joined(separator: "\n") + "\n"
        try? text.write(toFile: path, atomically: true, encoding: .utf8)
    }

    private func writeShapedTextSmokeLines(path: String) {
        let lines = [
            "shaped 日本語 \u{e0b0} e\u{301} Ω",
            "latin ABC xyz",
            "nerd \u{e0b2}\u{f013}",
            "combining a\u{0308}",
            "ambiguous ·→",
        ]
        let text = lines.joined(separator: "\n") + "\n"
        try? text.write(toFile: path, atomically: true, encoding: .utf8)
    }

    private func nvimAnimationSmokeSummary(
        _ shift: OutputScrollShift?,
        hasModelFrames: Bool,
        skiaFrames: Int
    ) -> String {
        let rendererSummary = "model-frames=\(hasModelFrames ? "yes" : "no") " +
            "skia-frames=\(skiaFrames > 0 ? "yes" : "no") count=\(skiaFrames)"
        guard let shift else {
            return "missing-scroll-region-shift \(rendererSummary)"
        }
        var columns = ""
        if let startCol = shift.startCol, let endCol = shift.endCol {
            columns = " cols=\(startCol)..\(endCol)"
        }
        return "rows=\(shift.rows) start=\(shift.startRow) end=\(shift.endRow)\(columns) " +
            rendererSummary
    }

    private func clearSmokeScrollShift() {
        if activePaneMode() == .neovim {
            lastNvimModelScrollShift = nil
            return
        }
        terminalTextView.clearLastScrollRegionShift()
    }

    private func consumeSmokeScrollShift() -> OutputScrollShift? {
        if activePaneMode() == .neovim {
            defer {
                lastNvimModelScrollShift = nil
            }
            return lastNvimModelScrollShift
        }
        return terminalTextView.consumeLastScrollRegionShift()
    }

    private func peekSmokeScrollShift() -> OutputScrollShift? {
        if activePaneMode() == .neovim {
            return lastNvimModelScrollShift
        }
        return terminalTextView.peekLastScrollRegionShift()
    }

    private func activePaneRendererScrollPosition() -> Double {
        guard let paneId = activePaneId,
              let pane = terminalPanes[paneId]
        else {
            return 0
        }
        return pane.rendererScrollPosition()
    }

    private func runNvimCommandOrWrite(_ command: String, fallback: Data) {
        if !runNvimCommand(command) {
            writeToActivePane(fallback)
        }
    }

    @discardableResult
    private func runNvimCommand(_ command: String) -> Bool {
        guard let paneId = activePaneId,
              let pane = terminalPanes[paneId]
        else {
            return false
        }
        guard pane.kind == .neovim else {
            return false
        }
        let ok = pane.runCommand(command)
        if ok {
            drainTerminalPanes()
        }
        return ok
    }

    private func syncFromCore() {
        guard let snapshot = core.snapshot() else {
            return
        }

        lastSnapshot = snapshot
        syncTabs(snapshot)
        syncActivePane(snapshot)
        view.window?.title = windowTitle(snapshot)
    }

    private func selectTab(_ index: Int) {
        _ = core.selectTab(index)
        syncFromCore()
        focusTerminal()
    }

    private func selectTabForContextMenu(_ index: Int) {
        guard core.selectTab(index) else {
            return
        }
        syncFromCore()
    }

    private func syncTabs(_ snapshot: TerminalCoreSnapshot) {
        syncingTabs = true
        tabControl.segmentCount = snapshot.tabs.count
        for (idx, tab) in snapshot.tabs.enumerated() {
            tabControl.setLabel(tab.title, forSegment: idx)
            tabControl.setWidth(tabWidth(for: tab.title), forSegment: idx)
        }
        if snapshot.active_tab < tabControl.segmentCount {
            tabControl.selectedSegment = snapshot.active_tab
        }
        terminalTextView.updateTabs(
            snapshot.tabs.map(\.title),
            themes: snapshot.tabs.map(\.theme),
            active: snapshot.active_tab
        )
        syncingTabs = false
    }

    private func themeMenuItem() -> NSMenuItem {
        let item = NSMenuItem(title: "Color Theme", action: nil, keyEquivalent: "")
        let submenu = NSMenu()
        for theme in themes {
            let themeItem = menuItem(theme, #selector(setThemeFromMenu(_:)))
            themeItem.representedObject = theme
            themeItem.image = colorSwatchImage(themeAccentColor(theme))
            themeItem.state = theme == activeTheme() ? .on : .off
            submenu.addItem(themeItem)
        }
        item.submenu = submenu
        return item
    }

    private func menuItem(_ title: String, _ action: Selector) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: "")
        item.target = self
        return item
    }

    private func activeTheme() -> String? {
        guard let snapshot = lastSnapshot else {
            return nil
        }
        return snapshot.tabs.first(where: { $0.index == snapshot.active_tab })?.theme
    }

    private func windowTitle(_ snapshot: TerminalCoreSnapshot) -> String {
        let tab = snapshot.tabs.first(where: { $0.index == snapshot.active_tab })
        return "\(tab?.title ?? "neovide-tabs") - native shell"
    }

    private func configureTabControl() {
        tabControl.translatesAutoresizingMaskIntoConstraints = false
        tabControl.segmentStyle = .capsule
        tabControl.trackingMode = .selectOne
        tabControl.target = self
        tabControl.action = #selector(tabControlChanged(_:))
    }

    private func tabWidth(for title: String) -> CGFloat {
        let measured = (title as NSString).size(withAttributes: [
            .font: NSFont.systemFont(ofSize: 13, weight: .semibold),
        ])
        return min(max(measured.width + 34, 112), 190)
    }

    private func configureTerminalTextView() {
        terminalTextView.translatesAutoresizingMaskIntoConstraints = false
        terminalTextView.wantsLayer = true
        terminalTextView.layer?.backgroundColor = terminalBackground.cgColor
        terminalTextView.layer?.zPosition = 1
    }

    private func syncActivePane(_ snapshot: TerminalCoreSnapshot) {
        guard let paneId = activePaneId(in: snapshot) else {
            activePaneId = nil
            terminalTextView.setFrame(nil)
            return
        }

        if activePaneId != paneId {
            terminalTextView.resetScrollAnimation()
            lastNvimModelScrollShift = nil
            commandBuffers[paneId] = TerminalInputCommandBuffer()
        }
        activePaneId = paneId
        _ = terminalPane(for: paneId)
        updateActiveFrame()
    }

    private func activePaneId(in snapshot: TerminalCoreSnapshot) -> Int? {
        snapshot.tabs.first(where: { $0.index == snapshot.active_tab })?.active_pane
    }

    private func terminalPane(for paneId: Int) -> NativePane? {
        if let pane = terminalPanes[paneId] {
            return pane
        }

        guard let pane = makePane(grid: terminalTextView.terminalGridSize()) else {
            return nil
        }
        terminalPanes[paneId] = pane
        return pane
    }

    private func makePane(
        grid: (rows: Int, cols: Int, widthPixels: Int, heightPixels: Int)
    ) -> NativePane? {
        switch defaultPaneMode {
        case .terminal:
            return RustTerminalPane(grid: grid)
        case .neovim:
            return RustNeovimPane(grid: grid)
        }
    }

    private func resizeTerminalPanesToGrid() {
        let gridSize = terminalTextView.terminalGridSize()
        for pane in terminalPanes.values {
            pane.resize(grid: gridSize)
        }
        updateActiveFrame()
    }

    private func writeToActivePane(_ data: Data) {
        guard let paneId = activePaneId,
              let pane = terminalPane(for: paneId)
        else {
            return
        }
        if handoffTerminalNeovimInput(data, paneId: paneId, pane: pane) {
            return
        }
        pane.write(data)
        drainTerminalPanes()
    }

    private func handoffTerminalNeovimInput(
        _ data: Data,
        paneId: Int,
        pane: NativePane
    ) -> Bool {
        guard pane.kind == .terminal else {
            return false
        }
        var buffer = commandBuffers[paneId] ?? TerminalInputCommandBuffer()
        let request = buffer.observe(data)
        commandBuffers[paneId] = buffer
        guard let request else {
            return false
        }

        pane.write(Data([0x15]))
        _ = pane.drain()
        guard switchTerminalPaneToNeovim(paneId: paneId, request: request) else {
            pane.write(data)
            drainTerminalPanes()
            return true
        }
        return true
    }

    private func switchTerminalPaneToNeovim(
        paneId: Int,
        request: NeovimLaunchRequest
    ) -> Bool {
        let cwd = terminalPanes[paneId]?.currentWorkingDirectory()
        guard let pane = RustNeovimPane(grid: terminalTextView.terminalGridSize(), cwd: cwd) else {
            return false
        }
        terminalPanes[paneId] = pane
        commandBuffers[paneId] = TerminalInputCommandBuffer()
        scrollRemainders[paneId] = 0
        terminalTextView.resetScrollAnimation()
        lastNvimModelScrollShift = nil
        if let file = request.file {
            _ = pane.runCommand(neovimEditCommand(file))
        }
        drainTerminalPanes()
        updateActiveFrame()
        return true
    }

    private func scrollActivePane(deltaRows: CGFloat) {
        guard let paneId = activePaneId,
              let pane = terminalPane(for: paneId)
        else {
            return
        }

        let requestedRows = wholeScrollRows(deltaRows, paneId: paneId)
        guard requestedRows != 0 else {
            return
        }

        let movedRows = pane.scroll(rows: requestedRows)
        guard movedRows != 0 else {
            return
        }
        terminalTextView.animateScrollRows(movedRows)
        terminalTextView.suppressNextOutputShift()
        updateActiveFrame()
    }

    private func wholeScrollRows(_ deltaRows: CGFloat, paneId: Int) -> Int {
        let accumulated = (scrollRemainders[paneId] ?? 0) + deltaRows
        let wholeRows = Int(accumulated.rounded(.towardZero))
        scrollRemainders[paneId] = accumulated - CGFloat(wholeRows)
        return wholeRows
    }

    private func activePaneMode() -> NativePaneMode {
        guard let paneId = activePaneId,
              let pane = terminalPanes[paneId]
        else {
            return defaultPaneMode
        }
        return pane.kind
    }

    private func neovimEditCommand(_ file: String) -> String {
        "execute 'edit' fnameescape('\(vimSingleQuoted(file))')"
    }

    private func vimSingleQuoted(_ value: String) -> String {
        value.replacingOccurrences(of: "'", with: "''")
    }

    private func startFrameTimer() {
        frameTimer = Timer.scheduledTimer(withTimeInterval: 1.0 / 60.0, repeats: true) { [weak self] _ in
            self?.drainTerminalPanes()
            self?.terminalTextView.advanceAnimation()
        }
    }

    private func drainTerminalPanes() {
        var activePaneChanged = false
        var exitedNvimPanes: [Int] = []
        for (paneId, pane) in terminalPanes {
            let changed = pane.drain()
            activePaneChanged = activePaneChanged || (changed && paneId == activePaneId)
            if pane.kind == .neovim && pane.isExited() {
                exitedNvimPanes.append(paneId)
            }
        }
        for paneId in exitedNvimPanes {
            replaceExitedNeovimPane(paneId)
            activePaneChanged = activePaneChanged || paneId == activePaneId
        }
        if activePaneChanged {
            updateActiveFrame()
        }
    }

    private func replaceExitedNeovimPane(_ paneId: Int) {
        guard let pane = RustTerminalPane(grid: terminalTextView.terminalGridSize()) else {
            terminalPanes.removeValue(forKey: paneId)
            return
        }
        terminalPanes[paneId] = pane
        commandBuffers[paneId] = TerminalInputCommandBuffer()
        scrollRemainders[paneId] = 0
        terminalTextView.resetScrollAnimation()
        lastNvimModelScrollShift = nil
    }

    private func updateActiveFrame() {
        guard let paneId = activePaneId,
              let pane = terminalPanes[paneId]
        else {
            terminalTextView.setFrame(nil)
            return
        }

        if pane.kind == .neovim, let model = pane.rendererModel() {
            if let scrollHint = model.scroll_hint {
                lastNvimModelScrollShift = scrollHint.outputShift
            }
            terminalTextView.setRendererModel(model)
            metalView.needsDisplay = true
            return
        }

        terminalTextView.setFrame(nil)
        metalView.needsDisplay = true
    }

    private func renderActiveMetalFrame(
        texture: MTLTexture,
        renderer: UnsafeMutableRawPointer?
    ) -> Bool {
        guard let renderer,
              let paneId = activePaneId,
              let renderHandle = terminalPanes[paneId]?.renderHandle()
        else {
            return false
        }

        let geometry = terminalTextView.skiaRenderGeometry()
        let pane = terminalPanes[paneId]
        if pane?.kind == .terminal {
            return nvterm_skia_metal_render_terminal(
                renderer,
                renderHandle,
                metalObjectPointer(texture),
                Int32(texture.width),
                Int32(texture.height),
                geometry.originX,
                geometry.originY,
                geometry.cellWidth,
                geometry.cellHeight
            ) != 0
        }

        return nvterm_skia_metal_render_nvim(
            renderer,
            renderHandle,
            metalObjectPointer(texture),
            Int32(texture.width),
            Int32(texture.height),
            geometry.originX,
            geometry.originY,
            geometry.cellWidth,
            geometry.cellHeight
        ) != 0
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    private var window: NSWindow?
    private var shellController: TerminalShellViewController?

    func applicationDidFinishLaunching(_ notification: Notification) {
        guard let core = RustCore() else {
            NSApp.terminate(nil)
            return
        }

        let controller = TerminalShellViewController(core: core)
        let contentRect = NSRect(x: 0, y: 0, width: 1100, height: 720)
        let window = NSWindow(
            contentRect: contentRect,
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )

        window.title = "neovide-tabs native shell"
        window.tabbingMode = .preferred
        controller.view.frame = NSRect(origin: .zero, size: contentRect.size)
        window.contentViewController = controller
        window.makeKeyAndOrderFront(nil)
        self.window = window
        self.shellController = controller

        buildMainMenu()
        applySmokeScenarioIfNeeded(controller)
        controller.focusTerminal()
        NSApp.activate(ignoringOtherApps: true)
        writeSmokeWindowIdIfNeeded(window)
        scheduleSmokeShotIfNeeded(window)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    @objc func newTab(_ sender: Any?) {
        shellController?.newTab(sender)
    }

    @objc func splitVertical(_ sender: Any?) {
        shellController?.splitVertical(sender)
    }

    @objc func splitHorizontal(_ sender: Any?) {
        shellController?.splitHorizontal(sender)
    }

    @objc func renameActiveTab(_ sender: Any?) {
        shellController?.renameActiveTab(sender)
    }

    @objc func zoomIn(_ sender: Any?) {
        shellController?.zoomIn(sender)
    }

    @objc func zoomOut(_ sender: Any?) {
        shellController?.zoomOut(sender)
    }

    @objc func resetZoom(_ sender: Any?) {
        shellController?.resetZoom(sender)
    }

    @objc func selectTabFromShortcut(_ sender: NSMenuItem) {
        shellController?.selectTabFromShortcut(sender.tag)
    }

    private func buildMainMenu() {
        let mainMenu = NSMenu()
        mainMenu.addItem(appMenuItem())
        mainMenu.addItem(viewMenuItem())
        mainMenu.addItem(sessionMenuItem())
        NSApp.mainMenu = mainMenu
    }

    private func applySmokeScenarioIfNeeded(_ controller: TerminalShellViewController) {
        let environment = ProcessInfo.processInfo.environment
        switch environment["NVTERM_NATIVE_SMOKE_SCENARIO"] {
        case "1":
            controller.applySmokeScenario(resultPath: environment["NVTERM_NATIVE_SMOKE_RESULT"])
        case "terminal-vim-scroll":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyTerminalVimScrollSmokeScenario(resultPath: path)
            }
        case "terminal-bottom-input":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyTerminalBottomInputSmokeScenario(resultPath: path)
            }
        case "terminal-nvim-handoff":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyTerminalNvimHandoffSmokeScenario(resultPath: path)
            }
        case "terminal-nvim-cwd":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyTerminalNvimCwdSmokeScenario(resultPath: path)
            }
        case "terminal-nvim-quit":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyTerminalNvimQuitSmokeScenario(resultPath: path)
            }
        case "nvim-scroll":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimScrollSmokeScenario(resultPath: path)
            }
        case "nvim-jump":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimJumpSmokeScenario(resultPath: path)
            }
        case "nvim-side-pane":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimSidePaneSmokeScenario(resultPath: path)
            }
        case "nvim-commandline":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCommandLineSmokeScenario(resultPath: path)
            }
        case "nvim-shaped-text":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimShapedTextSmokeScenario(resultPath: path)
            }
        case "nvim-skia":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimSkiaSmokeScenario(resultPath: path)
            }
        case "nvim-ui-surfaces":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimUiSurfacesSmokeScenario(resultPath: path)
            }
        case "nvim-popupmenu":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimPopupmenuSmokeScenario(resultPath: path)
            }
        case "nvim-cursor-switch":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCursorSwitchSmokeScenario(resultPath: path)
            }
        case "nvim-cursor-shape":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCursorShapeSmokeScenario(resultPath: path)
            }
        case "nvim-cursor-normal-shape":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCursorNormalShapeSmokeScenario(resultPath: path)
            }
        case "nvim-cursor-replace-shape":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCursorReplaceShapeSmokeScenario(resultPath: path)
            }
        case "nvim-cursor-blink":
            if let path = environment["NVTERM_NATIVE_SMOKE_RESULT"], !path.isEmpty {
                controller.applyNvimCursorBlinkSmokeScenario(resultPath: path)
            }
        default:
            break
        }
    }

    private func scheduleSmokeShotIfNeeded(_ window: NSWindow) {
        let environment = ProcessInfo.processInfo.environment
        guard let path = environment["NVTERM_NATIVE_SMOKE_SHOT"], !path.isEmpty else {
            return
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.8) { [weak window] in
            if let window {
                self.writeSmokeShot(path: path, window: window)
            }
            NSApp.terminate(nil)
        }
    }

    private func writeSmokeWindowIdIfNeeded(_ window: NSWindow) {
        let environment = ProcessInfo.processInfo.environment
        guard let path = environment["NVTERM_NATIVE_SMOKE_WINDOW_ID"], !path.isEmpty else {
            return
        }
        try? "\(window.windowNumber)\n".write(
            toFile: path,
            atomically: true,
            encoding: .utf8
        )
    }

    private func writeSmokeShot(path: String, window: NSWindow) {
        guard let contentView = window.contentView else {
            return
        }
        contentView.setFrameSize(contentView.window?.contentLayoutRect.size ?? contentView.frame.size)
        contentView.layoutSubtreeIfNeeded()
        contentView.displayIfNeeded()
        let bounds = contentView.bounds
        guard let bitmap = contentView.bitmapImageRepForCachingDisplay(in: bounds) else {
            return
        }
        contentView.cacheDisplay(in: bounds, to: bitmap)
        guard let data = bitmap.representation(using: .png, properties: [:]) else {
            return
        }
        try? data.write(to: URL(fileURLWithPath: path))
    }

    private func appMenuItem() -> NSMenuItem {
        let item = NSMenuItem()
        let menu = NSMenu()
        menu.addItem(withTitle: "Quit neovide-tabs", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q")
        item.submenu = menu
        return item
    }

    private func sessionMenuItem() -> NSMenuItem {
        let item = NSMenuItem()
        let menu = NSMenu(title: "Session")
        menu.addItem(targetedItem("New Tab", #selector(newTab(_:)), "t"))
        menu.addItem(targetedItem("Split Vertical", #selector(splitVertical(_:)), "d"))
        menu.addItem(targetedItem("Split Horizontal", #selector(splitHorizontal(_:)), "D"))
        menu.addItem(NSMenuItem.separator())
        for shortcutNumber in 1...9 {
            let title = shortcutNumber == 9 ? "Select Last Tab" : "Select Tab \(shortcutNumber)"
            let menuItem = targetedItem(title, #selector(selectTabFromShortcut(_:)), "\(shortcutNumber)")
            menuItem.tag = shortcutNumber
            menu.addItem(menuItem)
        }
        menu.addItem(NSMenuItem.separator())
        menu.addItem(targetedItem("Rename Session", #selector(renameActiveTab(_:)), "r"))
        item.submenu = menu
        return item
    }

    private func viewMenuItem() -> NSMenuItem {
        let item = NSMenuItem()
        let menu = NSMenu(title: "View")
        menu.addItem(targetedItem("Zoom In", #selector(zoomIn(_:)), "+"))
        menu.addItem(targetedItem("Zoom Out", #selector(zoomOut(_:)), "-"))
        menu.addItem(targetedItem("Actual Size", #selector(resetZoom(_:)), "0"))
        item.submenu = menu
        return item
    }

    private func targetedItem(_ title: String, _ action: Selector, _ key: String) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: action, keyEquivalent: key)
        item.target = self
        return item
    }
}

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate
app.setActivationPolicy(.regular)
app.run()
