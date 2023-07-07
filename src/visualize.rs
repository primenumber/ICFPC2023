use crate::common::*;
use anyhow::Result;
use std::path::PathBuf;
use svg::node::element::{Circle, Group, Rectangle};
use svg::Document;

fn rgb_from_hue(hue: f64) -> (u32, u32, u32) {
    let angle = (hue.fract() * 6.).floor() as u32;
    let frac = (hue.fract() * 6.).fract();
    let x = (frac * 255.999).floor() as u32;
    if angle == 0 {
        (255, x, 0)
    } else if angle == 1 {
        (255 - x, 255, 0)
    } else if angle == 2 {
        (0, 255, x)
    } else if angle == 3 {
        (0, 255 - x, 255)
    } else if angle == 4 {
        (x, 0, 255)
    } else if angle == 5 {
        (255, 0, 255 - x)
    } else {
        panic!();
    }
}

fn to_color(hue: f64) -> String {
    let (r, g, b) = rgb_from_hue(hue);
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn vis_attendee(attendee: &Attendee) -> Rectangle {
    let size = 4.;
    Rectangle::new()
        .set("x", attendee.x - size / 2.)
        .set("y", attendee.y - size / 2.)
        .set("width", size)
        .set("height", size)
        .set("stroke-width", 0.1)
        .set("stroke", "black")
        .set("fill", "white")
}

fn vis_attendees(attendees: &[Attendee]) -> Group {
    let mut g = Group::new();
    for attendee in attendees {
        g = g.add(vis_attendee(attendee));
    }
    g
}

fn vis_musician(kind: u32, max_kind: usize, place: Point) -> Circle {
    let ratio = kind as f64 / max_kind as f64;
    Circle::new()
        .set("cx", place.x)
        .set("cy", place.y)
        .set("r", 10.0)
        .set("stroke-width", 1)
        .set("stroke", "black")
        .set("fill", to_color(ratio))
        .set("fill-opacity", 0.5)
}

fn vis_musicians(kinds: &[u32], max_kind: usize, places: &[Point]) -> Group {
    let mut g = Group::new();
    for (&kind, &place) in kinds.iter().zip(places.iter()) {
        g = g.add(vis_musician(kind, max_kind, place));
    }
    g
}

fn vis_room(width: f64, height: f64) -> Rectangle {
    Rectangle::new()
        .set("x", 0)
        .set("y", 0)
        .set("width", width)
        .set("height", height)
        .set("fill", "silver")
}

fn vis_stage(width: f64, height: f64, bottom_left: &[f64]) -> Rectangle {
    Rectangle::new()
        .set("x", bottom_left[0])
        .set("y", bottom_left[1])
        .set("width", width)
        .set("height", height)
        .set("fill", "gray")
}

pub fn visualize(prob: &Problem, sol: &Solution, output: &PathBuf) -> Result<()> {
    let w = prob.room_width as i32;
    let h = prob.room_height as i32;
    let margin = 10;
    let mut doc = Document::new()
        .set("id", "vis")
        .set(
            "viewBox",
            (-margin, -margin, w + 2 * margin, h + 2 * margin),
        )
        .set("width", w + 2 * margin)
        .set("height", h + 2 * margin)
        .set("style", "background-color:white");
    doc = doc.add(vis_room(prob.room_width, prob.room_height));
    doc = doc.add(vis_stage(
        prob.stage_width,
        prob.stage_height,
        &prob.stage_bottom_left,
    ));
    let max_kind = prob.attendees.first().unwrap().tastes.len();
    doc = doc.add(vis_attendees(&prob.attendees));
    doc = doc.add(vis_musicians(&prob.musicians, max_kind, &sol.placements));
    let html = format!("<html><body>{}</body></html>", doc.to_string());
    std::fs::write(output, html)?;
    Ok(())
}
