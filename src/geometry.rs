use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn length(&self) -> f64 {
        self.norm().sqrt()
    }

    pub fn norm(&self) -> f64 {
        self.dot(*self)
    }

    pub fn dot(&self, rhs: Point) -> f64 {
        self.x * rhs.x + self.y * rhs.y
    }

    pub fn normalize(&self) -> Point {
        (1. / self.length()) * *self
    }
}

impl std::ops::Add<Point> for Point {
    type Output = Point;
    fn add(self, rhs: Point) -> Point {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Point;
    fn sub(self, rhs: Point) -> Point {
        Point {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<Point> for f64 {
    type Output = Point;
    fn mul(self, rhs: Point) -> Point {
        Point {
            x: self * rhs.x,
            y: self * rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub p1: Point,
    pub p2: Point,
}

#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub c: Point,
    pub r: f64,
}

pub fn norm_segment_point(line: Line, p: Point) -> f64 {
    let d = line.p2 - line.p1;
    let n = d.normalize();
    let ap = p - line.p1;
    let bp = p - line.p2;
    if n.dot(ap) <= 0. {
        return ap.norm();
    }
    if n.dot(bp) >= 0. {
        return bp.norm();
    }
    let apn = ap.dot(n);
    let apnn = apn * n;
    (ap - apnn).norm()
}

#[allow(dead_code)]
fn distance_segment_point(line: Line, p: Point) -> f64 {
    norm_segment_point(line, p).sqrt()
}

pub fn is_cross_line_circle(line: Line, circle: Circle) -> bool {
    norm_segment_point(line, circle.c) <= circle.r * circle.r
}
