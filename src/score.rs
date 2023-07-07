use anyhow::Result;
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};

use crate::common::{Point, Problem, Solution};

pub fn score(prob: &Problem, sol: &Solution) -> Result<()> {
    let n: usize = prob.attendees.len();

    if !is_valid_answer(sol) {
        println!("invalid solution");
        return Ok(());
    }

    let mut score: i64 = 0;

    let pb = ProgressBar::new(n as u64);
    'attendee: for attendee in prob.attendees.iter() {
        let attendee_point = Point {
            x: attendee.x,
            y: attendee.y,
        };
        let musicians: Vec<(usize, (&Point, &u32))> = sol
            .placements
            .iter()
            .zip(prob.musicians.iter())
            .enumerate()
            .collect();
        for (_musician_idx, musician) in musicians.iter() {
            let musician_area = Circle {
                c: *musician.0,
                r: 5.0,
            };
            if is_point_in_circle(&musician_area, &attendee_point) {
                let dsq = (attendee_point - *musician.0).norm();
                score += (1e6 * attendee.tastes[*musician.1 as usize] / dsq).ceil() as i64;
                continue 'attendee;
            }
        }
        for (musician_idx, musician) in &musicians {
            let mut valid_impact = true;
            for (check_musician_idx, check_musician) in &musicians {
                if musician_idx == check_musician_idx {
                    continue;
                }
                let musician_attendee_line = Line {
                    p1: Point {
                        x: attendee.x,
                        y: attendee.y,
                    },
                    p2: *musician.0,
                };
                let check_musician_area = Circle {
                    c: *check_musician.0,
                    r: 5.0,
                };
                if is_cross_line_circle(&musician_attendee_line, &check_musician_area) {
                    valid_impact = false;
                }
            }
            if valid_impact {
                let dsq = (attendee_point - *musician.0).norm();
                score += (1e6 * attendee.tastes[*musician.1 as usize] / dsq).ceil() as i64;
            }
        }
        pb.inc(1);
    }
    pb.finish_with_message("finish calculation");
    println!("score {score}");
    Ok(())
}

fn is_valid_answer(sol: &Solution) -> bool {
    let mut is_valid = true;
    for (musician_idx, musician_point) in sol.placements.iter().enumerate() {
        for (check_musician_idx, check_musician_point) in sol.placements.iter().enumerate() {
            if musician_idx == check_musician_idx {
                continue;
            }
            if (*musician_point - *check_musician_point).length() < 10.0 {
                is_valid = false;
                println!("{musician_idx} is not far enough from {musician_idx}");
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

fn is_point_in_circle(circle: &Circle, point: &Point) -> bool {
    let center = circle.c;
    let d: f64 = (center - *point).length();
    d < circle.r
}

fn is_cross_line_circle(line: &Line, circle: &Circle) -> bool {
    let d = line.p2 - line.p1;
    let n = d.normalize();
    let pa = line.p1 - circle.c;
    if n.dot(pa) <= 0. {
        return pa.length() < circle.r;
    }
    let pb = line.p2 - circle.c;
    if n.dot(pb) >= 0. {
        return pb.length() < circle.r;
    }
    let pan = pa.dot(n);
    let pann = pan * n;
    let lp = (pa - pann).length();
    lp <= circle.r
}
