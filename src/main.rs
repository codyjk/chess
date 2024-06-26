use chess::board::color::Color;
use chess::board::Board;
use chess::game::modes::{computer_vs_computer, play_computer, player_vs_player};
use chess::move_generator::MoveGenerator;
use log::debug;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "chess", about = "chess engine cli")]
enum Chess {
    CountPositions {
        #[structopt(short, long, default_value = "4")]
        depth: u8,
    },
    Play {
        #[structopt(short, long, default_value = "4")]
        depth: u8,
        #[structopt(short = "c", long = "color", default_value = "random")]
        color: Color,
    },
    Pvp,
    Watch {
        #[structopt(short, long, default_value = "4")]
        depth: u8,
    },
}

fn main() {
    env_logger::init();

    let args = Chess::from_args();

    match args {
        Chess::CountPositions { depth } => run_count_positions(depth),
        Chess::Play { depth, color } => play_computer(depth, color),
        Chess::Watch { depth } => computer_vs_computer(0, 1000, depth),
        Chess::Pvp => player_vs_player(),
    }
}

fn run_count_positions(depth: u8) {
    let depths = 0..=depth;
    let mut move_generator = MoveGenerator::new();

    for depth in depths {
        let mut board = Board::starting_position();
        let count = move_generator.count_positions(depth, &mut board, Color::White);

        debug!("depth: {}, positions: {}", depth, count);
    }
}
