use crate::common::*;
use anyhow::Result;
use ordered_float::NotNan;
use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap};
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

fn argcmp(p0: Point, p1: Point) -> Ordering {
    ((p0.y, p0.x) < (0., 0.))
        .cmp(&((p1.y, p1.x) < (0., 0.)))
        .then_with(|| (p1.x * p0.y).partial_cmp(&(p0.x * p1.y)).unwrap())
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

fn generate_candidates(prob: &Problem) -> Result<Vec<Point>> {
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
    Ok(placement_candidates)
}

pub fn solve_greedy(prob: &Problem) -> Result<Solution> {
    let mut placement_candidates = generate_candidates(prob)?;
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
