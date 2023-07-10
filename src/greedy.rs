use crate::cache::*;
use crate::common::*;
use crate::geometry::*;
use crate::hungarian::*;
use crate::score::*;
use anyhow::Result;
use rayon::prelude::*;
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
enum InterpolateMode {
    Strech,
    Corner(f64),
}

#[derive(Clone, Copy)]
enum PlacementMode {
    GridNormal(InterpolateMode),
    GridDiag,
    GridCompress,
}

fn interpolate(length: f64, total: usize, index: usize, mode: InterpolateMode) -> f64 {
    match mode {
        InterpolateMode::Strech => {
            if index == 0 {
                0.0
            } else {
                length * index as f64 / (total as f64 - 1.0)
            }
        }
        InterpolateMode::Corner(gap) => {
            if index * 2 < total {
                gap * index as f64
            } else {
                let rem = total - index - 1;
                length - 10. * rem as f64
            }
        }
    }
}

fn generate_candidate_grid_normal(
    bottom_left: Point,
    size: Point,
    mode: InterpolateMode,
) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    let min_distance = 10.0;
    let cols = (size.x / min_distance).floor() as usize + 1;
    let rows = (size.y / min_distance).floor() as usize + 1;
    for row in 0..rows {
        for col in 0..cols {
            let x = bottom_left.x + interpolate(size.x, cols, col, mode);
            let y = bottom_left.y + interpolate(size.y, rows, row, mode);
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidate_checker(
    bottom_left: Point,
    size: Point,
    rows: usize,
    cols: usize,
) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 1 {
                continue;
            }
            let x = bottom_left.x
                + if cols > 0 {
                    col as f64 * size.x / (cols - 1) as f64
                } else {
                    0.0
                };
            let y = bottom_left.y
                + if rows > 0 {
                    row as f64 * size.y / (rows - 1) as f64
                } else {
                    0.0
                };
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidate_grid_diag(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    let min_distance = 7.0711;
    let cols = (size.x / min_distance).floor() as usize + 1;
    let rows = (size.y / min_distance).floor() as usize + 1;
    generate_candidate_checker(bottom_left, size, rows, cols)
}

fn generate_candidate_grid_compress(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    if size.x < 10. {
        let min_sub_height = (100. - size.x * size.x).sqrt().max(5.);
        let rows = (size.y / min_sub_height).floor() as usize;
        return generate_candidate_checker(bottom_left, size, rows, 2);
    }
    if size.y < 10. {
        let min_sub_width = (100. - size.y * size.y).sqrt().max(5.);
        let cols = (size.x / min_sub_width).floor() as usize;
        return generate_candidate_checker(bottom_left, size, 2, cols);
    }
    let mut best_count = 0;
    let mut best_cols = 0;
    let mut best_rows = 0;
    for cols in 2.. {
        let sub_width = size.x / (cols - 1) as f64;
        if sub_width < 5. {
            break;
        }
        if sub_width * sub_width > 75. {
            continue;
        }
        let sub_height = (100. - sub_width * sub_width).sqrt();
        let rows = (size.y / sub_height).floor() as usize;
        let count = (rows * cols + 1) / 2;
        if count > best_count {
            best_count = count;
            best_cols = cols;
            best_rows = rows;
        }
    }
    generate_candidate_checker(bottom_left, size, best_rows, best_cols)
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
        PlacementMode::GridNormal(mode) => generate_candidate_grid_normal(bottom_left, size, mode)?,
        PlacementMode::GridDiag => generate_candidate_grid_diag(bottom_left, size)?,
        PlacementMode::GridCompress => generate_candidate_grid_compress(bottom_left, size)?,
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
    let placement_modes = [
        PlacementMode::GridNormal(InterpolateMode::Strech),
        PlacementMode::GridNormal(InterpolateMode::Corner(10.0)),
        PlacementMode::GridDiag,
        PlacementMode::GridCompress,
    ];
    let mut param_packs = Vec::new();
    for pmode in placement_modes {
        param_packs.push((pmode, false));
        if is_full_division_scoring(prob) {
            param_packs.push((pmode, true));
        }
    }
    let sol = param_packs
        .par_iter()
        .filter_map(|&(pmode, together_mode)| solve_greedy_impl(prob, pmode, together_mode).ok())
        .flat_map(|sol| {
            let hg = optimize_hungarian(prob, &sol).unwrap();
            [sol, hg]
        })
        .max_by_key(|sol| score(prob, sol, true).unwrap())
        .ok_or(SolveGreedyError::FailedToGenerateSolution)?;
    Ok(sol)
}
