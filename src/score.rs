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
                x: musician.0.x,
                y: musician.0.y,
                r: 5.0,
            };
            if is_point_in_circle(&musician_area, &attendee_point) {
                let d = distance_point_point(&attendee_point, musician.0);
                score += (1e6 * attendee.tastes[*musician.1 as usize] / d.powf(2.0)).ceil() as i64;
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
                    x1: attendee.x,
                    y1: attendee.y,
                    x2: musician.0.x,
                    y2: musician.0.y,
                };
                let check_musician_area = Circle {
                    x: check_musician.0.x,
                    y: check_musician.0.y,
                    r: 5.0,
                };
                if is_cross_line_circle(&musician_attendee_line, &check_musician_area) {
                    valid_impact = false;
                }
            }
            if valid_impact {
                let d = distance_point_point(&attendee_point, musician.0);
                score += (1e6 * attendee.tastes[*musician.1 as usize] / d.powf(2.0)).ceil() as i64;
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
            if distance_point_point(musician_point, check_musician_point) < 10.0 {
                is_valid = false;
                println!("{musician_idx} is not far enough from {musician_idx}");
            }
        }
    }
    is_valid
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Line {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
struct Circle {
    x: f64,
    y: f64,
    r: f64,
}

fn is_point_in_circle(circle: &Circle, point: &Point) -> bool {
    let center = Point {
        x: circle.x,
        y: circle.y,
    };
    let d: f64 = distance_point_point(&center, point);
    d < circle.r
}

fn is_cross_line_circle(line: &Line, circle: &Circle) -> bool {
    let line = Line {
        x1: line.x1 - circle.x,
        y1: line.y1 - circle.y,
        x2: line.x2 - circle.x,
        y2: line.y2 - circle.y,
    };
    let a = line.x1.powf(2.0) + line.x2.powf(2.0) + line.y1.powf(2.0) + line.y2.powf(2.0)
        - 2.0 * line.x1 * line.x2
        - 2.0 * line.y1 * line.y1;
    let b = 2.0 * line.x1 * line.x2 + 2.0 * line.y1 * line.y2
        - 2.0 * line.x1.powf(2.0)
        - 2.0 * line.y1.powf(2.0);
    let c = line.x1.powf(2.0) + line.y1.powf(2.0) - circle.r.powf(2.0);
    let d = b.powf(2.0) - 4.0 * a * c;
    d < 0.0
        || (0.0 <= -b + d.sqrt() && -b + d.sqrt() <= 2.0 * a)
        || (0.0 <= -b - d.sqrt() && -b - d.sqrt() <= 2.0 * a)
}

fn distance_point_point(p1: &Point, p2: &Point) -> f64 {
    ((p1.x - p2.x).powf(2.0) + (p1.y - p2.y).powf(2.0)).sqrt()
}
