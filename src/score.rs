use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::common::{Point, Problem, Solution};

pub fn score(prob: &Problem, sol: &Solution) -> Result<()> {
    let m: usize = prob.musicians.len();

    if !is_valid_answer(sol, m) {
        println!("score -1");
        return Ok(());
    }

    let mut score: i64 = 0;
    for attendee in prob.attendees.iter() {
        for musician_idx in 0..m {
            let mut valid_impact = true;
            for check_musician_idx in 0..m {
                if musician_idx == check_musician_idx {
                    continue;
                }
                let line = Line {
                    x1: attendee.x,
                    y1: attendee.y,
                    x2: sol.placements[musician_idx].x,
                    y2: sol.placements[musician_idx].y,
                };
                let circle = Circle {
                    x: sol.placements[check_musician_idx].x,
                    y: sol.placements[check_musician_idx].y,
                    r: 5.0,
                };
                if is_cross_line_circle(line, circle) {
                    valid_impact = false;
                }
            }
            if valid_impact {
                let p1 = Point {
                    x: attendee.x,
                    y: attendee.y,
                };
                let p2 = Point {
                    x: sol.placements[musician_idx].x,
                    y: sol.placements[musician_idx].y,
                };
                let d = distance_point_point(p1, p2);
                score += (1e6 * attendee.tastes[prob.musicians[musician_idx] as usize]
                    / d.powf(2.0))
                .ceil() as i64;
            }
        }
    }
    println!("score {score}");
    Ok(())
}

fn is_valid_answer(sol: &Solution, m: usize) -> bool {
    let mut is_valid = true;
    for musician_idx in 0..m {
        for check_musician_idx in 0..m {
            if musician_idx == check_musician_idx {
                continue;
            }
            let p1 = Point {
                x: sol.placements[musician_idx].x,
                y: sol.placements[musician_idx].y,
            };
            let p2 = Point {
                x: sol.placements[check_musician_idx].x,
                y: sol.placements[check_musician_idx].y,
            };
            if distance_point_point(p1, p2) < 10.0 {
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

fn is_cross_line_circle(line: Line, circle: Circle) -> bool {
    let xd = line.x2 - line.x1;
    let yd = line.y2 - line.y1;
    let x = line.x1 - circle.x;
    let y = line.y1 - circle.y;
    let a = xd.powf(2.0) + yd.powf(2.0);
    let b = xd.powf(x) + yd.powf(y);
    let c = x.powf(2.0) + y.powf(2.0) - circle.r.powf(2.0);
    let d = b.powf(2.0) - a * c;
    d < 0.0
}

fn distance_point_point(p1: Point, p2: Point) -> f64 {
    ((p1.x - p2.x).powf(2.0) + (p1.y - p2.y).powf(2.0)).sqrt()
}
