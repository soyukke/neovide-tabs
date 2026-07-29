# nvtermctl

`nvtermctl` controls a running Neovide Tabs instance through its local
Unix-domain socket. The executable is bundled inside `Neovide Tabs.app` and is
also placed on the `PATH` of every terminal pane.

## Discovery

Inside a pane, these variables are available:

```text
NVTERM_SOCKET   absolute path to the current control socket
NVTERM_TAB_ID   stable ID of the pane's tab
NVTERM_PANE_ID  stable ID of the pane
NVTERMCTL       absolute path to the bundled CLI
```

Outside the application, pass `--socket PATH`. If it is omitted, the CLI checks
`NVTERM_SOCKET` and then the default path under
`~/Library/Application Support/Neovide Tabs/run/control.sock`.

## Commands

```sh
nvtermctl list
nvtermctl read-screen --pane 1
nvtermctl send --pane 1 "printf 'hello\n'"
nvtermctl send --pane 1 - < input.txt
nvtermctl key --pane 1 Enter
nvtermctl key --pane 1 Ctrl+C

nvtermctl new-tab --cwd /path/to/project
nvtermctl split --pane 1 --vertical --cwd /path/to/project
nvtermctl split --pane 1 --horizontal
nvtermctl rename-tab --tab 1 "build"
nvtermctl set-theme --tab 1 Harbor

nvtermctl status set --pane 1 running "running tests"
nvtermctl status wait --pane 1 --timeout 300
nvtermctl status set --pane 1 done "tests passed"
```

`--pane` defaults to `NVTERM_PANE_ID`, and `--tab` defaults to
`NVTERM_TAB_ID`. Add `--json` for a stable machine-readable response.
`read-screen` returns the current visible terminal rows with trailing spaces
removed. `send` writes text to the pane's PTY; use `key` for Enter, Escape,
arrows, navigation keys, or `Ctrl+A` through `Ctrl+Z`.

Statuses are one of `idle`, `running`, `waiting`, `done`, `failed`, or
`blocked`. A wait returns immediately for a terminal status and otherwise
blocks until the next status update or the requested timeout.

## Protocol and security

Protocol version 1 is newline-delimited JSON. A request has `version: 1`, a
`command`, and command-specific fields. Responses contain `ok` and either
`result` or an error with stable `code` and human-readable `message` fields.

The socket is local-only, mode `0600`, and accepts peers only when their UID
matches the application. Its parent directory must be owner-only, and a second
instance cannot replace a socket that is still accepting connections. Requests
are limited to 1 MiB, concurrent clients to 32, normal requests to 30 seconds,
and status waits to one hour.

This boundary protects against other macOS users, not other processes running
as the same user. Do not move the socket into a shared directory or forward it
to another machine.
