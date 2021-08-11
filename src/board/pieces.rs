use super::bitboard::{Bitboard, EMPTY};
use super::piece::Piece;

#[derive(Clone, Copy, PartialEq)]
pub struct Pieces {
    pawns: Bitboard,
    rooks: Bitboard,
    knights: Bitboard,
    bishops: Bitboard,
    kings: Bitboard,
    queens: Bitboard,
    occupied: Bitboard,
}

impl Pieces {
    pub fn new() -> Self {
        Pieces {
            bishops: EMPTY,
            kings: EMPTY,
            knights: EMPTY,
            pawns: EMPTY,
            queens: EMPTY,
            rooks: EMPTY,

            occupied: EMPTY,
        }
    }

    pub fn locate(self, piece: Piece) -> Bitboard {
        match piece {
            Piece::Bishop => self.bishops,
            Piece::King => self.kings,
            Piece::Knight => self.knights,
            Piece::Pawn => self.pawns,
            Piece::Queen => self.queens,
            Piece::Rook => self.rooks,
        }
    }

    pub fn get(self, square: u64) -> Option<Piece> {
        if square & self.bishops > 0 {
            return Some(Piece::Bishop);
        } else if square & self.kings > 0 {
            return Some(Piece::King);
        } else if square & self.knights > 0 {
            return Some(Piece::Knight);
        } else if square & self.pawns > 0 {
            return Some(Piece::Pawn);
        } else if square & self.queens > 0 {
            return Some(Piece::Queen);
        } else if square & self.rooks > 0 {
            return Some(Piece::Rook);
        }

        None
    }

    pub fn occupied(self) -> Bitboard {
        self.occupied
    }

    pub fn is_occupied(self, square: u64) -> bool {
        square & self.occupied > 0
    }

    pub fn put(&mut self, square: u64, piece: Piece) -> Result<(), &'static str> {
        if self.is_occupied(square) {
            return Err("that square already has a piece on it");
        }

        match piece {
            Piece::Bishop => self.bishops |= square,
            Piece::King => self.kings |= square,
            Piece::Knight => self.knights |= square,
            Piece::Pawn => self.pawns |= square,
            Piece::Queen => self.queens |= square,
            Piece::Rook => self.rooks |= square,
        };

        self.occupied |= square;

        Ok(())
    }

    pub fn remove(&mut self, square: u64) -> Option<Piece> {
        let removed = self.get(square);
        let removed_piece = match removed {
            Some(piece) => piece,
            None => return None,
        };

        match removed_piece {
            Piece::Bishop => self.bishops ^= square,
            Piece::King => self.kings ^= square,
            Piece::Knight => self.knights ^= square,
            Piece::Pawn => self.pawns ^= square,
            Piece::Queen => self.queens ^= square,
            Piece::Rook => self.rooks ^= square,
        };

        self.occupied ^= square;

        removed
    }
}
