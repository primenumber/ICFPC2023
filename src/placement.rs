use crate::common::*;
use crate::geometry::*;
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneratePlacementError {
    #[error("Lack of candidates")]
    LackCandidatesError,
}

#[derive(Clone, Copy)]
pub enum InterpolateMode {
    Strech,
    Corner(f64),
}

#[derive(Clone, Copy)]
pub enum PlacementMode {
    GridNormal(InterpolateMode),
    GridDiag,
    GridCompress,
}

fn interpolate(length: f64, total: usize, index: usize, mode: InterpolateMode) -> f64 {
    match mode {
        InterpolateMode::Strech => {
            if index == 0 {
                0.0
            } else {
                length * index as f64 / (total as f64 - 1.0)
            }
        }
        InterpolateMode::Corner(gap) => {
            if index * 2 < total {
                gap * index as f64
            } else {
                let rem = total - index - 1;
                length - 10. * rem as f64
            }
        }
    }
}

fn generate_candidate_grid_normal(
    bottom_left: Point,
    size: Point,
    mode: InterpolateMode,
) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    let min_distance = 10.0;
    let cols = (size.x / min_distance).floor() as usize + 1;
    let rows = (size.y / min_distance).floor() as usize + 1;
    for row in 0..rows {
        for col in 0..cols {
            let x = bottom_left.x + interpolate(size.x, cols, col, mode);
            let y = bottom_left.y + interpolate(size.y, rows, row, mode);
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidate_checker(
    bottom_left: Point,
    size: Point,
    rows: usize,
    cols: usize,
) -> Result<Vec<Point>> {
    let mut placement_candidates = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            if (row + col) % 2 == 1 {
                continue;
            }
            let x = bottom_left.x
                + if cols > 0 {
                    col as f64 * size.x / (cols - 1) as f64
                } else {
                    0.0
                };
            let y = bottom_left.y
                + if rows > 0 {
                    row as f64 * size.y / (rows - 1) as f64
                } else {
                    0.0
                };
            placement_candidates.push(Point { x, y });
        }
    }
    Ok(placement_candidates)
}

fn generate_candidate_grid_diag(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    let min_distance = 7.0711;
    let cols = (size.x / min_distance).floor() as usize + 1;
    let rows = (size.y / min_distance).floor() as usize + 1;
    generate_candidate_checker(bottom_left, size, rows, cols)
}

fn generate_candidate_grid_compress(bottom_left: Point, size: Point) -> Result<Vec<Point>> {
    if size.x < 10. {
        let min_sub_height = (100. - size.x * size.x).sqrt().max(5.);
        let rows = (size.y / min_sub_height).floor() as usize;
        return generate_candidate_checker(bottom_left, size, rows, 2);
    }
    if size.y < 10. {
        let min_sub_width = (100. - size.y * size.y).sqrt().max(5.);
        let cols = (size.x / min_sub_width).floor() as usize;
        return generate_candidate_checker(bottom_left, size, 2, cols);
    }
    let mut best_count = 0;
    let mut best_cols = 0;
    let mut best_rows = 0;
    for cols in 2.. {
        let sub_width = size.x / (cols - 1) as f64;
        if sub_width < 5. {
            break;
        }
        if sub_width * sub_width > 75. {
            continue;
        }
        let sub_height = (100. - sub_width * sub_width).sqrt();
        let rows = (size.y / sub_height).floor() as usize;
        let count = (rows * cols + 1) / 2;
        if count > best_count {
            best_count = count;
            best_cols = cols;
            best_rows = rows;
        }
    }
    generate_candidate_checker(bottom_left, size, best_rows, best_cols)
}

pub fn generate_candidates(prob: &Problem, mode: PlacementMode) -> Result<Vec<Point>> {
    let padding = 10.0;
    let bottom_left = prob.stage_from()
        + Point {
            x: padding,
            y: padding,
        };
    let size = prob.stage_size()
        - Point {
            x: 2.0 * padding,
            y: 2.0 * padding,
        };
    let placement_candidates = match mode {
        PlacementMode::GridNormal(mode) => generate_candidate_grid_normal(bottom_left, size, mode)?,
        PlacementMode::GridDiag => generate_candidate_grid_diag(bottom_left, size)?,
        PlacementMode::GridCompress => generate_candidate_grid_compress(bottom_left, size)?,
    };
    if placement_candidates.len() < prob.musicians.len() {
        return Err(GeneratePlacementError::LackCandidatesError.into());
    }
    Ok(placement_candidates)
}
