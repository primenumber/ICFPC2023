use crate::common::*;
use crate::geometry::*;
use anyhow::Result;
use indicatif::ProgressBar;

use crate::common::{Problem, Solution};

pub fn impact_raw(attendee: &Attendee, kind: u32, place: Point) -> i64 {
    let dsq = (attendee.place() - place).norm();
    (1e6 * attendee.tastes[kind as usize] / dsq).ceil() as i64
}

fn check_pillars(attendee: &Attendee, place: Point, pillars: &[Pillar]) -> bool {
    let musician_attendee_line = Line {
        p1: attendee.place(),
        p2: place,
    };
    for pillar in pillars {
        let check_pillar_area = Circle {
            c: pillar.c(),
            r: pillar.radius,
        };
        if is_cross_line_circle(musician_attendee_line, check_pillar_area) {
            return false;
        }
    }
    true
}

fn check_other_musicians(attendee: &Attendee, placements: &[Point], musician_idx: usize) -> bool {
    let place = placements[musician_idx];
    let musician_attendee_line = Line {
        p1: attendee.place(),
        p2: place,
    };
    for (check_musician_idx, check_place) in placements.iter().enumerate() {
        if musician_idx == check_musician_idx {
            continue;
        }
        let check_musician_area = Circle {
            c: *check_place,
            r: 5.0,
        };
        if is_cross_line_circle(musician_attendee_line, check_musician_area) {
            return false;
        }
    }
    true
}

fn impact(
    attendee: &Attendee,
    kinds: &[u32],
    placements: &[Point],
    musician_idx: usize,
    pillars: &[Pillar],
) -> i64 {
    if !check_other_musicians(attendee, placements, musician_idx) {
        return 0;
    }
    let place = placements[musician_idx];
    if !check_pillars(attendee, place, pillars) {
        return 0;
    }
    let kind = kinds[musician_idx];
    impact_raw(attendee, kind, place)
}

fn happiness(
    attendee: &Attendee,
    musicians: &[u32],
    pillars: &[Pillar],
    sol: &Solution,
    scalar: &[f64],
) -> i64 {
    let mut score = 0;
    for (musician_idx, scale) in scalar.iter().enumerate() {
        score += (impact(attendee, &musicians, &sol.placements, musician_idx, pillars) as f64
            * scale)
            .ceil() as i64;
    }
    score
}

pub fn is_full_division_scoring(prob: &Problem) -> bool {
    !prob.pillars.is_empty()
}

fn play_together_scalar(prob: &Problem, sol: &Solution) -> Vec<f64> {
    if !is_full_division_scoring(prob) {
        return vec![1.0; sol.placements.len()];
    }
    sol.placements
        .iter()
        .zip(prob.musicians.iter())
        .enumerate()
        .map(|(i, (&pi, &ki))| {
            let mut scalar = 1.0;
            for (j, (&pj, &kj)) in sol.placements.iter().zip(prob.musicians.iter()).enumerate() {
                if i == j || ki != kj {
                    continue;
                }
                scalar += 1.0 / (pi - pj).length();
            }
            scalar
        })
        .collect()
}

pub fn score(prob: &Problem, sol: &Solution, quiet: bool) -> Result<i64> {
    let n: usize = prob.attendees.len();

    if !is_valid_answer(prob, sol) {
        println!("invalid solution");
        return Ok(0);
    }

    let mut score: i64 = 0;
    let scalar = play_together_scalar(prob, sol);

    let pb = ProgressBar::new(n as u64);
    for attendee in prob.attendees.iter() {
        score += happiness(&attendee, &prob.musicians, &prob.pillars, sol, &scalar);
        if !quiet {
            pb.inc(1);
        }
    }
    if !quiet {
        pb.finish_with_message("finish calculation");
    }
    Ok(score)
}

fn is_valid_answer(prob: &Problem, sol: &Solution) -> bool {
    let mut is_valid = true;
    let stage_left = prob.stage_from().x;
    let stage_bottom = prob.stage_from().y;
    let stage_right = prob.stage_to().x;
    let stage_top = prob.stage_to().y;
    for (musician_idx, musician_point) in sol.placements.iter().enumerate() {
        if musician_point.x < stage_left + 10.
            || musician_point.x > stage_right - 10.
            || musician_point.y < stage_bottom + 10.
            || musician_point.y > stage_top - 10.
        {
            is_valid = false;
            println!(
                "{musician_idx} is not inside the stage or too close to the edge of the stage"
            );
        }
        for (check_musician_idx, check_musician_point) in sol.placements.iter().enumerate() {
            if musician_idx == check_musician_idx {
                continue;
            }
            if (*musician_point - *check_musician_point).norm() < 100.0 {
                is_valid = false;
                println!("{musician_idx} is not far enough from {check_musician_idx}");
            }
        }
    }
    is_valid
}
