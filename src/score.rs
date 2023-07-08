use crate::common::*;
use anyhow::Result;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

use crate::common::{Point, Problem, Solution};

fn impact(attendee: &Attendee, kinds: &[u32], placements: &[Point], musician_idx: usize) -> i64 {
    let attendee_point = Point {
        x: attendee.x,
        y: attendee.y,
    };
    let place = placements[musician_idx];
    let kind = kinds[musician_idx];
    for (check_musician_idx, check_place) in placements.iter().enumerate() {
        if musician_idx == check_musician_idx {
            continue;
        }
        let musician_attendee_line = Line {
            p1: Point {
                x: attendee.x,
                y: attendee.y,
            },
            p2: place,
        };
        let check_musician_area = Circle {
            c: *check_place,
            r: 5.0,
        };
        if is_cross_line_circle(&musician_attendee_line, &check_musician_area) {
            return 0;
        }
    }
    let dsq = (attendee_point - place).norm();
    (1e6 * attendee.tastes[kind as usize] / dsq).ceil() as i64
}

fn happiness(attendee: &Attendee, musicians: &[u32], sol: &Solution) -> i64 {
    let mut score = 0;
    for (musician_idx, _) in musicians.iter().enumerate() {
        score += impact(attendee, &musicians, &sol.placements, musician_idx);
    }
    score
}

pub fn score(prob: &Problem, sol: &Solution) -> Result<i64> {
    let n: usize = prob.attendees.len();

    if !is_valid_answer(sol) {
        println!("invalid solution");
        return Ok(0);
    }

    let mut score: i64 = 0;

    let pb = ProgressBar::new(n as u64);
    for attendee in prob.attendees.iter() {
        score += happiness(&attendee, &prob.musicians, sol);
        pb.inc(1);
    }
    pb.finish_with_message("finish calculation");
    println!("score {score}");
    Ok(score)
}

fn is_valid_answer(sol: &Solution) -> bool {
    let mut is_valid = true;
    for (musician_idx, musician_point) in sol.placements.iter().enumerate() {
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Line {
    p1: Point,
    p2: Point,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Circle {
    c: Point,
    r: f64,
}

fn is_cross_line_circle(line: &Line, circle: &Circle) -> bool {
    let d = line.p2 - line.p1;
    let n = d.normalize();
    let ap = circle.c - line.p1;
    let bp = circle.c - line.p2;
    if n.dot(ap) <= 0. {
        return ap.norm() <= circle.r * circle.r;
    }
    if n.dot(bp) >= 0. {
        return bp.norm() <= circle.r * circle.r;
    }
    let apn = ap.dot(n);
    let apnn = apn * n;
    let norm = (ap - apnn).norm();
    norm <= circle.r * circle.r
}
