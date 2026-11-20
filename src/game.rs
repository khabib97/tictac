use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn other(self) -> Self {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn next(self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Medium,
            Difficulty::Medium => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Difficulty::Easy => Difficulty::Hard,
            Difficulty::Medium => Difficulty::Easy,
            Difficulty::Hard => Difficulty::Medium,
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Difficulty::Easy => "Random open cells",
            Difficulty::Medium => "Wins & blocks threats, otherwise random",
            Difficulty::Hard => "Unbeatable Minimax AI",
        }
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Difficulty::Easy => write!(f, "Easy"),
            Difficulty::Medium => write!(f, "Medium"),
            Difficulty::Hard => write!(f, "Hard"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    TwoPlayer,
    VsComputer {
        computer: Player,
        difficulty: Difficulty,
    },
}

impl GameMode {
    pub fn is_computer(self, player: Player) -> bool {
        matches!(self, GameMode::VsComputer { computer, .. } if computer == player)
    }

    pub fn badge(self) -> String {
        match self {
            GameMode::TwoPlayer => "2 Players".to_string(),
            GameMode::VsComputer { difficulty, .. } => format!("vs Computer ({difficulty})"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    OutOfRange,
    CellOccupied,
}

impl fmt::Display for MoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoveError::OutOfRange => write!(f, "Choose a cell between 1 and 9."),
            MoveError::CellOccupied => write!(f, "Cell already taken! Choose another."),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    InProgress,
    Won(Player),
    Draw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    cells: [Option<Player>; 9],
}

impl Board {
    pub const WINNING_LINES: [[usize; 3]; 8] = [
        [0, 1, 2], // top row
        [3, 4, 5], // middle row
        [6, 7, 8], // bottom row
        [0, 3, 6], // left column
        [1, 4, 7], // middle column
        [2, 5, 8], // right column
        [0, 4, 8], // main diagonal
        [2, 4, 6], // anti-diagonal
    ];

    pub fn new() -> Self {
        Self { cells: [None; 9] }
    }

    pub fn cell(&self, idx: usize) -> Option<Player> {
        self.cells[idx]
    }

    pub fn make_move(&mut self, position: usize, player: Player) -> Result<(), MoveError> {
        if !(1..=9).contains(&position) {
            return Err(MoveError::OutOfRange);
        }
        let idx = position - 1;
        if self.cells[idx].is_some() {
            return Err(MoveError::CellOccupied);
        }
        self.cells[idx] = Some(player);
        Ok(())
    }

    pub fn available_positions(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(idx, cell)| cell.is_none().then_some(idx + 1))
            .collect()
    }

    pub fn winning_line(&self) -> Option<(Player, [usize; 3])> {
        for line in Self::WINNING_LINES {
            let [a, b, c] = line;
            if let Some(player) = self.cells[a]
                && self.cells[b] == Some(player)
                && self.cells[c] == Some(player)
            {
                return Some((player, line));
            }
        }
        None
    }

    pub fn winner(&self) -> Option<Player> {
        self.winning_line().map(|(player, _)| player)
    }

    pub fn is_full(&self) -> bool {
        self.cells.iter().all(Option::is_some)
    }

    pub fn state(&self) -> GameState {
        if let Some(player) = self.winner() {
            GameState::Won(player)
        } else if self.is_full() {
            GameState::Draw
        } else {
            GameState::InProgress
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Scoreboard {
    pub x_wins: u32,
    pub o_wins: u32,
    pub draws: u32,
}

impl Scoreboard {
    pub fn record(&mut self, state: GameState) {
        match state {
            GameState::Won(Player::X) => self.x_wins += 1,
            GameState::Won(Player::O) => self.o_wins += 1,
            GameState::Draw => self.draws += 1,
            GameState::InProgress => {}
        }
    }

    pub fn format_for_mode(&self, mode: GameMode) -> String {
        match mode {
            GameMode::TwoPlayer => format!(
                "Player X: {}   |   Player O: {}   |   Draws: {}",
                self.x_wins, self.o_wins, self.draws
            ),
            GameMode::VsComputer { computer, .. } => {
                let (human_wins, comp_wins) = match computer {
                    Player::X => (self.o_wins, self.x_wins),
                    Player::O => (self.x_wins, self.o_wins),
                };
                format!(
                    "You ({}): {}   |   Computer ({}): {}   |   Draws: {}",
                    computer.other(),
                    human_wins,
                    computer,
                    comp_wins,
                    self.draws
                )
            }
        }
    }
}

/// Minimal xorshift64 PRNG so the AI needs no external random crate.
#[derive(Debug, Clone)]
pub struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    pub fn from_time() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xCAFE_BABE_DEAD_BEEF);
        Self::new(nanos)
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let idx = (self.next_u64() as usize) % items.len();
        Some(&items[idx])
    }
}

fn find_immediate_winning_move(board: &Board, player: Player) -> Option<usize> {
    for pos in board.available_positions() {
        let mut next = board.clone();
        next.cells[pos - 1] = Some(player);
        if next.winner() == Some(player) {
            return Some(pos);
        }
    }
    None
}

fn minimax(board: &mut Board, current: Player, ai: Player, depth: i32) -> i32 {
    match board.state() {
        GameState::Won(winner) if winner == ai => return 10 - depth,
        GameState::Won(_) => return depth - 10,
        GameState::Draw => return 0,
        GameState::InProgress => {}
    }

    let maximizing = current == ai;
    let mut best = if maximizing { i32::MIN } else { i32::MAX };

    for idx in 0..9 {
        if board.cells[idx].is_none() {
            board.cells[idx] = Some(current);
            let score = minimax(board, current.other(), ai, depth + 1);
            board.cells[idx] = None;

            if maximizing {
                best = best.max(score);
            } else {
                best = best.min(score);
            }
        }
    }

    best
}

pub fn choose_computer_move(
    board: &Board,
    ai: Player,
    difficulty: Difficulty,
    rng: &mut SimpleRng,
) -> usize {
    let available = board.available_positions();
    assert!(
        !available.is_empty(),
        "choose_computer_move called on a full board"
    );

    match difficulty {
        Difficulty::Easy => *rng.choose(&available).unwrap(),
        Difficulty::Medium => {
            if let Some(win_pos) = find_immediate_winning_move(board, ai) {
                return win_pos;
            }
            if let Some(block_pos) = find_immediate_winning_move(board, ai.other()) {
                return block_pos;
            }
            *rng.choose(&available).unwrap()
        }
        Difficulty::Hard => {
            // Short-circuit empty board to avoid unnecessary full-tree search on move 1.
            if available.len() == 9 {
                let openings = [1, 3, 5, 7, 9];
                return *rng.choose(&openings).unwrap();
            }

            let mut best_score = i32::MIN;
            let mut best_moves = Vec::new();
            let mut scratch = board.clone();

            for pos in available {
                scratch.cells[pos - 1] = Some(ai);
                let score = minimax(&mut scratch, ai.other(), ai, 0);
                scratch.cells[pos - 1] = None;

                if score > best_score {
                    best_score = score;
                    best_moves.clear();
                    best_moves.push(pos);
                } else if score == best_score {
                    best_moves.push(pos);
                }
            }

            *rng.choose(&best_moves).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_and_invalid_moves() {
        let mut board = Board::new();
        assert_eq!(board.make_move(0, Player::X), Err(MoveError::OutOfRange));
        assert_eq!(board.make_move(10, Player::X), Err(MoveError::OutOfRange));
        assert_eq!(board.make_move(5, Player::X), Ok(()));
        assert_eq!(board.make_move(5, Player::O), Err(MoveError::CellOccupied));
    }

    #[test]
    fn detects_row_col_and_diagonal_wins() {
        for line in Board::WINNING_LINES {
            let mut board = Board::new();
            for idx in line {
                board.make_move(idx + 1, Player::X).unwrap();
            }
            assert_eq!(board.state(), GameState::Won(Player::X));
            assert_eq!(board.winning_line(), Some((Player::X, line)));
        }
    }

    #[test]
    fn detects_draw() {
        let moves = [
            (1, Player::X),
            (2, Player::O),
            (3, Player::X),
            (5, Player::O),
            (4, Player::X),
            (6, Player::O),
            (8, Player::X),
            (7, Player::O),
            (9, Player::X),
        ];
        let mut board = Board::new();
        for (pos, player) in moves {
            board.make_move(pos, player).unwrap();
        }
        assert_eq!(board.state(), GameState::Draw);
        assert_eq!(board.winning_line(), None);
    }

    #[test]
    fn easy_ai_picks_valid_empty_cell() {
        let mut board = Board::new();
        for pos in 1..=8 {
            let player = if pos % 2 == 1 { Player::X } else { Player::O };
            board.make_move(pos, player).unwrap();
        }
        let mut rng = SimpleRng::new(42);
        assert_eq!(
            choose_computer_move(&board, Player::X, Difficulty::Easy, &mut rng),
            9
        );
    }

    #[test]
    fn medium_ai_wins_and_blocks() {
        let mut rng = SimpleRng::new(7);

        let mut win_board = Board::new();
        win_board.make_move(1, Player::X).unwrap();
        win_board.make_move(4, Player::O).unwrap();
        win_board.make_move(9, Player::X).unwrap();
        win_board.make_move(5, Player::O).unwrap();
        assert_eq!(
            choose_computer_move(&win_board, Player::O, Difficulty::Medium, &mut rng),
            6
        );

        let mut block_board = Board::new();
        block_board.make_move(1, Player::X).unwrap();
        block_board.make_move(5, Player::O).unwrap();
        block_board.make_move(2, Player::X).unwrap();
        assert_eq!(
            choose_computer_move(&block_board, Player::O, Difficulty::Medium, &mut rng),
            3
        );
    }

    #[test]
    fn hard_ai_is_unbeatable() {
        fn verify_never_loses(board: &Board, current: Player, ai: Player, rng: &mut SimpleRng) {
            match board.state() {
                GameState::Won(winner) => {
                    assert_eq!(winner, ai, "Hard AI lost!");
                    return;
                }
                GameState::Draw => return,
                GameState::InProgress => {}
            }

            if current == ai {
                let pos = choose_computer_move(board, ai, Difficulty::Hard, rng);
                let mut next = board.clone();
                next.make_move(pos, ai).unwrap();
                verify_never_loses(&next, current.other(), ai, rng);
            } else {
                for pos in board.available_positions() {
                    let mut next = board.clone();
                    next.make_move(pos, current).unwrap();
                    verify_never_loses(&next, current.other(), ai, rng);
                }
            }
        }

        let mut rng = SimpleRng::new(12345);
        verify_never_loses(&Board::new(), Player::X, Player::O, &mut rng);
        verify_never_loses(&Board::new(), Player::X, Player::X, &mut rng);
    }
}
