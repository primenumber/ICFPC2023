use crate::common::*;
use anyhow::Result;
use std::cmp::Reverse;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveGreedyError {
    #[error("Lack of candidates")]
    LackCandidatesError,
}

fn gain(musician: u32, attendees: &[Attendee], place: Point) -> i64 {
    let mut sum = 0;
    for attendee in attendees {
        let dsq = (Point {
            x: attendee.x,
            y: attendee.y,
        } - place)
            .norm();
        sum += (1e6 * attendee.tastes[musician as usize] / dsq).ceil() as i64;
    }
    sum
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let musician_left = prob.stage_bottom_left[0] + 10.;
    let musician_bottom = prob.stage_bottom_left[1] + 10.;
    let min_distance = 10.;
    let musicians_in_row = ((prob.stage_width - 10.) / min_distance).floor() as usize;
    let musicians_in_col = ((prob.stage_height - 10.) / min_distance).floor() as usize;
    let mut placement_candidates = Vec::new();
    for row in 0..musicians_in_col {
        for col in 0..musicians_in_row {
            placement_candidates.push(Point {
                x: musician_left + col as f64 * min_distance,
                y: musician_bottom + row as f64 * min_distance,
            });
        }
    }
    if placement_candidates.len() < prob.musicians.len() {
        return Err(SolveGreedyError::LackCandidatesError.into());
    }
    let mut gains = Vec::new();
    for (i, &musician) in prob.musicians.iter().enumerate() {
        for (j, &place) in placement_candidates.iter().enumerate() {
            gains.push((gain(musician, &prob.attendees, place), i, j));
        }
    }
    gains.sort_unstable_by_key(|e| Reverse(e.0));
    let mut musician_used = vec![false; prob.musicians.len()];
    let mut place_used = vec![false; placement_candidates.len()];
    let mut pairs = Vec::new();
    for (_g, i, j) in gains {
        if musician_used[i] || place_used[j] {
            continue;
        }
        pairs.push((i, j));
        musician_used[i] = true;
        place_used[j] = true;
    }
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs
        .into_iter()
        .map(|(_, j)| placement_candidates[j])
        .collect();
    Ok(Solution { placements })
}
