use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::SharedState;

pub struct AlertRule {
    pub name: &'static str,
    pub enabled: bool,
    pub threshold_pct: f32,
    pub sustain: Duration,
}

pub struct AlertEntry {
    pub time: String,
    pub message: String,
}

pub struct AlertsView {
    pub cpu_rule: AlertRule,
    pub ram_rule: AlertRule,
    pub disk_rule: AlertRule,
    pub log: VecDeque<AlertEntry>,
    cpu_since: Option<Instant>,
    ram_since: Option<Instant>,
    disk_since: Option<Instant>,
}

impl Default for AlertsView {
    fn default() -> Self {
        Self {
            cpu_rule: AlertRule {
                name: "CPU",
                enabled: true,
                threshold_pct: 90.0,
                sustain: Duration::from_secs(60),
            },
            ram_rule: AlertRule {
                name: "RAM",
                enabled: true,
                threshold_pct: 90.0,
                sustain: Duration::from_secs(60),
            },
            disk_rule: AlertRule {
                name: "Диск I/O (доля от пика)",
                enabled: false,
                threshold_pct: 95.0,
                sustain: Duration::from_secs(60),
            },
            log: VecDeque::new(),
            cpu_since: None,
            ram_since: None,
            disk_since: None,
        }
    }
}

impl AlertsView {
    fn check(
        since: &mut Option<Instant>,
        rule: &AlertRule,
        value_pct: f32,
        log: &mut VecDeque<AlertEntry>,
    ) {
        if !rule.enabled {
            *since = None;
            return;
        }
        if value_pct > rule.threshold_pct {
            let start = since.get_or_insert_with(Instant::now);
            if start.elapsed() >= rule.sustain {
                let message = format!(
                    "{}: {:.1}% выше порога {:.0}% в течение {} с",
                    rule.name,
                    value_pct,
                    rule.threshold_pct,
                    rule.sustain.as_secs()
                );
                let _ = notify_rust::Notification::new()
                    .summary("airbug-mon: алерт")
                    .body(&message)
                    .show();
                log.push_front(AlertEntry {
                    time: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    message,
                });
                if log.len() > 200 {
                    log.pop_back();
                }
                *since = None;
            }
        } else {
            *since = None;
        }
    }

    pub fn evaluate(&mut self, state: &SharedState) {
        let Some(latest) = state.latest() else { return };
        Self::check(
            &mut self.cpu_since,
            &self.cpu_rule,
            latest.cpu_total,
            &mut self.log,
        );
        let ram_pct = if latest.mem_total > 0 {
            100.0 * latest.mem_used as f32 / latest.mem_total as f32
        } else {
            0.0
        };
        Self::check(&mut self.ram_since, &self.ram_rule, ram_pct, &mut self.log);
        let disk_pct = {
            let peak = state
                .ring
                .iter()
                .map(|s| s.disk_read_rate + s.disk_write_rate)
                .fold(1.0f64, f64::max);
            100.0 * (latest.disk_read_rate + latest.disk_write_rate) as f32 / peak as f32
        };
        Self::check(
            &mut self.disk_since,
            &self.disk_rule,
            disk_pct,
            &mut self.log,
        );
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Пороги");
        for rule in [&mut self.cpu_rule, &mut self.ram_rule, &mut self.disk_rule] {
            ui.horizontal(|ui| {
                ui.checkbox(&mut rule.enabled, rule.name);
                ui.add(
                    egui::DragValue::new(&mut rule.threshold_pct)
                        .range(1.0..=100.0)
                        .suffix("%"),
                );
                ui.label("в течение");
                let mut secs = rule.sustain.as_secs() as u32;
                if ui
                    .add(egui::DragValue::new(&mut secs).range(1..=3600).suffix(" с"))
                    .changed()
                {
                    rule.sustain = Duration::from_secs(secs as u64);
                }
            });
        }
        ui.separator();
        ui.heading("Журнал алертов");
        if self.log.is_empty() {
            ui.label("Срабатываний нет.");
        }
        egui::ScrollArea::vertical().show(ui, |ui| {
            for entry in &self.log {
                ui.label(format!("[{}] {}", entry.time, entry.message));
            }
        });
    }
}
