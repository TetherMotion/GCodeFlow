use bevy_egui::egui;
use crate::app::AppState;
use crate::editor::EditorState;
use crate::ui::BenchmarkState;
use gcodeflow::benchmark::run_benchmark_from_file;
use crossbeam_channel::bounded;
use std::env;
use std::fs;
use uuid::Uuid;

pub fn draw_benchmark_window(
    ctx: &egui::Context,
    _app_state: &mut AppState,
    editor_state: &mut EditorState,
    bench_state: &mut BenchmarkState,
) {
    if !bench_state.open { return; }

    egui::Window::new("G-Code Benchmark")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.radio_value(&mut bench_state.use_current_buffer, true, "Use current buffer");
                ui.radio_value(&mut bench_state.use_current_buffer, false, "Select a file");
            });

            ui.add_space(6.0);

            if !bench_state.use_current_buffer {
                ui.horizontal(|ui| {
                    if ui.button("Choose file...").clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("GCode", &["gcode"]).pick_file() {
                            bench_state.selected_file = Some(p);
                        }
                    }
                    if let Some(ref p) = bench_state.selected_file {
                        ui.label(format!("{}", p.display()));
                    } else {
                        ui.label("(no file selected)");
                    }
                });
            }

            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.button(if bench_state.running { "Running..." } else { "Run" }).clicked() && !bench_state.running {
                    // Spawn background thread to run benchmark
                    bench_state.running = true;
                    bench_state.last_result = None;
                    bench_state.last_error = None;

                    let (tx, rx) = bounded(1);
                    bench_state.pending = Some(rx);

                    if bench_state.use_current_buffer {
                        // write current buffer to temp file
                        let tmp = env::temp_dir().join(format!("gcbm_{}.gcode", Uuid::new_v4()));
                        let contents = _app_state.gcode_content.clone();
                        let _ = fs::write(&tmp, contents);
                        std::thread::spawn(move || {
                            let res = run_benchmark_from_file(&tmp).map_err(|e| e.to_string());
                            let _ = tx.send(res);
                            let _ = fs::remove_file(&tmp);
                        });
                    } else if let Some(ref file) = bench_state.selected_file {
                        let p = file.clone();
                        std::thread::spawn(move || {
                            let res = run_benchmark_from_file(p).map_err(|e| e.to_string());
                            let _ = tx.send(res);
                        });
                    } else {
                        bench_state.running = false;
                        bench_state.last_error = Some("No file selected".to_string());
                    }
                }

                if ui.button("Close").clicked() {
                    bench_state.open = false;
                }
            });

            ui.separator();

            if bench_state.running {
                ui.label("Benchmark running...");
            }

            if let Some(ref err) = bench_state.last_error {
                ui.colored_label(egui::Color32::RED, err);
            }

            if let Some(ref res) = bench_state.last_result {
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
