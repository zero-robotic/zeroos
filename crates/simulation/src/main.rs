// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! MuJoCo differential-drive robot simulation binary.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use mujoco_rs::prelude::*;
use mujoco_rs::viewer::egui::{self, Key};
use mujoco_rs::viewer::MjViewer;
use zos_simulation::{
    scene_path, twist_to_planar_velocity_axes, BASE_LINK, SPEED_ANGULAR, SPEED_LINEAR,
};

#[derive(Default)]
struct Teleop {
    linear: f64,
    angular: f64,
}

fn run_viewer() -> Result<(), Box<dyn std::error::Error>> {
    let path = scene_path();
    if !path.is_file() {
        return Err(format!("diff-drive scene not found: {}", path.display()).into());
    }

    println!("Loading diff-drive model...");
    let model = MjModel::from_xml(&path)?;
    let mut data = model.make_data();

    let drive_x_id = model
        .actuator("drive_x")
        .ok_or("actuator drive_x not found")?
        .id;
    let drive_y_id = model
        .actuator("drive_y")
        .ok_or("actuator drive_y not found")?
        .id;
    let drive_yaw_id = model
        .actuator("drive_yaw")
        .ok_or("actuator drive_yaw not found")?
        .id;
    let teleop = Rc::new(RefCell::new(Teleop::default()));
    let teleop_keys = teleop.clone();

    let mut viewer = MjViewer::builder()
        .max_user_geoms(0)
        .build_passive(&model)?;

    viewer.add_ui_callback_detached(move |ctx| {
        ctx.input(|input| {
            let mut t = teleop_keys.borrow_mut();
            let w = input.key_down(Key::W);
            let s = input.key_down(Key::S);
            let a = input.key_down(Key::A);
            let d = input.key_down(Key::D);

            t.linear = match (w, s) {
                (true, false) => SPEED_LINEAR,
                (false, true) => -SPEED_LINEAR,
                _ => 0.0,
            };
            t.angular = match (a, d) {
                (true, false) => SPEED_ANGULAR,
                (false, true) => -SPEED_ANGULAR,
                _ => 0.0,
            };
        });

        egui::Window::new("Teleop")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let t = teleop_keys.borrow();
                ui.label("Hold W/S to drive forward/back, A/D to turn, release to stop");
                ui.label(format!(
                    "cmd_vel: linear {:.2} m/s, angular {:.2} rad/s",
                    t.linear, t.angular
                ));
            });
    });

    println!("Simulation started.");
    println!("Keys: hold W/S forward/back, A/D turn in place, release to stop. Drag to orbit view.");

    let timestep = model.opt().timestep;
    while viewer.running() {
        let t = teleop.borrow();
        let xmat = data.body(BASE_LINK).unwrap().view(&data).xmat;
        let (vx, vy, wz) =
            twist_to_planar_velocity_axes(t.linear, t.angular, xmat[0], xmat[3]);
        drop(t);

        {
            let ctrl = data.ctrl_mut();
            ctrl[drive_x_id] = vx;
            ctrl[drive_y_id] = vy;
            ctrl[drive_yaw_id] = wz;
        }

        data.step();
        viewer.sync_data(&mut data);
        viewer.render();

        std::thread::sleep(Duration::from_secs_f64(timestep));
    }

    println!("Simulation finished.");
    Ok(())
}

fn main() {
    if let Err(err) = run_viewer() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
