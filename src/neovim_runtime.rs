use std::{
    env,
    io::{BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{self, Receiver},
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

use anyhow::{Result, anyhow};
use rmpv::{Value, decode::read_value, encode::write_value};

use crate::{
    neovide_render::NeovideRendererModelSnapshot,
    neovim_editor::{NeovimEditor, NeovimFrameSnapshot},
    terminal_runtime::TerminalGridSize,
};

pub struct NativeNeovimRuntime {
    process: NeovimProcess,
    rx: Receiver<Value>,
    next_msg_id: u64,
    editor: NeovimEditor,
}

impl NativeNeovimRuntime {
    pub fn spawn(size: TerminalGridSize) -> Result<Self> {
        let (process, rx) = NeovimProcess::spawn()?;
        let mut runtime = Self {
            process,
            rx,
            next_msg_id: 1,
            editor: NeovimEditor::new(size.cols, size.rows),
        };
        runtime.attach(size)?;
        Ok(runtime)
    }

    pub fn resize(&mut self, size: TerminalGridSize) -> Result<()> {
        self.editor.resize_screen(size.cols, size.rows);
        self.request(
            "nvim_ui_try_resize",
            vec![size.cols.into(), size.rows.into()],
        )
    }

    pub fn input_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        let input = nvim_input_notation(bytes);
        if input.is_empty() {
            return Ok(());
        }
        self.request("nvim_input", vec![input.into()])
    }

    pub fn command(&mut self, command: &str) -> Result<()> {
        self.request("nvim_command", vec![command.into()])
    }

    pub fn drain(&mut self) -> Result<bool> {
        let mut changed = false;
        while let Ok(value) = self.rx.try_recv() {
            changed = self.handle_message(value) || changed;
        }
        Ok(changed)
    }

    pub fn frame(&mut self) -> Result<NeovimFrameSnapshot> {
        Ok(self.editor.snapshot())
    }

    pub fn renderer_model(&self) -> NeovideRendererModelSnapshot {
        self.editor.renderer_model()
    }

    pub fn renderer_model_with_pending_scroll(&mut self) -> NeovideRendererModelSnapshot {
        self.editor.renderer_model_with_pending_scroll()
    }

    pub fn advance_renderer_animations(&mut self, dt: f32) -> bool {
        self.editor.advance_renderer_animations(dt)
    }

    pub fn has_active_renderer_animation(&self) -> bool {
        self.editor.has_active_renderer_animation()
    }

    fn attach(&mut self, size: TerminalGridSize) -> Result<()> {
        let options = Value::Map(vec![
            ("ext_linegrid".into(), true.into()),
            ("ext_multigrid".into(), true.into()),
            ("ext_popupmenu".into(), true.into()),
            ("rgb".into(), true.into()),
        ]);
        self.request(
            "nvim_ui_attach",
            vec![size.cols.into(), size.rows.into(), options],
        )
    }

    fn request(&mut self, method: &str, args: Vec<Value>) -> Result<()> {
        let message = Value::Array(vec![
            0.into(),
            self.next_msg_id.into(),
            method.into(),
            Value::Array(args),
        ]);
        self.next_msg_id += 1;
        write_value(&mut self.process.stdin, &message)?;
        self.process.stdin.flush()?;
        Ok(())
    }

    fn handle_message(&mut self, value: Value) -> bool {
        let Some(items) = value.as_array() else {
            return false;
        };
        if items.len() < 3 || items[0].as_i64() != Some(2) {
            return false;
        }
        if items[1].as_str() != Some("redraw") {
            return false;
        }
        self.handle_redraw_batches(&items[2])
    }

    fn handle_redraw_batches(&mut self, batches: &Value) -> bool {
        let Some(batches) = batches.as_array() else {
            return false;
        };
        let mut changed = false;
        for batch in batches {
            changed = self.handle_redraw_batch(batch) || changed;
        }
        changed
    }

    fn handle_redraw_batch(&mut self, batch: &Value) -> bool {
        let Some(items) = batch.as_array() else {
            return false;
        };
        let Some(event) = items.first().and_then(Value::as_str) else {
            return false;
        };
        if event == "flush" {
            self.editor.flush_renderer();
            return true;
        }
        let mut changed = false;
        for args in &items[1..] {
            changed = self.editor.handle_event(event, args) || changed;
        }
        changed
    }
}

struct NeovimProcess {
    child: Child,
    stdin: ChildStdin,
}

impl NeovimProcess {
    fn spawn() -> Result<(Self, Receiver<Value>)> {
        let mut command = Command::new(nvim_command());
        configure_process_group(&mut command);
        let mut child = command
            .arg("--embed")
            .arg("--cmd")
            .arg("let g:neovide = v:true")
            .arg("--cmd")
            .arg("let g:neovide_tabs = v:true")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("nvim stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("nvim stdout unavailable"))?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || read_msgpack_loop(stdout, tx));
        Ok((Self { child, stdin }, rx))
    }
}

impl Drop for NeovimProcess {
    fn drop(&mut self) {
        terminate_process_tree(&mut self.child);
    }
}

#[cfg(unix)]
fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

#[cfg(not(unix))]
fn configure_process_group(_command: &mut Command) {}

#[cfg(unix)]
fn terminate_process_tree(child: &mut Child) {
    let Ok(pid) = i32::try_from(child.id()) else {
        let _ = child.kill();
        return;
    };
    signal_process_group(pid, libc::SIGTERM);
    if wait_for_child_exit(child) {
        return;
    }
    signal_process_group(pid, libc::SIGKILL);
    let _ = child.wait();
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(unix)]
fn signal_process_group(pid: i32, signal: i32) {
    // SAFETY: `kill` is called with a process-group id and does not dereference pointers.
    unsafe {
        libc::kill(-pid, signal);
    }
}

fn wait_for_child_exit(child: &mut Child) -> bool {
    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => return true,
        }
    }
    false
}

fn read_msgpack_loop(stdout: std::process::ChildStdout, tx: mpsc::Sender<Value>) {
    let mut reader = BufReader::new(stdout);
    while let Ok(value) = read_value(&mut reader) {
        if tx.send(value).is_err() {
            break;
        }
    }
}

fn nvim_command() -> String {
    env::var("NVTERM_NVIM").unwrap_or_else(|_| "nvim".to_owned())
}

fn nvim_input_notation(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut index = 0;
    while index < bytes.len() {
        if let Some((notation, consumed)) = special_key(&bytes[index..]) {
            output.push_str(notation);
            index += consumed;
            continue;
        }
        if bytes[index].is_ascii_control() {
            push_control_notation(bytes[index], &mut output);
            index += 1;
            continue;
        }
        let end = printable_run_end(bytes, index);
        output.push_str(&escape_nvim_input(&String::from_utf8_lossy(
            &bytes[index..end],
        )));
        index = end;
    }
    output
}

fn special_key(bytes: &[u8]) -> Option<(&'static str, usize)> {
    for (sequence, notation) in [
        (b"\x1b[A".as_slice(), "<Up>"),
        (b"\x1b[B".as_slice(), "<Down>"),
        (b"\x1b[C".as_slice(), "<Right>"),
        (b"\x1b[D".as_slice(), "<Left>"),
        (b"\x1b[H".as_slice(), "<Home>"),
        (b"\x1b[F".as_slice(), "<End>"),
        (b"\x1b[3~".as_slice(), "<Del>"),
        (b"\x1b[5~".as_slice(), "<PageUp>"),
        (b"\x1b[6~".as_slice(), "<PageDown>"),
    ] {
        if bytes.starts_with(sequence) {
            return Some((notation, sequence.len()));
        }
    }
    None
}

fn push_control_notation(byte: u8, output: &mut String) {
    match byte {
        b'\r' | b'\n' => output.push_str("<CR>"),
        b'\t' => output.push_str("<Tab>"),
        0x1b => output.push_str("<Esc>"),
        0x7f | 0x08 => output.push_str("<BS>"),
        1..=26 => {
            output.push_str("<C-");
            output.push(char::from(b'a' + byte - 1));
            output.push('>');
        }
        _ => {}
    }
}

fn printable_run_end(bytes: &[u8], start: usize) -> usize {
    let mut end = start;
    while end < bytes.len() && !bytes[end].is_ascii_control() {
        end += 1;
    }
    end
}

fn escape_nvim_input(input: &str) -> String {
    input.replace('<', "<lt>")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_terminal_control_bytes_to_nvim_notation() {
        assert_eq!(nvim_input_notation(&[0x10]), "<C-p>");
        assert_eq!(nvim_input_notation(b"\x1b[A"), "<Up>");
        assert_eq!(nvim_input_notation(b":edit file\r"), ":edit file<CR>");
    }
}
