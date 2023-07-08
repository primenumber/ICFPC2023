use crate::common::*;
use crate::geometry::*;
use anyhow::Result;
use indicatif::ProgressBar;

use crate::common::{Problem, Solution};

pub fn impact_raw(attendee: &Attendee, kind: u32, place: Point) -> i64 {
    let dsq = (attendee.place() - place).norm();
    (1e6 * attendee.tastes[kind as usize] / dsq).ceil() as i64
}

fn impact(
    attendee: &Attendee,
    kinds: &[u32],
    placements: &[Point],
    musician_idx: usize,
    pillars: &[Pillar],
) -> i64 {
    let place = placements[musician_idx];
    let kind = kinds[musician_idx];
    let musician_attendee_line = Line {
        p1: Point {
            x: attendee.x,
            y: attendee.y,
        },
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
            return 0;
        }
    }
    for pillar in pillars {
        let check_pillar_area = Circle {
            c: Point {
                x: pillar.center.0,
                y: pillar.center.1,
            },
            r: pillar.radius,
        };
        if is_cross_line_circle(musician_attendee_line, check_pillar_area) {
            return 0;
        }
    }
    impact_raw(attendee, kind, place)
}

fn happiness(attendee: &Attendee, musicians: &[u32], pillars: &[Pillar], sol: &Solution) -> i64 {
    let mut score = 0;
    for (musician_idx, _) in musicians.iter().enumerate() {
        score += impact(attendee, &musicians, &sol.placements, musician_idx, pillars);
    }
    score
}

pub fn score(prob: &Problem, sol: &Solution, quiet: bool) -> Result<i64> {
    let n: usize = prob.attendees.len();

    if !is_valid_answer(prob, sol) {
        println!("invalid solution");
        return Ok(0);
    }

    let mut score: i64 = 0;

    let pb = ProgressBar::new(n as u64);
    for attendee in prob.attendees.iter() {
        score += happiness(&attendee, &prob.musicians, &prob.pillars, sol);
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
    let stage_left = prob.stage_bottom_left[0];
    let stage_bottom = prob.stage_bottom_left[1];
    let stage_right = stage_left + prob.stage_width;
    let stage_top = stage_bottom + prob.stage_height;
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
