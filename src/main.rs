mod cache;
mod climbing;
mod common;
mod geometry;
mod greedy;
mod hungarian;
mod placement;
mod score;
mod visualize;
use crate::climbing::*;
use crate::common::*;
use crate::greedy::*;
use crate::hungarian::*;
use crate::score::*;
use crate::visualize::*;
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
    Climb {
        input: PathBuf,
        output: PathBuf,
    },
    Optimize {
        problem: PathBuf,
        solution: PathBuf,
        output: PathBuf,
    },
    Visualize {
        problem: PathBuf,
        solution: PathBuf,
        output: PathBuf,
    },
    Score {
        problem: PathBuf,
        solution: PathBuf,
        quiet: Option<bool>,
    },
    Submit {
        id: u32,
        solution: PathBuf,
        token: String,
    },
    Download {
        id_from: u32,
        id_to: u32,
        output: PathBuf,
    },
    ProbStats {
        problem: PathBuf,
    },
    UserName {
        new_name: String,
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
        Commands::Climb { input, output } => {
            let prob = Problem::load_from_file(input)?;
            let sol = solve_climbing(&prob)?;
            sol.save_to_file(output)?;
        }
        Commands::Optimize {
            problem,
            solution,
            output,
        } => {
            let prob = Problem::load_from_file(problem)?;
            let sol: Solution = Solution::load_from_file(solution)?;
            let opt_sol = optimize_hungarian(&prob, &sol)?;
            opt_sol.save_to_file(output)?;
        }
        Commands::Visualize {
            problem,
            solution,
            output,
        } => {
            let prob = Problem::load_from_file(problem)?;
            let sol = Solution::load_from_file(solution)?;
            visualize(&prob, &sol, output)?;
        }
        Commands::Submit {
            id,
            solution,
            token,
        } => {
            let sol = Solution::load_from_file(solution)?;
            sol.submit(*id, token)?;
        }
        Commands::Score {
            problem,
            solution,
            quiet,
        } => {
            let prob = Problem::load_from_file(problem)?;
            let sol = Solution::load_from_file(solution)?;
            let s = score(&prob, &sol, quiet.unwrap_or(false))?;
            println!("score: {}", s);
        }
        Commands::Download {
            id_from,
            id_to,
            output,
        } => {
            download_problems(*id_from, *id_to, output);
        }
        Commands::ProbStats { problem } => {
            let prob = Problem::load_from_file(problem)?;
            let left = prob.stage_bottom_left[0];
            let bottom = prob.stage_bottom_left[1];
            eprintln!(
                "(W, H)=({}, {}), (l, b, w, h)=({}, {}, {}, {}), N={}, M={}, K={}",
                prob.room_width,
                prob.room_height,
                left,
                bottom,
                prob.stage_width,
                prob.stage_height,
                prob.attendees.len(),
                prob.musicians.len(),
                prob.attendees.first().unwrap().tastes.len()
            );
        }
        Commands::UserName { new_name, token } => {
            change_user_name(new_name, token)?;
        }
    }
    Ok(())
}
