//! TrustMario64 — Skeletal Animation System
//! Body-part-based animation (like SM64) with keyframe interpolation
#![allow(dead_code)]

use alloc::vec::Vec;
use super::physics::{Vec3, PI, fast_sin, fast_cos, lerp_angle};
use crate::math::lerp;

// ======================== Body Parts ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyPart {
    Hips = 0,
    Torso = 1,
    Head = 2,
    LeftUpperArm = 3,
    LeftLowerArm = 4,
    RightUpperArm = 5,
    RightLowerArm = 6,
    LeftUpperLeg = 7,
    LeftLowerLeg = 8,
    RightUpperLeg = 9,
    RightLowerLeg = 10,
    Cap = 11,
}

pub const NUM_BODY_PARTS: usize = 12;

// ======================== Bone Pose ========================

/// Rotation angles (euler XYZ) for one body part in one keyframe
#[derive(Clone, Copy, Debug, Default)]
pub struct BonePose {
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
}

impl BonePose {
    pub const ZERO: BonePose = BonePose { rx: 0.0, ry: 0.0, rz: 0.0 };

    pub fn new(rx: f32, ry: f32, rz: f32) -> Self { Self { rx, ry, rz } }

    pub fn lerp(&self, other: &BonePose, t: f32) -> BonePose {
        BonePose {
            rx: lerp(self.rx, other.rx, t),
            ry: lerp(self.ry, other.ry, t),
            rz: lerp(self.rz, other.rz, t),
        }
    }
}

// ======================== Animation ========================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimId {
    Idle,
    Walk,
    Run,
    Jump,
    DoubleJump,
    TripleJump,
    Fall,
    Land,
    LongJump,
    Backflip,
    SideFlip,
    WallKick,
    GroundPound,
    GroundPoundLand,
    Crouch,
    Slide,
    Swim,
    Damaged,
    Victory,
    Dive,
}

#[derive(Clone)]
pub struct KeyFrame {
    pub time: f32, // 0.0 to 1.0 normalized
    pub root_offset: Vec3, // body center offset
    pub poses: [BonePose; NUM_BODY_PARTS],
}

#[derive(Clone)]
pub struct Animation {
    pub id: AnimId,
    pub frames: Vec<KeyFrame>,
    pub duration: f32,   // seconds
    pub looping: bool,
    pub speed: f32,      // playback speed multiplier
}

// ======================== Animation State ========================

pub struct AnimState {
    pub current: AnimId,
    pub time: f32,        // current time in seconds
    pub speed: f32,       // current playback speed
    pub blending: bool,
    pub blend_from: AnimId,
    pub blend_time: f32,
    pub blend_duration: f32,
    pub current_poses: [BonePose; NUM_BODY_PARTS],
    pub current_root_offset: Vec3,
}

impl AnimState {
    pub fn new() -> Self {
        Self {
            current: AnimId::Idle,
            time: 0.0,
            speed: 1.0,
            blending: false,
            blend_from: AnimId::Idle,
            blend_time: 0.0,
            blend_duration: 0.15,
            current_poses: [BonePose::ZERO; NUM_BODY_PARTS],
            current_root_offset: Vec3::ZERO,
        }
    }

    pub fn play(&mut self, anim: AnimId, speed: f32) {
        if self.current != anim {
            self.blend_from = self.current;
            self.blend_time = 0.0;
            self.blending = true;
            self.current = anim;
            self.time = 0.0;
        }
        self.speed = speed;
    }

    pub fn update(&mut self, dt: f32, library: &AnimLibrary) {
        self.time += dt * self.speed;

        if let Some(anim) = library.get(self.current) {
            if anim.looping {
                while self.time >= anim.duration {
                    self.time -= anim.duration;
                }
            } else if self.time >= anim.duration {
                self.time = anim.duration - 0.001;
            }

            let t = if anim.duration > 0.0 { self.time / anim.duration } else { 0.0 };
            let (poses, root) = sample_animation(anim, t);

            if self.blending {
                self.blend_time += dt;
                let bt = (self.blend_time / self.blend_duration).min(1.0);
                if bt >= 1.0 {
                    self.blending = false;
                }
                // Blend poses
                for i in 0..NUM_BODY_PARTS {
                    self.current_poses[i] = self.current_poses[i].lerp(&poses[i], bt);
                }
                self.current_root_offset = self.current_root_offset.lerp(root, bt);
            } else {
                self.current_poses = poses;
                self.current_root_offset = root;
            }
        }
    }
}

fn sample_animation(anim: &Animation, t: f32) -> ([BonePose; NUM_BODY_PARTS], Vec3) {
    if anim.frames.is_empty() {
        return ([BonePose::ZERO; NUM_BODY_PARTS], Vec3::ZERO);
    }
    if anim.frames.len() == 1 {
        return (anim.frames[0].poses, anim.frames[0].root_offset);
    }

    // Find the two keyframes to interpolate between
    let mut f0 = 0;
    let mut f1 = 1;
    for i in 0..anim.frames.len() - 1 {
        if t >= anim.frames[i].time && t <= anim.frames[i + 1].time {
            f0 = i;
            f1 = i + 1;
            break;
        }
    }

    let frame0 = &anim.frames[f0];
    let frame1 = &anim.frames[f1];
    let range = frame1.time - frame0.time;
    let local_t = if range > 0.001 { (t - frame0.time) / range } else { 0.0 };

    let mut poses = [BonePose::ZERO; NUM_BODY_PARTS];
    for i in 0..NUM_BODY_PARTS {
        poses[i] = frame0.poses[i].lerp(&frame1.poses[i], local_t);
    }
    let root = frame0.root_offset.lerp(frame1.root_offset, local_t);
    (poses, root)
}

// ======================== Animation Library ========================

pub struct AnimLibrary {
    pub anims: Vec<Animation>,
}

impl AnimLibrary {
    pub fn new() -> Self {
        let mut lib = Self { anims: Vec::new() };
        lib.build_all();
        lib
    }

    pub fn get(&self, id: AnimId) -> Option<&Animation> {
        self.anims.iter().find(|a| a.id == id)
    }

    fn build_all(&mut self) {
        self.anims.push(Self::build_idle());
        self.anims.push(Self::build_walk());
        self.anims.push(Self::build_run());
        self.anims.push(Self::build_jump());
        self.anims.push(Self::build_double_jump());
        self.anims.push(Self::build_fall());
        self.anims.push(Self::build_land());
        self.anims.push(Self::build_long_jump());
        self.anims.push(Self::build_backflip());
        self.anims.push(Self::build_ground_pound());
        self.anims.push(Self::build_crouch());
        self.anims.push(Self::build_swim());
        self.anims.push(Self::build_damaged());
        self.anims.push(Self::build_victory());
    }

    // Helper: create keyframe with specific bone overrides
    fn kf(time: f32, overrides: &[(usize, BonePose)]) -> KeyFrame {
        let mut poses = [BonePose::ZERO; NUM_BODY_PARTS];
        for &(idx, pose) in overrides {
            if idx < NUM_BODY_PARTS { poses[idx] = pose; }
        }
        KeyFrame { time, root_offset: Vec3::ZERO, poses }
    }

    fn kf_with_root(time: f32, root: Vec3, overrides: &[(usize, BonePose)]) -> KeyFrame {
        let mut poses = [BonePose::ZERO; NUM_BODY_PARTS];
        for &(idx, pose) in overrides {
            if idx < NUM_BODY_PARTS { poses[idx] = pose; }
        }
        KeyFrame { time, root_offset: root, poses }
    }

    fn build_idle() -> Animation {
        let p = 0.05; // subtle breathing
        Animation {
            id: AnimId::Idle,
            frames: alloc::vec![
                Self::kf(0.0, &[(1, BonePose::new(p, 0.0, 0.0))]),
                Self::kf(0.5, &[(1, BonePose::new(-p, 0.0, 0.0))]),
                Self::kf(1.0, &[(1, BonePose::new(p, 0.0, 0.0))]),
            ],
            duration: 2.0, looping: true, speed: 1.0,
        }
    }

    fn build_walk() -> Animation {
        let leg = 0.4;  // leg swing amplitude
        let arm = 0.3;  // arm swing amplitude
        Animation {
            id: AnimId::Walk,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (7, BonePose::new(leg, 0.0, 0.0)),   // left leg forward
                    (9, BonePose::new(-leg, 0.0, 0.0)),   // right leg back
                    (3, BonePose::new(-arm, 0.0, 0.0)),   // left arm back
                    (5, BonePose::new(arm, 0.0, 0.0)),    // right arm forward
                ]),
                Self::kf(0.5, &[
                    (7, BonePose::new(-leg, 0.0, 0.0)),
                    (9, BonePose::new(leg, 0.0, 0.0)),
                    (3, BonePose::new(arm, 0.0, 0.0)),
                    (5, BonePose::new(-arm, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (7, BonePose::new(leg, 0.0, 0.0)),
                    (9, BonePose::new(-leg, 0.0, 0.0)),
                    (3, BonePose::new(-arm, 0.0, 0.0)),
                    (5, BonePose::new(arm, 0.0, 0.0)),
                ]),
            ],
            duration: 0.6, looping: true, speed: 1.0,
        }
    }

    fn build_run() -> Animation {
        let leg = 0.7;
        let arm = 0.5;
        let knee = 0.6;
        Animation {
            id: AnimId::Run,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(0.1, 0.0, 0.0)),   // lean forward
                    (7, BonePose::new(leg, 0.0, 0.0)),
                    (8, BonePose::new(-knee, 0.0, 0.0)),  // knee bend
                    (9, BonePose::new(-leg, 0.0, 0.0)),
                    (3, BonePose::new(-arm, 0.0, 0.0)),
                    (5, BonePose::new(arm, 0.0, 0.0)),
                ]),
                Self::kf(0.5, &[
                    (0, BonePose::new(0.1, 0.0, 0.0)),
                    (7, BonePose::new(-leg, 0.0, 0.0)),
                    (10, BonePose::new(-knee, 0.0, 0.0)),
                    (9, BonePose::new(leg, 0.0, 0.0)),
                    (3, BonePose::new(arm, 0.0, 0.0)),
                    (5, BonePose::new(-arm, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (0, BonePose::new(0.1, 0.0, 0.0)),
                    (7, BonePose::new(leg, 0.0, 0.0)),
                    (8, BonePose::new(-knee, 0.0, 0.0)),
                    (9, BonePose::new(-leg, 0.0, 0.0)),
                    (3, BonePose::new(-arm, 0.0, 0.0)),
                    (5, BonePose::new(arm, 0.0, 0.0)),
                ]),
            ],
            duration: 0.35, looping: true, speed: 1.0,
        }
    }

    fn build_jump() -> Animation {
        Animation {
            id: AnimId::Jump,
            frames: alloc::vec![
                Self::kf_with_root(0.0, Vec3::new(0.0, 0.0, 0.0), &[
                    (7, BonePose::new(-0.3, 0.0, 0.0)),  // crouch
                    (9, BonePose::new(-0.3, 0.0, 0.0)),
                    (3, BonePose::new(0.0, 0.0, -0.5)),  // arms out
                    (5, BonePose::new(0.0, 0.0, 0.5)),
                ]),
                Self::kf_with_root(0.3, Vec3::new(0.0, 0.1, 0.0), &[
                    (7, BonePose::new(0.2, 0.0, 0.0)),   // legs extend
                    (9, BonePose::new(0.2, 0.0, 0.0)),
                    (3, BonePose::new(-1.2, 0.0, 0.0)),  // fist pump!
                    (5, BonePose::new(-0.5, 0.0, 0.0)),
                ]),
                Self::kf_with_root(1.0, Vec3::new(0.0, 0.0, 0.0), &[
                    (7, BonePose::new(0.1, 0.0, 0.0)),
                    (9, BonePose::new(0.1, 0.0, 0.0)),
                    (3, BonePose::new(-0.8, 0.0, 0.0)),
                    (5, BonePose::new(-0.3, 0.0, 0.0)),
                ]),
            ],
            duration: 0.5, looping: false, speed: 1.0,
        }
    }

    fn build_double_jump() -> Animation {
        Animation {
            id: AnimId::DoubleJump,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(0.0, 0.0, 0.0)),
                    (3, BonePose::new(0.0, 0.0, -1.5)),  // T-pose for spin
                    (5, BonePose::new(0.0, 0.0, 1.5)),
                ]),
                Self::kf(0.25, &[
                    (0, BonePose::new(PI, 0.0, 0.0)),    // full front flip
                    (3, BonePose::new(0.0, 0.0, -1.5)),
                    (5, BonePose::new(0.0, 0.0, 1.5)),
                ]),
                Self::kf(0.5, &[
                    (0, BonePose::new(PI * 2.0, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[]),
            ],
            duration: 0.6, looping: false, speed: 1.0,
        }
    }

    fn build_fall() -> Animation {
        Animation {
            id: AnimId::Fall,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (3, BonePose::new(-0.5, 0.0, -0.8)),  // arms spread
                    (5, BonePose::new(-0.5, 0.0, 0.8)),
                    (7, BonePose::new(0.2, 0.0, 0.0)),
                    (9, BonePose::new(0.2, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (3, BonePose::new(-0.3, 0.0, -0.9)),
                    (5, BonePose::new(-0.3, 0.0, 0.9)),
                    (7, BonePose::new(0.3, 0.0, 0.0)),
                    (9, BonePose::new(0.3, 0.0, 0.0)),
                ]),
            ],
            duration: 0.8, looping: true, speed: 1.0,
        }
    }

    fn build_land() -> Animation {
        Animation {
            id: AnimId::Land,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (7, BonePose::new(-0.5, 0.0, 0.0)),  // crouch on landing
                    (9, BonePose::new(-0.5, 0.0, 0.0)),
                    (8, BonePose::new(-0.3, 0.0, 0.0)),
                    (10, BonePose::new(-0.3, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[]),
            ],
            duration: 0.2, looping: false, speed: 1.0,
        }
    }

    fn build_long_jump() -> Animation {
        Animation {
            id: AnimId::LongJump,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(0.8, 0.0, 0.0)),   // lean forward
                    (3, BonePose::new(-1.0, 0.0, 0.0)),  // arms back
                    (5, BonePose::new(-1.0, 0.0, 0.0)),
                    (7, BonePose::new(-0.2, 0.0, 0.0)),
                    (9, BonePose::new(0.5, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (0, BonePose::new(0.6, 0.0, 0.0)),
                    (3, BonePose::new(-0.8, 0.0, 0.0)),
                    (5, BonePose::new(-0.8, 0.0, 0.0)),
                ]),
            ],
            duration: 0.8, looping: false, speed: 1.0,
        }
    }

    fn build_backflip() -> Animation {
        Animation {
            id: AnimId::Backflip,
            frames: alloc::vec![
                Self::kf(0.0, &[(0, BonePose::new(0.0, 0.0, 0.0))]),
                Self::kf(0.25, &[(0, BonePose::new(-PI, 0.0, 0.0))]),
                Self::kf(0.5, &[(0, BonePose::new(-PI * 2.0, 0.0, 0.0))]),
                Self::kf(1.0, &[(0, BonePose::new(0.0, 0.0, 0.0))]),
            ],
            duration: 0.7, looping: false, speed: 1.0,
        }
    }

    fn build_ground_pound() -> Animation {
        Animation {
            id: AnimId::GroundPound,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(0.0, PI, 0.0)),     // spin
                    (3, BonePose::new(0.0, 0.0, -1.5)),   // arms out
                    (5, BonePose::new(0.0, 0.0, 1.5)),
                ]),
                Self::kf(0.3, &[
                    (0, BonePose::new(0.0, 0.0, 0.0)),
                    (7, BonePose::new(0.8, 0.0, 0.0)),    // butt-first
                    (9, BonePose::new(0.8, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (7, BonePose::new(0.8, 0.0, 0.0)),
                    (9, BonePose::new(0.8, 0.0, 0.0)),
                ]),
            ],
            duration: 0.4, looping: false, speed: 1.0,
        }
    }

    fn build_crouch() -> Animation {
        Animation {
            id: AnimId::Crouch,
            frames: alloc::vec![
                Self::kf_with_root(0.0, Vec3::new(0.0, -0.3, 0.0), &[
                    (7, BonePose::new(-0.8, 0.0, 0.1)),
                    (9, BonePose::new(-0.8, 0.0, -0.1)),
                    (8, BonePose::new(-0.5, 0.0, 0.0)),
                    (10, BonePose::new(-0.5, 0.0, 0.0)),
                ]),
            ],
            duration: 0.3, looping: false, speed: 1.0,
        }
    }

    fn build_swim() -> Animation {
        Animation {
            id: AnimId::Swim,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(0.5, 0.0, 0.0)),
                    (3, BonePose::new(-0.5, 0.0, -1.0)),
                    (5, BonePose::new(-0.5, 0.0, 1.0)),
                    (7, BonePose::new(0.0, 0.0, 0.0)),
                    (9, BonePose::new(0.0, 0.0, 0.0)),
                ]),
                Self::kf(0.5, &[
                    (0, BonePose::new(0.5, 0.0, 0.0)),
                    (3, BonePose::new(0.5, 0.0, -0.3)),
                    (5, BonePose::new(0.5, 0.0, 0.3)),
                    (7, BonePose::new(-0.5, 0.0, 0.0)),
                    (9, BonePose::new(-0.5, 0.0, 0.0)),
                ]),
                Self::kf(1.0, &[
                    (0, BonePose::new(0.5, 0.0, 0.0)),
                    (3, BonePose::new(-0.5, 0.0, -1.0)),
                    (5, BonePose::new(-0.5, 0.0, 1.0)),
                ]),
            ],
            duration: 0.8, looping: true, speed: 1.0,
        }
    }

    fn build_damaged() -> Animation {
        Animation {
            id: AnimId::Damaged,
            frames: alloc::vec![
                Self::kf(0.0, &[
                    (0, BonePose::new(-0.3, 0.0, 0.0)),
                    (3, BonePose::new(0.0, 0.0, -1.2)),
                    (5, BonePose::new(0.0, 0.0, 1.2)),
                ]),
                Self::kf(1.0, &[]),
            ],
            duration: 0.5, looping: false, speed: 1.0,
        }
    }

    fn build_victory() -> Animation {
        Animation {
            id: AnimId::Victory,
            frames: alloc::vec![
                Self::kf_with_root(0.0, Vec3::new(0.0, 0.0, 0.0), &[
                    (3, BonePose::new(-2.5, 0.0, 0.0)),   // right arm up!
                ]),
                Self::kf_with_root(0.3, Vec3::new(0.0, 0.5, 0.0), &[
                    (3, BonePose::new(-2.8, 0.0, 0.0)),
                    (7, BonePose::new(-0.3, 0.0, 0.0)),
                    (9, BonePose::new(-0.3, 0.0, 0.0)),
                ]),
                Self::kf_with_root(0.6, Vec3::new(0.0, 0.0, 0.0), &[
                    (3, BonePose::new(-2.5, 0.0, 0.0)),
                ]),
                Self::kf_with_root(1.0, Vec3::new(0.0, 0.0, 0.0), &[]),
            ],
            duration: 1.5, looping: false, speed: 1.0,
        }
    }
}
