use crate::board::bitboard::{EMPTY, RANK_1, RANK_2, RANK_4, RANK_5, RANK_7, RANK_8};
use crate::board::color::Color;
use crate::board::error::BoardError;
use crate::board::piece::Piece;
use crate::board::square::*;
use crate::board::{
    Board, BLACK_KINGSIDE_RIGHTS, BLACK_QUEENSIDE_RIGHTS, WHITE_KINGSIDE_RIGHTS,
    WHITE_QUEENSIDE_RIGHTS,
};
use crate::moves::chess_move::{ChessMove, ChessOperation as Op};

type Capture = (Piece, Color);

impl Board {
    pub fn apply(&mut self, cm: ChessMove) -> Result<Option<Capture>, BoardError> {
        let result = match cm.op() {
            Op::Standard => self.apply_standard(cm.from_square(), cm.to_square(), cm.capture()),
            Op::EnPassant => self.apply_en_passant(cm.from_square(), cm.to_square()),
            Op::Promote { to_piece } => {
                self.apply_promote(cm.from_square(), cm.to_square(), cm.capture(), to_piece)
            }
            Op::Castle => self.apply_castle(cm.from_square(), cm.to_square()),
        };
        match result {
            Ok(Some(_capture)) => self.reset_halfmove_clock(),
            Ok(None) => self.increment_halfmove_clock(),
            _ => return result,
        };
        self.update_position_hash();
        self.count_current_position();
        self.increment_fullmove_clock();
        result
    }

    fn apply_standard(
        &mut self,
        from_square: u64,
        to_square: u64,
        expected_capture: Option<Capture>,
    ) -> Result<Option<Capture>, BoardError> {
        let maybe_piece = self.remove(from_square);
        let (piece_to_move, color) = match maybe_piece {
            None => return Err(BoardError::FromSquareIsEmpty { op: "apply" }),
            Some((piece, color)) => (piece, color),
        };

        if self.get(to_square) != expected_capture {
            return Err(BoardError::UnexpectedCaptureResult);
        }

        // check for en passant
        let mut en_passant_target = EMPTY;
        if piece_to_move == Piece::Pawn {
            let is_en_passant = match color {
                Color::White => (from_square & RANK_2 > 0) && (to_square & RANK_4 > 0),
                Color::Black => (from_square & RANK_7 > 0) && (to_square & RANK_5 > 0),
            };

            if is_en_passant {
                en_passant_target = match color {
                    Color::White => from_square << 8,
                    Color::Black => from_square >> 8,
                };
            }
        }
        self.push_en_passant_target(en_passant_target);

        let captured_piece = self.remove(to_square);

        // adjust castle rights if a rook or king moved
        let mut lost_castle_rights = match (piece_to_move, color, from_square) {
            (Piece::Rook, Color::White, A1) => WHITE_QUEENSIDE_RIGHTS,
            (Piece::Rook, Color::White, H1) => WHITE_KINGSIDE_RIGHTS,
            (Piece::Rook, Color::Black, A8) => BLACK_QUEENSIDE_RIGHTS,
            (Piece::Rook, Color::Black, H8) => BLACK_KINGSIDE_RIGHTS,
            (Piece::King, Color::White, E1) => WHITE_KINGSIDE_RIGHTS | WHITE_QUEENSIDE_RIGHTS,
            (Piece::King, Color::Black, E8) => BLACK_KINGSIDE_RIGHTS | BLACK_QUEENSIDE_RIGHTS,
            _ => 0,
        };

        // adjust castle rights if a rook is taken
        lost_castle_rights |= match (captured_piece, to_square) {
            (Some((Piece::Rook, Color::White)), A1) => WHITE_QUEENSIDE_RIGHTS,
            (Some((Piece::Rook, Color::White)), H1) => WHITE_KINGSIDE_RIGHTS,
            (Some((Piece::Rook, Color::Black)), A8) => BLACK_QUEENSIDE_RIGHTS,
            (Some((Piece::Rook, Color::Black)), H8) => BLACK_KINGSIDE_RIGHTS,
            _ => 0,
        };

        self.lose_castle_rights(lost_castle_rights);

        self.put(to_square, piece_to_move, color)
            .map(|_| captured_piece)
    }

    fn apply_en_passant(
        &mut self,
        from_square: u64,
        to_square: u64,
    ) -> Result<Option<Capture>, BoardError> {
        let maybe_piece = self.remove(from_square);
        let (piece_to_move, color) = match maybe_piece {
            None => return Err(BoardError::FromSquareIsEmpty { op: "apply" }),
            Some((piece, color)) => (piece, color),
        };

        if piece_to_move != Piece::Pawn {
            return Err(BoardError::EnPassantNonPawn { op: "apply" });
        }

        // the captured pawn is "behind" the target square
        let capture_square = match color {
            Color::White => to_square >> 8,
            Color::Black => to_square << 8,
        };

        let capture = match self.remove(capture_square) {
            Some((piece, color)) => (piece, color),
            None => return Err(BoardError::EnPassantNonCapture),
        };

        self.push_en_passant_target(EMPTY);

        self.preserve_castle_rights();

        self.put(to_square, piece_to_move, color)
            .map(|_| Some(capture))
    }

    fn apply_promote(
        &mut self,
        from_square: u64,
        to_square: u64,
        expected_capture: Option<Capture>,
        promote_to_piece: Piece,
    ) -> Result<Option<Capture>, BoardError> {
        self.apply_standard(from_square, to_square, expected_capture)
            .map(|capture| {
                let (_piece, color) = self.remove(to_square).unwrap();
                self.put(to_square, promote_to_piece, color).unwrap();
                capture
            })
    }

    fn apply_castle(
        &mut self,
        king_from: u64,
        king_to: u64,
    ) -> Result<Option<Capture>, BoardError> {
        let kingside = match king_to {
            b if b == king_from << 2 => true,
            b if b == king_from >> 2 => false,
            _ => return Err(BoardError::InvalidCastleMoveError),
        };

        let color = match ((king_from & RANK_1 > 0), (king_from & RANK_8 > 0)) {
            (true, false) => Color::White,
            (false, true) => Color::Black,
            _ => return Err(BoardError::InvalidCastleMoveError),
        };

        let (rook_from, rook_to) = match (color, kingside) {
            (Color::White, true) => (H1, F1),
            (Color::White, false) => (A1, D1),
            (Color::Black, true) => (H8, F8),
            (Color::Black, false) => (A8, D8),
        };

        if self.get(king_from) != Some((Piece::King, color)) {
            return Err(BoardError::InvalidCastleStateError {
                msg: "king_from is not a king",
            });
        }

        if self.get(king_to) != None {
            return Err(BoardError::InvalidCastleStateError {
                msg: "king_to is not empty",
            });
        }

        if self.get(rook_from) != Some((Piece::Rook, color)) {
            return Err(BoardError::InvalidCastleStateError {
                msg: "rook_from is not a rook",
            });
        }

        if self.get(rook_to) != None {
            return Err(BoardError::InvalidCastleStateError {
                msg: "rook_to is not empty",
            });
        }

        self.remove(king_from).unwrap();
        self.put(king_to, Piece::King, color).unwrap();
        self.remove(rook_from).unwrap();
        self.put(rook_to, Piece::Rook, color).unwrap();

        let lost_castle_rights = match color {
            Color::White => WHITE_KINGSIDE_RIGHTS | WHITE_QUEENSIDE_RIGHTS,
            Color::Black => BLACK_KINGSIDE_RIGHTS | BLACK_QUEENSIDE_RIGHTS,
        };

        self.push_en_passant_target(EMPTY);
        self.lose_castle_rights(lost_castle_rights);

        Ok(None)
    }

    pub fn undo(&mut self, cm: ChessMove) -> Result<Option<Capture>, BoardError> {
        self.uncount_current_position();
        let result = match cm.op() {
            Op::Standard => self.undo_standard(cm.from_square(), cm.to_square(), cm.capture()),
            Op::EnPassant => self.undo_en_passant(cm.from_square(), cm.to_square()),
            Op::Promote { to_piece } => {
                self.undo_promote(cm.from_square(), cm.to_square(), cm.capture(), to_piece)
            }
            Op::Castle => self.undo_castle(cm.from_square(), cm.to_square()),
        };
        self.pop_halfmove_clock();
        self.decrement_fullmove_clock();
        self.update_position_hash();
        result
    }

    fn undo_standard(
        &mut self,
        from_square: u64,
        to_square: u64,
        capture: Option<Capture>,
    ) -> Result<Option<Capture>, BoardError> {
        // remove the moved piece
        let maybe_piece = self.remove(to_square);
        let (piece_to_move_back, piece_color) = match maybe_piece {
            None => return Err(BoardError::ToSquareIsEmpty { op: "undo" }),
            Some((piece, color)) => (piece, color),
        };

        // put the captured piece back
        if capture.is_some() {
            let (piece, color) = capture.unwrap();
            self.put(to_square, piece, color).unwrap();
        }

        // return to the previous en passant state
        self.pop_en_passant_target();

        // return to the previous castle rights state
        self.pop_castle_rights();

        self.put(from_square, piece_to_move_back, piece_color)
            .map(|_| None)
    }

    fn undo_en_passant(
        &mut self,
        from_square: u64,
        to_square: u64,
    ) -> Result<Option<Capture>, BoardError> {
        // remove the moved pawn
        let maybe_piece = self.remove(to_square);
        let (piece_to_move_back, piece_color) = match maybe_piece {
            None => return Err(BoardError::ToSquareIsEmpty { op: "undo" }),
            Some((piece, color)) => (piece, color),
        };

        if piece_to_move_back != Piece::Pawn {
            return Err(BoardError::EnPassantNonPawn { op: "undo" });
        }

        // return the pawn to its original square
        self.put(from_square, piece_to_move_back, piece_color)
            .unwrap();

        // the captured pawn is "behind" the target square
        let capture_square = match piece_color {
            Color::White => to_square >> 8,
            Color::Black => to_square << 8,
        };

        // return to the previous en passant state
        self.pop_en_passant_target();

        // return to the previous castle rights state
        self.pop_castle_rights();

        self.put(capture_square, Piece::Pawn, piece_color.opposite())
            .map(|_| None)
    }

    fn undo_promote(
        &mut self,
        from_square: u64,
        to_square: u64,
        expected_capture: Option<Capture>,
        _promote_to_piece: Piece,
    ) -> Result<Option<Capture>, BoardError> {
        self.undo_standard(from_square, to_square, expected_capture)
            .map(|capture| {
                let (_piece, color) = self.remove(from_square).unwrap();
                self.put(from_square, Piece::Pawn, color).unwrap();
                capture
            })
    }

    fn undo_castle(&mut self, king_from: u64, king_to: u64) -> Result<Option<Capture>, BoardError> {
        let kingside = match king_to {
            b if b == king_from << 2 => true,
            b if b == king_from >> 2 => false,
            _ => return Err(BoardError::InvalidCastleMoveError),
        };

        let color = match ((king_from & RANK_1 > 0), (king_from & RANK_8 > 0)) {
            (true, false) => Color::White,
            (false, true) => Color::Black,
            _ => return Err(BoardError::InvalidCastleMoveError),
        };

        let (rook_from, rook_to) = match (color, kingside) {
            (Color::White, true) => (H1, F1),
            (Color::White, false) => (A1, D1),
            (Color::Black, true) => (H8, F8),
            (Color::Black, false) => (A8, D8),
        };

        if self.get(king_to) != Some((Piece::King, color)) {
            return Err(BoardError::InvalidCastleStateError {
                msg: "king_to is not a king",
            });
        }

        if self.get(king_from) != None {
            return Err(BoardError::InvalidCastleStateError {
                msg: "king_from is not empty",
            });
        }

        if self.get(rook_to) != Some((Piece::Rook, color)) {
            return Err(BoardError::InvalidCastleStateError {
                msg: "rook_to is not a rook",
            });
        }

        if self.get(rook_from) != None {
            return Err(BoardError::InvalidCastleStateError {
                msg: "rook_from is not empty",
            });
        }

        self.remove(king_to).unwrap();
        self.put(king_from, Piece::King, color).unwrap();
        self.remove(rook_to).unwrap();
        self.put(rook_from, Piece::Rook, color).unwrap();

        // return to the previous en passant state
        self.pop_en_passant_target();

        // return to the previous castle rights state
        self.pop_castle_rights();

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::square;

    #[test]
    fn test_apply_chess_move() {
        let mut board = Board::starting_position();
        println!("Testing board:\n{}", board);

        // using a queens gambit accepted opening to test basic chess move application
        let moves: Vec<(u64, u64, (Piece, Color), Option<(Piece, Color)>)> = vec![
            (E2, E4, (Piece::Pawn, Color::White), None),
            (E7, E5, (Piece::Pawn, Color::Black), None),
            (D2, D4, (Piece::Pawn, Color::White), None),
            (
                E5,
                D4,
                (Piece::Pawn, Color::Black),
                Some((Piece::Pawn, Color::White)),
            ),
        ];

        for (from_square, to_square, moved, expected_capture) in &moves {
            let captured = board
                .apply(ChessMove::new(*from_square, *to_square, *expected_capture))
                .unwrap();
            assert_eq!(board.get(*to_square).unwrap(), *moved);
            assert_eq!(captured, *expected_capture);
            println!("New board state:\n{}", board);
        }
    }

    #[test]
    fn test_undo_pawn_move() {
        let mut board = Board::starting_position();
        let original_board = format!("{}", board);
        println!("Testing board:\n{}", board);

        let chessmove = ChessMove::new(A2, A4, None);
        board.apply(chessmove).unwrap();
        println!("Result after applying move:\n{}", board);
        board.undo(chessmove).unwrap();
        println!("Result after undoing move:\n{}", board);

        let result_board = format!("{}", board);
        assert_eq!(original_board, result_board);
    }

    #[test]
    fn test_undo_knight_move() {
        let mut board = Board::starting_position();
        let original_board = format!("{}", board);
        println!("Testing board:\n{}", board);

        let chessmove = ChessMove::new(B1, C3, None);
        board.apply(chessmove).unwrap();
        println!("Result after applying move:\n{}", board);
        board.undo(chessmove).unwrap();
        println!("Result after undoing move:\n{}", board);

        let chessmove2 = ChessMove::new(B1, A3, None);
        board.apply(chessmove2).unwrap();
        println!("Result after applying move:\n{}", board);
        board.undo(chessmove2).unwrap();
        println!("Result after undoing move:\n{}", board);

        let result_board = format!("{}", board);
        assert_eq!(original_board, result_board);
    }

    #[test]
    fn test_undo_capture() {
        let mut board = Board::new();
        board.put(A2, Piece::Knight, Color::White).unwrap();
        board.put(B4, Piece::Pawn, Color::Black).unwrap();
        let capture = ChessMove::new(A2, B4, Some((Piece::Pawn, Color::Black)));

        board.apply(capture).unwrap();
        board.undo(capture).unwrap();

        assert_eq!(Some((Piece::Knight, Color::White)), board.get(A2));
        assert_eq!(Some((Piece::Pawn, Color::Black)), board.get(B4));
    }

    #[test]
    fn test_apply_and_undo_en_passant() {
        let mut board = Board::new();
        board.put(D2, Piece::Pawn, Color::White).unwrap();
        board.put(E4, Piece::Pawn, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        board.apply(ChessMove::new(D2, D4, None)).unwrap();
        println!("After move that reveals en passant:\n{}", board);
        assert_eq!(Some((Piece::Pawn, Color::White)), board.get(D4));
        assert_eq!(D3, board.peek_en_passant_target());

        let en_passant = ChessMove::en_passant(E4, D3, (Piece::Pawn, Color::White));

        board.apply(en_passant).unwrap();
        println!("After en passant:\n{}", board);
        assert_eq!(Some((Piece::Pawn, Color::Black)), board.get(D3));
        assert_eq!(None, board.get(D4));
        assert_eq!(EMPTY, board.peek_en_passant_target());

        board.undo(en_passant).unwrap();
        println!("Undo en passant:\n{}", board);
        assert_eq!(Some((Piece::Pawn, Color::White)), board.get(D4));
        assert_eq!(Some((Piece::Pawn, Color::Black)), board.get(E4));
        assert_eq!(D3, board.peek_en_passant_target());
    }

    #[test]
    fn test_apply_and_undo_promote() {
        let mut board = Board::new();
        board.put(A7, Piece::Pawn, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        let promotion = ChessMove::promote(A7, A8, None, Piece::Queen);

        board.apply(promotion).unwrap();
        println!("After applying promotion:\n{}", board);
        assert_eq!(None, board.get(A7));
        assert_eq!(Some((Piece::Queen, Color::White)), board.get(A8));

        board.undo(promotion).unwrap();
        println!("After undoing promotion:\n{}", board);
        assert_eq!(Some((Piece::Pawn, Color::White)), board.get(A7));
        assert_eq!(None, board.get(A8));
    }

    #[test]
    fn test_apply_and_undo_promote_with_capture() {
        let mut board = Board::new();
        board.put(A7, Piece::Pawn, Color::White).unwrap();
        board.put(B8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        let promotion = ChessMove::promote(A7, B8, Some((Piece::Rook, Color::Black)), Piece::Queen);

        board.apply(promotion).unwrap();
        println!("After applying promotion:\n{}", board);
        assert_eq!(None, board.get(A7));
        assert_eq!(Some((Piece::Queen, Color::White)), board.get(B8));

        board.undo(promotion).unwrap();
        println!("After undoing promotion:\n{}", board);
        assert_eq!(Some((Piece::Pawn, Color::White)), board.get(A7));
        assert_eq!(Some((Piece::Rook, Color::Black)), board.get(B8));
    }

    #[test]
    fn test_apply_and_undo_castle_white_kingside() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(H1, Piece::Rook, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        let castle = ChessMove::castle_kingside(Color::White);

        board.apply(castle).unwrap();
        println!("After applying castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::White)), board.get(G1));
        assert_eq!(Some((Piece::Rook, Color::White)), board.get(F1));

        board.undo(castle).unwrap();
        println!("After undoing castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::White)), board.get(E1));
        assert_eq!(Some((Piece::Rook, Color::White)), board.get(H1));
    }

    #[test]
    fn test_apply_and_undo_castle_black_kingside() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        let castle = ChessMove::castle_kingside(Color::Black);

        board.apply(castle).unwrap();
        println!("After applying castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::Black)), board.get(G8));
        assert_eq!(Some((Piece::Rook, Color::Black)), board.get(F8));

        board.undo(castle).unwrap();
        println!("After undoing castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::Black)), board.get(E8));
        assert_eq!(Some((Piece::Rook, Color::Black)), board.get(H8));
    }

    #[test]
    fn test_apply_and_undo_castle_white_queenside() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        let castle = ChessMove::castle_queenside(Color::White);

        board.apply(castle).unwrap();
        println!("After applying castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::White)), board.get(C1));
        assert_eq!(Some((Piece::Rook, Color::White)), board.get(D1));

        board.undo(castle).unwrap();
        println!("After undoing castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::White)), board.get(E1));
        assert_eq!(Some((Piece::Rook, Color::White)), board.get(A1));
    }

    #[test]
    fn test_apply_and_undo_castle_black_queenside() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(A8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        let castle = ChessMove::castle_queenside(Color::Black);

        board.apply(castle).unwrap();
        println!("After applying castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::Black)), board.get(C8));
        assert_eq!(Some((Piece::Rook, Color::Black)), board.get(D8));

        board.undo(castle).unwrap();
        println!("After undoing castle:\n{}", board);
        assert_eq!(Some((Piece::King, Color::Black)), board.get(E8));
        assert_eq!(Some((Piece::Rook, Color::Black)), board.get(A8));
    }

    #[test]
    fn test_white_lose_kingside_castle_rights() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(H1, Piece::Rook, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(H1, H2, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS);
    }

    #[test]
    fn test_white_lose_queenside_castle_rights() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(A1, A2, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS);
    }

    #[test]
    fn test_black_lose_kingside_castle_rights() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(H8, H2, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS);
    }

    #[test]
    fn test_black_lose_queenside_castle_rights() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(A8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(A8, A2, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS);
    }

    #[test]
    fn test_white_lose_queenside_castle_rights_from_capture() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        board.put(H1, Piece::Rook, Color::White).unwrap();
        board.put(H8, Piece::Bishop, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS > 0);
        board
            .apply(ChessMove::new(H8, A1, Some((Piece::Rook, Color::White))))
            .unwrap();
        assert_eq!(0, board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS);
    }

    #[test]
    fn test_white_lose_kingside_castle_rights_from_capture() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        board.put(H1, Piece::Rook, Color::White).unwrap();
        board.put(A8, Piece::Bishop, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS > 0);
        board
            .apply(ChessMove::new(A8, H1, Some((Piece::Rook, Color::White))))
            .unwrap();
        assert_eq!(0, board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS);
    }

    #[test]
    fn test_black_lose_queenside_castle_rights_from_capture() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(A8, Piece::Rook, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        board.put(H1, Piece::Bishop, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS > 0);
        board
            .apply(ChessMove::new(H1, A8, Some((Piece::Rook, Color::Black))))
            .unwrap();
        assert_eq!(0, board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS);
    }

    #[test]
    fn test_black_lose_kingside_castle_rights_from_capture() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(A8, Piece::Rook, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        board.put(A1, Piece::Bishop, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS > 0);
        board
            .apply(ChessMove::new(A1, H8, Some((Piece::Rook, Color::Black))))
            .unwrap();
        assert_eq!(0, board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS);
    }

    #[test]
    fn test_white_lose_all_castle_rights() {
        let mut board = Board::new();
        board.put(E1, Piece::King, Color::White).unwrap();
        board.put(A1, Piece::Rook, Color::White).unwrap();
        board.put(H1, Piece::Rook, Color::White).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS > 0);
        assert!(board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(E1, E2, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & WHITE_KINGSIDE_RIGHTS);
        assert_eq!(0, board.peek_castle_rights() & WHITE_QUEENSIDE_RIGHTS);
    }

    #[test]
    fn test_black_lose_all_castle_rights() {
        let mut board = Board::new();
        board.put(E8, Piece::King, Color::Black).unwrap();
        board.put(A8, Piece::Rook, Color::Black).unwrap();
        board.put(H8, Piece::Rook, Color::Black).unwrap();
        println!("Testing board:\n{}", board);

        assert!(board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS > 0);
        assert!(board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS > 0);
        board.apply(ChessMove::new(E8, E7, None)).unwrap();
        assert_eq!(0, board.peek_castle_rights() & BLACK_KINGSIDE_RIGHTS);
        assert_eq!(0, board.peek_castle_rights() & BLACK_QUEENSIDE_RIGHTS);
    }
}
