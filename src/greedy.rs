use crate::cache::*;
use crate::common::*;
use crate::geometry::*;
use crate::hungarian::*;
use crate::placement::*;
use crate::score::*;
use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveGreedyError {
    #[error("Failed to generate any valid solutions")]
    FailedToGenerateSolution,
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
