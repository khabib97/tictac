use std::io::{self, Stdout, Write};
use std::panic;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{
    Attribute, Color, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor,
};
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode,
};
use crossterm::{execute, queue};

use crate::app::{Action, App, MenuRow, Screen, StatusTone};
use crate::game::{GameState, Player, SimpleRng};

const FRAME_WIDTH: u16 = 58;
const FRAME_HEIGHT: u16 = 23;
const INNER_WIDTH: usize = (FRAME_WIDTH - 2) as usize;

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(err) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(err);
        }
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, ResetColor, Show, LeaveAlternateScreen);
    }
}

fn install_panic_hook() {
    let default_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, ResetColor, Show, LeaveAlternateScreen);
        default_hook(info);
    }));
}

pub fn run() -> io::Result<()> {
    install_panic_hook();
    let _guard = TerminalGuard::enter()?;
    let mut stdout = io::stdout();
    let mut app = App::new(SimpleRng::from_time());

    loop {
        render(&mut stdout, &app)?;

        if app.should_quit {
            break;
        }

        match event::read()? {
            Event::Key(key) => {
                // On Windows, crossterm emits Press, Repeat, and Release events.
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if let Some(action) = map_key(key) {
                    app.handle_action(action);
                    if app.should_quit {
                        break;
                    }
                }
            }
            Event::Resize(_, _) => {
                // Re-renders cleanly on the next loop iteration.
            }
            _ => {}
        }
    }

    Ok(())
}

fn map_key(key: KeyEvent) -> Option<Action> {
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Some(Action::Quit);
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('w' | 'W' | 'k' | 'K') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('s' | 'S' | 'j' | 'J') => Some(Action::Down),
        KeyCode::Left | KeyCode::Char('a' | 'A' | 'h' | 'H') => Some(Action::Left),
        KeyCode::Right | KeyCode::Char('d' | 'D' | 'l' | 'L') => Some(Action::Right),
        KeyCode::Enter => Some(Action::Confirm),
        KeyCode::Char(' ') => Some(Action::Toggle),
        KeyCode::Char(c @ '1'..='9') => Some(Action::Digit((c as u8 - b'0') as usize)),
        KeyCode::Char('m' | 'M') | KeyCode::Esc => Some(Action::Menu),
        KeyCode::Char('r' | 'R') => Some(Action::Restart),
        KeyCode::Char('q' | 'Q') => Some(Action::Quit),
        _ => None,
    }
}

fn render(stdout: &mut Stdout, app: &App) -> io::Result<()> {
    let (term_w, term_h) = terminal::size()?;
    queue!(stdout, ResetColor, Clear(ClearType::All))?;

    if term_w < FRAME_WIDTH || term_h < FRAME_HEIGHT {
        render_too_small(stdout, term_w, term_h)?;
        stdout.flush()?;
        return Ok(());
    }

    let ox = (term_w - FRAME_WIDTH) / 2;
    let oy = (term_h - FRAME_HEIGHT) / 2;

    draw_frame_outline(stdout, ox, oy)?;

    match app.screen {
        Screen::Menu => render_menu(stdout, app, ox, oy)?,
        Screen::Playing => render_playing(stdout, app, ox, oy)?,
    }

    queue!(stdout, ResetColor)?;
    stdout.flush()
}

fn render_too_small(stdout: &mut Stdout, term_w: u16, term_h: u16) -> io::Result<()> {
    let msg1 = format!("Terminal too small ({term_w}x{term_h})");
    let msg2 = format!("Need at least {FRAME_WIDTH}x{FRAME_HEIGHT}");
    let msg3 = "Resize window or press Q to quit";

    let cy = term_h / 2;
    for (offset, text, color) in [
        (-1i16, msg1.as_str(), Color::Yellow),
        (0, msg2.as_str(), Color::White),
        (1, msg3, Color::DarkGrey),
    ] {
        let y = (cy as i16 + offset).clamp(0, term_h.saturating_sub(1) as i16) as u16;
        let clipped: String = text.chars().take(term_w as usize).collect();
        let x = term_w.saturating_sub(clipped.chars().count() as u16) / 2;
        queue!(
            stdout,
            MoveTo(x, y),
            SetForegroundColor(color),
            crossterm::style::Print(clipped)
        )?;
    }
    Ok(())
}

fn draw_frame_outline(stdout: &mut Stdout, ox: u16, oy: u16) -> io::Result<()> {
    let horiz = "─".repeat(INNER_WIDTH);
    let empty = " ".repeat(INNER_WIDTH);

    queue!(stdout, SetForegroundColor(Color::Cyan))?;
    queue!(
        stdout,
        MoveTo(ox, oy),
        crossterm::style::Print(format!("╭{horiz}╮"))
    )?;

    for row in 1..(FRAME_HEIGHT - 1) {
        queue!(
            stdout,
            MoveTo(ox, oy + row),
            crossterm::style::Print(format!("│{empty}│"))
        )?;
    }

    queue!(
        stdout,
        MoveTo(ox, oy + FRAME_HEIGHT - 1),
        crossterm::style::Print(format!("╰{horiz}╯"))
    )?;

    Ok(())
}

fn draw_divider(stdout: &mut Stdout, ox: u16, y: u16) -> io::Result<()> {
    let horiz = "─".repeat(INNER_WIDTH);
    queue!(
        stdout,
        MoveTo(ox, y),
        SetForegroundColor(Color::Cyan),
        crossterm::style::Print(format!("├{horiz}┤"))
    )
}

fn draw_centered(
    stdout: &mut Stdout,
    ox: u16,
    y: u16,
    text: &str,
    fg: Color,
    bold: bool,
) -> io::Result<()> {
    let clipped: String = text.chars().take(INNER_WIDTH).collect();
    let len = clipped.chars().count();
    let pad = INNER_WIDTH.saturating_sub(len) / 2;
    queue!(
        stdout,
        MoveTo(ox + 1 + pad as u16, y),
        SetForegroundColor(fg)
    )?;
    if bold {
        queue!(stdout, SetAttribute(Attribute::Bold))?;
    }
    queue!(
        stdout,
        crossterm::style::Print(clipped),
        SetAttribute(Attribute::Reset)
    )
}

fn render_menu(stdout: &mut Stdout, app: &App, ox: u16, oy: u16) -> io::Result<()> {
    draw_centered(
        stdout,
        ox,
        oy + 1,
        "T I C - T A C - T O E",
        Color::White,
        true,
    )?;
    draw_divider(stdout, ox, oy + 2)?;

    draw_centered(stdout, ox, oy + 4, "Game Setup", Color::Yellow, true)?;

    let mode_value = if app.vs_computer {
        "vs Computer"
    } else {
        "2 Players"
    };
    draw_menu_option(
        stdout,
        ox,
        oy + 7,
        "Mode",
        mode_value,
        app.menu_row == MenuRow::Mode,
        true,
    )?;

    let diff_value = app.difficulty.to_string();
    draw_menu_option(
        stdout,
        ox,
        oy + 9,
        "Difficulty",
        &diff_value,
        app.menu_row == MenuRow::Difficulty,
        app.vs_computer,
    )?;

    let mark_value = match app.human_mark {
        Player::X => "X (First)",
        Player::O => "O (Second)",
    };
    draw_menu_option(
        stdout,
        ox,
        oy + 11,
        "Your Mark",
        mark_value,
        app.menu_row == MenuRow::HumanMark,
        app.vs_computer,
    )?;

    let desc = if app.vs_computer {
        format!("AI: {}", app.difficulty.description())
    } else {
        "Play locally with a friend on one keyboard".to_string()
    };
    draw_centered(stdout, ox, oy + 14, &desc, Color::DarkGrey, false)?;

    // Start button
    let start_selected = app.menu_row == MenuRow::Start;
    let btn_text = if start_selected {
        "  ▶ START GAME ◀  "
    } else {
        "[   START GAME   ]"
    };
    let pad = INNER_WIDTH.saturating_sub(btn_text.chars().count()) / 2;
    queue!(stdout, MoveTo(ox + 1 + pad as u16, oy + 17))?;
    if start_selected {
        queue!(
            stdout,
            SetBackgroundColor(Color::Green),
            SetForegroundColor(Color::Black),
            SetAttribute(Attribute::Bold),
            crossterm::style::Print(btn_text),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
    } else {
        queue!(
            stdout,
            SetForegroundColor(Color::Green),
            SetAttribute(Attribute::Bold),
            crossterm::style::Print(btn_text),
            SetAttribute(Attribute::Reset)
        )?;
    }

    draw_divider(stdout, ox, oy + 20)?;
    draw_centered(
        stdout,
        ox,
        oy + 21,
        "Arrows:Navigate  Space:Change  Enter:Start  Q:Quit",
        Color::DarkGrey,
        false,
    )?;

    Ok(())
}

fn draw_menu_option(
    stdout: &mut Stdout,
    ox: u16,
    y: u16,
    label: &str,
    value: &str,
    focused: bool,
    enabled: bool,
) -> io::Result<()> {
    let pointer = if focused { "▶" } else { " " };
    let formatted_value = if enabled {
        format!("◄ {:^13} ►", value)
    } else {
        format!("  {:^13}  ", "—")
    };
    let line = format!(" {pointer} {label:<12} {formatted_value} ");
    let pad = INNER_WIDTH.saturating_sub(line.chars().count()) / 2;

    queue!(stdout, MoveTo(ox + 1 + pad as u16, y))?;
    if focused {
        queue!(
            stdout,
            SetBackgroundColor(Color::DarkBlue),
            SetForegroundColor(Color::White),
            SetAttribute(Attribute::Bold),
            crossterm::style::Print(line),
            SetAttribute(Attribute::Reset),
            ResetColor
        )?;
    } else if enabled {
        queue!(
            stdout,
            SetForegroundColor(Color::White),
            crossterm::style::Print(line)
        )?;
    } else {
        queue!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            crossterm::style::Print(line)
        )?;
    }
    Ok(())
}

fn render_playing(stdout: &mut Stdout, app: &App, ox: u16, oy: u16) -> io::Result<()> {
    let header = format!("TIC-TAC-TOE   •   {}", app.mode.badge());
    draw_centered(stdout, ox, oy + 1, &header, Color::White, true)?;
    draw_divider(stdout, ox, oy + 2)?;

    let score = app.scoreboard.format_for_mode(app.mode);
    draw_centered(stdout, ox, oy + 3, &score, Color::Yellow, false)?;

    render_board(stdout, app, ox, oy + 5)?;

    let status_color = match app.status_tone {
        StatusTone::Info => Color::White,
        StatusTone::Success => Color::Green,
        StatusTone::Warning => Color::Yellow,
        StatusTone::Error => Color::Red,
    };
    let bold_status = app.board.state() != GameState::InProgress;
    draw_centered(stdout, ox, oy + 19, &app.status, status_color, bold_status)?;

    draw_divider(stdout, ox, oy + 20)?;

    let footer = if app.board.state() == GameState::InProgress {
        "Arrows/WASD:Move  Enter/1-9:Play  M:Menu  Q:Quit"
    } else {
        "Enter/R: Play Again   M: Change Mode   Q: Quit"
    };
    draw_centered(stdout, ox, oy + 21, footer, Color::DarkGrey, false)?;

    Ok(())
}

fn render_board(stdout: &mut Stdout, app: &App, ox: u16, top_y: u16) -> io::Result<()> {
    // 3 cells * 7 chars wide + 4 vertical bars = 25 chars wide, 13 rows tall
    const BOARD_W: usize = 25;
    let bx = ox + 1 + ((INNER_WIDTH - BOARD_W) / 2) as u16;
    let winning = app.board.winning_line();
    let in_progress = app.board.state() == GameState::InProgress;

    queue!(
        stdout,
        MoveTo(bx, top_y),
        SetForegroundColor(Color::DarkGrey),
        crossterm::style::Print("┌───────┬───────┬───────┐")
    )?;

    for row in 0..3 {
        let cell_top = top_y + 1 + (row as u16) * 4;

        for sub_row in 0..3 {
            let y = cell_top + sub_row;
            queue!(
                stdout,
                MoveTo(bx, y),
                SetForegroundColor(Color::DarkGrey),
                crossterm::style::Print("│")
            )?;

            for col in 0..3 {
                let idx = row * 3 + col;
                let cell = app.board.cell(idx);
                let is_cursor = in_progress && app.cursor == idx;
                let is_win_cell = winning.is_some_and(|(_, line)| line.contains(&idx));

                let bg = if is_win_cell {
                    let winner = winning.unwrap().0;
                    if app.mode.is_computer(winner) {
                        Some(Color::DarkRed)
                    } else {
                        Some(Color::DarkGreen)
                    }
                } else if is_cursor {
                    Some(Color::DarkBlue)
                } else {
                    None
                };

                if let Some(bg_color) = bg {
                    queue!(stdout, SetBackgroundColor(bg_color))?;
                }

                if sub_row == 1 {
                    let (ch, fg, bold) = match cell {
                        Some(Player::X) => ("X".to_string(), Color::Cyan, true),
                        Some(Player::O) => ("O".to_string(), Color::Magenta, true),
                        None => (
                            (idx + 1).to_string(),
                            if is_cursor {
                                Color::White
                            } else {
                                Color::DarkGrey
                            },
                            is_cursor,
                        ),
                    };
                    let fg = if is_win_cell { Color::White } else { fg };
                    queue!(stdout, SetForegroundColor(fg))?;
                    if bold {
                        queue!(stdout, SetAttribute(Attribute::Bold))?;
                    }
                    queue!(
                        stdout,
                        crossterm::style::Print(format!("   {ch}   ")),
                        SetAttribute(Attribute::Reset)
                    )?;
                } else {
                    queue!(stdout, crossterm::style::Print("       "))?;
                }

                queue!(
                    stdout,
                    ResetColor,
                    SetForegroundColor(Color::DarkGrey),
                    crossterm::style::Print("│")
                )?;
            }
        }

        let border = if row < 2 {
            "├───────┼───────┼───────┤"
        } else {
            "└───────┴───────┴───────┘"
        };
        queue!(
            stdout,
            MoveTo(bx, cell_top + 3),
            SetForegroundColor(Color::DarkGrey),
            crossterm::style::Print(border)
        )?;
    }

    Ok(())
}
