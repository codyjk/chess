use crate::board::color::Color;
use crate::board::Board;
use crate::moves::chess_move::ChessMove;
use crate::moves::targets::Targets;
use crate::{evaluate, moves};
use log::{debug, log_enabled, trace, Level};
use rustc_hash::FxHashMap;
use thiserror::Error;

type SearchNode = (u64, u8, u8); // (board_hash, depth, current_turn)
type SearchResult = f32; // best_score

pub struct Searcher {
    search_depth: u8,
    search_result_cache: FxHashMap<SearchNode, SearchResult>,
    pub last_searched_position_count: u32,
    pub last_cache_hit_count: u32,
    pub last_alpha_beta_termination_count: u32,
}

#[derive(Error, Debug)]
pub enum SearchError {
    #[error("no available moves")]
    NoAvailableMoves,
}

impl Searcher {
    pub fn new(depth: u8) -> Self {
        Self {
            search_depth: depth,
            search_result_cache: FxHashMap::default(),
            last_searched_position_count: 0,
            last_cache_hit_count: 0,
            last_alpha_beta_termination_count: 0,
        }
    }

    pub fn search(
        &mut self,
        board: &mut Board,
        targets: &mut Targets,
    ) -> Result<ChessMove, SearchError> {
        self.last_searched_position_count = 0;
        self.last_cache_hit_count = 0;
        self.last_alpha_beta_termination_count = 0;

        debug!("starting `search` depth={}", self.search_depth);

        let current_turn = board.turn();
        let candidates = moves::generate(board, current_turn, targets);

        if candidates.len() == 0 {
            return Err(SearchError::NoAvailableMoves);
        }

        let mut results = candidates
            .iter()
            .map(|&chessmove| {
                board.apply(chessmove).unwrap();
                board.next_turn();
                let score = self.alpha_beta_max(
                    self.search_depth,
                    board,
                    targets,
                    f32::NEG_INFINITY,
                    f32::INFINITY,
                );
                board.undo(chessmove).unwrap();
                board.prev_turn();
                (score, chessmove)
            })
            .collect::<Vec<(f32, ChessMove)>>();

        results.sort_by(|(a, _mv_a), (b, _mv_b)| a.partial_cmp(b).unwrap());
        let (_score, best_move) = results[0];

        if log_enabled!(Level::Debug) {
            debug!("ending `search`. results:");
            for (score, chessmove) in results {
                debug!("chessmove={} score={}", chessmove, score);
            }
            debug!("best_move={}", best_move);
        }

        Ok(best_move)
    }

    fn alpha_beta_max(
        &mut self,
        depth: u8,
        board: &mut Board,
        targets: &mut Targets,
        mut alpha: f32,
        beta: f32,
    ) -> f32 {
        self.last_searched_position_count += 1;

        trace!(
            "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) begin",
            depth,
            alpha,
            beta,
            board.current_position_hash()
        );

        if depth == 0 {
            let score = evaluate::score(board, targets, board.turn());
            trace!(
                "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) returning score={}",
                depth,
                alpha,
                beta,
                board.current_position_hash(),
                score
            );
            return score;
        }

        self.check_cache(board.current_position_hash(), depth, board.turn())
            .map(|score| {
                trace!(
                    "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) cached score={}",
                    depth,
                    alpha,
                    beta,
                    board.current_position_hash(),
                    score
                );
                return score;
            });

        let candidates = moves::generate(board, board.turn(), targets);

        for chessmove in candidates {
            trace!(
                "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) evaluating chessmove={}",
                depth,
                alpha,
                beta,
                board.current_position_hash(),
                chessmove
            );
            board.apply(chessmove).unwrap();
            board.next_turn();
            let score = self.alpha_beta_min(depth - 1, board, targets, alpha, beta);
            board.undo(chessmove).unwrap();
            board.prev_turn();
            trace!("alpha_beta_max(depth={}, alpha={}, beta={}, position={}) evaluated chessmove={} score={}", depth, alpha, beta, board.current_position_hash(), chessmove, score);

            if score >= beta {
                self.last_alpha_beta_termination_count += 1;
                trace!("alpha_beta_max(depth={}, alpha={}, beta={}, position={}) hard beta cutoff returning score=beta={}", depth, alpha, beta, board.current_position_hash(), beta);
                self.set_cache(board.current_position_hash(), depth, board.turn(), beta);
                return beta;
            }

            if score > alpha {
                alpha = score;
                trace!(
                    "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) new alpha={}",
                    depth,
                    alpha,
                    beta,
                    board.current_position_hash(),
                    alpha
                );
            }
        }

        self.set_cache(board.current_position_hash(), depth, board.turn(), alpha);

        trace!(
            "alpha_beta_max(depth={}, alpha={}, beta={}, position={}) end",
            depth,
            alpha,
            beta,
            board.current_position_hash()
        );

        return alpha;
    }

    fn alpha_beta_min(
        &mut self,
        depth: u8,
        board: &mut Board,
        targets: &mut Targets,
        alpha: f32,
        mut beta: f32,
    ) -> f32 {
        self.last_searched_position_count += 1;

        trace!(
            "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) begin",
            depth,
            alpha,
            beta,
            board.current_position_hash()
        );

        if depth == 0 {
            let score = -1.0 * evaluate::score(board, targets, board.turn());
            trace!(
                "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) returning score={}",
                depth,
                alpha,
                beta,
                board.current_position_hash(),
                score
            );
            return score;
        }

        self.check_cache(board.current_position_hash(), depth, board.turn())
            .map(|score| {
                trace!(
                    "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) cached score={}",
                    depth,
                    alpha,
                    beta,
                    board.current_position_hash(),
                    score
                );
                return score;
            });

        let candidates = moves::generate(board, board.turn(), targets);

        for chessmove in candidates {
            trace!(
                "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) evaluating chessmove={}",
                depth,
                alpha,
                beta,
                board.current_position_hash(),
                chessmove
            );
            board.apply(chessmove).unwrap();
            board.next_turn();
            let score = self.alpha_beta_max(depth - 1, board, targets, alpha, beta);
            board.undo(chessmove).unwrap();
            board.prev_turn();
            trace!("alpha_beta_min(depth={}, alpha={}, beta={}, position={}) evaluated chessmove={} score={}", depth, alpha, beta, board.current_position_hash(), chessmove, score);

            if score <= alpha {
                self.last_alpha_beta_termination_count += 1;
                self.set_cache(board.current_position_hash(), depth, board.turn(), alpha);
                trace!("alpha_beta_min(depth={}, alpha={}, beta={}, position={}) hard alpha cutoff returning score=alpha={}", depth, alpha, beta, board.current_position_hash(), alpha);

                return alpha;
            }

            if score < beta {
                beta = score;
                trace!(
                    "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) new beta={}",
                    depth,
                    alpha,
                    beta,
                    board.current_position_hash(),
                    beta
                );
            }
        }

        self.set_cache(board.current_position_hash(), depth, board.turn(), beta);

        trace!(
            "alpha_beta_min(depth={}, alpha={}, beta={}, position={}) end",
            depth,
            alpha,
            beta,
            board.current_position_hash()
        );

        return beta;
    }

    fn set_cache(&mut self, position_hash: u64, depth: u8, current_turn: Color, score: f32) {
        let search_node = (position_hash, depth, current_turn as u8);
        self.search_result_cache.insert(search_node, score);
    }

    fn check_cache(&mut self, position_hash: u64, depth: u8, current_turn: Color) -> Option<f32> {
        let search_node = (position_hash, depth, current_turn as u8);
        match self.search_result_cache.get(&search_node) {
            Some(&prev_best_score) => {
                self.last_cache_hit_count += 1;
                Some(prev_best_score)
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::color::Color;
    use crate::board::piece::Piece;
    use crate::board::{square, ALL_CASTLE_RIGHTS};

    #[test]
    fn test_find_mate_in_1_white() {
        let mut board = Board::new();
        let mut targets = Targets::new();
        let mut searcher = Searcher::new(1);

        board.put(C2, Piece::King, Color::White).unwrap();
        board.put(A2, Piece::King, Color::Black).unwrap();
        board.put(B8, Piece::Queen, Color::White).unwrap();
        board.set_turn(Color::White);
        board.lose_castle_rights(ALL_CASTLE_RIGHTS);
        board.update_position_hash();
        println!("Testing board:\n{}", board);

        let chessmove = searcher.search(&mut board, &mut targets).unwrap();
        let valid_checkmates = vec![
            ChessMove::new(B8, B2, None),
            ChessMove::new(B8, A8, None),
            ChessMove::new(B8, A7, None),
        ];
        assert!(
            valid_checkmates.contains(&chessmove),
            "{} does not leed to checkmate",
            chessmove
        );
    }

    #[test]
    fn test_find_mate_in_1_black() {
        let mut board = Board::new();
        let mut targets = Targets::new();
        let mut searcher = Searcher::new(1);

        board.put(C2, Piece::King, Color::Black).unwrap();
        board.put(A2, Piece::King, Color::White).unwrap();
        board.put(B8, Piece::Queen, Color::Black).unwrap();
        board.set_turn(Color::Black);
        board.lose_castle_rights(ALL_CASTLE_RIGHTS);

        println!("Testing board:\n{}", board);

        let chessmove = searcher.search(&mut board, &mut targets).unwrap();

        let valid_checkmates = vec![
            ChessMove::new(B8, B2, None),
            ChessMove::new(B8, A8, None),
            ChessMove::new(B8, A7, None),
        ];
        assert!(valid_checkmates.contains(&chessmove));
    }

    #[test]
    fn test_find_back_rank_mate_in_2_white() {
        let mut board = Board::new();
        let mut targets = Targets::new();
        let mut searcher = Searcher::new(2);

        board.put(A7, Piece::Pawn, Color::Black).unwrap();
        board.put(B7, Piece::Pawn, Color::Black).unwrap();
        board.put(C7, Piece::Pawn, Color::Black).unwrap();
        board.put(B8, Piece::King, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        board.put(D1, Piece::Rook, Color::White).unwrap();
        board.put(D2, Piece::Queen, Color::White).unwrap();
        board.put(A1, Piece::King, Color::White).unwrap();
        board.set_turn(Color::White);
        board.lose_castle_rights(ALL_CASTLE_RIGHTS);

        println!("Testing board:\n{}", board);

        let expected_moves = [
            ChessMove::new(D2, D8, None),
            ChessMove::new(H8, D8, Some((Piece::Queen, Color::White))),
            ChessMove::new(D1, D8, Some((Piece::Rook, Color::Black))),
        ];

        let move1 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move1).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[0], move1);
        println!("Testing board:\n{}", board);

        let move2 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move2).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[1], move2);
        println!("Testing board:\n{}", board);

        let move3 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move3).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[2], move3);
        println!("Testing board:\n{}", board);
    }

    #[test]
    fn test_find_back_rank_mate_in_2_black() {
        let mut board = Board::new();
        let mut targets = Targets::new();
        let mut searcher = Searcher::new(3);

        board.put(F2, Piece::Pawn, Color::White).unwrap();
        board.put(G2, Piece::Pawn, Color::White).unwrap();
        board.put(H2, Piece::Pawn, Color::White).unwrap();
        board.put(G1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        board.put(E8, Piece::Rook, Color::Black).unwrap();
        board.put(E7, Piece::Queen, Color::Black).unwrap();
        board.put(H8, Piece::King, Color::Black).unwrap();
        board.set_turn(Color::Black);
        board.lose_castle_rights(ALL_CASTLE_RIGHTS);

        println!("Testing board:\n{}", board);

        let expected_moves = [
            ChessMove::new(E7, E1, None),
            ChessMove::new(A1, E1, Some((Piece::Queen, Color::Black))),
            ChessMove::new(E8, E1, Some((Piece::Rook, Color::White))),
        ];

        let move1 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move1).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[0], move1);
        println!("Testing board:\n{}", board);

        let move2 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move2).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[1], move2);
        println!("Testing board:\n{}", board);

        let move3 = searcher.search(&mut board, &mut targets).unwrap();
        board.apply(move3).unwrap();
        board.next_turn();
        assert_eq!(expected_moves[2], move3);
        println!("Testing board:\n{}", board);
    }
}
