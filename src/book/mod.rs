use common::bitboard::{bitboard::Bitboard, square};
use rustc_hash::FxHashMap;

type BookMove = (Bitboard, Bitboard);

#[derive(Default)]
struct BookNode {
    lines: FxHashMap<BookMove, Box<BookNode>>,
}

impl BookNode {
    fn new() -> Self {
        Default::default()
    }
}

/// A book of common opening lines, represented as a basic DAG.
/// Each node in the DAG represents a move in the opening line.
pub struct Book {
    root: BookNode,
}

impl Default for Book {
    fn default() -> Self {
        let mut book = Self {
            root: BookNode::default(),
        };
        populate_opening_book(&mut book);
        book
    }
}

impl Book {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_line(&mut self, line: &str) {
        let moves = line.split(' ');
        let mut curr_node = &mut self.root;

        for raw_move in moves {
            let raw_from_square: String = raw_move.to_string().chars().take(2).collect();
            let raw_to_square: String = raw_move.to_string().chars().skip(2).take(2).collect();
            let from_square = square::from_algebraic(&raw_from_square);
            let to_square = square::from_algebraic(&raw_to_square);
            let book_move = book_move(from_square, to_square);

            let next_node = curr_node
                .lines
                .entry(book_move)
                .or_insert(Box::new(BookNode::new()));
            curr_node = next_node;
        }
    }

    pub fn get_next_moves(&self, line: Vec<BookMove>) -> Vec<BookMove> {
        let mut curr_node = &self.root;

        for book_move in line {
            let next = curr_node.lines.get(&book_move);
            if next.is_none() {
                return vec![];
            }

            curr_node = next.unwrap();
        }

        curr_node.lines.keys().copied().collect()
    }
}

pub fn book_move(from_square: Bitboard, to_square: Bitboard) -> BookMove {
    (from_square, to_square)
}

// TODO(codyjk): Maybe move this to precompile somehow?

fn populate_opening_book(book: &mut Book) {
    // Four knights

    book.add_line("e2e4 e7e5 b1c3 g8f6 g1f3 b8c6 f1b5 f8b4 e1g1 e8g8 d2d3 d7d6 c1g5 b4c3 b2c3 d8e7 f1e1 c6d8 d3d4 d8e6");

    // Italian

    book.add_line(
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d4 e5d4 c3d4 c5b4 c1d2 b4d2 b1d2 d7d5",
    );
    book.add_line(
        "e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 c2c3 g8f6 d2d3 d7d6 b2b4 c5b6 a2a4 a7a5 b4b5 c6e7",
    );
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1c4 f8c5 d2d3 g8f6 c2c3");

    // Two knights

    book.add_line("e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 f3g5 d7d5 e4d5 c6a5 c4b5 c7c6 d5c6 b7c6 b5e2 h7h6 g5f3 e5e4 f3e5 f8d6 f2f4 e4f3 e5f3 e8g8 d2d4 c6c5");
    book.add_line(
        "e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d4 e5d4 e1g1 f6e4 f1e1 d7d5 c4d5 d8d5 b1c3 d5h5 c3e4 c8e6",
    );
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1c4 g8f6 d2d3 f8e7 e1g1 e8g8 c2c3 d7d6 c4b3");

    // Scotch

    book.add_line("e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 g8f6 d4c6 b7c6 e4e5 d8e7 d1e2 f6d5 c2c4 c8a6 g2g3 g7g6 b2b3 f8g7 c1b2 e8g8 f1g2 a8e8 e1g1");
    book.add_line("e2e4 e7e5 g1f3 b8c6 d2d4 e5d4 f3d4 f8c5 c1e3 d8f6 c2c3 g8e7 g2g3 e8g8 f1g2");

    // Ruy Lopez

    book.add_line("e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8 h2h3 c6a5 b3c2 c7c5 d2d4 d8c7");
    book.add_line(
        "e2e4 e7e5 g1f3 b8c6 f1b5 f8c5 c2c3 g8f6 e1g1 e8g8 d2d4 c5b6 f1e1 d7d6 h2h3 c6e7 b1d2",
    );
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f8c5 f3e5 c6e5 d2d4 c7c6 d4e5 f6e4 b5d3 d7d5 e5d6 e4f6 f1e1");
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1b5 g8f6 e1g1 f6e4 d2d4 f8e7 d1e2 e4d6 b5c6 b7c6 d4e5 d6b7 b1c3 e8g8 f3d4");
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f6e4 d2d4 b7b5 a4b3 d7d5 d4e5 c8e6 c2c3 e4c5 b3c2 e6g4");
    book.add_line("e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5a4 g8f6 e1g1 f8e7 f1e1 b7b5 a4b3 d7d6 c2c3 e8g8 h2h3 c8b7 d2d4 f8e8 b1d2 e7f8 a2a3 h7h6 b3c2 c6b8 b2b4 b8d7 c1b2 g7g6");

    // Petroff

    book.add_line(
        "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4 d6d5 f1d3 b8c6 e1g1 c8g4 c2c4 e4f6",
    );
    book.add_line(
        "e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 d2d4 d6d5 f1d3 b8c6 e1g1 f8e7 c2c4 c6b4",
    );
    book.add_line("e2e4 e7e5 g1f3 g8f6 f3e5 d7d6 e5f3 f6e4 b1c3 e4c3 d2c3");
    book.add_line("e2e4 e7e5 g1f3 g8f6 d2d4 f6e4 f1d3 d7d5 f3e5 b8d7 e5d7 c8d7 e1g1 f8d6 c2c4 c7c6 b1c3 e4c3 b2c3");

    // Sicilian

    book.add_line("e2e4 c7c5 c2c3 d7d5 e4d5 d8d5 d2d4 e7e6");
    book.add_line(
        "e2e4 c7c5 c2c3 g8f6 e4e5 f6d5 d2d4 c5d4 g1f3 e7e6 c3d4 b7b6 b1c3 d5c3 b2c3 d8c7",
    );
    book.add_line(
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 f1e2 e7e5 d4b3 f8e7 e1g1 e8g8 a2a4 b7b6",
    );
    book.add_line("e2e4 c7c5 b1c3 b8c6 g2g3 g7g6 f1g2 f8g7 d2d3 e7e6 c1e3 d7d6 g1e2 c6d4 d1d2");
    book.add_line(
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 c1g5 e7e6 d1d2 f8e7 e1c1 e8g8",
    );
    book.add_line(
        "e2e4 c7c5 g1f3 d7d6 d2d4 c5d4 f3d4 g8f6 b1c3 a7a6 g2g3 e7e5 d4e2 b7b5 f1g2 c8b7 e1g1 b8d7",
    );
    book.add_line("e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g7g6 b1c3 f8g7 c1e3 g8f6 f1c4 e8g8");
    book.add_line(
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g8f6 b1c3 d7d6 f1e2 e7e5 d4b3 f8e7 e1g1 e8g8 c1e3 c8e6",
    );
    book.add_line(
        "e2e4 c7c5 g1f3 b8c6 d2d4 c5d4 f3d4 g8f6 b1c3 d7d6 f1e2 g7g6 c1e3 f8g7 e1g1 e8g8 d4b3 c8e6",
    );

    // French

    book.add_line("e2e4 e7e6 d2d4 d7d5 e4e5 c7c5 c2c3 b8c6 g1f3 d8b6 a2a3 c5c4");
    book.add_line("e2e4 e7e6 d2d4 d7d5 b1c3 g8f6 c1g5 f8e7 e4e5 f6d7 g5e7 d8e7 f2f4 e8g8 d1d2 c7c5 g1f3 b8c6 e1c1 c5c4");
    book.add_line("e2e4 e7e6 d2d4 d7d5 b1c3 d5e4 c3e4 b8d7 g1f3 g8f6 e4f6 d7f6 f1d3 b7b6 d1e2 c8b7? c1g5 f8e7");
    book.add_line(
        "e2e4 e7e6 d2d4 d7d5 b1c3 f8b4 e4e5 g8e7 a2a3 b4c3 b2c3 c7c5 g1f3 b8c6 a3a4 d8a5 d1d2 c8d7",
    );
    book.add_line(
        "e2e4 e7e6 d2d4 d7d5 b1c3 f8b4 e4e5 g8e7 a2a3 b4c3 b2c3 c7c5 a3a4 b8c6 g1f3 d8a5 c1d2 c8d7",
    );
    book.add_line("e2e4 e7e6 d2d4 d7d5 b1c3 f8b4 e4e5 c7c5 a2a3 b4c3 b2c3 g8e7 d1g4 d8c7 g4g7 h8g8 g7h7 c5d4 g1e2 b8c6 f2f4 c8d7");
    book.add_line(
        "e2e4 e7e6 d2d4 d7d5 b1d2 c7c5 e4d5 e6d5 g1f3 b8c6 f1b5 f8d6 d4c5 d6c5 e1g1 g8e7",
    );
    book.add_line(
        "e2e4 e7e6 d2d4 d7d5 b1d2 c7c5 g1f3 g8f6 e4d5 e6d5 f1b5 c8d7 b5d7 b8d7 e1g1 f8e7",
    );
    book.add_line("e2e4 e7e6 d2d4 d7d5 b1d2 g8f6 e4e5 f6d7 f1d3 c7c5 c2c3 b8c6 g1e2 c5d4 c3d4 f7f6 e5f6 d7f6 e1g1 f8d6");

    // Caro-Kann

    book.add_line("e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 b8d7 g1f3 g8f6 e4f6 d7f6 f3e5");
    book.add_line("e2e4 c7c6 b1c3 d7d5 d2d4 d5e4 c3e4 b8d7 f1c4 g8f6 e4g5 e7e6 d1e2 d7b6");
    book.add_line("e2e4 c7c6 b1c3 d7d5 d2d4 d5e4 c3e4 b8d7 e4g5 g8f6 f1d3 e7e6");
    book.add_line("e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6 g1f3 g8f6-f3e5 g6h7 f1d3 b8d7 d3h7");
    book.add_line("e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6 g1f3 b8d7 h4h5 g6h7 f1d3 h7d3 d1d3 g8f6 c1d2 e7e6 e1c1");
    book.add_line("e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 h2h4 h7h6 g1f3 b8d7 h4h5 g6h7 f1d3 h7d3 d1d3 e7e6 c1d2 g8f6 e1c1");
    book.add_line("e2e4 c7c6 d2d4 d7d5 b1c3 d5e4 c3e4 c8f5 e4g3 f5g6 g1f3 b8d7 h2h4 h7h6 f1d3 g6d3 d1d3 e7e6 c1d2 g8f6 e1c1");
    book.add_line("e2e4 c7c6 d2d4 d7d5 b1d2 d5e4 d2e4");
    book.add_line("e2e4 c7c6 d2d4 d7d5 e4d5 c6d5 c2c4 g8f6 b1c3 e7e6 g1f3");
    book.add_line(
        "e2e4 c7c6 d2d4 d7d5 e4d5 c6d5 c2c4 g8f6 b1c3 b8c6 c1g5 e7e6 c4c5 f8e7 f1b5 e8g8 g1f3 f6e4",
    );
    book.add_line("e2e4 c7c6 d2d4 d7d5 e4e5 c8f5 f1d3 f5d3 d1d3 e7e6 b1c3 d8b6");
    book.add_line(
        "e2e4 c7c6 b1c3 d7d5 g1f3 c8g4 h2h3 g4f3 d1f3 e7e6 d2d4 g8f6 f1d3 d5e4 c3e4 d8d4 c2c3 d4d8",
    );

    // Pirc and modern

    book.add_line(
        "d2d4 d7d6 e2e4 g8f6 b1c3 g7g6 f1c4 c7c6 d1e2 f8g7 g1f3 e8g8 c1g5 b7b5 c4d3 d8c7",
    );
    book.add_line("e2e4 d7d6 d2d4 g8f6 b1c3 g7g6 c1g5 f8g7 d1d2 b8d7 e1c1 e7e5 d4e5 d6e5 g1f3 h7h6 g5h4 g6g5 h4g3 d8e7");

    // QGA

    book.add_line(
        "d2d4 d7d5 c2c4 d5c4 g1f3 g8f6 e2e3 e7e6 f1c4 c7c5 e1g1 a7a6 d1e2 b7b5 c4d3 c5d4 e3d4 b8c6",
    );

    // QGD

    book.add_line("d2d4 d7d5 c2c4 e7e6 b1c3 f8e7 g1f3 g8f6 c4d5 e6d5");
    book.add_line(
        "c2c4 e7e6 d2d4 d7d5 b1c3 c7c5 c4d5 e6d5 g1f3 b8c6 g2g3 g8f6 f1g2 f8e7 e1g1 e8g8",
    );
    book.add_line("c2c4 e7e6 b1c3 d7d5 d2d4 g8f6 c1g5 f8e7 e2e3 e8g8 g1f3 b8d7 a1c1 c7c6");
    book.add_line(
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 b8d7 c4d5 e6d5 e2e3 c7c6 f1d3 f8e7 d1c2 e8g8 g1e2 f8e8",
    );
    book.add_line(
        "d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 c1g5 b8d7 e2e3 c7c6 g1f3 d8a5 f3d2 f8b4 d1c2 e8g8 g5h4 c6c5",
    );
    book.add_line("d2d4 d7d5 c2c4 e7e6 b1c3 g8f6 g1f3 f8e7 c4d5 e6d5 c1g5 e8g8");

    // Slav

    book.add_line("d2d4 d7d5 c2c4 c7c6 b1c3 g8f6 g1f3 d5c4 a2a4 c8f5 f3e5 e7e6 f2f3 f8b4 c1g5 h7h6 g5f6 d8f6 e2e4 f5h7");
    book.add_line("d2d4 d7d5 c2c4 c7c6 g1f3 g8f6 b1c3 e7e6 e2e3 b8d7 f1d3 f8d6");

    // Catalan

    book.add_line(
        "d2d4 e7e6 c2c4 d7d5 g2g3 g8f6 g1f3 f8e7 f1g2 e8g8 e1g1 f6d7 d1c2 c7c6 b1d2 b7b6 e2e4 c8b7",
    );

    // Nimzo-Indian

    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 d1c2 c7c5 d4c5 e8g8 a2a3 b4c5 g1f3 b7b6");
    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 d1c2 e8g8 a2a3 b4c3 c2c3 b7b6 c1g5 c8b7");
    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 g1f3 b7b6 g2g3 c8b7 f1g2");
    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 a2a3 b4c3 b2c3 e8g8 f2f3 d7d5 c4d5 e6d5 e2e3 c8f5 g1e2 b8d7 e2g3 f5g6");
    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 c1d2 e8g8 e2e3 d7d5 g1f3 c7c5 a2a3 b4c3 d2c3 f6e4 a1c1 e4c3 c1c3 c5d4");
    book.add_line("d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 e2e3 e8g8 f1d3 d7d5 g1f3 c7c5 e1g1 b8c6 a2a3 b4c3 b2c3 d5c4 d3c4 d8c7");
    book.add_line(
        "d2d4 g8f6 c2c4 e7e6 b1c3 f8b4 d1c2 d7d5 a2a3 b4c3 c2c3 b8c6 g1f3 f6e4 c3b3 c6a5 b3a4 c7c6",
    );

    // Queen's Indian

    book.add_line(
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 g2g3 c8b7 f1g2 f8e7 e1g1 e8g8 b1c3 f6e4 d1c2 e4c3 c2c3",
    );
    book.add_line(
        "d2d4 g8f6 c2c4 e7e6 g1f3 b7b6 e2e3 c8b7 f1d3 f8e7 b1c3 d7d5 e1g1 e8g8 d1e2 b8d7",
    );

    // King's Indian

    book.add_line(
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 f2f3 e8g8 c1e3 e7e5 d4d5 f6h5 d1d2 f7f5 e1c1 b8d7",
    );
    book.add_line("d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5 d4d5 a7a5");
    book.add_line(
        "d2d4 g8f6 c2c4 g7g6 g2g3 f8g7 f1g2 e8g8 b1c3 d7d6 g1f3 b8d7 e1g1 e7e5 e2e4 c7c6 h2h3 d8b6",
    );
    book.add_line(
        "d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 f2f4 c7c5 g1f3 e8g8 d4d5 e7e6 f1d3 e6d5 c4d5 d8b6",
    );
    book.add_line("d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 e2e4 d7d6 g1f3 e8g8 f1e2 e7e5 e1g1 b8c6 d4d5 c6e7 f3e1 f6e8 f2f3 f7f5");
    book.add_line("d2d4 g8f6 c2c4 g7g6 b1c3 f8g7 g1f3 e8g8 c1f4 d7d6 h2h3 b8d7 e2e3 c7c6");

    // Grunfeld

    book.add_line("d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c1f4 f8g7");
    book.add_line("d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 c4d5 f6d5 e2e4 d5c3 b2c3 c7c5 f1c4 f8g7 g1e2 e8g8 e1g1 c5d4 c3d4 b8c6");
    book.add_line(
        "d2d4 g8f6 c2c4 g7g6 b1c3 d7d5 g1f3 f8g7 d1b3 d5c4 b3c4 e8g8 e2e4 c8g4 c1e3 f6d7 e1c1 b8c6",
    );

    // Benoni

    book.add_line("d2d4 g8f6 c2c4 c7c5 d4d5 e7e6 b1c3 e6d5 c4d5 d7d6 e2e4 g7g6 f1d3 f8g7 g1e2 e8g8 e1g1 a7a6 a2a4 d8c7");

    // Dutch

    book.add_line("d2d4 f7f5 g2g3 e7e6 f1g2 g8f6 g1f3 f8e7 e1g1 e8g8 c2c4 d7d6 b1c3 d8e8 d1c2 e8h5 b2b3 b8c6 c1a3 a7a5");

    // Queen's Pawn

    book.add_line("d2d4 d7d5 g1f3 g8f6 c1f4 c7c5 e2e3 b8c6 c2c3 d8b6 d1c1 c8f5 d4c5 b6c5 b1d2 a8c8 f3d4 c6d4 e3d4 c5b6");

    // English

    book.add_line(
        "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 e2e4 f8b4 d2d3 d7d6 f1e2 e8g8 e1g1 b4c3 b2c3 d8e7",
    );
    book.add_line(
        "c2c4 e7e5 b1c3 g8f6 g1f3 b8c6 g2g3 d7d5 c4d5 f6d5 f1g2 d5b6 e1g1 f8e7 d2d3 e8g8 c1e3 f7f5",
    );
    book.add_line("c2c4 g8f6 b1c3 d7d5 c4d5 f6d5 e2e4 d5f4 f1c4 c8e6 c4e6 f7e6");
    book.add_line("c2c4 g8f6 b1c3 e7e5 g1f3 b8c6 g2g3 f8c5 f1g2 d7d6 e1g1 e8g8 d2d3 h7h6");
    book.add_line(
        "c2c4 g8f6 b1c3 e7e5 g1f3 b8c6 g2g3 f8b4 f1g2 e8g8 e1g1 e5e4 f3e1 b4c3 d2c3 h7h6 e1c2 b7b6",
    );

    book.add_line("c2c4 c7c5 g1f3 b8c6 b1c3 g8f6 g2g3 g7g6 f1g2 f8g7 e1g1 e8g8 d2d4 c5d4 f3d4 c6d4 d1d4 d7d6 d4d3");

    // Reti
    book.add_line("g1f3 d7d5 g2g3 g8f6 f1g2 g7g6 e1g1 f8g7 d2d3 e8g8 b1d2 b8c6 e2e4 e7e5 c2c3 a7a5 f1e1 d5e4 d3e4");
    book.add_line("g1f3 d7d5 c2c4 e7e6 g2g3 g8f6 f1g2 f8e7 e1g1 e8g8 b2b3 c7c5 c4d5 f6d5 c1b2 b8c6 d2d4 b7b6 b1c3 d5c3");
    book.add_line(
        "g1f3 d7d5 c2c4 d5c4 e2e3 c7c5 f1c4 e7e6 e1g1 g8f6 b2b3 b8c6 c1b2 a7a6 a2a4 f8e7",
    );
}
