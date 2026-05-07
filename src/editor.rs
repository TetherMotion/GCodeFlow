//! GCode text editor with syntax highlighting
//!
//! Provides a fully functional syntax-highlighted text editor with:
//! - Full cursor navigation and editing
//! - Line numbers display (separate from editable text)
//! - Syntax highlighting for GCode elements
//! - Selection-based visualization filtering

use bevy::prelude::*;
use bevy_egui::egui;

use crate::app::AppState;
use crate::trajectory::TrajectoryUpdateEvent;
use crate::config::SyntaxColors;

// Use the extracted editor crate for UI/editor functionality
use gcode_editor::{TokenType, tokenize_line_pure_rust};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        // Insert the local EditorState resource (wrapping the gcode_editor inner state)
        app.insert_resource(EditorState::default())
           .add_event::<EditorChangeEvent>()
           // UI is rendered by `UiPlugin` so the editor can be placed in the
           // correct right-hand pane alongside the rest of the hierarchical layout.
           ;
    }
}

/// Editor state (local Bevy resource)
#[derive(Resource, Clone)]
pub struct EditorState {
    /// Inner reusable editor state from `gcode_editor` crate
    pub inner: gcode_editor::EditorState,

    /// ID counter for unique widget IDs
    id_counter: u64,
}

impl Default for EditorState {
    fn default() -> Self {
        Self {
            inner: gcode_editor::EditorState::default(),
            id_counter: 0,
        }
    }
}

impl EditorState {
    /// Get a unique ID for widgets
    pub fn next_id(&mut self) -> u64 {
        self.id_counter += 1;
        self.id_counter
    }
}

// Allow convenient field access to the inner editor state
impl std::ops::Deref for EditorState {
    type Target = gcode_editor::EditorState;
    fn deref(&self) -> &Self::Target { &self.inner }
}

impl std::ops::DerefMut for EditorState {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.inner }
}

/// Event emitted when editor content changes
#[derive(Event, Clone)]
pub struct EditorChangeEvent {
    pub new_content: String,
    pub selected_lines: Option<(usize, usize)>,
}

pub(crate) fn editor_panel(
    ctx: &egui::Context,
    state: &mut AppState,
    editor_state: &mut EditorState,
    change_events: &mut EventWriter<EditorChangeEvent>,
    trajectory_events: &mut EventWriter<TrajectoryUpdateEvent>,
) {
    // Editor panel on the right side - resizable by user
    egui::SidePanel::right("editor_panel")
        .default_width(500.0)
        .resizable(true)
        .show(ctx, |ui| {
            // Toolbar
            ui.horizontal(|ui| {
                if ui.button("📂 Open").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("GCode", &["gcode", "nc", "ngc", "g"])
                        .add_filter("All files", &["*"])
                        .pick_file()
                    {
                        match std::fs::read_to_string(&path) {
                            Ok(content) => {
                                state.gcode_content = content.clone();
                                state.current_file = Some(path);
                                state.modified = false;
                                change_events.send(EditorChangeEvent {
                                    new_content: content,
                                    selected_lines: None,
                                });
                                trajectory_events.send(TrajectoryUpdateEvent {
                                    selected_lines: None,
                                });
                            }
                            Err(e) => {
                                log::error!("Failed to open file: {}", e);
                            }
                        }
                    }
                }
                
                if ui.button("💾 Save").clicked() {
                    if let Some(ref path) = state.current_file {
                        if let Err(e) = std::fs::write(path, &state.gcode_content) {
                            log::error!("Failed to save: {}", e);
                        } else {
                            state.modified = false;
                        }
                    }
                }
                
                ui.separator();
                
                if ui.button("🔍").clicked() {
                    editor_state.show_search = !editor_state.show_search;
                }
                
                ui.separator();
                
                // Selection info
                if let Some((start, end)) = editor_state.selected_lines {
                    ui.label(format!("Lines {}-{} selected", start + 1, end + 1));
                    if ui.button("Clear Selection").clicked() {
                        editor_state.selected_lines = None;
                        trajectory_events.send(TrajectoryUpdateEvent {
                            selected_lines: None,
                        });
                    }
                }
            });
            
            // Search bar
            if editor_state.show_search {
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let search_response = ui.text_edit_singleline(&mut editor_state.search_text);
                    if ui.button("Find Next").clicked() || (search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                        // Find next occurrence
                        if !editor_state.search_text.is_empty() {
                            if let Some(pos) = state.gcode_content.find(&editor_state.search_text) {
                                // Calculate line number
                                let line = state.gcode_content[..pos].lines().count();
                                editor_state.cursor_pos = (line, 0);
                            }
                        }
                    }
                    if ui.button("×").clicked() {
                        editor_state.show_search = false;
                    }
                });
            }
            
            ui.separator();
            
            // Get config values before mutable borrow
            let font_size = state.config.editor.font_size;
            let show_line_numbers = state.config.editor.show_line_numbers;
            let colors = state.config.editor.colors.clone();
            
            // Main editor area with line numbers and syntax highlighting
            // Reserve space for the status bar at the bottom; the remaining space should be used
            // by the editor area and scale with the panel height.
            let available_height = (ui.available_height() - 30.0).max(0.0);

            // Make the text editor height dynamic instead of a fixed number of rows.
            let row_height = ui.fonts(|f| f.row_height(&egui::FontId::monospace(font_size)));
            let desired_rows = ((available_height / row_height).floor() as usize).max(5);
            let current_sim_line = state.simulation.current_line;
            
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(available_height)
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        // Line numbers column (read-only, separate from editor)
                        if show_line_numbers {
                            let line_count = state.gcode_content.lines().count().max(1);
                            let font_id = egui::FontId::monospace(font_size);
                            let mut job = egui::text::LayoutJob::default();

                            let base = egui::TextFormat {
                                font_id: font_id.clone(),
                                color: egui::Color32::from_rgb(100, 100, 100),
                                ..Default::default()
                            };
                            let current = egui::TextFormat {
                                font_id: font_id.clone(),
                                color: egui::Color32::from_rgb(0, 0, 0),
                                background: egui::Color32::from_rgb(255, 200, 0),
                                ..Default::default()
                            };

                            for idx in 0..line_count {
                                let fmt = if idx == current_sim_line { current.clone() } else { base.clone() };
                                job.append(&format!("{:4}\n", idx + 1), 0.0, fmt);
                            }

                            ui.add(egui::Label::new(job).selectable(false));
                            
                            ui.separator();
                        }
                        
                        // Main text editor with syntax highlighting
                        // Use the reusable editor component from `gcode_editor` which
                        // provides a sane layouter and consistent behavior.
                        let g_colors = gcode_editor::SyntaxColors {
                            gcode: colors.gcode,
                            mcode: colors.mcode,
                            axis: colors.axis,
                            axis_chars: std::collections::HashSet::from(['X', 'Y', 'Z', 'A', 'B', 'C']),
                            axis_overrides: std::collections::HashMap::new(),
                            parameter: colors.parameter,
                            parameter_key: colors.parameter,
                            parameter_value: colors.parameter,
                            p_parameter: colors.parameter,
                            number: colors.number,
                            comment: colors.comment,
                            ocode: colors.ocode,
                            operator: colors.operator,
                            variable: colors.variable,
                            error: colors.error,
                        };

                        let events = gcode_editor::show_editor(ui, &mut state.gcode_content, &mut editor_state.inner, &g_colors, font_size);
                        for ev in events {
                            match ev {
                                gcode_editor::EditorEvent::ContentChanged(change) => {
                                    change_events.send(EditorChangeEvent {
                                        new_content: change.new_content,
                                        selected_lines: change.selected_lines,
                                    });
                                    state.modified = true;
                                }
                                gcode_editor::EditorEvent::ActiveLineChanged { .. } => {
                                    // Handle active line change if needed
                                }
                                gcode_editor::EditorEvent::SelectionChanged { .. } => {
                                    trajectory_events.send(TrajectoryUpdateEvent {
                                        selected_lines: editor_state.selected_lines,
                                    });
                                }
                            }
                        }

                        // Keep focused state in sync with inner editor
                        editor_state.focused = editor_state.inner.focused;
                    });
                });
            
            // Status bar
            ui.separator();
            ui.horizontal(|ui| {
                let line_count = state.gcode_content.lines().count();
                let char_count = state.gcode_content.len();
                // Fixed-width format to prevent jumping
                ui.monospace(format!("{:>5} lines, {:>6} chars", line_count, char_count));
                
                ui.separator();
                
                // Always show status for consistent width
                if state.modified {
                    ui.colored_label(egui::Color32::YELLOW, "● Modified");
                } else {
                    ui.label("✓ Saved  ");  // Extra space for consistent width
                }
                
                // Right-aligned path with placeholder for consistent layout
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(ref path) = state.current_file {
                        ui.label(path.display().to_string());
                    } else {
                        ui.label("(no file)");
                    }
                });
            });
        });
}

/// Apply syntax highlighting to GCode text
/// Returns a LayoutJob with colored text spans
pub fn highlight_gcode(text: &str, colors: &SyntaxColors, font_size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let font_id = egui::FontId::monospace(font_size);
    
    for line in text.lines() {
        let tokens = tokenize_line_pure_rust(line, None);
        let mut pos = 0;
        
        for token in &tokens {
            // Skip tokens that overlap with already-processed text
            // This handles cases where the tokenizer returns overlapping tokens
            // (e.g., 'G21' as GCode and '21' as Number)
            if token.start < pos {
                continue;
            }
            
            // Add whitespace/text before token (clamp indices to avoid panics)
            let line_len = line.len();
            if token.start > pos {
                let gap_end = token.start.min(line_len);
                if pos < gap_end {
                    let gap = &line[pos..gap_end];
                    job.append(
                        gap,
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color: egui::Color32::WHITE,
                            ..default()
                        },
                    );
                }
            }

            // Get color for token type
            let color = match token.token_type {
                TokenType::GCode => array_to_color32(&colors.gcode),
                TokenType::MCode => array_to_color32(&colors.mcode),
                TokenType::Axis => array_to_color32(&colors.axis),
                TokenType::AxisNamed(_) => array_to_color32(&colors.axis),
                TokenType::Parameter => array_to_color32(&colors.parameter),
                TokenType::ParameterKey => array_to_color32(&colors.parameter),
                TokenType::ParameterValue => array_to_color32(&colors.parameter),
                TokenType::ParameterP => array_to_color32(&colors.parameter),
                TokenType::Number => array_to_color32(&colors.number),
                TokenType::Comment => array_to_color32(&colors.comment),
                TokenType::OCode => array_to_color32(&colors.ocode),
                TokenType::Operator => array_to_color32(&colors.operator),
                TokenType::Variable => array_to_color32(&colors.variable),
                TokenType::Error => array_to_color32(&colors.error),
                TokenType::Unknown => egui::Color32::WHITE,
            };

            // Clamp token range to the line bounds. If the token is malformed, skip it.
            let token_start = token.start.min(line_len);
            let token_end = token.start.saturating_add(token.length).min(line_len);
            if token_start >= token_end {
                // malformed, skip
                continue;
            }

            let token_text = &line[token_start..token_end];
            job.append(
                token_text,
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color,
                    ..default()
                },
            );

            pos = token_end;
        }
        
        // Add remaining text
        if pos < line.len() {
            job.append(
                &line[pos..],
                0.0,
                egui::TextFormat {
                    font_id: font_id.clone(),
                    color: egui::Color32::WHITE,
                    ..default()
                },
            );
        }
        
        // Add newline
        job.append(
            "\n",
            0.0,
            egui::TextFormat {
                font_id: font_id.clone(),
                color: egui::Color32::WHITE,
                ..default()
            },
        );
    }
    
    job
}

/// Create a LayoutJob for syntax highlighting with wrap width
/// This version is used by the TextEdit layouter callback
fn highlight_gcode_layouter(text: &str, colors: &SyntaxColors, font_size: f32, wrap_width: f32) -> egui::text::LayoutJob {
    let mut job = highlight_gcode(text, colors, font_size);
    job.wrap = egui::text::TextWrapping {
        max_width: wrap_width,
        ..Default::default()
    };
    job
}

fn array_to_color32(arr: &[f32; 4]) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        (arr[0] * 255.0) as u8,
        (arr[1] * 255.0) as u8,
        (arr[2] * 255.0) as u8,
        (arr[3] * 255.0) as u8,
    )
}

// ============================================================================
// UNIT TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_editor_state_default() {
        let state = EditorState::default();
        assert_eq!(state.cursor_pos, (0, 0));
        assert!(state.selection_start.is_none());
        assert!(state.selection_end.is_none());
        assert!(state.selected_lines.is_none());
        assert!(!state.focused);
        assert!(state.search_text.is_empty());
    }
    
    #[test]
    fn test_editor_state_next_id() {
        let mut state = EditorState::default();
        assert_eq!(state.next_id(), 1);
        assert_eq!(state.next_id(), 2);
        assert_eq!(state.next_id(), 3);
    }
    
    #[test]
    fn test_editor_change_event() {
        let event = EditorChangeEvent {
            new_content: "G0 X10 Y20".to_string(),
            selected_lines: Some((0, 5)),
        };
        assert_eq!(event.new_content, "G0 X10 Y20");
        assert_eq!(event.selected_lines, Some((0, 5)));
    }
    
    #[test]
    fn test_array_to_color32() {
        let arr = [1.0, 0.5, 0.25, 1.0];
        let color = array_to_color32(&arr);
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 127);
        assert_eq!(color.b(), 63);
        assert_eq!(color.a(), 255);
    }
    
    #[test]
    fn test_array_to_color32_zero() {
        let arr = [0.0, 0.0, 0.0, 0.0];
        let color = array_to_color32(&arr);
        assert_eq!(color.r(), 0);
        assert_eq!(color.g(), 0);
        assert_eq!(color.b(), 0);
        assert_eq!(color.a(), 0);
    }

    #[test]
    fn test_highlight_single_g() {
        // Ensure single-character input doesn't panic and produces at least one section
        let colors = crate::config::SyntaxColors::default();
        let job = highlight_gcode("G", &colors, 12.0);
        assert!(!job.sections.is_empty());
    }
}
