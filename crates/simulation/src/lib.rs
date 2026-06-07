// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! MuJoCo differential-drive simulation helpers.

use std::path::PathBuf;

/// Must match `robots/diff_drive/robot.xml` (wheel radius 0.05 m, track width 0.32 m).
pub const WHEEL_RADIUS: f64 = 0.05;
pub const WHEEL_BASE: f64 = 0.32;

/// Teleop linear speed (m/s). 0.8 ≈ brisk walk; actuator limit is 2.0.
pub const SPEED_LINEAR: f64 = 0.8;
/// Teleop turn rate (rad/s). ~115°/s; actuator limit is 3.0.
pub const SPEED_ANGULAR: f64 = 2.0;

/// ROS-style frame names (+X forward, +Y left, +Z up).
pub const BASE_LINK: &str = "base_link";
pub const IMU_LINK: &str = "imu_link";
pub const LIDAR_LINK: &str = "lidar_link";
pub const CAMERA_LINK: &str = "camera_link";

/// Ground-truth odometry sample (world pose + body-frame twist).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Odom {
    pub x: f64,
    pub y: f64,
    pub yaw: f64,
    pub linear: f64,
    pub angular: f64,
}

/// IMU sample in the sensor (body) frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuSample {
    pub accel: [f64; 3],
    pub gyro: [f64; 3],
}

pub fn robot_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("robots/diff_drive/robot.xml")
}

pub fn scene_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("robots/diff_drive/scene.xml")
}

pub fn office_floor_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("robots/diff_drive/environments/office_floor.xml")
}

/// Convert body-frame cmd_vel to left/right wheel angular velocities (rad/s).
pub fn twist_to_wheel_speeds(linear: f64, angular: f64) -> (f64, f64) {
    let left = (linear - angular * WHEEL_BASE / 2.0) / WHEEL_RADIUS;
    let right = (linear + angular * WHEEL_BASE / 2.0) / WHEEL_RADIUS;
    (left, right)
}

/// Convert body-frame cmd_vel to world-frame planar velocities (m/s, m/s, rad/s).
pub fn twist_to_planar_velocity(linear: f64, angular: f64, yaw_rad: f64) -> (f64, f64, f64) {
    let (sin, cos) = yaw_rad.sin_cos();
    (linear * cos, linear * sin, angular)
}

/// Convert body-frame cmd_vel using the body +X axis projected onto the world XY plane.
///
/// `forward_x` / `forward_y` are the first column of `data.body(..).xmat` (row-major 3×3).
pub fn twist_to_planar_velocity_axes(
    linear: f64,
    angular: f64,
    forward_x: f64,
    forward_y: f64,
) -> (f64, f64, f64) {
    (linear * forward_x, linear * forward_y, angular)
}

/// Read planar ground-truth odometry from MuJoCo joint states.
#[cfg(feature = "viewer")]
pub fn read_odom<M: std::ops::Deref<Target = mujoco_rs::prelude::MjModel>>(
    data: &mujoco_rs::prelude::MjData<M>,
) -> Odom {
    let slide_x = data.joint("slide_x").expect("slide_x");
    let slide_y = data.joint("slide_y").expect("slide_y");
    let yaw = data.joint("yaw").expect("yaw");

    let x = slide_x.view(data).qpos[0];
    let y = slide_y.view(data).qpos[0];
    let yaw_rad = yaw.view(data).qpos[0];
    let vx_world = slide_x.view(data).qvel[0];
    let vy_world = slide_y.view(data).qvel[0];
    let omega = yaw.view(data).qvel[0];

    let (sin, cos) = yaw_rad.sin_cos();
    let linear = vx_world * cos + vy_world * sin;

    Odom {
        x,
        y,
        yaw: yaw_rad,
        linear,
        angular: omega,
    }
}

/// Read IMU accelerometer / gyroscope samples.
#[cfg(feature = "viewer")]
pub fn read_imu<M: std::ops::Deref<Target = mujoco_rs::prelude::MjModel>>(
    data: &mujoco_rs::prelude::MjData<M>,
) -> ImuSample {
    let accel = data.sensor("imu_accel").expect("imu_accel").view(data).data;
    let gyro = data.sensor("imu_gyro").expect("imu_gyro").view(data).data;
    ImuSample {
        accel: [accel[0], accel[1], accel[2]],
        gyro: [gyro[0], gyro[1], gyro[2]],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn twist_to_wheel_speeds_forward() {
        let (left, right) = twist_to_wheel_speeds(SPEED_LINEAR, 0.0);
        assert!((left - 16.0).abs() < 1e-9);
        assert!((right - 16.0).abs() < 1e-9);
    }

    #[test]
    fn twist_to_wheel_speeds_turn_in_place() {
        let (left, right) = twist_to_wheel_speeds(0.0, 1.2);
        assert!((left + right).abs() < 1e-9);
        assert!(left < 0.0);
        assert!(right > 0.0);
    }

    #[test]
    fn twist_to_planar_velocity_forward() {
        let (vx, vy, wz) = twist_to_planar_velocity(0.4, 0.0, 0.0);
        assert!((vx - 0.4).abs() < 1e-9);
        assert!(vy.abs() < 1e-9);
        assert!(wz.abs() < 1e-9);
    }

    #[test]
    fn twist_to_planar_velocity_yaw_90() {
        let yaw = std::f64::consts::FRAC_PI_2;
        let (vx, vy, wz) = twist_to_planar_velocity(0.4, 0.0, yaw);
        assert!(vx.abs() < 1e-9);
        assert!((vy - 0.4).abs() < 1e-9);
        assert!(wz.abs() < 1e-9);
    }

    /// Planar base must not gain altitude when driving forward.
    #[cfg(feature = "viewer")]
    #[test]
    fn forward_drive_stays_on_ground() {
        use mujoco_rs::prelude::*;

        let path = scene_path();
        let model = MjModel::from_xml(&path).expect("load scene");
        let mut data = model.make_data();

        let drive_x = model.actuator("drive_x").expect("drive_x").id;
        let drive_y = model.actuator("drive_y").expect("drive_y").id;
        let drive_yaw = model.actuator("drive_yaw").expect("drive_yaw").id;
        let base = data.body(BASE_LINK).expect(BASE_LINK);
        data.forward();
        let initial_z = base.view(&data).xpos[2];
        let initial_x = base.view(&data).xpos[0];

        let (vx, vy, wz) = twist_to_planar_velocity(SPEED_LINEAR, 0.0, 0.0);
        for _ in 0..400 {
            let ctrl = data.ctrl_mut();
            ctrl[drive_x] = vx;
            ctrl[drive_y] = vy;
            ctrl[drive_yaw] = wz;
            data.step();
        }

        let pos = base.view(&data).xpos;
        assert!(
            (pos[2] - initial_z).abs() < 0.01,
            "base altitude changed: {initial_z} -> {}",
            pos[2]
        );

        let dx = pos[0] - initial_x;
        assert!(dx > 0.1, "expected forward motion, dx={dx}");
    }

    /// Steady-state speed should track cmd_vel after a short settle time.
    #[cfg(feature = "viewer")]
    #[test]
    fn forward_drive_steady_speed() {
        use mujoco_rs::prelude::*;

        let path = scene_path();
        let model = MjModel::from_xml(&path).expect("load scene");
        let mut data = model.make_data();

        let drive_x = model.actuator("drive_x").expect("drive_x").id;
        let drive_y = model.actuator("drive_y").expect("drive_y").id;
        let drive_yaw = model.actuator("drive_yaw").expect("drive_yaw").id;
        let slide_x = data.joint("slide_x").expect("slide_x");

        let (vx, vy, wz) = twist_to_planar_velocity(SPEED_LINEAR, 0.0, 0.0);
        for _ in 0..600 {
            let ctrl = data.ctrl_mut();
            ctrl[drive_x] = vx;
            ctrl[drive_y] = vy;
            ctrl[drive_yaw] = wz;
            data.step();
        }

        let measured = slide_x.view(&data).qvel[0];
        assert!(
            (measured - SPEED_LINEAR).abs() < 0.08,
            "expected ~{SPEED_LINEAR} m/s, got {measured}"
        );
    }

    /// After turning ~90°, forward cmd_vel should move along +Y, not +X.
    #[cfg(feature = "viewer")]
    #[test]
    fn forward_after_yaw_90_moves_along_body_heading() {
        use mujoco_rs::prelude::*;

        let path = scene_path();
        let model = MjModel::from_xml(&path).expect("load scene");
        let mut data = model.make_data();

        let drive_x = model.actuator("drive_x").expect("drive_x").id;
        let drive_y = model.actuator("drive_y").expect("drive_y").id;
        let drive_yaw = model.actuator("drive_yaw").expect("drive_yaw").id;
        let base = data.body(BASE_LINK).expect(BASE_LINK);

        let timestep = model.opt().timestep;
        let turn_steps =
            (std::f64::consts::FRAC_PI_2 / (SPEED_ANGULAR * timestep)).ceil() as usize;
        for _ in 0..turn_steps {
            let ctrl = data.ctrl_mut();
            ctrl[drive_x] = 0.0;
            ctrl[drive_y] = 0.0;
            ctrl[drive_yaw] = SPEED_ANGULAR;
            data.step();
        }

        let xmat = base.view(&data).xmat;
        let forward_x = xmat[0];
        let forward_y = xmat[3];

        data.forward();
        let x0 = base.view(&data).xpos[0];
        let y0 = base.view(&data).xpos[1];

        let (vx, vy, wz) =
            twist_to_planar_velocity_axes(SPEED_LINEAR, 0.0, forward_x, forward_y);
        for _ in 0..400 {
            let ctrl = data.ctrl_mut();
            ctrl[drive_x] = vx;
            ctrl[drive_y] = vy;
            ctrl[drive_yaw] = wz;
            data.step();
        }

        let dx = base.view(&data).xpos[0] - x0;
        let dy = base.view(&data).xpos[1] - y0;
        assert!(
            dy.abs() > dx.abs(),
            "expected body-frame forward; forward=({forward_x},{forward_y}), dx={dx}, dy={dy}, vx={vx}, vy={vy}"
        );
    }

    #[cfg(feature = "viewer")]
    #[test]
    fn scene_includes_robot_frames_and_sensors() {
        use mujoco_rs::prelude::*;

        let model = MjModel::from_xml(&scene_path()).expect("load scene");
        let data = model.make_data();

        assert!(data.body(BASE_LINK).is_some());
        assert!(data.body(LIDAR_LINK).is_some());
        assert!(data.body(CAMERA_LINK).is_some());
        assert!(model.sensor("imu_accel").is_some());
        assert!(model.sensor("imu_gyro").is_some());
        assert!(model.camera("front_camera").is_some());
        assert!(model.geom("office_floor").is_some());
        assert!(model.geom("meet_table").is_some());
    }

    #[cfg(feature = "viewer")]
    #[test]
    fn read_odom_and_imu_after_step() {
        use mujoco_rs::prelude::*;

        let model = MjModel::from_xml(&scene_path()).expect("load scene");
        let mut data = model.make_data();
        let drive_x = model.actuator("drive_x").expect("drive_x").id;

        data.ctrl_mut()[drive_x] = SPEED_LINEAR;
        data.step();

        let odom = read_odom(&data);
        assert!(odom.linear > 0.0);

        let imu = read_imu(&data);
        assert!(imu.accel[2].is_finite());
        assert!(imu.gyro[2].is_finite());
    }
}
