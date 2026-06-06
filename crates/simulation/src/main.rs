// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! MuJoCo differential-drive robot simulation binary.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use mujoco_rs::prelude::*;
use mujoco_rs::viewer::egui::{self, FontData, FontDefinitions, FontFamily};
use mujoco_rs::viewer::MjViewer;
use zos_simulation::{
    scene_path, twist_to_planar_velocity_axes, BASE_LINK, SPEED_ANGULAR, SPEED_LINEAR,
};

/// Platform CJK-capable fonts for egui (default Latin-only fonts show □ for Chinese).
fn cjk_font_candidates() -> &'static [&'static str] {
    #[cfg(target_os = "macos")]
    {
        &[
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
        ]
    }
    #[cfg(target_os = "windows")]
    {
        &[
            "C:\\Windows\\Fonts\\msyh.ttc",
            "C:\\Windows\\Fonts\\simhei.ttf",
        ]
    }
    #[cfg(target_os = "linux")]
    {
        &[
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        ]
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        &[]
    }
}

fn setup_cjk_fonts(ctx: &egui::Context) {
    let Some(bytes) = cjk_font_candidates()
        .iter()
        .find_map(|path| std::fs::read(path).ok())
    else {
        eprintln!("警告：未找到中文字体，界面中文可能显示为方框");
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("cjk".to_owned(), FontData::from_owned(bytes).into());
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "cjk".to_owned());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "cjk".to_owned());
    ctx.set_fonts(fonts);
}

#[derive(Default)]
struct Teleop {
    linear: f64,
    angular: f64,
}

fn run_viewer() -> Result<(), Box<dyn std::error::Error>> {
    let path = scene_path();
    if !path.is_file() {
        return Err(format!("未找到差分轮场景: {}", path.display()).into());
    }

    println!("正在加载差分轮模型...");
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

    viewer.with_ui_egui_ctx(setup_cjk_fonts);

    viewer.add_ui_callback_detached(move |ctx| {
        use mujoco_rs::viewer::egui::Key;

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

        egui::Window::new("遥控")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                let t = teleop_keys.borrow();
                ui.label("按住 W/S 前进后退，按住 A/D 左转右转，松开即停");
                ui.label(format!(
                    "cmd_vel: linear {:.2} m/s, angular {:.2} rad/s",
                    t.linear, t.angular
                ));
            });
    });

    println!("差分轮仿真已启动。");
    println!("键盘：按住 W/S 前进后退，按住 A/D 左转右转，松开即停，鼠标拖拽视角。");

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

    println!("仿真结束。");
    Ok(())
}

fn main() {
    if let Err(err) = run_viewer() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
