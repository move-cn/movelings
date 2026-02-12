mod cli;
mod handlers;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use handlers::*;
use crate::utils::find_exercise_by_prefix;
 
fn main() {
    let cli = Cli::parse();

    let command_handler = |name: &str, action: fn(&str) -> bool| {
        match find_exercise_by_prefix(name) {
            Ok(exercise_name) => {
                action(&exercise_name);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    let hint_handler = |name: &str| {
        match find_exercise_by_prefix(name) {
            Ok(exercise_name) => {
                show_hint(&exercise_name);
                true
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    };

    match cli.command {
        Some(Commands::List) => list_exercises(),
        Some(Commands::Run { name }) => command_handler(&name, check_exercise),
        Some(Commands::Hint { name }) => {
            hint_handler(&name);
        },
        Some(Commands::Reset) => reset_progress(),
        Some(Commands::Progress) => show_progress(),
        Some(Commands::Watch) => watch_mode(),
        Some(Commands::Exercise(args)) => {
            if let Some(exercise_name) = args.first() {
                command_handler(exercise_name, check_exercise);
            } else {
                eprintln!("Error: No exercise name provided");
                std::process::exit(1);
            }
        },
        None => show_menu(),
    }
}