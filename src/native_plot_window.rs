use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

/// Messages sent from the application to the native plot window
#[derive(Debug)]
pub enum PlotMessage {
    /// Cartesian points as [x, y]
    Cartesian(Vec<[f64; 2]>),
    /// Time-based points as [t, value]
    Time(Vec<[f64; 2]>),
}

/// Handle stored as a Bevy resource to send updates to the native window
#[derive(Clone, bevy::prelude::Resource)]
pub struct NativePlotWindowHandle {
    pub sender: Sender<PlotMessage>,
    pub is_open: Arc<AtomicBool>,
}

/// Spawn a new native OS window running an eframe app that displays the plot.
/// Returns a handle that can be inserted as a Bevy resource for sending updates.
use once_cell::sync::Lazy;
use std::sync::Mutex;

static GLOBAL_HANDLE: Lazy<Mutex<Option<NativePlotWindowHandle>>> = Lazy::new(|| Mutex::new(None));

pub fn spawn_native_plot_window() -> NativePlotWindowHandle {
    let (tx, rx): (Sender<PlotMessage>, Receiver<PlotMessage>) = unbounded();
    let is_open = Arc::new(AtomicBool::new(true));
    let is_open_clone = is_open.clone();

    std::thread::spawn(move || {
        let options = eframe::NativeOptions::default();
        let app = PlotWindowApp::new(rx, is_open_clone);
        // This will run until the window is closed
        // eframe expects the initialization closure to return a Result containing the boxed App
        let _ = eframe::run_native("GCodeFlow 2D Projection", options, Box::new(|_cc| Ok(Box::new(app) as Box<dyn eframe::App>)));
    });

    let handle = NativePlotWindowHandle { sender: tx, is_open };
    // store in the global so other parts of the app can access it without requiring Commands
    *GLOBAL_HANDLE.lock().expect("lock global handle") = Some(handle.clone());
    handle
}

/// Return a cloned handle if a native plot window has been spawned
pub fn current_handle() -> Option<NativePlotWindowHandle> {
    GLOBAL_HANDLE.lock().expect("lock global handle").clone()
}

struct PlotWindowApp {
    receiver: Receiver<PlotMessage>,
    cartesian: Vec<[f64; 2]>,
    time: Vec<[f64; 2]>,
    is_open: Arc<AtomicBool>,
}

impl PlotWindowApp {
    fn new(receiver: Receiver<PlotMessage>, is_open: Arc<AtomicBool>) -> Self {
        Self { receiver, cartesian: Vec::new(), time: Vec::new(), is_open }
    }
}

impl eframe::App for PlotWindowApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Drain messages
        for msg in self.receiver.try_iter() {
            match msg {
                PlotMessage::Cartesian(v) => self.cartesian = v,
                PlotMessage::Time(v) => self.time = v,
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Close").clicked() {
                    // Mark closed; the Bevy side sees this flag and can respawn later.
                    self.is_open.store(false, Ordering::SeqCst);
                }
                if ui.button("Fit").clicked() {
                    // No-op for now; reliance on auto-bounds in the plot itself
                }
            });

            ui.separator();

            // Cartesian plot (if we have data)
            if !self.cartesian.is_empty() {
                let points_arr: Vec<[f64; 2]> = self.cartesian.iter().map(|p| [p[0], p[1]]).collect();
                let line = egui_plot::Line::new(egui_plot::PlotPoints::from(points_arr)).name("Cartesian Path");
                egui_plot::Plot::new("native_cartesian_plot").show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
            }

            // Time plot (if we have data)
            if !self.time.is_empty() {
                let points_arr: Vec<[f64; 2]> = self.time.iter().map(|p| [p[0], p[1]]).collect();
                let line = egui_plot::Line::new(egui_plot::PlotPoints::from(points_arr)).name("Time Series");
                egui_plot::Plot::new("native_time_plot").show(ui, |plot_ui| {
                    plot_ui.line(line);
                });
            }
        });
    }
}
