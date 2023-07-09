use crate::cache::*;
use crate::common::*;
use crate::geometry::*;
use crate::score::*;
use anyhow::Result;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveGreedyError {
    #[error("Lack of candidates")]
    LackCandidatesError,
}

fn generate_candidates(prob: &Problem, diag: bool) -> Result<Vec<Point>> {
    let musician_left = prob.stage_from().x + 10.;
    let musician_bottom = prob.stage_from().y + 10.;
    let width_without_pad = prob.stage_width - 20.;
    let height_without_pad = prob.stage_height - 20.;
    let min_distance = if diag { 7.0711 } else { 10. };
    let musicians_in_row = ((prob.stage_width - 20.) / min_distance).floor() as usize + 1;
    let musicians_in_col = ((prob.stage_height - 20.) / min_distance).floor() as usize + 1;
    let mut placement_candidates = Vec::new();
    for row in 0..musicians_in_col {
        for col in 0..musicians_in_row {
            if diag && (row + col) % 2 == 1 {
                continue;
            }
            let x = musician_left
                + if col > 0 {
                    col as f64 * width_without_pad / (musicians_in_row - 1) as f64
                } else {
                    0.0
                };
            let y = musician_bottom
                + if row > 0 {
                    row as f64 * height_without_pad / (musicians_in_col - 1) as f64
                } else {
                    0.0
                };
            placement_candidates.push(Point { x, y });
        }
    }
    if placement_candidates.len() < prob.musicians.len() {
        return Err(SolveGreedyError::LackCandidatesError.into());
    }
    Ok(placement_candidates)
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let diag_mode = false;

    let placement_candidates = generate_candidates(prob, diag_mode)?;

    let mut cache = DiffCache::new(prob, &placement_candidates);

    // place musicians greedy
    let mut musicians: HashMap<_, _> = prob.musicians.iter().enumerate().collect();
    let mut pairs = Vec::new();
    while !musicians.is_empty() {
        let (i, j, _d) = cache.find_best_matching();
        cache.add_matching(prob, i, j);
        let new_place = placement_candidates[i];
        musicians.remove(&j);
        pairs.push((j, new_place));
    }

    // construct Solution
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs.into_iter().map(|(_, place)| place).collect();
    Ok(Solution { placements })
}
