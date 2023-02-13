use std::{cell::RefCell, f32::consts::PI, rc::Rc};

use glam::{vec3, Quat, Vec3};
use wasm_bindgen::prelude::*;

const MAX_ROTATION_PER_SUBSTEP: f32 = 0.5;
const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 60.0;
const GRAB_JOINT_COMPLIANCE: f32 = 10.0;

#[derive(Clone, Copy, Default)]
pub struct Pose {
    p: Vec3,
    q: Quat,
}

impl Pose {
    fn rotate(&self, v: &mut Vec3) {
        *v = self.q.mul_vec3(*v);
    }

    fn inv_rotate(&self, v: &mut Vec3) {
        *v = self.q.conjugate().mul_vec3(*v);
    }

    #[allow(dead_code)]
    fn transform(&self, v: &mut Vec3) {
        *v = self.q.mul_vec3(*v);
        *v += self.p;
    }

    fn inv_transform(&self, v: &mut Vec3) {
        *v -= self.p;
        self.inv_rotate(v);
    }

    fn transform_pose(&self, pose: &mut Pose) {
        pose.q = self.q * pose.q;
        self.rotate(&mut pose.p);
        pose.p += self.p;
    }
}

fn get_quat_axis_0(q: Quat) -> Vec3 {
    let x2 = q.x * 2.0;
    let w2 = q.w * 2.0;
    Vec3::new(
        (q.w * w2) - 1.0 + q.x * x2,
        (q.z * w2) + q.y * x2,
        (-q.y * w2) + q.z * x2,
    )
}

fn get_quat_axis_1(q: Quat) -> Vec3 {
    let y2 = q.y * 2.0;
    let w2 = q.w * 2.0;
    Vec3::new(
        (-q.z * w2) + q.x * y2,
        (q.w * w2) - 1.0 + q.y * y2,
        (q.x * w2) + q.z * y2,
    )
}

#[allow(dead_code)]
fn get_quat_axis_2(q: Quat) -> Vec3 {
    let z2 = q.z * 2.0;
    let w2 = q.w * 2.0;
    Vec3::new(
        (q.y * w2) + q.x * z2,
        (-q.x * w2) + q.y * z2,
        (q.w * w2) - 1.0 + q.z * z2,
    )
}

pub struct Body {
    pose: Pose,
    prev_pose: Pose,
    vel: Vec3,
    omega: Vec3,

    inv_mass: f32,
    inv_inertia: Vec3,
}

impl Body {
    fn new(pose: Pose) -> Self {
        Self {
            pose,
            prev_pose: pose,
            vel: Vec3::ZERO,
            omega: Vec3::ZERO,

            inv_mass: 1.0,
            inv_inertia: Vec3::ONE,
        }
    }

    fn set_box(&mut self, size: Vec3, density: f32) {
        let mut mass = size.x * size.y * size.z * density;
        self.inv_mass = 1.0 / mass;
        mass /= 12.0;
        self.inv_inertia = Vec3::new(
            1.0 / (size.y * size.y + size.z * size.z) / mass,
            1.0 / (size.z * size.z + size.x * size.x) / mass,
            1.0 / (size.x * size.x + size.y * size.y) / mass,
        );
    }

    fn apply_rotation(&mut self, rot: Vec3, scale: Option<f32>) {
        let mut scale = if let Some(s) = scale { s } else { 1.0 };

        // Safety clamping. This happens very rarely if the solver
        // wants to turn the body by more than 30 degrees in the
        // orders of milliseconds.
        let phi = rot.length();
        if phi * scale > MAX_ROTATION_PER_SUBSTEP {
            scale = MAX_ROTATION_PER_SUBSTEP / phi;
        }
        let mut dq = Quat::from_xyzw(rot.x * scale, rot.y * scale, rot.z * scale, 0.0);
        dq = dq.mul_quat(self.pose.q);
        self.pose.q = Quat::from_xyzw(
            self.pose.q.x + 0.5 * dq.x,
            self.pose.q.y + 0.5 * dq.y,
            self.pose.q.z + 0.5 * dq.z,
            self.pose.q.w + 0.5 * dq.w,
        )
        .normalize();
    }

    fn integrate(&mut self, dt: f32) {
        self.prev_pose = self.pose;
        self.vel += GRAVITY * dt;
        self.pose.p += self.vel * dt;
        self.apply_rotation(self.omega, Some(dt));
    }

    fn update(&mut self, dt: f32) {
        self.vel = self.pose.p - self.prev_pose.p;
        self.vel *= 1.0 / dt;
        let dq = self.pose.q * self.prev_pose.q.conjugate();
        self.omega = dq.xyz() * 2.0 / dt;
        if dq.w < 0.0 {
            self.omega = -self.omega;
        }
    }

    fn get_vel_at(&self, p: Vec3) -> Vec3 {
        let mut vel = p - self.pose.p;
        vel = vel.cross(self.omega);
        self.vel - vel
    }

    fn get_inv_mass(&self, normal: Vec3, pos: Option<Vec3>) -> f32 {
        let mut n = if let Some(p) = pos {
            (p - self.pose.p).cross(normal)
        } else {
            normal
        };

        self.pose.inv_rotate(&mut n);
        let mut w = n.x * n.x * self.inv_inertia.x
            + n.y * n.y * self.inv_inertia.y
            + n.z * n.z * self.inv_inertia.z;
        if pos.is_some() {
            w += self.inv_mass;
        }
        w
    }

    fn apply_correction(&mut self, corr: Vec3, pos: Option<Vec3>, vel_level: bool) {
        let mut dq = if let Some(p) = pos {
            if vel_level {
                self.vel += corr * self.inv_mass;
            } else {
                self.pose.p += corr * self.inv_mass;
            }
            (p - self.pose.p).cross(corr)
        } else {
            corr
        };

        self.pose.inv_rotate(&mut dq);
        dq = Vec3::new(
            self.inv_inertia.x * dq.x,
            self.inv_inertia.y * dq.y,
            self.inv_inertia.z * dq.z,
        );
        self.pose.rotate(&mut dq);
        if vel_level {
            self.omega += dq;
        } else {
            self.apply_rotation(dq, None);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_pair_correction(
        body0: &Option<Rc<RefCell<Self>>>,
        body1: &Option<Rc<RefCell<Self>>>,
        corr: Vec3,
        compliance: f32,
        dt: f32,
        pos0: Option<Vec3>,
        pos1: Option<Vec3>,
        vel_level: bool,
    ) {
        let c = corr.length();
        if c == 0.0 {
            return;
        }

        let mut normal = corr.normalize();
        let w0 = if let Some(body0) = &body0 {
            body0.borrow().get_inv_mass(normal, pos0)
        } else {
            0.0
        };
        let w1 = if let Some(body1) = &body1 {
            body1.borrow().get_inv_mass(normal, pos1)
        } else {
            0.0
        };

        let w = w0 + w1;
        if w == 0.0 {
            return;
        }

        let lambda = -c / (w + compliance / dt / dt);
        normal *= -lambda;
        if let Some(body0) = &body0 {
            body0.borrow_mut().apply_correction(normal, pos0, vel_level);
        }
        if let Some(body1) = &body1 {
            normal *= -1.0;
            body1.borrow_mut().apply_correction(normal, pos1, vel_level);
        }
    }

    /// The key function to handle all angular joint limits
    #[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
    fn limit_angle(
        body0: &Option<Rc<RefCell<Self>>>,
        body1: &Option<Rc<RefCell<Self>>>,
        n: Vec3,
        a: Vec3,
        b: Vec3,
        min_angle: f32,
        max_angle: f32,
        compliance: f32,
        dt: f32,
        max_corr: f32,
    ) {
        let c = a.cross(b);

        let mut phi = f32::asin(c.dot(n));
        if a.dot(b) < 0.0 {
            phi = PI - phi;
        }

        if phi > PI {
            phi -= 2.0 * PI;
        } else if phi < -PI {
            phi += 2.0 * PI;
        }

        if phi < min_angle || phi > max_angle {
            phi = phi.clamp(min_angle, max_angle);

            let q = Quat::from_axis_angle(n, phi);
            let mut omega = (q.mul_vec3(a)).cross(b);

            phi = omega.length();
            if phi > max_corr {
                omega *= max_corr / phi;
            }

            Self::apply_pair_correction(body0, body1, omega, compliance, dt, None, None, false);
        }
    }
}

#[allow(dead_code)]
enum JointKind {
    Spherical,
    Hinge,
    Fixed,
}

pub struct Joint {
    kind: JointKind,
    body0: Option<Rc<RefCell<Body>>>,
    body1: Option<Rc<RefCell<Body>>>,
    local_pose_0: Pose,
    local_pose_1: Pose,
    global_pose_0: Pose,
    global_pose_1: Pose,

    compliance: f32,
    rot_damping: f32,
    pos_damping: f32,
    has_swing_limits: bool,
    min_swing_angle: f32,
    max_swing_angle: f32,
    swing_limits_compliance: f32,
    has_twist_limits: bool,
    min_twist_angle: f32,
    max_twist_angle: f32,
    twist_limits_compliance: f32,
}

impl Joint {
    fn new(
        kind: JointKind,
        body0: Option<Rc<RefCell<Body>>>,
        body1: Option<Rc<RefCell<Body>>>,
        local_pose_0: Pose,
        local_pose_1: Pose,
    ) -> Self {
        Self {
            kind,
            body0,
            body1,
            local_pose_0,
            local_pose_1,
            global_pose_0: local_pose_0,
            global_pose_1: local_pose_1,

            compliance: 0.0,
            rot_damping: 0.0,
            pos_damping: 0.0,
            has_swing_limits: false,
            min_swing_angle: -2.0 * PI,
            max_swing_angle: 2.0 * PI,
            swing_limits_compliance: 0.0,
            has_twist_limits: false,
            min_twist_angle: -2.0 * PI,
            max_twist_angle: 2.0 * PI,
            twist_limits_compliance: 0.0,
        }
    }

    fn update_global_poses(&mut self) {
        self.global_pose_0 = self.local_pose_0;
        if let Some(body0) = &self.body0 {
            body0.borrow().pose.transform_pose(&mut self.global_pose_0);
        }
        self.global_pose_1 = self.local_pose_1;
        if let Some(body1) = &self.body1 {
            body1.borrow().pose.transform_pose(&mut self.global_pose_1);
        }
    }

    #[allow(clippy::too_many_lines)]
    fn solve_pos(&mut self, dt: f32) {
        self.update_global_poses();

        // orientation
        match self.kind {
            JointKind::Fixed => {
                let mut q = self.global_pose_0.q.conjugate();
                q = self.global_pose_1.q * q;
                let mut omega = q.xyz() * 2.0;
                if q.w < 0.0 {
                    omega *= -1.0;
                }
                Body::apply_pair_correction(
                    &self.body0,
                    &self.body1,
                    omega,
                    self.compliance,
                    dt,
                    None,
                    None,
                    false,
                );
            }
            JointKind::Hinge => {
                // align axes
                let mut a0 = get_quat_axis_0(self.global_pose_0.q);
                let a1 = get_quat_axis_0(self.global_pose_1.q);
                a0 = a0.cross(a1);
                Body::apply_pair_correction(
                    &self.body0,
                    &self.body1,
                    a0,
                    0.0,
                    dt,
                    None,
                    None,
                    false,
                );

                // limits
                if self.has_swing_limits {
                    self.update_global_poses();
                    let n = get_quat_axis_0(self.global_pose_0.q);
                    let b0 = get_quat_axis_1(self.global_pose_0.q);
                    let b1 = get_quat_axis_1(self.global_pose_1.q);
                    Body::limit_angle(
                        &self.body0,
                        &self.body1,
                        n,
                        b0,
                        b1,
                        self.min_swing_angle,
                        self.max_swing_angle,
                        self.swing_limits_compliance,
                        dt,
                        PI,
                    );
                }
            }
            JointKind::Spherical => {
                // swing limits
                if self.has_swing_limits {
                    self.update_global_poses();
                    let a0 = get_quat_axis_0(self.global_pose_0.q);
                    let a1 = get_quat_axis_0(self.global_pose_1.q);
                    let n = a0.cross(a1).normalize();
                    Body::limit_angle(
                        &self.body0,
                        &self.body1,
                        n,
                        a0,
                        a1,
                        self.min_swing_angle,
                        self.max_swing_angle,
                        self.swing_limits_compliance,
                        dt,
                        PI,
                    );
                }

                // twist limits
                if self.has_twist_limits {
                    self.update_global_poses();
                    let n0 = get_quat_axis_0(self.global_pose_0.q);
                    let n1 = get_quat_axis_0(self.global_pose_1.q);
                    let n = (n0 + n1).normalize();
                    let mut a0 = get_quat_axis_1(self.global_pose_0.q);
                    a0 += n * -n.dot(a0);
                    a0 = a0.normalize();
                    let mut a1 = get_quat_axis_1(self.global_pose_1.q);
                    a1 += n * -n.dot(a1);
                    a1 = a1.normalize();

                    // handle gimbal lock problem
                    let max_corr = if n0.dot(n1) > -0.5 {
                        2.0 * PI
                    } else {
                        1.0 * dt
                    };

                    Body::limit_angle(
                        &self.body0,
                        &self.body1,
                        n,
                        a0,
                        a1,
                        self.min_twist_angle,
                        self.max_twist_angle,
                        self.twist_limits_compliance,
                        dt,
                        max_corr,
                    );
                }
            }
        }

        // position
        // simple attachment
        self.update_global_poses();
        let corr = self.global_pose_1.p - self.global_pose_0.p;
        Body::apply_pair_correction(
            &self.body0,
            &self.body1,
            corr,
            self.compliance,
            dt,
            Some(self.global_pose_0.p),
            Some(self.global_pose_1.p),
            false,
        );
    }

    fn solve_vel(&mut self, dt: f32) {
        // Gauss-Seidel lets us make damping unconditionally stable in a
        // very simple way. We clamp the correction for each constraint
        // to the magnitude of the current velocity making sure that
        // we never subtract more than there actually is.

        if self.rot_damping > 0.0 {
            let mut omega = Vec3::ZERO;
            if let Some(body0) = &self.body0 {
                omega -= body0.borrow().omega;
            }
            if let Some(body1) = &self.body1 {
                omega += body1.borrow().omega;
            }
            omega *= f32::min(1.0, self.rot_damping * dt);
            Body::apply_pair_correction(&self.body0, &self.body1, omega, 0.0, dt, None, None, true);
        }

        if self.pos_damping > 0.0 {
            self.update_global_poses();
            let mut vel = Vec3::ZERO;
            if let Some(body0) = &self.body0 {
                vel -= body0.borrow().get_vel_at(self.global_pose_0.p);
            }
            if let Some(body1) = &self.body1 {
                vel += body1.borrow().get_vel_at(self.global_pose_1.p);
            }
            vel *= f32::min(1.0, self.pos_damping * dt);
            Body::apply_pair_correction(
                &self.body0,
                &self.body1,
                vel,
                0.0,
                dt,
                Some(self.global_pose_0.p),
                Some(self.global_pose_1.p),
                true,
            );
        }
    }
}

#[wasm_bindgen]
pub struct BodyChainSimulation {
    #[wasm_bindgen(readonly)]
    pub num_objects: usize,
    bodies: Vec<Rc<RefCell<Body>>>,
    joints: Vec<Joint>,

    grab_joint: Option<usize>,

    poses: Vec<[f32; 7]>, // Vec<Vec3 position, Quat rotation> flattened to f32

    #[wasm_bindgen(readonly)]
    pub num_substeps: u8,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    inv_dt: f32,
    #[wasm_bindgen(readonly)]
    pub rot_damping: f32,
    #[wasm_bindgen(readonly)]
    pub pos_damping: f32,
    #[wasm_bindgen(readonly)]
    pub compliance: f32,
}

#[wasm_bindgen]
impl BodyChainSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        num_objects: usize,
        object_size: &[f32],
        last_object_size: &[f32],
        num_substeps: u8,
        rot_damping: f32,
        pos_damping: f32,
        compliance: f32,
    ) -> Self {
        let dt = TIME_STEP / f32::from(num_substeps);
        let mut chain = Self {
            num_objects,
            bodies: Vec::with_capacity(num_objects),
            joints: Vec::with_capacity(num_objects + 1), // additional grab joint

            grab_joint: None,

            poses: vec![[0.0; 7]; num_objects],

            num_substeps,
            dt,
            inv_dt: 1.0 / dt,

            rot_damping,
            pos_damping,
            compliance,
        };
        chain.reset(object_size, last_object_size);
        chain
    }

    pub fn reset(&mut self, object_size: &[f32], last_object_size: &[f32]) {
        self.bodies.clear();
        self.joints.clear();

        let object_size = Vec3::from_slice(object_size);
        let last_object_size = Vec3::from_slice(last_object_size);

        let pos = Vec3::new(
            0.0,
            (self.num_objects as f32 * object_size.y + last_object_size.y) * 1.4 + 0.2,
            0.0,
        );
        let mut pose = Pose::default();
        let mut last_body: Option<Rc<RefCell<Body>>> = None;
        let mut joint_pose_0 = Pose::default();
        let mut joint_pose_1 = Pose::default();
        joint_pose_0.q = Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), 0.5 * PI);
        joint_pose_1.q = Quat::from_axis_angle(vec3(0.0, 0.0, 1.0), 0.5 * PI);
        let mut last_size = object_size;

        for i in 0..self.num_objects {
            let size = if i < self.num_objects - 1 {
                object_size
            } else {
                last_object_size
            };
            pose.p = pos;
            pose.p.y -= i as f32 * object_size.y;

            let box_body = Rc::new(RefCell::new(Body::new(pose)));
            box_body.borrow_mut().set_box(size, 1.0);
            self.bodies.push(box_body.clone());

            let s = if i % 2 == 0 { -0.5 } else { 0.5 };
            joint_pose_0.p = Vec3::new(s, 0.5, s) * size;
            joint_pose_1.p = Vec3::new(s, -0.5, s) * last_size;

            if last_body.is_none() {
                joint_pose_1 = joint_pose_0;
                joint_pose_1.p += pose.p;
            }

            let mut joint = Joint::new(
                JointKind::Spherical,
                Some(box_body.clone()),
                last_body,
                joint_pose_0,
                joint_pose_1,
            );
            joint.rot_damping = self.rot_damping;
            joint.pos_damping = self.pos_damping;
            joint.compliance = self.compliance;
            self.joints.push(joint);

            last_body = Some(box_body);
            last_size = size;
        }
    }

    pub fn step(&mut self) {
        for _ in 0..self.num_substeps {
            self.bodies
                .iter_mut()
                .for_each(|b| b.borrow_mut().integrate(self.dt));

            self.joints.iter_mut().for_each(|j| j.solve_pos(self.dt));

            self.bodies
                .iter_mut()
                .for_each(|b| b.borrow_mut().update(self.dt));

            self.joints.iter_mut().for_each(|j| j.solve_vel(self.dt));
        }

        self.poses.iter_mut().enumerate().for_each(|(i, p)| {
            let pose = self.bodies[i].borrow().pose;
            pose.p.write_to_slice(&mut p[0..3]);
            pose.q.write_to_slice(&mut p[3..7]);
        });
    }

    #[wasm_bindgen(getter)]
    pub fn poses(&self) -> *const [f32; 7] {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the poses array pointer after heap allocations have
        // occurred (which move the location).
        // Poses is a Vec<[f32; 7]>, which is a linear array of f32s in
        // memory.
        self.poses.as_ptr()
    }

    #[wasm_bindgen(setter)]
    pub fn set_num_substeps(&mut self, num_substeps: u8) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / Into::<f32>::into(num_substeps);
        self.inv_dt = 1.0 / self.dt;
    }

    #[wasm_bindgen(setter)]
    pub fn set_pos_damping(&mut self, damping: f32) {
        self.pos_damping = damping;
        self.joints.iter_mut().for_each(|b| b.pos_damping = damping);
    }

    #[wasm_bindgen(setter)]
    pub fn set_rot_damping(&mut self, damping: f32) {
        self.rot_damping = damping;
        self.joints.iter_mut().for_each(|b| b.rot_damping = damping);
    }

    #[wasm_bindgen(setter)]
    pub fn set_compliance(&mut self, compliance: f32) {
        self.compliance = compliance;
        self.joints
            .iter_mut()
            .for_each(|b| b.compliance = compliance);
    }

    pub fn start_grab(&mut self, id: usize, pos: &[f32]) {
        let mut pose0 = Pose::default();
        let mut pose1 = Pose::default();
        let mut hit = Vec3::from_slice(pos);
        pose1.p = hit;
        let body = &self.bodies[id];
        body.borrow().pose.inv_transform(&mut hit);
        pose0.p = hit;
        let mut grab_joint =
            Joint::new(JointKind::Spherical, Some(body.clone()), None, pose0, pose1);
        grab_joint.compliance = GRAB_JOINT_COMPLIANCE;
        self.joints.push(grab_joint);
        self.grab_joint = Some(self.joints.len() - 1);
    }

    pub fn move_grabbed(&mut self, _: usize, pos: &[f32]) {
        if let Some(grab_joint) = self.grab_joint {
            self.joints[grab_joint].local_pose_1.p = Vec3::from_slice(pos);
        }
    }

    pub fn end_grab(&mut self, _: usize, _: &[f32]) {
        if self.grab_joint.is_some() {
            self.joints.pop();
            self.grab_joint = None;
        }
    }
}
