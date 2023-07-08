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

fn check_non_blocking_pillars(
    attendee_place: Point,
    musician_place: Point,
    pillars: &[Pillar],
) -> bool {
    let segment = Line {
        p1: attendee_place,
        p2: musician_place,
    };
    for pillar in pillars {
        let circle = Circle {
            c: pillar.c(),
            r: pillar.radius,
        };
        if is_cross_line_circle(segment, circle) {
            return false;
        }
    }
    return true;
}

fn find_best_pair(
    current_impact: &[Vec<i64>],
    used_places: &[bool],
    used_musicians: &[bool],
) -> (usize, usize) {
    let (i, j, _) = current_impact
        .iter()
        .enumerate()
        .filter(|(i, _)| !used_places[*i])
        .map(|(i, impacts)| {
            let (j, impact) = impacts
                .iter()
                .enumerate()
                .filter(|(j, _)| !used_musicians[*j])
                .max_by_key(|(_, &impact)| impact)
                .unwrap();
            (i, j, *impact)
        })
        .max_by_key(|(_, _, v)| *v)
        .unwrap();
    (i, j)
}

fn update_impact(
    current_impact: &mut [Vec<i64>],
    visible: &mut [Vec<bool>],
    prob: &Problem,
    new_place: Point,
    placement_candidates: &[Point],
) {
    let circle = Circle {
        c: new_place,
        r: 5.0,
    };
    for (j, atd) in prob.attendees.iter().enumerate() {
        let atd_place = Point { x: atd.x, y: atd.y };
        for (i, &candi_place) in placement_candidates.iter().enumerate() {
            let segment = Line {
                p1: atd_place,
                p2: candi_place,
            };
            if !is_cross_line_circle(segment, circle) || !visible[i][j] {
                continue;
            }
            visible[i][j] = false;
            for (k, &kind) in prob.musicians.iter().enumerate() {
                current_impact[i][k] -= impact_raw(atd, kind, candi_place);
            }
        }
    }
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let diag_mode = false;
    let placement_candidates = generate_candidates(prob, diag_mode)?;
    let mut visible = vec![vec![true; prob.attendees.len()]; placement_candidates.len()];
    let mut current_impact = vec![vec![0; prob.musicians.len()]; placement_candidates.len()];
    for (i, &place) in placement_candidates.iter().enumerate() {
        for (j, vis) in visible[i].iter_mut().enumerate() {
            let atd = &prob.attendees[j];
            let atd_place = Point { x: atd.x, y: atd.y };
            *vis = check_non_blocking_pillars(atd_place, place, &prob.pillars);
            if !*vis {
                continue;
            }
            for (k, &kind) in prob.musicians.iter().enumerate() {
                current_impact[i][k] += impact_raw(atd, kind, place);
            }
        }
    }
    let mut used_places = vec![false; placement_candidates.len()];
    let mut used_musicians = vec![false; prob.musicians.len()];
    let mut musicians: HashMap<_, _> = prob.musicians.iter().enumerate().collect();
    let mut pairs = Vec::new();
    while !musicians.is_empty() {
        let (i, j) = find_best_pair(&current_impact, &used_places, &used_musicians);
        let new_place = placement_candidates[i];
        used_places[i] = true;
        used_musicians[j] = true;
        musicians.remove(&j);
        pairs.push((j, new_place));
        update_impact(
            &mut current_impact,
            &mut visible,
            prob,
            new_place,
            &placement_candidates,
        );
    }
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs.into_iter().map(|(_, place)| place).collect();
    Ok(Solution { placements })
}
