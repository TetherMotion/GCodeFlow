use bevy_egui::egui;
use crate::app::AppState;
#[cfg(feature = "editor")]
use crate::editor::EditorState;
use crate::ui::{BenchmarkState, UiState};
use gcodeflow::benchmark::run_benchmark_from_file;
use crossbeam_channel::bounded;
use std::env;
use std::fs;
use uuid::Uuid;

#[cfg(feature = "editor")]
pub fn draw_benchmark_window(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    editor_state: &mut EditorState,
    benchmark_state: &mut BenchmarkState,
) {
    draw_benchmark_window_impl(ctx, ui_state, Some(editor_state), benchmark_state);
}

#[cfg(not(feature = "editor"))]
pub fn draw_benchmark_window(
    ctx: &egui::Context,
    ui_state: &mut UiState,
    benchmark_state: &mut BenchmarkState,
) {
    draw_benchmark_window_impl(ctx, ui_state, None, benchmark_state);
}

#[cfg(feature = "editor")]
fn draw_benchmark_window_impl(
    ctx: &egui::Context,
    _ui_state: &mut UiState,
    editor_state: &mut EditorState,
    benchmark_state: &mut BenchmarkState,
) {
    draw_benchmark_window_impl_inner(ctx, _ui_state, Some(editor_state), benchmark_state);
}

#[cfg(not(feature = "editor"))]
fn draw_benchmark_window_impl(
    ctx: &egui::Context,
    _ui_state: &mut UiState,
    benchmark_state: &mut BenchmarkState,
) {
    draw_benchmark_window_impl_inner(ctx, _ui_state, None, benchmark_state);
}

fn draw_benchmark_window_impl_inner(
    ctx: &egui::Context,
    _ui_state: &mut UiState,
    editor_state: Option<&mut EditorState>,
    benchmark_state: &mut BenchmarkState,
) {
    if !benchmark_state.open { return; }

    egui::Window::new("G-Code Benchmark")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut benchmark_state.use_current_buffer, true, "Use current buffer");
                ui.radio_value(&mut benchmark_state.use_current_buffer, false, "Select a file");
            });

            ui.add_space(6.0);

            if !benchmark_state.use_current_buffer {
                ui.horizontal(|ui| {
                    if ui.button("Choose file...").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("GCode", &["gcode"]).pick_file() {
                            benchmark_state.selected_file = Some(p);
                        }
                    }
                    if let Some(ref p) = benchmark_state.selected_file {
                        ui.label(format!("{}", p.display()));
                    } else {
                        ui.label("(no file selected)");
                    }
                });
            }

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(if benchmark_state.running { "Running..." } else { "Run" }).clicked() && !benchmark_state.running {
                    // Spawn background thread to run benchmark
                    benchmark_state.running = true;
                    benchmark_state.last_result = None;
                    benchmark_state.last_error = None;

                    let (tx, rx) = bounded(1);
                    benchmark_state.pending = Some(rx);

                    if benchmark_state.use_current_buffer {
                        // write current buffer to temp file
                        let tmp = env::temp_dir().join(format!("gcbm_{}.gcode", Uuid::new_v4()));
                        #[cfg(feature = "editor")]
                        let contents = editor_state.as_ref().map(|e| e.get_content()).unwrap_or_default();
                        #[cfg(not(feature = "editor"))]
                        let contents = String::new();
                        let _ = fs::write(&tmp, contents);
                        std::thread::spawn(move || {
                            let res = run_benchmark_from_file(&tmp).map_err(|e| e.to_string());
                            let _ = tx.send(res);
                            let _ = fs::remove_file(&tmp);
                        });
                    } else if let Some(ref file) = benchmark_state.selected_file {
                        let p = file.clone();
                        std::thread::spawn(move || {
                            let res = run_benchmark_from_file(p).map_err(|e| e.to_string());
                            let _ = tx.send(res);
                        });
                    } else {
                        benchmark_state.running = false;
                        benchmark_state.last_error = Some("No file selected".to_string());
                    }
                }

                if ui.button("Close").clicked() {
                    benchmark_state.open = false;
                }
            });

            ui.separator();

            if benchmark_state.running {
                ui.label("Benchmark running...");
            }

            if let Some(ref err) = benchmark_state.last_error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some(ref res) = benchmark_state.last_result {
                ui.group(|ui| {
                    ui.label(format!("File: {} ({} lines, {} bytes)", res.file_name, res.line_count, res.file_size));
                    ui.label(format!("File read: {:.2} ms", res.file_read_ms));
                    ui.label(format!("Parse: {:.2} ms", res.parse_ms));
                    ui.label(format!("Interpret+copy: {:.2} ms", res.interp_ms + res.copy_segments_ms));
                    ui.label(format!("Path length: {:.2} mm", res.path_length_mm));
                    ui.separator();
                    ui.label("Time tests:");
                    for t in &res.time_tests { ui.label(format!("  {}: {:.2} ms ({} points)", t.name, t.duration_ms, t.points)); }
                    ui.label("Deviation tests:");
                    for t in &res.deviation_tests { ui.label(format!("  {}: {:.2} ms ({} points)", t.name, t.duration_ms, t.points)); }
                    ui.separator();
                    ui.label(format!("Total time: {:.2} ms", res.total_ms));
                    ui.label(format!("Throughput: {:.0} lines/sec  ({:.2} MB/sec)", res.lines_per_second, res.mb_per_sec));
                });
            }
        });
}
