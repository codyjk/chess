use super::color::Color;
use super::piece::Piece;
use super::square;
use super::{
    Board, BLACK_KINGSIDE_RIGHTS, BLACK_QUEENSIDE_RIGHTS, WHITE_KINGSIDE_RIGHTS,
    WHITE_QUEENSIDE_RIGHTS,
};
use regex::Regex;

pub const STARTING_POSITION_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

impl Board {
    /// A FEN record contains six fields. The separator between fields is a space. The fields are:
    ///   1. Piece placement (from White's perspective). Each rank is described, starting with rank 8
    ///     and ending with rank 1; within each rank, the contents of each square are described from
    ///     file `a` through file `h`. Following the Standard Algebraic Notation (SAN), each piece is
    ///     identified by a single letter taken from the standard English names (pawn = `P`,
    ///     knight = `N`, bishop = `B`, rook = `R`, queen = `Q` and king = `K`). White pieces are
    ///     designated using upper-case letters (`PNBRQK`) while black pieces use lowercase
    ///     (`pnbrqk`). Empty squares are noted using digits 1 through 8 (the number of empty
    ///     squares), and `/` separates ranks.
    ///   2. Active color. `w` means White moves next, `b` means Black moves next.
    ///   3. Castling availability. If neither side can castle, this is `-`. Otherwise, this has one
    ///     or more letters: `K` (White can castle kingside), `Q` (White can castle queenside), `k`
    ///     (Black can castle kingside), and/or `q` (Black can castle queenside). A move that
    ///     temporarily prevents castling does not negate this notation.
    ///   4. En passant target square in algebraic notation. If there's no en passant target square,
    ///     this is `-`. If a pawn has just made a two-square move, this is the position `behind` the
    ///     pawn. This is recorded regardless of whether there is a pawn in position to make an en
    ///     passant capture.
    ///   5. Halfmove clock: The number of halfmoves since the last capture or pawn advance, used for
    ///     the fifty-move rule.
    ///   6. Fullmove number: The number of the full move. It starts at 1, and is incremented after
    ///     Black's move.
    ///
    /// Starting position FEN: `rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1`
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let re = Regex::new(
            r"(?x)
            # `(?x)` - insignificant whitespace mode. makes it easier to comment
            # `\x20` - character code for a single space ` `
            ^
            ([pnbrqkPNBRQK1-8]{1,8}) # first rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # second rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # third rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # fourth rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # fifth rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # sixth rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # seventh rank
            /
            ([pnbrqkPNBRQK1-8]{1,8}) # eighth rank
            \x20
            (b|w)                    # current turn
            \x20
            ([kqKQ]{1,4}|-)          # castling rights
            \x20
            ([a-h][1-8]|-)           # en passant target square
            \x20
            (0|[1-9][0-9]*)          # halfmove count
            \x20
            ([1-9][0-9]*)            # fullmove count
            $

        ",
        )
        .unwrap();

        let caps = match re.captures(&fen) {
            Some(captures) => captures,
            None => return Err(format!("invalid FEN; could not parse board from `{}`", fen)),
        };

        // blank board
        let mut board = Self::new();

        // parse ranks
        for capture_group in 1..=8 {
            let rank = &caps[capture_group];
            let row = 8 - capture_group;
            let mut col = 0;

            for fen_char in rank.chars() {
                let square = square::from_row_col(row, col);
                assert!(col < 8);
                match Piece::from_fen(fen_char) {
                    Some((piece, color)) => {
                        board.put(square, piece, color).unwrap();
                        col += 1;
                    }
                    None => {
                        // must be empty square. parse it and advance col counter
                        let empty_square_count = fen_char.to_digit(10).unwrap();
                        col += empty_square_count as usize;
                    }
                };
            }
        }

        // parse turn
        board.turn = match &caps[9] {
            "b" => Some(Color::Black),
            "w" => Some(Color::White),
            _ => None,
        }
        .unwrap();

        // parse castling rights
        let raw_rights = &caps[10];
        let mut lost_rights = 0b000;

        if raw_rights != "-" {
            if !raw_rights.contains('K') {
                lost_rights |= WHITE_KINGSIDE_RIGHTS;
            }

            if !raw_rights.contains('Q') {
                lost_rights |= WHITE_QUEENSIDE_RIGHTS;
            }

            if !raw_rights.contains('k') {
                lost_rights |= BLACK_KINGSIDE_RIGHTS;
            }

            if !raw_rights.contains('q') {
                lost_rights |= BLACK_QUEENSIDE_RIGHTS;
            }
        }

        board.lose_castle_rights(lost_rights);

        // parse en passant target square
        let en_passant_target = &caps[11];

        if !en_passant_target.contains('-') {
            let square = square::from_algebraic(en_passant_target);
            board.push_en_passant_target(square);
        }

        // halfmove clock
        let raw_halfmove_clock = &caps[12];
        let halfmove_clock = raw_halfmove_clock.parse::<u8>().unwrap();
        board.push_halfmove_clock(halfmove_clock);

        // fullmove clock
        let raw_fullmove_clock = &caps[13];
        let fullmove_clock = raw_fullmove_clock.parse::<u8>().unwrap();
        board.set_fullmove_clock(fullmove_clock);

        Ok(board)
    }
}

impl Piece {
    pub fn to_fen(&self, color: Color) -> char {
        match (self, color) {
            (Piece::Bishop, Color::Black) => 'b',
            (Piece::Bishop, Color::White) => 'B',
            (Piece::King, Color::Black) => 'k',
            (Piece::King, Color::White) => 'K',
            (Piece::Knight, Color::Black) => 'n',
            (Piece::Knight, Color::White) => 'N',
            (Piece::Pawn, Color::Black) => 'p',
            (Piece::Pawn, Color::White) => 'P',
            (Piece::Queen, Color::Black) => 'q',
            (Piece::Queen, Color::White) => 'Q',
            (Piece::Rook, Color::Black) => 'r',
            (Piece::Rook, Color::White) => 'R',
        }
    }

    pub fn from_fen(c: char) -> Option<(Piece, Color)> {
        match c {
            'b' => Some((Piece::Bishop, Color::Black)),
            'B' => Some((Piece::Bishop, Color::White)),
            'k' => Some((Piece::King, Color::Black)),
            'K' => Some((Piece::King, Color::White)),
            'n' => Some((Piece::Knight, Color::Black)),
            'N' => Some((Piece::Knight, Color::White)),
            'p' => Some((Piece::Pawn, Color::Black)),
            'P' => Some((Piece::Pawn, Color::White)),
            'q' => Some((Piece::Queen, Color::Black)),
            'Q' => Some((Piece::Queen, Color::White)),
            'r' => Some((Piece::Rook, Color::Black)),
            'R' => Some((Piece::Rook, Color::White)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fen() {
        // based off of examples from https://www.chess.com/terms/fen-chess
        let board = Board::from_fen("8/8/8/4p1K1/2k1P3/8/8/8 b - - 4 11").unwrap();
        println!("Testing board:\n{}", board.to_ascii());
        let tests = vec![
            (square::C4, Piece::King, Color::Black),
            (square::E5, Piece::Pawn, Color::Black),
            (square::E4, Piece::Pawn, Color::White),
            (square::G5, Piece::King, Color::White),
        ];

        for (square, piece, color) in &tests {
            assert_eq!(board.get(*square).unwrap(), (*piece, *color));
        }
        let occupied_squares: Vec<u64> = tests
            .into_iter()
            .map(|(square, _expected_piece, _expected_color)| square.clone())
            .collect();

        for square in &square::ORDERED {
            if occupied_squares.contains(&square) {
                continue;
            }
            assert!(matches!(board.get(*square), None));
        }

        assert_eq!(Color::Black, board.turn());
        assert_eq!(
            0b0000
                | WHITE_KINGSIDE_RIGHTS
                | WHITE_QUEENSIDE_RIGHTS
                | BLACK_KINGSIDE_RIGHTS
                | BLACK_QUEENSIDE_RIGHTS,
            board.peek_castle_rights()
        );
        assert_eq!(0, board.peek_en_passant_target());
        assert_eq!(4, board.peek_halfmove_clock());
        assert_eq!(11, board.fullmove_clock());
    }
}
