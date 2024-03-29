use crate::board::color::Color;
use crate::board::piece::{Piece, ALL_PIECES};
use crate::board::Board;
use crate::moves;
use crate::moves::targets::{self, Targets};

mod bonus_tables;

#[derive(Debug)]
pub enum GameEnding {
    Checkmate,
    Stalemate,
    Draw,
}

fn current_player_is_in_check(board: &Board, targets: &mut Targets) -> bool {
    let current_player = board.turn();
    let king = board.pieces(current_player).locate(Piece::King);

    let attacked_squares =
        targets::generate_attack_targets(board, current_player.opposite(), targets);

    king & attacked_squares > 0
}

pub fn game_ending(
    board: &mut Board,
    targets: &mut Targets,
    current_turn: Color,
) -> Option<GameEnding> {
    if board.max_seen_position_count() == 3 {
        return Some(GameEnding::Draw);
    }

    let candidates = moves::generate(board, current_turn, targets);
    let check = current_player_is_in_check(board, targets);

    if candidates.len() == 0 {
        if check {
            return Some(GameEnding::Checkmate);
        } else {
            return Some(GameEnding::Stalemate);
        }
    }

    return None;
}

pub fn score(board: &mut Board, targets: &mut Targets, current_turn: Color) -> f32 {
    match (game_ending(board, targets, current_turn), current_turn) {
        (Some(GameEnding::Checkmate), Color::White) => return f32::INFINITY,
        (Some(GameEnding::Checkmate), Color::Black) => return f32::NEG_INFINITY,
        (Some(GameEnding::Stalemate), Color::White) => return f32::NEG_INFINITY,
        (Some(GameEnding::Stalemate), Color::Black) => return f32::INFINITY,
        (Some(GameEnding::Draw), Color::White) => return f32::NEG_INFINITY,
        (Some(GameEnding::Draw), Color::Black) => return f32::INFINITY,
        _ => (),
    };

    material_score(board, Color::White) - material_score(board, Color::Black)
}

fn material_score(board: &Board, color: Color) -> f32 {
    let mut material = 0.;
    let pieces = board.pieces(color);

    for piece in &ALL_PIECES {
        let bonuses = bonus_tables::get(*piece);
        let squares = pieces.locate(*piece);
        let piece_value = f32::from(piece.material_value());

        for i in 0..64 {
            let sq = 1 << i;

            if sq & squares == 0 {
                continue;
            }

            // need to flip around the bonuses if calculating for black
            let bonus_i = match color {
                Color::White => i,
                Color::Black => {
                    let rank = i / 8;
                    let file = i % 8;
                    (7 - file) + ((7 - rank) * 8)
                }
            };

            material += piece_value + bonuses[bonus_i];
        }
    }

    material
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;

    #[test]
    fn test_starting_material_score() {
        let board = Board::starting_position();
        // (piece value) * (piece quantity) * (starting tile bonus)
        // 8 * 1 * 1.0 = 8 pawns
        // 1 * 9 * 1.0 = 9 queens
        // 2 * 5 * 1.0 = 10 rooks
        // 2 * 3 * .75 = 4.5 knights
        // 2 * 3 * 1.0 = 6 bishops
        // total = 37.5
        assert_eq!(
            material_score(&board, Color::White),
            material_score(&board, Color::Black)
        );
    }
}
