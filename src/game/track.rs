use std::cell::RefCell;

use physics::advance_frame::frame_after;

use crate::game::line::Line;
use crate::game::vector::Vector2D;
use crate::linestore::grid::Grid;
use crate::rider::{Entity, EntityPoint};
use crate::{physics, LineBuilder, DEBUG_PRINT};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct TrackMeta {
    line_extension_ratio: f64,
    gravity_well_height: f64,
    // Suggestion: This should probably be per-rider instead of per-engine
    remount: bool,
    cell_size: f64,
}

impl Default for TrackMeta {
    fn default() -> Self {
        TrackMeta {
            line_extension_ratio: 0.25,
            cell_size: 14.0,
            gravity_well_height: 10.0,
            remount: false,
        }
    }
}

/// A track in linerider.
#[derive(Debug)]
pub struct Track {
    pub meta: TrackMeta,

    grid: Grid,

    precomputed_rider_positions: RefCell<Vec<Vec<Entity>>>,
}

impl Track {
    pub fn new(starting_positions: Vec<Entity>, lines: Vec<Line>) -> Track {
        let meta: TrackMeta = Default::default();
        Track {
            meta,
            grid: Grid::new(lines, meta.cell_size),
            precomputed_rider_positions: RefCell::new(vec![starting_positions]),
        }
    }
    pub fn new_with_meta(
        starting_positions: Vec<Entity>,
        lines: Vec<Line>,
        meta: TrackMeta,
    ) -> Track {
        Track {
            meta,
            grid: Grid::new(lines, meta.cell_size),
            precomputed_rider_positions: RefCell::new(vec![starting_positions]),
        }
    }

    pub fn line_builder(&self) -> LineBuilder {
        Line::builder().extension_ratio(self.meta.line_extension_ratio)
    }

    /// Gets all lines in the track.
    pub fn all_lines(&self) -> &Vec<Line> {
        self.grid.all_lines()
    }

    /// Adds a line to the track.
    pub fn add_line(&mut self, line: Line) {
        self.grid.add_line(line);
        self.precomputed_rider_positions.borrow_mut().drain(1..);
    }

    /// Removes a single line from the track.
    pub fn remove_line(&mut self, line: &Line) {
        self.grid.remove_line(line);
        self.precomputed_rider_positions.borrow_mut().drain(1..);
    }

    /// Gets all of the lines near a point.
    pub fn lines_near(&self, point: Vector2D) -> Vec<&Line> {
        self.grid.lines_near(point, 1)
    }

    /// Gets all of the lines in a rectangle.
    pub fn lines_near_box(&self, p1: Vector2D, p2: Vector2D) -> Vec<&Line> {
        self.grid.lines_near_box(p1, p2)
    }

    /// Gets the rider positions for a zero-indexed frame.
    pub fn entity_positions_at(&self, frame: usize) -> Vec<Entity> {
        let mut position_cache = self.precomputed_rider_positions.borrow_mut();
        if let Some(riders) = position_cache.get(frame) {
            riders.clone()
        } else {
            let len = position_cache.len();
            for i in len..=frame {
                if DEBUG_PRINT {
                    println!("Frame {}", i);
                }
                let next_positions = frame_after(position_cache.last().unwrap(), self);
                position_cache.push(next_positions);
            }

            position_cache.last().unwrap().clone()
        }
    }

    /// Adds a new rider to the track.
    pub fn create_entity(&mut self, entity: Entity) {
        let position_cache = self.precomputed_rider_positions.get_mut();
        let initial_frame = position_cache.get_mut(0).unwrap();
        initial_frame.push(entity);

        position_cache.drain(1..);
    }

    /// Removes a rider from the track.
    pub fn remove_entity(&mut self, entity: Entity) -> Option<()> {
        let position_cache = self.precomputed_rider_positions.get_mut();
        let initial_frame = position_cache.get_mut(0).unwrap();
        initial_frame.remove(initial_frame.iter().position(|e| *e == entity)?);

        position_cache.drain(1..);
        Some(())
    }

    /// Snaps a point to the nearest line ending, or returns `to_snap` if
    /// there are no nearby points.
    pub fn snap_point(&self, max_dist: f64, to_snap: Vector2D) -> Vector2D {
        let max_dist_sq = max_dist * max_dist;

        self.lines_near(to_snap)
            .iter()
            .flat_map(|l| [l.ends.0.location, l.ends.1.location])
            .map(|p| (p, p.distance_squared(to_snap)))
            .filter(|(_, dist)| dist.total_cmp(&max_dist_sq).is_lt())
            .min_by(|(_, d1), (_, d2)| d1.total_cmp(d2))
            .unwrap_or((to_snap, 0.0))
            .0
    }

    /// Returns the distance below the line, or 0 if applicable. "below" is the direction
    /// 90 degrees to the right of the vector created from `self.points.0` to `self.points.1`.
    ///
    /// Returns 0 when:
    ///  * the point is above the line
    ///  * the point is moving "upward"
    ///  * the point is outside of the line, including extensions
    pub fn distance_below_line(&self, line: &Line, point: &EntityPoint) -> f64 {
        let line_vec = line.as_vector2d();
        let point_from_start = point.location - line.ends.0.location;
        let perpendicular = line.perpendicular();

        let is_moving_into_line = {
            let dot = perpendicular.dot_product(point.momentum);
            dot < 0.0
        };
        if !is_moving_into_line {
            return 0.0;
        }

        let line_length = line_vec.length_squared().sqrt();
        let line_normalized = line_vec / line_length;

        let (ext_l, ext_r) = line.hitbox_extensions();
        let parallel_component = point_from_start.dot_product(line_normalized);
        if parallel_component < -ext_l || ext_r + line_length < parallel_component {
            return 0.0;
        }

        let distance_below = (-perpendicular).dot_product(point_from_start);
        if 0.0 < distance_below && distance_below < self.meta.gravity_well_height {
            distance_below
        } else {
            0.0
        }
    }
}

impl Clone for Track {
    fn clone(&self) -> Self {
        Track {
            meta: self.meta.clone(),
            grid: self.grid.clone(),
            precomputed_rider_positions: self.precomputed_rider_positions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use lr_formatter_rs::trackjson::read;
    use std::fs;
    use std::vec;

    use crate::rider::PointIndex;
    use crate::{rider::Entity, Line, LineType, Track, Vector2D};

    // Suggestion: Implement entity.avg_position by averaging the position of all entity points (and similar for velocity)
    // Custom function to average entity vectors together
    fn average(entity: &Entity) -> Vector2D {
        let mut result = Vector2D(0.0, 0.0);

        for (_, vector) in entity.points.iter() {
            result.0 += vector.location.0;
            result.1 += vector.location.1;
        }

        if entity.points.len() > 0 {
            result.0 /= entity.points.len() as f64;
            result.1 /= entity.points.len() as f64;
        }

        return result;
    }

    #[test]
    fn zero_lines() {
        let engine = Track::new(vec![Entity::default_boshsled()], vec![]);
        let entities0 = engine.entity_positions_at(0);
        let entities1 = engine.entity_positions_at(1);
        let rider0 = entities0.get(0).expect("Rider frame 0 should exist");
        let rider1 = entities1.get(0).expect("Rider frame 1 should exist");
        let avg_position0 = average(rider0);
        let avg_position1 = average(rider1);
        assert!(
            avg_position1.0 > avg_position0.0,
            "Rider should have increased x"
        );
        assert!(
            avg_position1.1 > avg_position0.1,
            "Rider should have increased y"
        );
    }

    #[test]
    fn one_line() {
        let y = 5.0;
        let line = Line::builder()
            .id(0)
            .line_type(LineType::Normal)
            .point(0.0, 5.0)
            .point(30.0, 5.0)
            .build();
        let frame = 11;

        let engine0 = Track::new(vec![Entity::default_boshsled()], vec![]);
        let mut engine1 = engine0.clone();
        engine1.add_line(line);

        let entities0 = engine0.entity_positions_at(frame);
        let entities1 = engine1.entity_positions_at(frame);

        let rider_falling = entities0.get(0).expect("Rider falling should exist");
        let rider_colliding = entities1.get(0).expect("Rider colliding should exist");

        let rider_falling_points = average(rider_falling);
        let rider_colliding_points = average(rider_colliding);

        assert!(
            rider_colliding_points.1 < rider_falling_points.1,
            "Rider should have collided with line"
        );
        assert!(
            rider_colliding_points.1 < y - 0.01,
            "Rider should not have been flattened"
        );
    }

    #[test]
    fn two_lines() {
        let y = 5.0;
        let line1 = Line::builder()
            .id(0)
            .line_type(LineType::Normal)
            .point(0.0, 5.0)
            .point(30.0, 5.0)
            .build();
        let line2 = Line::builder()
            .id(1)
            .line_type(LineType::Normal)
            .point(-7.0, 0.0)
            .point(-7.0, 10.0)
            .flipped(true)
            .build();
        let frame = 15;

        let engine = Track::new(vec![Entity::default_boshsled()], vec![line1, line2]);
        let entities = engine.entity_positions_at(frame);
        let rider = entities.get(0).expect("Rider should exist");
        let sled = entities
            .get(1)
            .expect("Sled should exist as separate entity");
        let shoulder = rider.point_at(PointIndex::BoshShoulder);
        let butt = rider.point_at(PointIndex::BoshButt);
        let nose = sled.point_at(PointIndex::SledNose);

        println!(
            "Shoulder ({}, {})",
            shoulder.location.0, shoulder.location.1
        );
        println!("Nose ({}, {})", shoulder.location.0, shoulder.location.1);
        println!("Butt ({}, {})", butt.location.0, butt.location.1);
        assert!(
            shoulder.location.0 > 0.0 && nose.location.0 < 0.0,
            "Rider should have been separated from sled"
        );
        assert!(
            shoulder.location.1 < y - 0.01,
            "Rider should not have been flattened"
        );
        assert!(
            butt.location.1 > y - 0.01
                && butt.location.0 > 0.0
                && shoulder.location.0 > butt.location.0,
            "Rider should be sitting and leaning forward"
        );
    }

    #[test]
    fn scenery_line() {
        let line = Line::builder()
            .id(0)
            .line_type(LineType::Scenery)
            .point(0.0, 5.0)
            .point(30.0, 5.0)
            .build();
        let frame = 11;

        let engine0 = Track::new(vec![Entity::default_boshsled()], vec![]);
        let mut engine1 = engine0.clone();
        engine1.add_line(line);

        let entities0 = engine0.entity_positions_at(frame);
        let entities1 = engine1.entity_positions_at(frame);

        let rider_falling = entities0.get(0).expect("Rider falling should exist");
        let rider_colliding = entities1.get(0).expect("Rider colliding should exist");

        let rider_falling_points = average(rider_falling);
        let rider_colliding_points = average(rider_colliding);

        assert!(
            rider_colliding_points.1 == rider_falling_points.1,
            "Rider should not have collided with line"
        );
    }

    #[test]
    fn crash() {
        let track_bytes =
            fs::read_to_string("./fixtures/crash.track.json").expect("Failed to read file");
        let track = read(&track_bytes).expect("Failed to parse file");
        println!("{}", track.title);
    }

    #[test]
    fn cycloid() {
        let track_bytes =
            fs::read_to_string("./fixtures/cycloid.track.json").expect("Failed to read file");
        let track = read(&track_bytes).expect("Failed to parse file");
        println!("{}", track.title);
    }

    #[test]
    fn legacy_test() {
        let track_bytes = fs::read_to_string("./fixtures/legacyTestTrack.track.json")
            .expect("Failed to read file");
        let track = read(&track_bytes).expect("Failed to parse file");
        println!("{}", track.title);
    }

    #[test]
    fn modern_test() {
        let track_bytes =
            fs::read_to_string("./fixtures/testTrack.track.json").expect("Failed to read file");
        let track = read(&track_bytes).expect("Failed to parse file");
        println!("{}", track.title);
    }
}
