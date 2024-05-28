use super::bitboard::EMPTY;
use super::piece::Piece;
use super::BoardError;

#[derive(Clone, PartialEq)]
pub struct PieceSet {
    // [pawns, rooks, knights, bishops, kings, queens]
    bitboards: [u64; 6],
    occupied: u64,
}

impl Default for PieceSet {
    fn default() -> Self {
        PieceSet {
            bitboards: [EMPTY; 6],
            occupied: EMPTY,
        }
    }
}

impl PieceSet {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn locate(&self, piece: Piece) -> u64 {
        self.bitboards[piece as usize]
    }

    pub fn get(&self, square: u64) -> Option<Piece> {
        for (i, &bitboard) in self.bitboards.iter().enumerate() {
            if square & bitboard > 0 {
                return Some(Piece::from_usize(i));
            }
        }
        None
    }

    pub fn occupied(&self) -> u64 {
        self.occupied
    }

    pub fn is_occupied(&self, square: u64) -> bool {
        square & self.occupied > 0
    }

    pub fn put(&mut self, square: u64, piece: Piece) -> Result<(), BoardError> {
        if self.is_occupied(square) {
            return Err(BoardError::SquareOccupied);
        }

        self.bitboards[piece as usize] |= square;
        self.occupied |= square;

        Ok(())
    }

    pub fn remove(&mut self, square: u64) -> Option<Piece> {
        let removed = self.get(square);
        let removed_piece = match removed {
            Some(piece) => piece,
            None => return None,
        };

        self.bitboards[removed_piece as usize] ^= square;
        self.occupied ^= square;

        removed
    }
}
