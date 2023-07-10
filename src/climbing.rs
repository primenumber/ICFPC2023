use crate::cache::*;
use crate::common::*;
use crate::geometry::*;
use crate::hungarian::*;
use crate::placement::*;
use crate::score::*;
use anyhow::Result;
use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use std::cmp::min;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveClimbingError {
    #[error("Failed to generate any valid solutions")]
    FailedToGenerateSolution,
}

fn score_partial(prob: &Problem, placements: &[Option<Point>], volume: &[f64]) -> i64 {
    let mut result = 0;
    for (midx, &opt_place) in placements.iter().enumerate() {
        let Some(place) = opt_place else { continue; };
        'attendee: for attendee in prob.attendees.iter() {
            if !check_pillars(attendee, place, &prob.pillars) {
                continue;
            }
            for (another_midx, &another_place) in placements.iter().enumerate() {
                if another_midx == midx {
                    continue;
                }
                let Some(another_place) = another_place else {continue;};
                if is_blocked_by_another(attendee, place, another_place) {
                    continue 'attendee;
                }
            }
            result += (volume[midx] * impact_raw(attendee, prob.musicians[midx], place) as f64)
                .ceil() as i64;
        }
    }
    result
}

fn convert_to_parital_placement(
    musician_to_place: &[Option<usize>],
    placements: &[Point],
) -> Vec<Option<Point>> {
    musician_to_place
        .iter()
        .map(|&e| -> Option<Point> { Some(placements[e?]) })
        .collect()
}

fn solve_climbing_impl(prob: &Problem, placement_mode: PlacementMode) -> Result<Solution> {
    let placement_candidates = generate_candidates(prob, placement_mode)?;
    let mut musician_to_place = vec![None; prob.musicians.len()];
    let mut place_to_musician = vec![None; placement_candidates.len()];
    let mut volumes = vec![10.0; prob.musicians.len()];
    let mut current_basic_score = 0;
    let mut best_basic_score = i64::MIN;
    let mut best_sol = None;
    let mut best_p2m = None;
    let mut best_m2p = None;
    let mut rng = SmallRng::from_entropy();

    for _i in 0..100 {
        let mut cache = DiffCache::new(
            prob,
            &placement_candidates,
            &musician_to_place,
            &place_to_musician,
            &volumes,
        );
        let mut remain = musician_to_place.iter().filter(|e| e.is_none()).count();

        // place musicians greedy
        while remain > 0 {
            let (i, j, d, v) = cache.find_best_matching();
            current_basic_score += d;
            musician_to_place[j] = Some(i);
            place_to_musician[i] = Some(j);
            volumes[j] = v;
            cache.add_matching(prob, i, j, &volumes);
            remain -= 1;
        }

        let placements = musician_to_place
            .iter()
            .map(|e| placement_candidates[e.unwrap()])
            .collect();

        if current_basic_score > best_basic_score {
            best_sol = Some(Solution {
                placements,
                volumes: volumes.clone(),
            });
            best_basic_score = current_basic_score;
            best_p2m = Some(place_to_musician.clone());
            best_m2p = Some(musician_to_place.clone());
        } else {
            place_to_musician.copy_from_slice(&best_p2m.clone().unwrap());
            musician_to_place.copy_from_slice(&best_m2p.clone().unwrap());
            volumes.copy_from_slice(&best_sol.clone().unwrap().volumes);
        }

        let k = min(30, prob.musicians.len() / 2);
        let destruction_target: Vec<_> = musician_to_place
            .choose_multiple(&mut rng, k)
            .cloned()
            .collect();
        for opt_pidx in destruction_target {
            let pidx = opt_pidx.unwrap();
            let midx = place_to_musician[pidx].unwrap();
            place_to_musician[pidx] = None;
            musician_to_place[midx] = None;
            volumes[midx] = 10.0;
        }
        let partial_placement =
            convert_to_parital_placement(&musician_to_place, &placement_candidates);
        current_basic_score = score_partial(prob, &partial_placement, &volumes);
    }
    best_sol.ok_or(SolveClimbingError::FailedToGenerateSolution.into())
}

pub fn solve_climbing(prob: &Problem) -> Result<Solution> {
    let placement_modes = [
        PlacementMode::GridNormal(InterpolateMode::Strech),
        PlacementMode::GridNormal(InterpolateMode::Corner(10.0)),
        PlacementMode::GridDiag,
        PlacementMode::GridCompress,
    ];
    let mut param_packs = Vec::new();
    for pmode in placement_modes {
        param_packs.push(pmode);
    }
    let sol = param_packs
        .par_iter()
        .filter_map(|&pmode| solve_climbing_impl(prob, pmode).ok())
        .flat_map(|sol| {
            let hg = optimize_hungarian(prob, &sol).unwrap();
            [sol, hg]
        })
        .max_by_key(|sol| score(prob, sol, true).unwrap())
        .ok_or(SolveClimbingError::FailedToGenerateSolution)?;
    Ok(sol)
}
