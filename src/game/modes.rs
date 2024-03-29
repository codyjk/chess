use super::command::{Command, MakeWaterfallMove};
use super::{Game, GameEnding};
use crate::board::color::Color;
use crate::input_handler;
use std::time::SystemTime;
use std::{thread, time};
use termion::clear;

pub fn play_computer(depth: u8, player_color: Color) {
    let game = &mut Game::new(depth);

    println!("{}", clear::All);
    println!("you are {}", player_color);
    loop {
        println!("{}", game.board);

        match game.check_game_over_for_current_turn() {
            Some(GameEnding::Checkmate) => {
                println!("checkmate!");
                break;
            }
            Some(GameEnding::Stalemate) => {
                println!("stalemate!");
                break;
            }
            _ => (),
        };

        let command: Box<dyn Command> = if player_color == game.board.turn() {
            match input_handler::parse_command() {
                Ok(command) => command,
                Err(msg) => {
                    println!("{}", msg);
                    continue;
                }
            }
        } else {
            Box::new(MakeWaterfallMove {})
        };

        let start_time = SystemTime::now();
        match command.execute(game) {
            Ok(chessmove) => {
                let duration = SystemTime::now().duration_since(start_time).unwrap();
                println!("{}", clear::All);
                let player = match player_color {
                    c if c == game.board.turn() => "you",
                    _ => "computer",
                };
                game.board.next_turn();
                let score = game.score(game.board.turn());
                println!(
                    "{} chose {} (depth={}, took={}ms, halfmove_clock={}, fullmove_clock={}, score={})",
                    player,
                    chessmove,
                    depth,
                    duration.as_millis(),
                    game.board.halfmove_clock(),
                    game.board.fullmove_clock(),
                    score,
                );

                if game.board.turn() == player_color {
                    println!(
                        "(positions_searched={}, cache_hits={}, alpha_beta_terminations={})",
                        game.last_searched_position_count(),
                        game.last_cache_hit_count(),
                        game.last_alpha_beta_termination_count(),
                    );
                }
                continue;
            }
            Err(error) => println!("error: {}", error),
        }
    }
}

pub fn computer_vs_computer(move_limit: u8, sleep_between_turns_in_ms: u64, depth: u8) {
    let game = &mut Game::new(depth);

    println!("{}", clear::All);

    loop {
        println!("{}", game.board);
        thread::sleep(time::Duration::from_millis(sleep_between_turns_in_ms));

        match game.check_game_over_for_current_turn() {
            Some(GameEnding::Checkmate) => {
                println!("checkmate!");
                break;
            }
            Some(GameEnding::Stalemate) => {
                println!("stalemate!");
                break;
            }
            Some(GameEnding::Draw) => {
                println!("draw!");
                break;
            }
            _ => (),
        };

        if move_limit > 0 && game.fullmove_clock() > move_limit {
            break;
        }

        let start_time = SystemTime::now();
        match game.make_waterfall_book_then_alpha_beta_move() {
            Ok(chessmove) => {
                let duration = SystemTime::now().duration_since(start_time).unwrap();
                println!("{}", clear::All);
                game.board.next_turn();
                let score = game.score(game.board.turn());
                println!(
                    "{} chose {} (depth={}, took={}ms, halfmove_clock={}, fullmove_clock={}, score={})",
                    game.board.turn(),
                    chessmove,
                    depth,
                    duration.as_millis(),
                    game.board.halfmove_clock(),
                    game.board.fullmove_clock(),
                    score,
                );

                println!(
                    "(positions_searched={}, cache_hits={}, alpha_beta_terminations={})",
                    game.last_searched_position_count(),
                    game.last_cache_hit_count(),
                    game.last_alpha_beta_termination_count(),
                );
                continue;
            }
            Err(error) => {
                println!("error: {}", error);
                break;
            }
        }
    }
}

pub fn player_vs_player() {
    let game = &mut Game::new(0);
    loop {
        println!("turn: {}", game.board.turn());
        println!("{}", game.board);

        match game.check_game_over_for_current_turn() {
            Some(GameEnding::Checkmate) => {
                println!("checkmate!");
                break;
            }
            Some(GameEnding::Stalemate) => {
                println!("stalemate!");
                break;
            }
            Some(GameEnding::Draw) => {
                println!("draw!");
                break;
            }
            _ => (),
        };

        let command = match input_handler::parse_command() {
            Ok(command) => command,
            Err(msg) => {
                println!("{}", msg);
                continue;
            }
        };

        match command.execute(game) {
            Ok(_chessmove) => {
                game.board.next_turn();
                continue;
            }
            Err(error) => println!("error: {}", error),
        }
    }
}
