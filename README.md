# Tic-Tac-Toe (`tictac`)

A cross-platform, keyboard-driven **Tic-Tac-Toe** game rendered inside a fixed, non-scrolling terminal frame (`58×23`). Play locally with a friend or challenge the built-in computer AI across three difficulty levels on **Linux, Windows, and macOS**.

```text
╭────────────────────────────────────────────────────────╮
│          TIC-TAC-TOE   •   vs Computer (Hard)          │
├────────────────────────────────────────────────────────┤
│     You (X): 0   |   Computer (O): 0   |   Draws: 1    │
│                                                        │
│                ┌───────┬───────┬───────┐               │
│                │       │       │       │               │
│                │   X   │   2   │   O   │               │
│                │       │       │       │               │
│                ├───────┼───────┼───────┤               │
│                │       │       │       │               │
│                │   4   │   X   │   6   │               │
│                │       │       │       │               │
│                ├───────┼───────┼───────┤               │
│                │       │       │       │               │
│                │   7   │   8   │   9   │               │
│                │       │       │       │               │
│                └───────┴───────┴───────┘               │
│                                                        │
│             Computer (O) played 3. Your turn (X)       │
├────────────────────────────────────────────────────────┤
│    Arrows/WASD:Move  Enter/1-9:Play  M:Menu  Q:Quit    │
╰────────────────────────────────────────────────────────╯
```

## Features

- **2 Game Modes**:
  - **2 Players**: Local Human vs Human on one keyboard.
  - **1 Player vs Computer**: Choose to play as `X` (first) or `O` (second).
- **3 AI Difficulty Levels**:
  - **Easy** — picks random available cells.
  - **Medium** — takes immediate wins and blocks immediate threats; otherwise plays randomly.
  - **Hard** — unbeatable Minimax AI with depth-aware scoring and randomized tie-breaking.
- **Fixed Non-Scrolling Terminal UI**:
  - Renders inside a centered `58×23` frame on the terminal's alternate screen buffer.
  - Highlights the active cursor cell, colors `X` and `O`, and highlights the winning 3-in-a-row line.
  - Gracefully handles terminal resizing and restores your terminal cleanly on exit.
- **Cross-Platform**: Single standalone binary with no runtime dependencies for **Linux, Windows, and macOS**.

---

## Download & Play (No Rust Required)

Download the archive for your platform from the **[Releases](../../releases)** page:

### Windows
1. Download `tictac-windows-x86_64.zip` and extract it.
2. Double-click `tictac.exe` (or run `.\tictac.exe` in Windows Terminal / PowerShell).

### macOS
1. Download `tictac-macos-arm64.tar.gz` (Apple Silicon M1/M2/M3/M4) or `tictac-macos-x86_64.tar.gz` (Intel).
2. Extract and run in Terminal:
   ```bash
   tar -xzf tictac-macos-*.tar.gz
   ./tictac
   ```
   *(If macOS Gatekeeper blocks an unsigned binary, run `xattr -d com.apple.quarantine ./tictac` once.)*

### Linux
1. Download `tictac-linux-x86_64.tar.gz` (statically linked musl binary).
2. Extract and run:
   ```bash
   tar -xzf tictac-linux-x86_64.tar.gz
   ./tictac
   ```

> **Note:** Make sure your terminal window is at least **58 columns wide × 23 rows tall**.

---

## Build from Source

Requires [Rust](https://rustup.rs/) (edition 2024).

```bash
git clone https://github.com/khabib97/tictac.git
cd tictac
cargo run --release
```

The standalone binary will be created at:
- **Linux / macOS**: `target/release/tictac`
- **Windows**: `target\release\tictac.exe`

---

## Controls

### Setup Menu
| Key | Action |
| --- | --- |
| `↑` / `↓` or `W` / `S` / `J` / `K` | Move between options |
| `←` / `→` or `A` / `D` or `Space` | Change Mode, Difficulty, or Mark |
| `Enter` | Start Game |
| `Q` / `Esc` / `Ctrl+C` | Quit |

### In Game
| Key | Action |
| --- | --- |
| `Arrow keys` or `WASD` or `HJKL` | Move cursor on the 3×3 grid |
| `Enter` / `Space` | Place mark at cursor (or start next round when finished) |
| `1` – `9` | Place mark directly in cells 1–9 |
| `R` | Restart round / Play again |
| `M` / `Esc` | Return to Setup Menu |
| `Q` / `Ctrl+C` | Quit |

---

## Project Structure

```text
src/
├── main.rs   # Entry point
├── game.rs   # Pure game rules, board, scoreboard, and Easy/Medium/Hard AI
├── app.rs    # Headless UI state machine and input actions
└── tui.rs    # Cross-platform fixed-frame terminal renderer (crossterm)
```

### Running Tests & Checks

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all -- --check
```

---

## Publishing a Release on GitHub

This repository includes two GitHub Actions workflows:
- [`.github/workflows/ci.yml`](.github/workflows/ci.yml): Runs formatting, Clippy, tests, and release builds on Linux, Windows, and macOS for every push and PR.
- [`.github/workflows/release.yml`](.github/workflows/release.yml): Automatically builds and publishes standalone archives for **Linux (musl), Windows, macOS (Apple Silicon), and macOS (Intel)** along with SHA-256 checksums whenever you push a version tag.

To publish your first release:

```bash
git init -b main
git add .
git commit -m "Initial release: cross-platform TUI Tic-Tac-Toe"
git remote add origin https://github.com/<your-username>/tictac.git
git push -u origin main

# Tag and trigger automated cross-platform release builds:
git tag v0.1.0
git push origin v0.1.0
```

Within a few minutes, ready-to-play downloads will appear under your repository's **Releases** tab.

---

## License

Licensed under the [MIT License](LICENSE).
