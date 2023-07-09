use std::collections::HashSet;

use crate::common::Pillar;
use crate::common::*;
use crate::geometry::*;
use crate::score::*;
use anyhow::Result;

pub fn optimize_hungarian(prob: &Problem, sol: &Solution) -> Result<Solution> {
    let m: usize = prob.musicians.len();
    let mut score_contrib_table = create_score_contrib_table(prob, &sol.placements, &prob.pillars);

    for score_contrib_musician in score_contrib_table.iter_mut() {
        let min = score_contrib_musician.iter().min().unwrap_or(&0);
        *score_contrib_musician = score_contrib_musician
            .iter()
            .map(|f| f - min)
            .collect::<Vec<i64>>();
    }
    for score_contrib_placement_idx in 0..m {
        let min = (0..m)
            .map(|x| score_contrib_table[x][score_contrib_placement_idx])
            .min()
            .unwrap_or(0);
        for score_contrib_musician_idx in 0..m {
            score_contrib_table[score_contrib_musician_idx][score_contrib_placement_idx] -= min
        }
    }

    let mut placements = vec![Point { x: 0.0, y: 0.0 }; m];
    loop {
        let zero_coordinates = get_zero_coordinate(&score_contrib_table, m);
        let select = is_finish_hungarian(&zero_coordinates, m);
        let len = zero_coordinates.len();
        println!("{len}");
        if let Some(select) = select {
            for (musician_idx, placement_idx) in select {
                placements[musician_idx] = sol.placements[placement_idx];
            }
            break;
        }

        let (musician_line, placement_line) = create_cover_line(zero_coordinates, m);
        update_score_contrib(&mut score_contrib_table, musician_line, placement_line, m);
    }

    Ok(Solution { placements })
}

fn create_score_contrib_table(
    prob: &Problem,
    placements: &[Point],
    pillars: &[Pillar],
) -> Vec<Vec<i64>> {
    let m: usize = placements.len();

    let mut score_contrib_table = vec![vec![0; m]; m];
    for (placement_idx, _) in placements.iter().enumerate() {
        let impact_attendees = prob
            .attendees
            .iter()
            .filter(|&attendee| check_impact_attendee(attendee, pillars, placements, placement_idx))
            .collect::<Vec<&Attendee>>();
        for (musician_idx, kind) in prob.musicians.iter().enumerate() {
            score_contrib_table[musician_idx][placement_idx] =
                calc_score_contrib(&impact_attendees, kind, placements, placement_idx);
        }
    }
    let max = *score_contrib_table.iter().flatten().max().unwrap_or(&0);
    score_contrib_table
        .iter()
        .map(|v| v.iter().map(|x| max - x).collect::<Vec<i64>>())
        .collect::<Vec<Vec<i64>>>()
}

fn check_impact_attendee(
    attendee: &Attendee,
    pillars: &[Pillar],
    placements: &[Point],
    placement_idx: usize,
) -> bool {
    if !check_other_musicians(attendee, placements, placement_idx) {
        return false;
    }
    let place = placements[placement_idx];
    if !check_pillars(attendee, place, pillars) {
        return false;
    }
    true
}
fn calc_score_contrib(
    attendees: &[&Attendee],
    kind: &u32,
    placements: &[Point],
    placement_idx: usize,
) -> i64 {
    let mut score_contrib = 0;
    for attendee in attendees.iter() {
        score_contrib += impact_raw(attendee, *kind, placements[placement_idx]);
    }
    score_contrib
}

fn get_zero_coordinate(score_contrib_table: &[Vec<i64>], m: usize) -> Vec<(usize, usize)> {
    let mut zero_coordinates = Vec::new();
    for (musician_idx, score_contrib_musician) in score_contrib_table.iter().enumerate() {
        for (placement_idx, contr) in score_contrib_musician.iter().enumerate() {
            if contr == &0 {
                zero_coordinates.push((musician_idx, placement_idx));
            }
        }
    }
    zero_coordinates
}

fn is_finish_hungarian(
    zero_coordinates: &[(usize, usize)],
    m: usize,
) -> Option<Vec<(usize, usize)>> {
    let mut select = vec![(0, 0); m];
    let mut musician_idx_use: HashSet<usize> = HashSet::new();
    let mut placement_idx_use: HashSet<usize> = HashSet::new();
    for (musician_idx, placement_idx) in zero_coordinates {
        if !musician_idx_use.contains(musician_idx) && !placement_idx_use.contains(placement_idx) {
            select.push((*musician_idx, *placement_idx));
            musician_idx_use.insert(*musician_idx);
            placement_idx_use.insert(*placement_idx);
        }
    }
    if select.len() != m {
        return None;
    }
    Some(select)
}

fn create_cover_line(
    mut zero_coordinates: Vec<(usize, usize)>,
    m: usize,
) -> (HashSet<usize>, HashSet<usize>) {
    let mut musician_line = HashSet::new();
    let mut placement_line = HashSet::new();
    while !zero_coordinates.is_empty() {
        let mut musician_zero_count = vec![0; m];
        let mut placement_zero_count = vec![0; m];
        for (musician, placement) in zero_coordinates.iter() {
            musician_zero_count[*musician] += 1;
            placement_zero_count[*placement] += 1;
        }
        let musician_max = get_max_idx(musician_zero_count);
        let placement_max = get_max_idx(placement_zero_count);
        if placement_max.1 < musician_max.1 {
            musician_line.insert(musician_max.0);
            zero_coordinates = zero_coordinates
                .into_iter()
                .filter(|zero_coordinate| zero_coordinate.0 != musician_max.0)
                .collect::<Vec<(usize, usize)>>();
        } else {
            placement_line.insert(placement_max.0);
            zero_coordinates = zero_coordinates
                .into_iter()
                .filter(|zero_coordinate| zero_coordinate.1 != placement_max.0)
                .collect::<Vec<(usize, usize)>>();
        }
    }
    (musician_line, placement_line)
}

fn get_max_idx(vec: Vec<i32>) -> (usize, i32) {
    vec.iter().enumerate().fold(
        (0, 0),
        |(max_idx, max), (idx, &v)| if v > max { (idx, v) } else { (max_idx, max) },
    )
}

fn update_score_contrib(
    score_contrib_table: &mut [Vec<i64>],
    musician_line: HashSet<usize>,
    placement_line: HashSet<usize>,
    m: usize,
) {
    let mut min = i64::MAX;
    for musician_idx in 0..m {
        for placement_idx in 0..m {
            if !musician_line.contains(&musician_idx)
                && !placement_line.contains(&placement_idx)
                && score_contrib_table[musician_idx][placement_idx] < min
            {
                min = score_contrib_table[musician_idx][placement_idx];
            }
        }
    }
    for musician_idx in 0..m {
        for placement_idx in 0..m {
            if !musician_line.contains(&musician_idx) && !placement_line.contains(&placement_idx) {
                score_contrib_table[musician_idx][placement_idx] -= min;
            } else if musician_line.contains(&musician_idx)
                && placement_line.contains(&placement_idx)
            {
                score_contrib_table[musician_idx][placement_idx] += min;
            }
        }
    }
}
