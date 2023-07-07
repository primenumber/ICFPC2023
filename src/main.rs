mod common;
mod greedy;
use crate::common::*;
use crate::greedy::*;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Solve {
        input: PathBuf,
        output: PathBuf,
    },
    Submit {
        id: u32,
        solution: PathBuf,
        token: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Solve { input, output } => {
            let prob = Problem::load_from_file(input)?;
            let sol = solve_greedy(&prob)?;
            sol.save_to_file(output)?;
        }
        Commands::Submit {
            id,
            solution,
            token,
        } => {
            let sol = Solution::load_from_file(solution)?;
            sol.submit(*id, token)?;
        }
    }
    Ok(())
}
