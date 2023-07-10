use crate::common::Pillar;
use crate::common::*;
use crate::geometry::*;
use crate::score::*;
use anyhow::Result;
use std::cmp::max;

// https://ei1333.github.io/library/graph/flow/hungarian.hpp
fn hungarian_impl(mat: &[Vec<i64>]) -> Vec<usize> {
    let m = mat.len();
    let mut p = vec![0; m];
    let mut way = vec![0; m];
    let mut u = vec![0_i128; m];
    let mut v = vec![0_i128; m];
    let mut min_v;
    let mut used;

    for i in 1..m {
        p[0] = i;
        min_v = vec![i128::MAX; m];
        used = vec![false; m];
        let mut j0 = 0;
        while p[j0] != 0 {
            let i0 = p[j0];
            let mut j1 = 0;
            used[j0] = true;
            let mut delta = i128::MAX;
            for j in 1..m {
                if used[j] {
                    continue;
                }
                let curr = mat[i0][j] as i128 - u[i0] - v[j];
                if curr < min_v[j] {
                    min_v[j] = curr;
                    way[j] = j0;
                }
                if min_v[j] < delta {
                    delta = min_v[j];
                    j1 = j;
                }
            }
            for j in 0..m {
                if used[j] {
                    u[p[j]] += delta;
                    v[j] -= delta;
                } else {
                    min_v[j] -= delta;
                }
            }
            j0 = j1;
        }
        while {
            p[j0] = p[way[j0]];
            j0 = way[j0];
            j0 != 0
        } {}
    }
    p[0] = 0;

    p
}

pub fn optimize_hungarian(prob: &Problem, sol: &Solution) -> Result<Solution> {
    let m: usize = prob.musicians.len();
    let score_contrib_table = create_score_contrib_table(prob, &sol.placements, &prob.pillars);
    let mut placements = vec![Point { x: 0.0, y: 0.0 }; m];
    let mat = convert_matrix_for_hungarian(&score_contrib_table);
    let p = hungarian_impl(&mat);

    for i in 1..=m {
        let new_musician_idx = p[i] - 1;
        let placement_idx = i - 1;
        placements[new_musician_idx] = sol.placements[placement_idx];
    }
    Ok(Solution {
        placements,
        volumes: sol.volumes.clone(),
    })
}

pub fn optimize_hungarian_2(prob: &Problem, sol: &Solution) -> (Solution, Vec<usize>) {
    let m: usize = prob.musicians.len();
    let score_contrib_table = create_score_contrib_table(prob, &sol.placements, &prob.pillars);
    let mut placements = vec![Point { x: 0.0, y: 0.0 }; m];
    let mat = convert_matrix_for_hungarian(&score_contrib_table);
    let p = hungarian_impl(&mat);

    let mut perm = vec![0; m];
    for i in 1..=m {
        let new_musician_idx = p[i] - 1;
        let placement_idx = i - 1;
        placements[new_musician_idx] = sol.placements[placement_idx];
        perm[new_musician_idx] = placement_idx;
    }
    (
        Solution {
            placements,
            volumes: sol.volumes.clone(),
        },
        perm,
    )
}

fn convert_matrix_for_hungarian(orig: &[Vec<i64>]) -> Vec<Vec<i64>> {
    let mut mat = orig.to_vec();
    //for row in mat.iter_mut() {
    //    for elem in row.iter_mut() {
    //        *elem = max(*elem, 0);
    //    }
    //}
    let max = *mat.iter().flatten().max().unwrap_or(&0);
    let m = mat.len();
    for i in 1..m {
        for j in 1..m {
            mat[i][j] = max - mat[i][j];
        }
    }
    mat
}

fn create_score_contrib_table(
    prob: &Problem,
    placements: &[Point],
    pillars: &[Pillar],
) -> Vec<Vec<i64>> {
    let m: usize = placements.len();

    let mut score_contrib_table = vec![vec![0; m + 1]; m + 1];
    for (placement_idx, _) in placements.iter().enumerate() {
        let impact_attendees = prob
            .attendees
            .iter()
            .filter(|&attendee| check_impact_attendee(attendee, pillars, placements, placement_idx))
            .collect::<Vec<&Attendee>>();
        for (musician_idx, kind) in prob.musicians.iter().enumerate() {
            score_contrib_table[musician_idx + 1][placement_idx + 1] =
                calc_score_contrib(&impact_attendees, kind, placements, placement_idx);
        }
    }
    score_contrib_table
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hungarian() {
        let mat = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 5, 4, 7, 6],
            vec![0, 6, 7, 3, 2],
            vec![0, 8, 11, 2, 5],
            vec![0, 9, 8, 6, 7],
        ];
        let p = hungarian_impl(&mat);
        assert_eq!(p, vec![0, 4, 1, 3, 2]);
    }
}
