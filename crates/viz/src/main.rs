// Copyright (c) 2026 ZeroOS Authors
// SPDX-License-Identifier: Apache-2.0

//! ZeroOS desktop message visualizer (`zos-viz`).

use std::sync::Arc;
use std::thread;

use eframe::egui;
use zos_msg::Twist;
use zos_runtime::{init, Executor, Node, NodeOptions, RuntimeError};
use zos_viz::{new_shared_state, SharedVizState};

fn spawn_runtime(state: SharedVizState) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("tokio runtime");

        if let Err(err) = rt.block_on(run_subscriber(state)) {
            eprintln!("zos-viz runtime error: {err}");
        }
    })
}

async fn run_subscriber(state: SharedVizState) -> Result<(), RuntimeError> {
    init().await?;
    let mut node = Node::new(NodeOptions::new().name("viz"));

    node.create_subscriber_builder::<Twist>("cmd_vel")
        .register(move |msg| {
            let state = Arc::clone(&state);
            async move {
                if let Ok(mut guard) = state.lock() {
                    guard.update_cmd_vel(msg.linear, msg.angular);
                }
            }
        })?;

    println!("zos-viz subscribed to cmd_vel, waiting for messages...");
    Executor::spin_node(&mut node).await
}

struct VizApp {
    state: SharedVizState,
    _runtime: thread::JoinHandle<()>,
}

impl VizApp {
    fn new(state: SharedVizState, runtime: thread::JoinHandle<()>) -> Self {
        Self {
            state,
            _runtime: runtime,
        }
    }
}

impl eframe::App for VizApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(100));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("ZeroOS Message Viz");
            ui.separator();

            let snapshot = self
                .state
                .lock()
                .map(|guard| guard.clone())
                .unwrap_or_default();

            ui.collapsing("Topics", |ui| {
                ui.label("cmd_vel  (zos.geometry.Twist)");
                ui.indent("cmd_vel_indent", |ui| {
                    if snapshot.cmd_vel.has_data() {
                        ui.label(format!(
                            "linear  = {:.3} m/s",
                            snapshot.cmd_vel.linear
                        ));
                        ui.label(format!(
                            "angular = {:.3} rad/s",
                            snapshot.cmd_vel.angular
                        ));
                        if let Some(age) = snapshot.cmd_vel.age() {
                            ui.label(format!("updated {:.2} s ago", age.as_secs_f64()));
                        }
                        ui.label(format!("{} messages received", snapshot.cmd_vel_count));
                    } else {
                        ui.label("no data yet (waiting for publisher...)");
                    }
                });

                ui.add_space(8.0);
                ui.label("scan     (planned: LaserScan)");
                ui.label("camera   (planned: CompressedImage)");
            });

            ui.separator();
            ui.label("Transport: zos-runtime / Zenoh (local or LAN peers)");
        });
    }
}

fn main() -> eframe::Result<()> {
    let state = new_shared_state();
    let runtime = spawn_runtime(Arc::clone(&state));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([480.0, 360.0])
            .with_title("zos-viz"),
        ..Default::default()
    };

    eframe::run_native(
        "zos-viz",
        options,
        Box::new(|_cc| Ok(Box::new(VizApp::new(state, runtime)))),
    )
}
