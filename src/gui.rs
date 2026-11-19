use std::path::PathBuf;
use std::time::{Duration, Instant};

use eframe::egui;
use egui::{Color32, Stroke};

use crate::csv;
use crate::format::{format_compact, format_number, format_percent};
use crate::settings::{self, Settings};
use crate::usage::{self, DayTotals, SessionTotals, UsageSnapshot};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Overview,
    Sessions,
    Models,
    Projects,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Label,
    Updated,
    Input,
    Output,
    Reasoning,
    CacheRead,
    Fresh,
    Total,
}

impl SortKey {
    fn label(self) -> &'static str {
        match self {
            Self::Label => "Session",
            Self::Updated => "Last active",
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Reasoning => "Reasoning",
            Self::CacheRead => "Cache read",
            Self::Fresh => "Fresh",
            Self::Total => "Total",
        }
    }

    fn value(self, row: &SessionTotals) -> i64 {
        match self {
            Self::Label => 0,
            Self::Updated => row.updated_at,
            Self::Input => row.input,
            Self::Output => row.output,
            Self::Reasoning => row.reasoning,
            Self::CacheRead => row.cache_read,
            Self::Fresh => row.fresh,
            Self::Total => row.total,
        }
    }
}

const SORT_KEYS: [SortKey; 8] = [
    SortKey::Label,
    SortKey::Updated,
    SortKey::Input,
    SortKey::Output,
    SortKey::Reasoning,
    SortKey::CacheRead,
    SortKey::Fresh,
    SortKey::Total,
];

enum Data {
    Ready(Box<UsageSnapshot>),
    Error(String),
}

struct DashboardApp {
    db_path: PathBuf,
    data: Data,
    tab: Tab,
    search: String,
    filter_model: String,
    filter_project: String,
    sort: SortKey,
    descending: bool,
    all_days: bool,
    notice: Option<String>,
    styled: bool,
    sync_seconds: u64,
    config_path: PathBuf,
    last_check: Instant,
    last_fingerprint: Vec<(u64, u128)>,
    last_synced: Option<String>,
}

impl DashboardApp {
    fn new(db_path: PathBuf) -> Self {
        let data = match usage::get_usage(&db_path) {
            Ok(snapshot) => Data::Ready(Box::new(snapshot)),
            Err(error) => Data::Error(error),
        };
        let config_path = settings::config_path();
        let config = settings::load_from(&config_path);
        Self {
            db_path: db_path.clone(),
            data,
            tab: Tab::Overview,
            search: String::new(),
            filter_model: String::new(),
            filter_project: String::new(),
            sort: SortKey::Total,
            descending: true,
            all_days: config.all_days,
            notice: None,
            styled: false,
            sync_seconds: config.auto_sync_seconds,
            config_path,
            last_check: Instant::now(),
            last_fingerprint: usage::db_fingerprint(&db_path),
            last_synced: Some(now_label()),
        }
    }

    fn persist(&mut self) {
        let settings = Settings {
            auto_sync_seconds: self.sync_seconds,
            all_days: self.all_days,
        };
        if let Err(error) = settings::save_to(&self.config_path, &settings) {
            self.notice = Some(format!("Could not save settings: {error}"));
        }
    }

    fn reload(&mut self) {
        self.data = match usage::get_usage(&self.db_path) {
            Ok(snapshot) => Data::Ready(Box::new(snapshot)),
            Err(error) => Data::Error(error),
        };
        self.last_fingerprint = usage::db_fingerprint(&self.db_path);
        self.last_synced = Some(now_label());
    }

    fn refresh(&mut self) {
        self.reload();
        self.notice = Some("Reloaded from the database".to_string());
    }

    fn export_csv(&mut self, content: String, filename: &str) {
        match std::fs::write(filename, content) {
            Ok(()) => self.notice = Some(format!("Saved {filename}")),
            Err(error) => self.notice = Some(format!("Could not save file: {error}")),
        }
    }
}

fn now_label() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

fn style() -> egui::Style {
    let mut style = egui::Style {
        visuals: egui::Visuals::dark(),
        ..Default::default()
    };
    style.visuals.panel_fill = Color32::from_gray(8);
    style.visuals.window_fill = Color32::from_gray(16);
    style.visuals.extreme_bg_color = Color32::from_gray(4);
    style.visuals.faint_bg_color = Color32::from_gray(20);
    style.visuals.selection.bg_fill = Color32::from_gray(70);
    style.visuals.selection.stroke = Stroke::new(1.0, Color32::from_gray(200));
    style.visuals.widgets.inactive.bg_fill = Color32::from_gray(28);
    style.visuals.widgets.hovered.bg_fill = Color32::from_gray(42);
    style.visuals.widgets.active.bg_fill = Color32::from_gray(56);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_gray(20);
    style
}

fn stat_card(ui: &mut egui::Ui, label: &str, value: &str, meta: &str) {
    egui::Frame::new()
        .fill(Color32::from_gray(16))
        .stroke(Stroke::new(1.0, Color32::from_gray(38)))
        .corner_radius(12.0)
        .inner_margin(egui::Margin::same(14))
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(label)
                    .size(13.0)
                    .color(Color32::from_gray(160)),
            );
            ui.label(egui::RichText::new(value).size(29.0).monospace());
            ui.label(
                egui::RichText::new(meta)
                    .size(12.0)
                    .color(Color32::from_gray(160)),
            );
        });
}

fn draw_chart(ui: &mut egui::Ui, days: &[DayTotals]) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), 240.0),
        egui::Sense::hover(),
    );
    if days.is_empty() || rect.width() < 60.0 {
        return;
    }
    let painter = ui.painter_at(rect);
    let inset_left = 46.0;
    let inset_right = 6.0;
    let inset_top = 8.0;
    let inset_bottom = 24.0;
    let plot = egui::Rect::from_min_max(
        egui::pos2(rect.left() + inset_left, rect.top() + inset_top),
        egui::pos2(rect.right() - inset_right, rect.bottom() - inset_bottom),
    );

    let max_total = days
        .iter()
        .map(|day| day.total.max(0) as f64)
        .fold(0.0, f64::max)
        .max(1.0);
    let step = crate::chart::nice_step(max_total / 4.0);
    let y_max = step * (max_total / step).ceil().max(1.0);
    let x = |index: usize| -> f32 {
        if days.len() <= 1 {
            plot.left()
        } else {
            plot.left() + (index as f32 / (days.len() - 1) as f32) * plot.width()
        }
    };
    let y = |value: f64| -> f32 {
        plot.bottom() - (value / y_max).clamp(0.0, 1.0) as f32 * plot.height()
    };

    let mut tick = 0.0;
    while tick <= y_max + 0.001 {
        let line_y = y(tick);
        painter.line_segment(
            [
                egui::pos2(plot.left(), line_y),
                egui::pos2(plot.right(), line_y),
            ],
            Stroke::new(1.0, Color32::from_gray(40)),
        );
        painter.text(
            egui::pos2(plot.left() - 6.0, line_y),
            egui::Align2::RIGHT_CENTER,
            format_compact(tick.round() as i64),
            egui::FontId::proportional(11.0),
            Color32::from_gray(150),
        );
        tick += step;
    }

    let mut fresh = Vec::new();
    fresh.push(egui::pos2(x(0), plot.bottom()));
    for (i, day) in days.iter().enumerate() {
        fresh.push(egui::pos2(x(i), y(day.fresh as f64)));
    }
    fresh.push(egui::pos2(x(days.len() - 1), plot.bottom()));
    painter.add(egui::Shape::convex_polygon(
        fresh,
        Color32::from_gray(120),
        Stroke::NONE,
    ));

    let mut cache = Vec::new();
    for (i, day) in days.iter().enumerate() {
        cache.push(egui::pos2(x(i), y(day.fresh as f64)));
    }
    for (i, day) in days.iter().enumerate().rev() {
        cache.push(egui::pos2(
            x(i),
            y((day.fresh + day.cache_read + day.cache_write) as f64),
        ));
    }
    painter.add(egui::Shape::convex_polygon(
        cache,
        Color32::from_gray(38),
        Stroke::NONE,
    ));

    let label_stride = (days.len() / 5).max(1);
    for i in (0..days.len()).step_by(label_stride) {
        painter.text(
            egui::pos2(x(i), rect.bottom() - 8.0),
            egui::Align2::CENTER_CENTER,
            &days[i].day,
            egui::FontId::proportional(11.0),
            Color32::from_gray(150),
        );
    }
}

fn table_header(ui: &mut egui::Ui, labels: &[&str]) {
    for label in labels {
        ui.label(egui::RichText::new(*label).color(Color32::from_gray(160)));
    }
    ui.end_row();
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.styled {
            ctx.set_style_of(egui::Theme::Dark, style());
            ctx.set_style_of(egui::Theme::Light, style());
            self.styled = true;
        }

        let mut switch_to = None;
        let mut refresh = false;
        ui.input(|input| {
            if input.key_pressed(egui::Key::Num1) {
                switch_to = Some(Tab::Overview);
            } else if input.key_pressed(egui::Key::Num2) {
                switch_to = Some(Tab::Sessions);
            } else if input.key_pressed(egui::Key::Num3) {
                switch_to = Some(Tab::Models);
            } else if input.key_pressed(egui::Key::Num4) {
                switch_to = Some(Tab::Projects);
            }
            if input.key_pressed(egui::Key::R)
                && !input.modifiers.ctrl
                && !input.modifiers.command
                && !input.modifiers.alt
            {
                refresh = true;
            }
        });
        if let Some(tab) = switch_to {
            self.tab = tab;
            self.notice = None;
        }
        if refresh {
            self.refresh();
        }

        if self.sync_seconds > 0 {
            let now = Instant::now();
            if now.duration_since(self.last_check) >= Duration::from_secs(self.sync_seconds) {
                self.last_check = now;
                let fingerprint = usage::db_fingerprint(&self.db_path);
                if fingerprint != self.last_fingerprint {
                    self.reload();
                }
            }
            ctx.request_repaint_after(Duration::from_millis(500));
        }

        egui::Frame::new()
            .inner_margin(egui::Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("token usage").size(22.0));
                    ui.label("Auto sync");
                    let current = settings::SYNC_OPTIONS
                        .iter()
                        .find(|(value, _)| *value == self.sync_seconds)
                        .map(|(_, label)| *label)
                        .unwrap_or("1 second");
                    egui::ComboBox::from_id_salt("auto-sync")
                        .selected_text(current)
                        .show_ui(ui, |ui| {
                            for (value, label) in settings::SYNC_OPTIONS {
                                if ui
                                    .selectable_value(&mut self.sync_seconds, value, label)
                                    .clicked()
                                {
                                    self.persist();
                                }
                            }
                        });
                    if ui.button("Refresh").clicked() {
                        self.refresh();
                    }
                });
                ui.label(
                    egui::RichText::new(format!("Reading {}", self.db_path.display()))
                        .size(12.0)
                        .color(Color32::from_gray(150)),
                );
                if let Some(time) = &self.last_synced {
                    ui.label(
                        egui::RichText::new(format!("Last synced {time}"))
                            .size(12.0)
                            .color(Color32::from_gray(150)),
                    );
                }
                if let Some(notice) = &self.notice {
                    ui.label(
                        egui::RichText::new(notice)
                            .size(12.0)
                            .color(Color32::from_gray(170)),
                    );
                }
                ui.separator();

                ui.horizontal(|ui| {
                    for (tab, label) in [
                        (Tab::Overview, "Overview"),
                        (Tab::Sessions, "Sessions"),
                        (Tab::Models, "Models"),
                        (Tab::Projects, "Projects"),
                    ] {
                        if ui.selectable_value(&mut self.tab, tab, label).clicked() {
                            self.notice = None;
                        }
                    }
                });
                ui.separator();

                match &self.data {
                    Data::Error(error) => {
                        ui.colored_label(
                            Color32::from_rgb(238, 138, 138),
                            "Could not load token usage",
                        );
                        ui.label(error);
                        ui.label("Set OPENCODE_DB_PATH or pass --db-path to use a different file.");
                    }
                    Data::Ready(snapshot) => {
                        let snapshot = snapshot.as_ref().clone();
                        match self.tab {
                            Tab::Overview => self.overview(ui, &snapshot),
                            Tab::Sessions => self.sessions(ui, &snapshot),
                            Tab::Models => self.models(ui, &snapshot),
                            Tab::Projects => self.projects(ui, &snapshot),
                        }
                    }
                }
            });
    }
}

impl DashboardApp {
    fn overview(&mut self, ui: &mut egui::Ui, snapshot: &UsageSnapshot) {
        ui.columns(4, |columns| {
            stat_card(
                &mut columns[0],
                "Today, all tokens",
                &format_compact(snapshot.today.total),
                &format!("Yesterday {}", format_compact(snapshot.yesterday.total)),
            );
            stat_card(
                &mut columns[1],
                "Today, fresh",
                &format_compact(snapshot.today.fresh),
                &format!(
                    "{} of today",
                    format_percent(snapshot.today.fresh, snapshot.today.total)
                ),
            );
            stat_card(
                &mut columns[2],
                "All time, all tokens",
                &format_compact(snapshot.all_time.total),
                &format!("{} messages", format_number(snapshot.message_count as i64)),
            );
            stat_card(
                &mut columns[3],
                "All time, fresh",
                &format_compact(snapshot.all_time.fresh),
                &format!(
                    "{} of all time",
                    format_percent(snapshot.all_time.fresh, snapshot.all_time.total)
                ),
            );
        });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Daily usage").size(17.0));
            if ui
                .selectable_value(&mut self.all_days, false, "Last 30 days")
                .clicked()
                || ui
                    .selectable_value(&mut self.all_days, true, "All time")
                    .clicked()
            {
                self.persist();
            }
        });
        let days: Vec<DayTotals> = if self.all_days {
            snapshot.per_day.clone()
        } else {
            snapshot
                .per_day
                .iter()
                .rev()
                .take(30)
                .rev()
                .cloned()
                .collect()
        };
        draw_chart(ui, &days);
        ui.label(
            egui::RichText::new(format!(
                "Previous 7 days daily average: {} tokens. Yesterday: {} tokens. Fresh means input, output and reasoning.",
                format_compact(snapshot.seven_day_average.total),
                format_compact(snapshot.yesterday.total)
            ))
            .size(12.0)
            .color(Color32::from_gray(150)),
        );

        ui.add_space(10.0);
        ui.label(egui::RichText::new("Daily detail").size(17.0));
        if ui.button("Export daily CSV").clicked() {
            let mut rows = days.clone();
            rows.reverse();
            self.export_csv(csv::daily_to_csv(&rows), "opencode-token-usage-daily.csv");
        }
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                egui::Grid::new("daily-grid").striped(true).show(ui, |ui| {
                    table_header(ui, &["Day", "Fresh", "Cache read", "Total"]);
                    for day in days.iter().rev() {
                        ui.label(&day.day);
                        ui.label(format_compact(day.fresh));
                        ui.label(format_compact(day.cache_read));
                        ui.label(format_number(day.total));
                        ui.end_row();
                    }
                });
            });

        ui.add_space(10.0);
        ui.label(egui::RichText::new("Sessions using the most tokens").size(17.0));
        egui::Grid::new("top-sessions")
            .striped(true)
            .show(ui, |ui| {
                table_header(ui, &["Session", "Model", "Fresh", "Cache read", "Total"]);
                for row in snapshot.sessions.iter().take(8) {
                    ui.label(&row.label);
                    ui.label(&row.model);
                    ui.label(format_compact(row.fresh));
                    ui.label(format_compact(row.cache_read));
                    ui.label(format_number(row.total));
                    ui.end_row();
                }
            });
    }

    fn sessions(&mut self, ui: &mut egui::Ui, snapshot: &UsageSnapshot) {
        ui.horizontal(|ui| {
            ui.label("Search");
            ui.text_edit_singleline(&mut self.search);
            ui.label("Model");
            let model_label = if self.filter_model.is_empty() {
                "All".to_string()
            } else {
                self.filter_model.clone()
            };
            egui::ComboBox::from_id_salt("filter-model")
                .selected_text(model_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_model, String::new(), "All");
                    for model in &snapshot.models {
                        ui.selectable_value(
                            &mut self.filter_model,
                            model.key.clone(),
                            &model.label,
                        );
                    }
                });
            ui.label("Project");
            let project_label = if self.filter_project.is_empty() {
                "All".to_string()
            } else {
                self.filter_project.clone()
            };
            egui::ComboBox::from_id_salt("filter-project")
                .selected_text(project_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_project, String::new(), "All");
                    for project in &snapshot.projects {
                        ui.selectable_value(
                            &mut self.filter_project,
                            project.label.clone(),
                            &project.label,
                        );
                    }
                });
            ui.label("Sort");
            egui::ComboBox::from_label("")
                .selected_text(self.sort.label())
                .show_ui(ui, |ui| {
                    for key in SORT_KEYS {
                        ui.selectable_value(&mut self.sort, key, key.label());
                    }
                });
            ui.selectable_value(&mut self.descending, false, "Ascending");
            ui.selectable_value(&mut self.descending, true, "Descending");
            if ui.button("Export CSV").clicked() {
                let rows = self.filtered_sessions(snapshot);
                self.export_csv(
                    csv::sessions_to_csv(&rows),
                    "opencode-token-usage-sessions.csv",
                );
            }
        });

        let rows = self.filtered_sessions(snapshot);
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("sessions-grid")
                .striped(true)
                .show(ui, |ui| {
                    table_header(
                        ui,
                        &[
                            "Session",
                            "Last active",
                            "Input",
                            "Output",
                            "Reasoning",
                            "Cache read",
                            "Fresh",
                            "Total",
                        ],
                    );
                    for row in &rows {
                        ui.label(&row.label);
                        ui.label(
                            chrono::DateTime::from_timestamp_millis(row.updated_at)
                                .map(|date| date.format("%m/%d/%Y").to_string())
                                .unwrap_or_default(),
                        );
                        ui.label(format_compact(row.input));
                        ui.label(format_compact(row.output));
                        ui.label(format_compact(row.reasoning));
                        ui.label(format_compact(row.cache_read));
                        ui.label(format_compact(row.fresh));
                        ui.label(format_number(row.total));
                        ui.end_row();
                    }
                });
        });
        if rows.is_empty() {
            ui.label("No sessions match this search.");
        }
    }

    fn models(&mut self, ui: &mut egui::Ui, snapshot: &UsageSnapshot) {
        self.breakdown(ui, &snapshot.models, snapshot.all_time.total, "models-grid");
    }

    fn projects(&mut self, ui: &mut egui::Ui, snapshot: &UsageSnapshot) {
        self.breakdown(
            ui,
            &snapshot.projects,
            snapshot.all_time.total,
            "projects-grid",
        );
    }

    fn breakdown(
        &mut self,
        ui: &mut egui::Ui,
        rows: &[crate::usage::NamedTotals],
        all_time: i64,
        id: &'static str,
    ) {
        let max = rows.first().map(|row| row.total).unwrap_or(0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new(id).striped(true).show(ui, |ui| {
                table_header(
                    ui,
                    &["Name", "Messages", "Fresh", "Cache read", "Total", "Share"],
                );
                for row in rows {
                    ui.label(&row.label);
                    ui.label(format_number(row.messages));
                    ui.label(format_compact(row.fresh));
                    ui.label(format_compact(row.cache_read));
                    ui.label(format_number(row.total));
                    let fraction = if max > 0 {
                        row.total as f32 / max as f32
                    } else {
                        0.0
                    };
                    ui.add(egui::ProgressBar::new(fraction).desired_width(70.0));
                    ui.label(format_percent(row.total, all_time));
                    ui.end_row();
                }
            });
        });
    }

    fn filtered_sessions(&self, snapshot: &UsageSnapshot) -> Vec<SessionTotals> {
        let mut rows = crate::usage::filter_sessions(
            &snapshot.sessions,
            &self.search,
            &self.filter_model,
            &self.filter_project,
        );
        let descending = self.descending;
        rows.sort_by(|a, b| {
            let order = match self.sort {
                SortKey::Label => a.label.cmp(&b.label),
                other => other.value(a).cmp(&other.value(b)),
            };
            if descending {
                order.reverse()
            } else {
                order
            }
        });
        rows
    }
}

pub fn run(db_path: PathBuf) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Token usage — OpenCode dashboard")
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Token usage",
        options,
        Box::new(move |_cc| Ok(Box::new(DashboardApp::new(db_path)))),
    )
}
