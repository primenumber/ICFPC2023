use crate::common::*;
use anyhow::Result;

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let musician_left = prob.stage_bottom_left[0] + 10.;
    let musician_bottom = prob.stage_bottom_left[1] + 10.;
    let musicians_in_row = ((prob.stage_width - 10.) / 10.).floor() as usize;
    let mut placements = Vec::new();
    for (i, _) in prob.musicians.iter().enumerate() {
        let row = i / musicians_in_row;
        let col = i % musicians_in_row;
        placements.push(Point {
            x: musician_left + col as f64 * 10.,
            y: musician_bottom + row as f64 * 10.,
        });
    }
    Ok(Solution { placements })
}
