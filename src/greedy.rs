use crate::common::*;
use crate::geometry::*;
use anyhow::Result;
use ordered_float::NotNan;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolveGreedyError {
    #[error("Lack of candidates")]
    LackCandidatesError,
}

fn visible_attendees(attendees: &[Attendee], candidate: Point, placed: &[Point]) -> Vec<usize> {
    let mut attendee_angles = Vec::new();
    for (idx, attendee) in attendees.iter().enumerate() {
        let d = Point {
            x: attendee.x,
            y: attendee.y,
        } - candidate;
        attendee_angles.push((d.y.atan2(d.x), d.norm(), idx));
    }
    attendee_angles.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap());
    let mut blocked_ranges = Vec::new();
    let block_radius = 5.;
    let tau = std::f64::consts::TAU;
    for &place in placed.iter() {
        let diff = place - candidate;
        let distance = diff.length();
        let angle = diff.y.atan2(diff.x);
        let delta = (block_radius / distance).asin();
        let from = angle - delta;
        let to = angle + delta;
        blocked_ranges.push((from, to, distance * delta.cos()));
        if to >= tau {
            blocked_ranges.push((from - tau, to - tau, distance * delta.cos()));
        }
        if from < 0. {
            blocked_ranges.push((from + tau, to + tau, distance * delta.cos()));
        }
    }
    let mut itr = attendee_angles.iter().peekable();
    blocked_ranges.sort_by(|lhs, rhs| lhs.partial_cmp(rhs).unwrap());
    let mut heap = BinaryHeap::<(Reverse<NotNan<f64>>, NotNan<f64>)>::new();
    let mut res = Vec::new();
    for (from, to, di) in blocked_ranges {
        loop {
            let Some(&(angle, dj, j)) = itr.peek() else { break };
            if angle > &from {
                break;
            }
            let _ = itr.next();
            let mut to_be_pushed = true;
            while let Some((dk, to_k)) = heap.peek() {
                if to_k.into_inner() < *angle {
                    heap.pop();
                    continue;
                }
                if dk.0.into_inner() <= *dj {
                    to_be_pushed = false;
                }
                break;
            }
            if to_be_pushed {
                res.push(*j);
            }
        }
        heap.push((Reverse(NotNan::new(di).unwrap()), NotNan::new(to).unwrap()));
    }
    loop {
        let Some(&(angle, dj, j)) = itr.peek() else { break };
        let _ = itr.next();
        let mut to_be_pushed = true;
        while let Some((dk, to_k)) = heap.peek() {
            if to_k.into_inner() < *angle {
                heap.pop();
                continue;
            }
            if dk.0.into_inner() <= *dj {
                to_be_pushed = false;
            }
            break;
        }
        if to_be_pushed {
            res.push(*j);
        }
    }
    res
}

fn generate_candidates(prob: &Problem, diag: bool) -> Result<Vec<Point>> {
    let musician_left = prob.stage_bottom_left[0] + 10.;
    let musician_bottom = prob.stage_bottom_left[1] + 10.;
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

pub fn solve_greedy_optimized(prob: &Problem) -> Result<Solution> {
    let diag_mode = false;
    let mut placement_candidates = generate_candidates(prob, diag_mode)?;
    let mut visible = Vec::new();
    for _ in placement_candidates.iter() {
        visible.push(vec![true; prob.attendees.len()]);
    }
    let mut placed = Vec::new();
    let mut musicians: HashMap<_, _> = prob.musicians.iter().enumerate().collect();
    let mut pairs = Vec::new();
    while !musicians.is_empty() {
        let mut max_gain = i64::MIN;
        let mut max_gain_pidx = 0;
        let mut max_gain_midx = 0;
        for (pidx, &place) in placement_candidates.iter().enumerate() {
            let atd_indices = visible_attendees(&prob.attendees, place, &placed);
            for (midx, &&kind) in &musicians {
                let mut impact_sum = 0;
                for &aidx in &atd_indices {
                    let atd = &prob.attendees[aidx];
                    let dsq = (Point { x: atd.x, y: atd.y } - place).norm();
                    impact_sum += (1e6 * atd.tastes[kind as usize] / dsq).ceil() as i64;
                }
                if impact_sum > max_gain {
                    max_gain = impact_sum;
                    max_gain_pidx = pidx;
                    max_gain_midx = *midx;
                }
            }
        }
        let place = placement_candidates.swap_remove(max_gain_pidx);
        placed.push(place);
        musicians.remove(&max_gain_midx);
        pairs.push((max_gain_midx, place));
    }
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs.into_iter().map(|(_, place)| place).collect();
    Ok(Solution { placements })
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let diag_mode = false;
    let mut placement_candidates = generate_candidates(prob, diag_mode)?;
    let mut placed = Vec::new();
    let mut musicians: HashMap<_, _> = prob.musicians.iter().enumerate().collect();
    let mut pairs = Vec::new();
    while !musicians.is_empty() {
        let mut max_gain = i64::MIN;
        let mut max_gain_pidx = 0;
        let mut max_gain_midx = 0;
        for (pidx, &place) in placement_candidates.iter().enumerate() {
            let atd_indices = visible_attendees(&prob.attendees, place, &placed);
            for (midx, &&kind) in &musicians {
                let mut impact_sum = 0;
                for &aidx in &atd_indices {
                    let atd = &prob.attendees[aidx];
                    let dsq = (Point { x: atd.x, y: atd.y } - place).norm();
                    impact_sum += (1e6 * atd.tastes[kind as usize] / dsq).ceil() as i64;
                }
                if impact_sum > max_gain {
                    max_gain = impact_sum;
                    max_gain_pidx = pidx;
                    max_gain_midx = *midx;
                }
            }
        }
        let place = placement_candidates.swap_remove(max_gain_pidx);
        placed.push(place);
        musicians.remove(&max_gain_midx);
        pairs.push((max_gain_midx, place));
    }
    pairs.sort_unstable_by_key(|e| e.0);
    let placements = pairs.into_iter().map(|(_, place)| place).collect();
    Ok(Solution { placements })
}
