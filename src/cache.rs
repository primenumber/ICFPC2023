use crate::common::*;
use crate::geometry::*;
use crate::score::*;

pub struct DiffCache {
    places: Vec<Point>,
    visible: Vec<Vec<bool>>,
    impact_diff: Vec<Vec<i64>>,
    impact_diff_blocking: Vec<Vec<i64>>,
    musician_to_place: Vec<Option<usize>>,
    place_to_musician: Vec<Option<usize>>,
}

impl DiffCache {
    pub fn new(prob: &Problem, places: &[Point]) -> DiffCache {
        let mut visible = vec![vec![true; prob.attendees.len()]; places.len()];
        let mut impact_diff = vec![vec![0; prob.musicians.len()]; places.len()];
        for (i, &place) in places.iter().enumerate() {
            for (j, vis) in visible[i].iter_mut().enumerate() {
                let atd = &prob.attendees[j];
                *vis = check_pillars(atd, place, &prob.pillars);
                if !*vis {
                    continue;
                }
                for (k, &kind) in prob.musicians.iter().enumerate() {
                    impact_diff[i][k] += impact_raw(atd, kind, place);
                }
            }
        }
        let impact_diff_blocking = vec![vec![0; prob.musicians.len()]; places.len()];
        let musician_to_place = vec![None; prob.musicians.len()];
        let place_to_musician = vec![None; places.len()];
        DiffCache {
            places: places.to_vec(),
            visible,
            impact_diff,
            impact_diff_blocking,
            musician_to_place,
            place_to_musician,
        }
    }

    pub fn find_best_matching(&self) -> (usize, usize, i64) {
        self.impact_diff
            .iter()
            .enumerate()
            .filter(|(i, _)| self.place_to_musician[*i].is_none())
            .map(|(i, impacts)| {
                let (j, impact) = impacts
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| self.musician_to_place[*j].is_none())
                    .max_by_key(|(_, &impact)| impact)
                    .unwrap();
                let penalty: i64 = self.impact_diff_blocking[i].iter().sum();
                (i, j, *impact + penalty)
            })
            .max_by_key(|(_, _, v)| *v)
            .unwrap()
    }

    fn update_direct(&mut self, prob: &Problem, pidx: usize, midx: usize) -> i64 {
        let place_self = self.places[pidx];
        for (i, &place_another) in self.places.iter().enumerate() {
            if self.place_to_musician[i].is_some() {
                continue;
            }
            let block_area_self = Circle {
                c: place_self,
                r: 5.0,
            };
            for (j, atd) in prob.attendees.iter().enumerate() {
                let segment_another = Line {
                    p1: place_another,
                    p2: atd.place(),
                };
                if is_cross_line_circle(segment_another, block_area_self) && self.visible[i][j] {
                    self.visible[i][j] = false;
                    for (k, &kind) in prob.musicians.iter().enumerate() {
                        self.impact_diff[i][k] -= impact_raw(atd, kind, place_another);
                    }
                }
            }
        }
        self.impact_diff[pidx][midx]
    }

    fn update_block_dec(
        &mut self,
        prob: &Problem,
        pidx: usize,
        i: usize,
        place_another: Point,
        midx_another: usize,
    ) {
        let place_self = self.places[pidx];
        let block_area_self = Circle {
            c: place_self,
            r: 5.0,
        };
        for (j, atd) in prob.attendees.iter().enumerate() {
            let segment_another = Line {
                p1: place_another,
                p2: atd.place(),
            };
            if is_cross_line_circle(segment_another, block_area_self) && self.visible[i][j] {
                self.visible[i][j] = false;
                let kind = prob.musicians[midx_another];
                for (ii, &place2) in self.places.iter().enumerate() {
                    if self.place_to_musician[ii].is_some() {
                        continue;
                    }
                    let block_area_2 = Circle { c: place2, r: 5.0 };
                    if is_cross_line_circle(segment_another, block_area_2) {
                        self.impact_diff_blocking[ii][midx_another] +=
                            impact_raw(atd, kind, place_another);
                    }
                }
            }
        }
    }

    fn update_block_inc(
        &mut self,
        prob: &Problem,
        pidx: usize,
        midx: usize,
        i: usize,
        place_another: Point,
    ) {
        let place_self = self.places[pidx];
        let kind_self = prob.musicians[midx];
        let block_area_another = Circle {
            c: place_another,
            r: 5.0,
        };
        for (j, atd) in prob.attendees.iter().enumerate() {
            let segment_self = Line {
                p1: place_self,
                p2: atd.place(),
            };
            if is_cross_line_circle(segment_self, block_area_another) && self.visible[pidx][j] {
                self.impact_diff_blocking[i][midx] -= impact_raw(atd, kind_self, place_self);
            }
        }
    }

    fn update_block(&mut self, prob: &Problem, pidx: usize, midx: usize) -> i64 {
        let mut diff = 0;
        for (k, opt_place) in self.musician_to_place.iter().enumerate() {
            if let Some(_) = opt_place {
                diff += self.impact_diff_blocking[pidx][k];
            }
        }
        let num = self.places.len();
        for i in 0..num {
            if i == pidx {
                continue;
            }
            match self.place_to_musician[i] {
                Some(midx_another) => {
                    self.update_block_dec(prob, pidx, i, self.places[i], midx_another);
                }
                None => {
                    self.update_block_inc(prob, pidx, midx, i, self.places[i]);
                }
            }
        }
        diff
    }

    pub fn add_matching(&mut self, prob: &Problem, pidx: usize, midx: usize) -> i64 {
        assert!(self.musician_to_place[midx].is_none());
        assert!(self.place_to_musician[pidx].is_none());
        self.musician_to_place[midx] = Some(pidx);
        self.place_to_musician[pidx] = Some(midx);
        self.update_direct(prob, pidx, midx) + self.update_block(prob, pidx, midx)
    }
}
