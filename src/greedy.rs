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
    #[error("Failed to generate any valid solutions")]
    FailedToGenerateSolution,
}

#[derive(Clone, Copy)]
enum PlacementMode {
    GridNormal,
    GridDiag,
}

fn generate_candidate_grid_normal(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    let min_distance = 10.0;
    let musicians_in_row = (size.x / min_distance).floor() as usize + 1;
    let musicians_in_col = (size.y / min_distance).floor() as usize + 1;
    for row in 0..musicians_in_col {
        for col in 0..musicians_in_row {
            let x = bottom_left.x
                + if col > 0 {
                    col as f64 * size.x / (musicians_in_row - 1) as f64
                } else {
                    0.0
                };
            let y = bottom_left.y
                + if row > 0 {
                    row as f64 * size.y / (musicians_in_col - 1) as f64
                } else {
                    0.0
                };
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidate_grid_diag(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    let min_distance = 7.0711;
    let musicians_in_row = (size.x / min_distance).floor() as usize + 1;
    let musicians_in_col = (size.y / min_distance).floor() as usize + 1;
    for row in 0..musicians_in_col {
        for col in 0..musicians_in_row {
            if (row + col) % 2 == 1 {
                continue;
            }
            let x = bottom_left.x
                + if col > 0 {
                    col as f64 * size.x / (musicians_in_row - 1) as f64
                } else {
                    0.0
                };
            let y = bottom_left.y
                + if row > 0 {
                    row as f64 * size.y / (musicians_in_col - 1) as f64
                } else {
                    0.0
                };
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidates(prob: &Problem, mode: PlacementMode) -> Result<Vec<Point>> {
    let padding = 10.0;
    let bottom_left = prob.stage_from()
        + Point {
            x: padding,
            y: padding,
        };
    let size = prob.stage_size()
        - Point {
            x: 2.0 * padding,
            y: 2.0 * padding,
        };
    let placement_candidates = match mode {
        PlacementMode::GridNormal => generate_candidate_grid_normal(bottom_left, size)?,
        PlacementMode::GridDiag => generate_candidate_grid_diag(bottom_left, size)?,
    };
    if placement_candidates.len() < prob.musicians.len() {
        return Err(SolveGreedyError::LackCandidatesError.into());
    }
    Ok(placement_candidates)
}

fn solve_greedy_play_together(
    prob: &Problem,
    mut volumes: Vec<f64>,
    placement_candidates: &[Point],
    mut musician_to_place: Vec<Option<usize>>,
    mut place_to_musician: Vec<Option<usize>>,
) -> Result<Solution> {
    let k = prob.attendees.first().unwrap().tastes.len();
    loop {
        let placements: Vec<_> = musician_to_place
            .iter()
            .map(|opt_pidx| match opt_pidx {
                Some(pidx) => placement_candidates[*pidx],
                None => Point { x: -10., y: -10. },
            })
            .collect();
        if musician_to_place.iter().all(|e| e.is_some()) {
            return Ok(Solution {
                placements,
                volumes,
            });
        }
        let current_impact: Vec<_> = musician_to_place
            .iter()
            .enumerate()
            .map(|(midx, _)| {
                prob.attendees
                    .iter()
                    .map(|attendee| {
                        impact(attendee, &prob.musicians, &placements, midx, &prob.pillars)
                    })
                    .sum::<i64>()
            })
            .collect();
        let mut scalar_gain = vec![vec![0.; k]; placement_candidates.len()];
        for (i, &place) in placement_candidates.iter().enumerate() {
            if place_to_musician[i].is_some() {
                continue;
            }
            for (j, pidx) in musician_to_place.iter().enumerate() {
                if pidx.is_none() {
                    continue;
                }
                let place_another = placement_candidates[pidx.unwrap()];
                scalar_gain[i][prob.musicians[j] as usize] +=
                    current_impact[j] as f64 / (place - place_another).length();
            }
        }
        let (i, j, _gain) = scalar_gain
            .iter()
            .enumerate()
            .filter(|(i, _)| place_to_musician[*i].is_none())
            .map(|(i, per_candi)| {
                let (j, gain) = prob
                    .musicians
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| musician_to_place[*j].is_none())
                    .map(|(j, kind)| (j, per_candi[*kind as usize].ceil() as i64))
                    .max_by_key(|(_, gain)| *gain)
                    .unwrap();
                (i, j, gain)
            })
            .max_by_key(|(_, _, gain)| *gain)
            .unwrap();
        place_to_musician[i] = Some(j);
        musician_to_place[j] = Some(i);
        volumes[j] = 0.0;
    }
}

fn solve_greedy_impl(
    prob: &Problem,
    placement_mode: PlacementMode,
    together_mode: bool,
) -> Result<Solution> {
    let placement_candidates = generate_candidates(prob, placement_mode)?;

    let mut cache = DiffCache::new(prob, &placement_candidates);

    // place musicians greedy
    let mut musicians: HashMap<_, _> = prob.musicians.clone().into_iter().enumerate().collect();
    let mut pairs = Vec::new();
    let mut volumes = vec![10.0; musicians.len()];
    while !musicians.is_empty() {
        let (i, j, d, v) = cache.find_best_matching();
        if together_mode && d == 0 {
            return solve_greedy_play_together(
                prob,
                volumes,
                &placement_candidates,
                cache.musician_to_place,
                cache.place_to_musician,
            );
        }
        volumes[j] = v;
        cache.add_matching(prob, i, j, &volumes);
        let new_place = placement_candidates[i];
        musicians.remove(&j);
        pairs.push((j, new_place));
    }

    // construct Solution
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs.into_iter().map(|(_, place)| place).collect();
    Ok(Solution {
        placements,
        volumes,
    })
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let placement_modes = [PlacementMode::GridNormal, PlacementMode::GridDiag];
    let mut sols = Vec::new();
    for pmode in placement_modes {
        if let Ok(sol) = solve_greedy_impl(prob, pmode, false) {
            sols.push((score(prob, &sol, true)?, sol));
        }
        if is_full_division_scoring(prob) {
            if let Ok(sol) = solve_greedy_impl(prob, pmode, true) {
                sols.push((score(prob, &sol, true)?, sol));
            }
        }
    }
    let (_, sol) = sols
        .into_iter()
        .max_by_key(|(s, _)| *s)
        .ok_or(SolveGreedyError::FailedToGenerateSolution)?;
    Ok(sol)
}
