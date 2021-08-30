pub mod command;

use crate::board::color::Color;
use crate::board::Board;
use crate::moves;
use crate::moves::ray_table::RayTable;
use rand::{self, Rng};
use thiserror::Error;

pub struct Game {
    board: Board,
    ray_table: RayTable,
}

#[derive(Error, Debug)]
pub enum GameError {
    #[error("that is not a valid move")]
    InvalidMove,
    #[error("no available moves")]
    NoAvailableMoves,
    #[error("board error: {msg:?}")]
    BoardError { msg: &'static str },
}

impl Game {
    pub fn new() -> Self {
        let mut ray_table = RayTable::new();
        ray_table.populate();

        Self {
            board: Board::starting_position(),
            ray_table: ray_table,
        }
    }

    pub fn turn(&self) -> Color {
        self.board.turn()
    }

    pub fn next_turn(&mut self) -> Color {
        self.board.next_turn()
    }

    pub fn render_board(&self) -> String {
        self.board.to_ascii()
    }

    pub fn make_move(&mut self, from_square: u64, to_square: u64) -> Result<(), GameError> {
        let turn = self.turn();
        let candidates = moves::generate(&mut self.board, turn, &self.ray_table);
        let maybe_chessmove = candidates
            .iter()
            .find(|&m| m.from_square() == from_square && m.to_square() == to_square);
        let chessmove = match maybe_chessmove {
            Some(result) => *result,
            None => return Err(GameError::InvalidMove),
        };
        match self.board.apply(chessmove) {
            Ok(_capture) => Ok(()),
            Err(error) => Err(GameError::BoardError { msg: error }),
        }
    }

    pub fn make_random_move(&mut self) -> Result<(), GameError> {
        let turn = self.turn();
        let candidates = moves::generate(&mut self.board, turn, &self.ray_table);
        let chessmove = match candidates.len() {
            0 => return Err(GameError::NoAvailableMoves),
            _ => {
                let rng = rand::thread_rng().gen_range(0..candidates.len());
                candidates[rng]
            }
        };
        match self.board.apply(chessmove) {
            Ok(_capture) => Ok(()),
            Err(error) => Err(GameError::BoardError { msg: error }),
        }
    }
}