use crate::game::{
    Board, Difficulty, GameMode, GameState, Player, Scoreboard, SimpleRng, choose_computer_move,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuRow {
    Mode,
    Difficulty,
    HumanMark,
    Start,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Toggle,
    Confirm,
    Digit(usize),
    Menu,
    Restart,
    Quit,
}

pub struct App {
    pub screen: Screen,
    pub vs_computer: bool,
    pub difficulty: Difficulty,
    pub human_mark: Player,
    pub menu_row: MenuRow,

    pub mode: GameMode,
    pub board: Board,
    pub scoreboard: Scoreboard,
    pub starting_player: Player,
    pub current_player: Player,
    pub cursor: usize,
    pub status: String,
    pub status_tone: StatusTone,
    pub should_quit: bool,

    rng: SimpleRng,
}

impl App {
    pub fn new(rng: SimpleRng) -> Self {
        Self {
            screen: Screen::Menu,
            vs_computer: false,
            difficulty: Difficulty::Medium,
            human_mark: Player::X,
            menu_row: MenuRow::Mode,
            mode: GameMode::TwoPlayer,
            board: Board::new(),
            scoreboard: Scoreboard::default(),
            starting_player: Player::X,
            current_player: Player::X,
            cursor: 4,
            status: String::new(),
            status_tone: StatusTone::Info,
            should_quit: false,
            rng,
        }
    }

    pub fn active_menu_rows(&self) -> &'static [MenuRow] {
        if self.vs_computer {
            &[
                MenuRow::Mode,
                MenuRow::Difficulty,
                MenuRow::HumanMark,
                MenuRow::Start,
            ]
        } else {
            &[MenuRow::Mode, MenuRow::Start]
        }
    }

    pub fn handle_action(&mut self, action: Action) {
        if action == Action::Quit {
            self.should_quit = true;
            return;
        }

        match self.screen {
            Screen::Menu => self.handle_menu_action(action),
            Screen::Playing => self.handle_playing_action(action),
        }
    }

    fn handle_menu_action(&mut self, action: Action) {
        match action {
            Action::Up => self.move_menu_focus(-1),
            Action::Down => self.move_menu_focus(1),
            Action::Left => self.adjust_menu_option(false),
            Action::Right | Action::Toggle => {
                if self.menu_row == MenuRow::Start {
                    self.start_session();
                } else {
                    self.adjust_menu_option(true);
                }
            }
            Action::Confirm => self.start_session(),
            Action::Digit(1) => {
                self.vs_computer = false;
                self.menu_row = MenuRow::Mode;
            }
            Action::Digit(2) => {
                self.vs_computer = true;
                self.menu_row = MenuRow::Difficulty;
            }
            Action::Menu => self.should_quit = true,
            Action::Digit(_) | Action::Restart | Action::Quit => {}
        }
    }

    fn move_menu_focus(&mut self, delta: isize) {
        let rows = self.active_menu_rows();
        let idx = rows.iter().position(|&r| r == self.menu_row).unwrap_or(0) as isize;
        let next = (idx + delta).rem_euclid(rows.len() as isize) as usize;
        self.menu_row = rows[next];
    }

    fn adjust_menu_option(&mut self, forward: bool) {
        match self.menu_row {
            MenuRow::Mode => {
                self.vs_computer = !self.vs_computer;
                if !self.vs_computer && !self.active_menu_rows().contains(&self.menu_row) {
                    self.menu_row = MenuRow::Mode;
                }
            }
            MenuRow::Difficulty => {
                self.difficulty = if forward {
                    self.difficulty.next()
                } else {
                    self.difficulty.prev()
                };
            }
            MenuRow::HumanMark => {
                self.human_mark = self.human_mark.other();
            }
            MenuRow::Start => {}
        }
    }

    fn start_session(&mut self) {
        self.mode = if self.vs_computer {
            GameMode::VsComputer {
                computer: self.human_mark.other(),
                difficulty: self.difficulty,
            }
        } else {
            GameMode::TwoPlayer
        };
        self.scoreboard = Scoreboard::default();
        self.starting_player = Player::X;
        self.screen = Screen::Playing;
        self.start_round();
    }

    fn start_round(&mut self) {
        self.board = Board::new();
        self.current_player = self.starting_player;
        self.cursor = 4;

        if self.mode.is_computer(self.current_player) {
            self.perform_computer_turn();
        } else {
            self.set_turn_status(None);
        }
    }

    fn handle_playing_action(&mut self, action: Action) {
        let finished = self.board.state() != GameState::InProgress;

        match action {
            Action::Menu => {
                self.screen = Screen::Menu;
                self.menu_row = MenuRow::Mode;
            }
            Action::Restart => {
                if finished {
                    self.starting_player = self.starting_player.other();
                }
                self.start_round();
            }
            Action::Confirm | Action::Toggle if finished => {
                self.starting_player = self.starting_player.other();
                self.start_round();
            }
            Action::Up if !finished => {
                let row = self.cursor / 3;
                let col = self.cursor % 3;
                self.cursor = ((row + 2) % 3) * 3 + col;
            }
            Action::Down if !finished => {
                let row = self.cursor / 3;
                let col = self.cursor % 3;
                self.cursor = ((row + 1) % 3) * 3 + col;
            }
            Action::Left if !finished => {
                let row = self.cursor / 3;
                let col = self.cursor % 3;
                self.cursor = row * 3 + (col + 2) % 3;
            }
            Action::Right if !finished => {
                let row = self.cursor / 3;
                let col = self.cursor % 3;
                self.cursor = row * 3 + (col + 1) % 3;
            }
            Action::Confirm | Action::Toggle => {
                self.apply_human_move(self.cursor + 1);
            }
            Action::Digit(pos) if !finished && (1..=9).contains(&pos) => {
                self.cursor = pos - 1;
                self.apply_human_move(pos);
            }
            _ => {}
        }
    }

    fn apply_human_move(&mut self, position: usize) {
        if let Err(err) = self.board.make_move(position, self.current_player) {
            self.status = err.to_string();
            self.status_tone = StatusTone::Error;
            return;
        }

        if self.finish_if_round_over() {
            return;
        }

        self.current_player = self.current_player.other();

        if self.mode.is_computer(self.current_player) {
            self.perform_computer_turn();
        } else {
            self.set_turn_status(None);
        }
    }

    fn perform_computer_turn(&mut self) {
        let GameMode::VsComputer {
            computer,
            difficulty,
        } = self.mode
        else {
            return;
        };

        let pos = choose_computer_move(&self.board, computer, difficulty, &mut self.rng);
        self.board
            .make_move(pos, computer)
            .expect("computer chose invalid move");

        if self.finish_if_round_over() {
            return;
        }

        self.current_player = self.current_player.other();
        self.set_turn_status(Some(format!("Computer ({computer}) played {pos}.")));
    }

    fn finish_if_round_over(&mut self) -> bool {
        let state = self.board.state();
        if state == GameState::InProgress {
            return false;
        }

        self.scoreboard.record(state);
        match state {
            GameState::Won(winner) => {
                if self.mode.is_computer(winner) {
                    self.status = format!("Computer ({winner}) wins!");
                    self.status_tone = StatusTone::Error;
                } else if matches!(self.mode, GameMode::VsComputer { .. }) {
                    self.status = format!("You ({winner}) win!");
                    self.status_tone = StatusTone::Success;
                } else {
                    self.status = format!("Player {winner} wins!");
                    self.status_tone = StatusTone::Success;
                }
            }
            GameState::Draw => {
                self.status = "It's a draw!".to_string();
                self.status_tone = StatusTone::Warning;
            }
            GameState::InProgress => unreachable!(),
        }
        true
    }

    fn set_turn_status(&mut self, prefix: Option<String>) {
        let turn_msg = match self.mode {
            GameMode::TwoPlayer => format!("Player {}'s turn", self.current_player),
            GameMode::VsComputer { .. } => format!("Your turn ({})", self.current_player),
        };
        self.status = match prefix {
            Some(p) => format!("{p} {turn_msg}"),
            None => turn_msg,
        };
        self.status_tone = StatusTone::Info;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_navigation_skips_ai_options_in_two_player_mode() {
        let mut app = App::new(SimpleRng::new(1));
        assert_eq!(app.screen, Screen::Menu);
        assert!(!app.vs_computer);
        assert_eq!(app.menu_row, MenuRow::Mode);

        app.handle_action(Action::Down);
        assert_eq!(app.menu_row, MenuRow::Start);

        app.handle_action(Action::Up);
        assert_eq!(app.menu_row, MenuRow::Mode);

        app.handle_action(Action::Right);
        assert!(app.vs_computer);

        app.handle_action(Action::Down);
        assert_eq!(app.menu_row, MenuRow::Difficulty);
        app.handle_action(Action::Right);
        assert_eq!(app.difficulty, Difficulty::Hard);

        app.handle_action(Action::Down);
        assert_eq!(app.menu_row, MenuRow::HumanMark);
        app.handle_action(Action::Right);
        assert_eq!(app.human_mark, Player::O);
    }

    #[test]
    fn two_player_game_flow_and_replay() {
        let mut app = App::new(SimpleRng::new(1));
        app.handle_action(Action::Confirm);
        assert_eq!(app.screen, Screen::Playing);
        assert_eq!(app.mode, GameMode::TwoPlayer);

        // X plays 1, O plays 4, X plays 2, O plays 5, X plays 3 -> X wins
        for digit in [1, 4, 2, 5, 3] {
            app.handle_action(Action::Digit(digit));
        }
        assert_eq!(app.board.state(), GameState::Won(Player::X));
        assert_eq!(app.scoreboard.x_wins, 1);

        // Press Enter to start round 2 (O goes first)
        app.handle_action(Action::Confirm);
        assert_eq!(app.board.state(), GameState::InProgress);
        assert_eq!(app.current_player, Player::O);
    }

    #[test]
    fn computer_goes_first_when_human_picks_o() {
        let mut app = App::new(SimpleRng::new(42));
        app.handle_action(Action::Right); // vs Computer
        app.handle_action(Action::Down); // Difficulty
        app.handle_action(Action::Right); // Hard
        app.handle_action(Action::Down); // HumanMark
        app.handle_action(Action::Right); // Human = O, Computer = X
        app.handle_action(Action::Confirm); // Start

        assert_eq!(app.screen, Screen::Playing);
        assert_eq!(app.board.available_positions().len(), 8);
        assert_eq!(app.current_player, Player::O);
        assert!(app.status.contains("Computer (X) played"));
    }

    #[test]
    fn cursor_wraps_around_grid_and_rejects_occupied_cell() {
        let mut app = App::new(SimpleRng::new(1));
        app.handle_action(Action::Confirm); // starts at cursor 4 (center)
        app.handle_action(Action::Confirm); // X at 5
        assert_eq!(app.board.cell(4), Some(Player::X));

        // Pressing confirm again on occupied center shows error
        app.handle_action(Action::Confirm);
        assert_eq!(app.status_tone, StatusTone::Error);
        assert_eq!(app.current_player, Player::O);

        // Move right twice from center (col 1 -> col 2 -> col 0)
        app.handle_action(Action::Right);
        assert_eq!(app.cursor, 5);
        app.handle_action(Action::Right);
        assert_eq!(app.cursor, 3);
    }
}
