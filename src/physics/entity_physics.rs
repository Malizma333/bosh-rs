use crate::game::Track;
use crate::game::Vector2D;
use crate::physics::bone_physics::{joint_should_break, next_bone_locations};
use crate::physics::line_physics::apply_gravity_wells;
use crate::rider::{Entity, EntityPoint};
use crate::DEBUG_PRINT;

pub type PhysicsEntity = Entity;

impl PhysicsEntity {
    /// Pushes the points of `self` in accordance to gravity well logic.
    pub fn apply_gravity_wells(&mut self, track: &Track) {
        self.mutate_points(|p| apply_gravity_wells(p, track))
    }

    /// Applies bone physics to a list of bones. Moves self because
    /// a BoshSled may break, causing `self` to become unusable.
    pub fn apply_bones(mut self) -> UpdateBonesResult {
        let mut broken = false;
        for (i, bone) in self.bones.clone().iter().enumerate() {
            if DEBUG_PRINT {
                println!("Subiteration {}", i);
            }
            if let Some((next_p1, next_p2)) = next_bone_locations(&bone, &self, broken) {
                self.point_at_mut(bone.p1).location = next_p1;
                self.point_at_mut(bone.p2).location = next_p2;
                if DEBUG_PRINT {
                    print_points(self.clone());
                }
            } else {
                broken = true;
                if DEBUG_PRINT {
                    println!("Break");
                }
            }
        }

        if broken {
            let (bosh, sled) = self.split();
            UpdateBonesResult::Broken(bosh, sled)
        } else {
            UpdateBonesResult::Same(self)
        }
    }

    /// Performs the logic of stepping the points of the rider to the next frame.
    /// Does not actually do any physics besides applying gravity.
    pub fn next_points(&mut self, gravity: Vector2D) {
        self.mutate_points(|p| {
            // Suggestion: This function should probably also apply friction and acceleration since it basically serves as the momentum tick:
            // 0.0: Momentum from previous frame
            // 0.1: Friction
            // 0.2: Acceleration
            // 0.3 (Iteration 0): Gravity

            let new_velocity = (p.location - p.previous_location) + gravity;

            *p = EntityPoint {
                previous_location: p.location,
                location: p.location + new_velocity,
                momentum: new_velocity,
                friction: p.friction,
            };
        })
    }

    /// applies joint logic
    /// does nothing on non-boshsleds
    pub fn apply_all_joints(self) -> UpdateBonesResult {
        if self.joints.iter().any(|j| joint_should_break(j, &self)) {
            let (bosh, sled) = self.split();
            UpdateBonesResult::Broken(bosh, sled)
        } else {
            UpdateBonesResult::Same(self)
        }
    }

    /// Applies all physics steps to the rider in the correct order.
    /// Moves `self` because it may become unusable after the sled breaks.
    pub fn apply_all_physics_ez(self, track: &Track) -> UpdateBonesResult {
        self.apply_all_physics(track, Vector2D(0.0, 0.175), 6)
    }

    /// Applies all physics steps to the rider in the correct order.
    /// Moves `self` because it may become unusable after the sled breaks.
    pub fn apply_all_physics(
        mut self,
        track: &Track,
        gravity: Vector2D,
        iterations: u64,
    ) -> UpdateBonesResult {
        self.next_points(gravity);

        if DEBUG_PRINT {
            println!("\nIteration {}", 0);
            print_points(self.clone());
        }

        let mut result = UpdateBonesResult::Same(self);

        for i in 0..iterations {
            if DEBUG_PRINT {
                println!("\nEnter iteration {}", i + 1);
            }
            result = match result {
                UpdateBonesResult::Same(same) => same.apply_bones(),
                UpdateBonesResult::Broken(bosh, sled) => {
                    let bosh = bosh.apply_bones().unwrap_same();
                    let sled = sled.apply_bones().unwrap_same();

                    UpdateBonesResult::Broken(bosh, sled)
                }
            };
            if DEBUG_PRINT {
                println!("Iteration {}", i + 1);
            }
            match &mut result {
                UpdateBonesResult::Same(same) => {
                    same.apply_gravity_wells(track);
                    if DEBUG_PRINT {
                        print_points(same.clone());
                    }
                }
                UpdateBonesResult::Broken(bosh, sled) => {
                    bosh.apply_gravity_wells(track);
                    sled.apply_gravity_wells(track);
                    if DEBUG_PRINT {
                        print_points(sled.clone());
                        print_points(bosh.clone());
                    }
                }
            };
        }

        match result {
            UpdateBonesResult::Same(same) => same.apply_all_joints(),
            UpdateBonesResult::Broken(_, _) => result,
        }
    }
}

#[derive(Clone)]
pub enum UpdateBonesResult {
    Same(PhysicsEntity),
    Broken(PhysicsEntity, PhysicsEntity),
}

impl UpdateBonesResult {
    pub fn unwrap_same(self) -> PhysicsEntity {
        if let UpdateBonesResult::Same(entity) = self {
            entity
        } else {
            panic!("unwrap_same called on UpdateBonesResult::Broken")
        }
    }
}

/// Print points in order of LRO coordinate menu for quick diff comparisons
fn print_points(entity: Entity) {
    print_point(&entity.points, "SledTL", crate::rider::PointIndex::SledPeg);
    print_point(&entity.points, "SledBL", crate::rider::PointIndex::SledTail);
    print_point(&entity.points, "SledBR", crate::rider::PointIndex::SledNose);
    print_point(&entity.points, "SledTR", crate::rider::PointIndex::SledRope);
    print_point(&entity.points, "BodyBu", crate::rider::PointIndex::BoshButt);
    print_point(
        &entity.points,
        "BodySh",
        crate::rider::PointIndex::BoshShoulder,
    );
    print_point(
        &entity.points,
        "BodyHL",
        crate::rider::PointIndex::BoshLeftHand,
    );
    print_point(
        &entity.points,
        "BodyHR",
        crate::rider::PointIndex::BoshRightHand,
    );
    print_point(
        &entity.points,
        "BodyFL",
        crate::rider::PointIndex::BoshLeftFoot,
    );
    print_point(
        &entity.points,
        "BodyFR",
        crate::rider::PointIndex::BoshRightFoot,
    );
}

fn print_point(
    points: &std::collections::HashMap<crate::rider::PointIndex, crate::rider::EntityPoint>,
    label: &str,
    index: crate::rider::PointIndex,
) {
    if let Some(p) = points.get(&index) {
        println!("{}: ({:?})", label, p.location);
    }
}
