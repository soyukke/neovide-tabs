use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr;

use serde::Serialize;

use crate::core::{RendererContract, SplitAxis, TerminalCore};
use crate::neovim_runtime::NativeNeovimRuntime;
use crate::skia_metal::{NativeSkiaMetalRenderer, SkiaRenderGeometry};
use crate::terminal_runtime::{NativeTerminalRuntime, TerminalGridSize};

const NVTERM_SPLIT_VERTICAL: u32 = 0;
const NVTERM_SPLIT_HORIZONTAL: u32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_create() -> *mut TerminalCore {
    Box::into_raw(Box::new(TerminalCore::new()))
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `handle` must be either null or a pointer returned by `nvterm_core_create`.
/// Non-null handles must be passed to this function at most once.
pub unsafe extern "C" fn nvterm_core_destroy(handle: *mut TerminalCore) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` must come from `nvterm_core_create` and is consumed once here.
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_tab_count(handle: *const TerminalCore) -> usize {
    core_ref(handle).map_or(0, |core| core.snapshot().tabs.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_new_tab(handle: *mut TerminalCore) -> usize {
    let Some(core) = core_mut(handle) else {
        return usize::MAX;
    };
    core.new_tab()
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_split_active(handle: *mut TerminalCore, axis: u32) -> usize {
    let Some(core) = core_mut(handle) else {
        return usize::MAX;
    };
    let Some(axis) = split_axis(axis) else {
        return usize::MAX;
    };
    core.split_active(axis).unwrap_or(usize::MAX)
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_select_tab(handle: *mut TerminalCore, index: usize) -> u8 {
    core_mut(handle).is_some_and(|core| core.select_tab(index)) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_rename_tab(
    handle: *mut TerminalCore,
    index: usize,
    title: *const c_char,
) -> u8 {
    let Some(core) = core_mut(handle) else {
        return 0;
    };
    let Some(title) = c_string(title) else {
        return 0;
    };
    core.rename_tab(index, title) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_set_tab_theme(
    handle: *mut TerminalCore,
    index: usize,
    theme: *const c_char,
) -> u8 {
    let Some(core) = core_mut(handle) else {
        return 0;
    };
    let Some(theme) = c_string(theme) else {
        return 0;
    };
    core.set_tab_theme(index, theme) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_core_snapshot_json(handle: *const TerminalCore) -> *mut c_char {
    let Some(core) = core_ref(handle) else {
        return ptr::null_mut();
    };
    json_ptr(&core.snapshot())
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_renderer_contract_json() -> *mut c_char {
    json_ptr(&RendererContract::current())
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_create(
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
) -> *mut NativeTerminalRuntime {
    let size = TerminalGridSize {
        rows,
        cols,
        pixel_width,
        pixel_height,
    };
    match NativeTerminalRuntime::spawn(size) {
        Ok(runtime) => Box::into_raw(Box::new(runtime)),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `handle` must be either null or a pointer returned by `nvterm_runtime_create`.
/// Non-null handles must be passed to this function at most once.
pub unsafe extern "C" fn nvterm_runtime_destroy(handle: *mut NativeTerminalRuntime) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` must come from `nvterm_runtime_create` and is consumed once here.
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_resize(
    handle: *mut NativeTerminalRuntime,
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
) -> u8 {
    let Some(runtime) = runtime_mut(handle) else {
        return 0;
    };
    let size = TerminalGridSize {
        rows,
        cols,
        pixel_width,
        pixel_height,
    };
    runtime.resize(size).is_ok() as u8
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `bytes` must point to `len` readable bytes for the duration of the call.
pub unsafe extern "C" fn nvterm_runtime_write(
    handle: *mut NativeTerminalRuntime,
    bytes: *const u8,
    len: usize,
) -> u8 {
    let Some(runtime) = runtime_mut(handle) else {
        return 0;
    };
    if bytes.is_null() {
        return 0;
    }

    // SAFETY: The caller promises that `bytes` points to `len` readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(bytes, len) };
    runtime.write_all(bytes).is_ok() as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_drain(handle: *mut NativeTerminalRuntime) -> u8 {
    let Some(runtime) = runtime_mut(handle) else {
        return 0;
    };
    runtime.drain().unwrap_or(false) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_scroll(
    handle: *mut NativeTerminalRuntime,
    requested_rows: isize,
) -> isize {
    let Some(runtime) = runtime_mut(handle) else {
        return 0;
    };
    runtime.scroll_delta(requested_rows).unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_frame_json(handle: *mut NativeTerminalRuntime) -> *mut c_char {
    let Some(runtime) = runtime_mut(handle) else {
        return ptr::null_mut();
    };
    match runtime.frame() {
        Ok(frame) => json_ptr(&frame),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_runtime_renderer_scroll_position(
    handle: *const NativeTerminalRuntime,
) -> f32 {
    runtime_ref(handle).map_or(0.0, NativeTerminalRuntime::renderer_scroll_position)
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_create(
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
) -> *mut NativeNeovimRuntime {
    let size = TerminalGridSize {
        rows,
        cols,
        pixel_width,
        pixel_height,
    };
    match NativeNeovimRuntime::spawn(size) {
        Ok(runtime) => Box::into_raw(Box::new(runtime)),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `handle` must be either null or a pointer returned by `nvterm_nvim_create`.
/// Non-null handles must be passed to this function at most once.
pub unsafe extern "C" fn nvterm_nvim_destroy(handle: *mut NativeNeovimRuntime) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` must come from `nvterm_nvim_create` and is consumed once here.
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_resize(
    handle: *mut NativeNeovimRuntime,
    rows: u16,
    cols: u16,
    pixel_width: u16,
    pixel_height: u16,
) -> u8 {
    let Some(runtime) = nvim_mut(handle) else {
        return 0;
    };
    let size = TerminalGridSize {
        rows,
        cols,
        pixel_width,
        pixel_height,
    };
    runtime.resize(size).is_ok() as u8
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `bytes` must point to `len` readable bytes for the duration of the call.
pub unsafe extern "C" fn nvterm_nvim_input(
    handle: *mut NativeNeovimRuntime,
    bytes: *const u8,
    len: usize,
) -> u8 {
    let Some(runtime) = nvim_mut(handle) else {
        return 0;
    };
    if bytes.is_null() {
        return 0;
    }

    // SAFETY: The caller promises that `bytes` points to `len` readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(bytes, len) };
    runtime.input_bytes(bytes).is_ok() as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_command(
    handle: *mut NativeNeovimRuntime,
    command: *const c_char,
) -> u8 {
    let Some(runtime) = nvim_mut(handle) else {
        return 0;
    };
    let Some(command) = c_string(command) else {
        return 0;
    };
    runtime.command(&command).is_ok() as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_drain(handle: *mut NativeNeovimRuntime) -> u8 {
    let Some(runtime) = nvim_mut(handle) else {
        return 0;
    };
    runtime.drain().unwrap_or(false) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_exited(handle: *mut NativeNeovimRuntime) -> u8 {
    let Some(runtime) = nvim_mut(handle) else {
        return 1;
    };
    runtime.is_exited() as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_frame_json(handle: *mut NativeNeovimRuntime) -> *mut c_char {
    let Some(runtime) = nvim_mut(handle) else {
        return ptr::null_mut();
    };
    match runtime.frame() {
        Ok(frame) => json_ptr(&frame),
        Err(_) => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_nvim_renderer_model_json(handle: *mut NativeNeovimRuntime) -> *mut c_char {
    let Some(runtime) = nvim_mut(handle) else {
        return ptr::null_mut();
    };
    json_ptr(&runtime.renderer_model_with_pending_scroll())
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `device` and `command_queue` must be live Metal protocol object pointers.
pub unsafe extern "C" fn nvterm_skia_metal_create(
    device: *mut c_void,
    command_queue: *mut c_void,
) -> *mut NativeSkiaMetalRenderer {
    // SAFETY: The caller guarantees both pointers are live Metal protocol objects.
    match unsafe { NativeSkiaMetalRenderer::new(device, command_queue) } {
        Some(renderer) => Box::into_raw(Box::new(renderer)),
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `handle` must be either null or a pointer returned by `nvterm_skia_metal_create`.
/// Non-null handles must be passed to this function at most once.
pub unsafe extern "C" fn nvterm_skia_metal_destroy(handle: *mut NativeSkiaMetalRenderer) {
    if handle.is_null() {
        return;
    }

    // SAFETY: `handle` must come from `nvterm_skia_metal_create` and is consumed once here.
    unsafe {
        drop(Box::from_raw(handle));
    }
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `renderer`, `nvim`, and `texture` must be live pointers for the duration of the call.
pub unsafe extern "C" fn nvterm_skia_metal_render_nvim(
    renderer: *mut NativeSkiaMetalRenderer,
    nvim: *mut NativeNeovimRuntime,
    texture: *mut c_void,
    width: i32,
    height: i32,
    origin_x: f32,
    origin_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> u8 {
    let Some(renderer) = skia_renderer_mut(renderer) else {
        return 0;
    };
    let Some(nvim) = nvim_mut(nvim) else {
        return 0;
    };
    let geometry = SkiaRenderGeometry {
        width,
        height,
        origin_x,
        origin_y,
        cell_width,
        cell_height,
    };
    // SAFETY: The caller guarantees the renderer, nvim runtime, and drawable texture are live.
    (unsafe { renderer.render_nvim(nvim, texture, geometry) }) as u8
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `renderer`, `runtime`, and `texture` must be live pointers for the duration of the call.
pub unsafe extern "C" fn nvterm_skia_metal_render_terminal(
    renderer: *mut NativeSkiaMetalRenderer,
    runtime: *mut NativeTerminalRuntime,
    texture: *mut c_void,
    width: i32,
    height: i32,
    origin_x: f32,
    origin_y: f32,
    cell_width: f32,
    cell_height: f32,
) -> u8 {
    let Some(renderer) = skia_renderer_mut(renderer) else {
        return 0;
    };
    let Some(runtime) = runtime_mut(runtime) else {
        return 0;
    };
    let geometry = SkiaRenderGeometry {
        width,
        height,
        origin_x,
        origin_y,
        cell_width,
        cell_height,
    };
    // SAFETY: The caller guarantees the renderer, terminal runtime, and drawable texture are live.
    (unsafe { renderer.render_terminal(runtime, texture, geometry) }) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_skia_metal_needs_animation_frame(
    renderer: *const NativeSkiaMetalRenderer,
) -> u8 {
    skia_renderer_ref(renderer).is_some_and(NativeSkiaMetalRenderer::needs_animation_frame) as u8
}

#[unsafe(no_mangle)]
pub extern "C" fn nvterm_skia_metal_next_frame_delay_ms(
    renderer: *const NativeSkiaMetalRenderer,
) -> u64 {
    skia_renderer_ref(renderer)
        .and_then(NativeSkiaMetalRenderer::next_frame_delay_ms)
        .unwrap_or(u64::MAX)
}

#[unsafe(no_mangle)]
/// # Safety
///
/// `value` must be either null or a pointer returned by this crate through
/// `CString::into_raw`. Non-null pointers must be passed at most once.
pub unsafe extern "C" fn nvterm_string_free(value: *mut c_char) {
    if value.is_null() {
        return;
    }

    // SAFETY: `value` must be a pointer returned by this crate through `CString::into_raw`.
    unsafe {
        drop(CString::from_raw(value));
    }
}

fn split_axis(axis: u32) -> Option<SplitAxis> {
    match axis {
        NVTERM_SPLIT_VERTICAL => Some(SplitAxis::Vertical),
        NVTERM_SPLIT_HORIZONTAL => Some(SplitAxis::Horizontal),
        _ => None,
    }
}

fn core_ref<'a>(handle: *const TerminalCore) -> Option<&'a TerminalCore> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an opaque pointer created by `nvterm_core_create`.
    unsafe { handle.as_ref() }
}

fn core_mut<'a>(handle: *mut TerminalCore) -> Option<&'a mut TerminalCore> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an exclusive opaque pointer created by `nvterm_core_create`.
    unsafe { handle.as_mut() }
}

fn runtime_mut<'a>(handle: *mut NativeTerminalRuntime) -> Option<&'a mut NativeTerminalRuntime> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an exclusive opaque pointer created by `nvterm_runtime_create`.
    unsafe { handle.as_mut() }
}

fn runtime_ref<'a>(handle: *const NativeTerminalRuntime) -> Option<&'a NativeTerminalRuntime> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an opaque pointer created by `nvterm_runtime_create`.
    unsafe { handle.as_ref() }
}

fn nvim_mut<'a>(handle: *mut NativeNeovimRuntime) -> Option<&'a mut NativeNeovimRuntime> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an exclusive opaque pointer created by `nvterm_nvim_create`.
    unsafe { handle.as_mut() }
}

fn skia_renderer_mut<'a>(
    handle: *mut NativeSkiaMetalRenderer,
) -> Option<&'a mut NativeSkiaMetalRenderer> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an exclusive opaque pointer created by `nvterm_skia_metal_create`.
    unsafe { handle.as_mut() }
}

fn skia_renderer_ref<'a>(
    handle: *const NativeSkiaMetalRenderer,
) -> Option<&'a NativeSkiaMetalRenderer> {
    if handle.is_null() {
        return None;
    }

    // SAFETY: Callers pass an opaque pointer created by `nvterm_skia_metal_create`.
    unsafe { handle.as_ref() }
}

fn c_string(value: *const c_char) -> Option<String> {
    if value.is_null() {
        return None;
    }

    // SAFETY: The native host passes a valid NUL-terminated C string.
    unsafe { CStr::from_ptr(value) }
        .to_str()
        .ok()
        .map(str::to_owned)
}

fn json_ptr(value: &impl Serialize) -> *mut c_char {
    let Ok(json) = serde_json::to_string(value) else {
        return ptr::null_mut();
    };
    string_ptr(json)
}

fn string_ptr(value: String) -> *mut c_char {
    match CString::new(value) {
        Ok(value) => value.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffi_snapshot_json_exposes_tabs() {
        let handle = nvterm_core_create();
        assert!(!handle.is_null());

        assert_eq!(nvterm_core_new_tab(handle), 1);
        let json = owned_string(nvterm_core_snapshot_json(handle));

        // SAFETY: `handle` was created by `nvterm_core_create` in this test.
        unsafe {
            nvterm_core_destroy(handle);
        }
        assert!(json.contains("\"active_tab\":1"));
        assert!(json.contains("\"session 2\""));
    }

    #[test]
    fn ffi_renderer_contract_mentions_metal() {
        let json = owned_string(nvterm_renderer_contract_json());

        assert!(json.contains("\"backend\":\"metal\""));
        assert!(json.contains("\"view\":\"MTKView\""));
    }

    fn owned_string(value: *mut c_char) -> String {
        assert!(!value.is_null());
        // SAFETY: `value` is returned by this module and remains valid until freed below.
        let string = unsafe { CStr::from_ptr(value) }
            .to_string_lossy()
            .into_owned();
        // SAFETY: `value` was returned by an FFI function in this module.
        unsafe {
            nvterm_string_free(value);
        }
        string
    }
}
