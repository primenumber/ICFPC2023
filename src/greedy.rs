use crate::common::*;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveGreedyError {
    #[error("Lack of candidates")]
    LackCandidatesError,
}

fn gain(_musician: u32, _attendees: &[Attendee], _place: Point) -> i64 {
    0
}

fn solve_greedy_impl(
    musician: u32,
    attendees: &[Attendee],
    candidates: &mut Vec<Point>,
) -> Result<Point> {
    let (idx, _score) = candidates
        .iter()
        .enumerate()
        .max_by_key(|(_, &place)| gain(musician, attendees, place))
        .ok_or(SolveGreedyError::LackCandidatesError)?;
    Ok(candidates.swap_remove(idx))
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let musician_left = prob.stage_bottom_left[0] + 10.;
    let musician_bottom = prob.stage_bottom_left[1] + 10.;
    let musicians_in_row = ((prob.stage_width - 10.) / 10.).floor() as usize;
    let musicians_in_col = ((prob.stage_height - 10.) / 10.).floor() as usize;
    let mut placement_candidates = Vec::new();
    for row in 0..musicians_in_col {
        for col in 0..musicians_in_row {
            placement_candidates.push(Point {
                x: musician_left + col as f64 * 10.,
                y: musician_bottom + row as f64 * 10.,
            });
        }
    }
    let mut placements = Vec::new();
    for &musician in &prob.musicians {
        placements.push(solve_greedy_impl(
            musician,
            &prob.attendees,
            &mut placement_candidates,
        )?);
    }
    Ok(Solution { placements })
}
