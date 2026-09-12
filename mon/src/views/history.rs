use egui_plot::{Legend, Line, Plot, PlotPoints};

use super::{fmt_bytes, history_storage};
use crate::storage::HistoryPoint;

#[derive(Clone, Copy, PartialEq)]
enum Metric {
    Cpu,
    Ram,
    Gpu,
    Net,
    Disk,
}

impl Metric {
    fn label(self) -> &'static str {
        match self {
            Metric::Cpu => "CPU",
            Metric::Ram => "RAM",
            Metric::Gpu => "GPU",
            Metric::Net => "Сеть",
            Metric::Disk => "Диск",
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Range {
    Hour,
    SixHours,
    Day,
    Week,
    Month,
}

impl Range {
    fn label(self) -> &'static str {
        match self {
            Range::Hour => "1ч",
            Range::SixHours => "6ч",
            Range::Day => "24ч",
            Range::Week => "7д",
            Range::Month => "30д",
        }
    }

    fn secs(self) -> i64 {
        match self {
            Range::Hour => 3600,
            Range::SixHours => 6 * 3600,
            Range::Day => 24 * 3600,
            Range::Week => 7 * 24 * 3600,
            Range::Month => 30 * 24 * 3600,
        }
    }

    fn aggregated(self) -> bool {
        matches!(self, Range::Week | Range::Month)
    }
}

pub struct HistoryView {
    metric: Metric,
    range: Range,
    points: Vec<HistoryPoint>,
    status: String,
}

impl Default for HistoryView {
    fn default() -> Self {
        Self {
            metric: Metric::Cpu,
            range: Range::Hour,
            points: Vec::new(),
            status: String::new(),
        }
    }
}

impl HistoryView {
    fn reload(&mut self) {
        let from = chrono::Utc::now().timestamp() - self.range.secs();
        match history_storage() {
            Some(st) => match st.history(from, self.range.aggregated()) {
                Ok(pts) => {
                    self.status = format!("{} точек", pts.len());
                    self.points = pts;
                }
                Err(e) => self.status = format!("Ошибка: {e}"),
            },
            None => self.status = "БД недоступна".into(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for m in [Metric::Cpu, Metric::Ram, Metric::Gpu, Metric::Net, Metric::Disk] {
                if ui
                    .selectable_label(self.metric == m, m.label())
                    .clicked()
                {
                    self.metric = m;
                }
            }
            ui.separator();
            for r in [
                Range::Hour,
                Range::SixHours,
                Range::Day,
                Range::Week,
                Range::Month,
            ] {
                if ui.selectable_label(self.range == r, r.label()).clicked() {
                    self.range = r;
                }
            }
            ui.separator();
            if ui.button("Загрузить").clicked() {
                self.reload();
            }
            if ui.button("Экспорт CSV").clicked() {
                let from = chrono::Utc::now().timestamp() - self.range.secs();
                let path = std::env::temp_dir().join(format!(
                    "airbug_mon_export_{}.csv",
                    chrono::Utc::now().timestamp()
                ));
                match history_storage()
                    .map(|st| st.export_csv(from, self.range.aggregated(), &path))
                {
                    Some(Ok(())) => self.status = format!("Экспортировано: {}", path.display()),
                    Some(Err(e)) => self.status = format!("Ошибка экспорта: {e}"),
                    None => self.status = "БД недоступна".into(),
                }
            }
            ui.label(&self.status);
        });
        ui.separator();

        if self.points.is_empty() {
            ui.label("Нет данных. Нажмите «Загрузить».");
            return;
        }

        let series: Vec<(&str, PlotPoints)> = match self.metric {
            Metric::Cpu => vec![(
                "CPU %",
                self.points
                    .iter()
                    .map(|p| [p.ts as f64, p.cpu_total as f64])
                    .collect(),
            )],
            Metric::Ram => vec![(
                "RAM %",
                self.points
                    .iter()
                    .map(|p| {
                        [
                            p.ts as f64,
                            if p.mem_total > 0 {
                                100.0 * p.mem_used as f64 / p.mem_total as f64
                            } else {
                                0.0
                            },
                        ]
                    })
                    .collect(),
            )],
            Metric::Gpu => vec![(
                "GPU %",
                self.points
                    .iter()
                    .map(|p| [p.ts as f64, p.gpu_util.unwrap_or(0.0) as f64])
                    .collect(),
            )],
            Metric::Net => vec![
                ("rx B/s", self.points.iter().map(|p| [p.ts as f64, p.net_rx]).collect()),
                ("tx B/s", self.points.iter().map(|p| [p.ts as f64, p.net_tx]).collect()),
            ],
            Metric::Disk => vec![
                ("read B/s", self.points.iter().map(|p| [p.ts as f64, p.disk_r]).collect()),
                ("write B/s", self.points.iter().map(|p| [p.ts as f64, p.disk_w]).collect()),
            ],
        };

        let y_label = match self.metric {
            Metric::Cpu | Metric::Ram | Metric::Gpu => "%".to_string(),
            _ => "байт/с".to_string(),
        };
        Plot::new("history_plot")
            .height(ui.available_height() - 30.0)
            .legend(Legend::default())
            .x_axis_formatter(|mark, _| {
                chrono::DateTime::from_timestamp(mark.value as i64, 0)
                    .map(|d| d.format("%m-%d %H:%M").to_string())
                    .unwrap_or_default()
            })
            .y_axis_formatter(move |mark, _| {
                let v = mark.value;
                if y_label == "%" {
                    format!("{v:.0}%")
                } else {
                    fmt_bytes(v)
                }
            })
            .show(ui, |plot_ui| {
                for (name, pts) in series {
                    plot_ui.line(Line::new(name, pts));
                }
            });
    }
}
