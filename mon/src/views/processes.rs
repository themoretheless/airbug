use std::collections::HashMap;

use egui_plot::{Line, Plot, PlotPoints};

use super::{fmt_bytes, SharedState};
use crate::sampler::ProcSample;

#[derive(Clone, Copy, PartialEq)]
enum SortCol {
    Name,
    Pid,
    Cpu,
    Mem,
    Read,
    Write,
}

pub struct ProcessesView {
    filter: String,
    sort_col: SortCol,
    sort_desc: bool,
    selected_pid: Option<u32>,
    history: HashMap<u32, VecDequeShim>,
}

type VecDequeShim = std::collections::VecDeque<(f32, u64)>;

impl Default for ProcessesView {
    fn default() -> Self {
        Self {
            filter: String::new(),
            sort_col: SortCol::Cpu,
            sort_desc: true,
            selected_pid: None,
            history: HashMap::new(),
        }
    }
}

impl ProcessesView {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let latest = match state.latest() {
            Some(s) => s.clone(),
            None => {
                ui.label("Ожидание данных…");
                return;
            }
        };

        for p in &latest.top_procs {
            let entry = self.history.entry(p.pid).or_default();
            if entry.len() >= super::RING_CAPACITY {
                entry.pop_front();
            }
            entry.push_back((p.cpu, p.mem));
        }

        ui.horizontal(|ui| {
            ui.label("Фильтр:");
            ui.text_edit_singleline(&mut self.filter);
        });
        ui.separator();

        let mut procs: Vec<ProcSample> = latest
            .top_procs
            .iter()
            .filter(|p| {
                self.filter.is_empty()
                    || p.name.to_lowercase().contains(&self.filter.to_lowercase())
                    || p.pid.to_string().contains(&self.filter)
            })
            .cloned()
            .collect();

        let desc = self.sort_desc;
        procs.sort_by(|a, b| {
            let ord = match self.sort_col {
                SortCol::Name => a.name.cmp(&b.name),
                SortCol::Pid => a.pid.cmp(&b.pid),
                SortCol::Cpu => a.cpu.partial_cmp(&b.cpu).unwrap_or(std::cmp::Ordering::Equal),
                SortCol::Mem => a.mem.cmp(&b.mem),
                SortCol::Read => a
                    .read_rate
                    .partial_cmp(&b.read_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
                SortCol::Write => a
                    .write_rate
                    .partial_cmp(&b.write_rate)
                    .unwrap_or(std::cmp::Ordering::Equal),
            };
            if desc {
                ord.reverse()
            } else {
                ord
            }
        });

        let header = |ui: &mut egui::Ui, col: SortCol, label: &str, view: &mut ProcessesView| {
            let text = if view.sort_col == col {
                format!("{} {}", label, if view.sort_desc { "▼" } else { "▲" })
            } else {
                label.to_string()
            };
            if ui.selectable_label(false, text).clicked() {
                if view.sort_col == col {
                    view.sort_desc = !view.sort_desc;
                } else {
                    view.sort_col = col;
                    view.sort_desc = true;
                }
            }
        };

        egui::Grid::new("proc_grid")
            .striped(true)
            .num_columns(6)
            .show(ui, |ui| {
                header(ui, SortCol::Name, "Имя", self);
                header(ui, SortCol::Pid, "PID", self);
                header(ui, SortCol::Cpu, "CPU %", self);
                header(ui, SortCol::Mem, "RAM", self);
                header(ui, SortCol::Read, "Read/s", self);
                header(ui, SortCol::Write, "Write/s", self);
                ui.end_row();
                for p in &procs {
                    let selected = self.selected_pid == Some(p.pid);
                    if ui.selectable_label(selected, &p.name).clicked()
                        || ui.selectable_label(selected, p.pid.to_string()).clicked()
                    {
                        self.selected_pid = if selected { None } else { Some(p.pid) };
                    }
                    ui.label(format!("{:.1}", p.cpu));
                    ui.label(fmt_bytes(p.mem as f64));
                    ui.label(fmt_bytes(p.read_rate));
                    ui.label(fmt_bytes(p.write_rate));
                    ui.end_row();
                }
            });

        if let Some(pid) = self.selected_pid {
            if let Some(hist) = self.history.get(&pid) {
                if !hist.is_empty() {
                    ui.separator();
                    ui.label(format!("История процесса PID {pid}"));
                    let cpu_pts: PlotPoints = hist
                        .iter()
                        .enumerate()
                        .map(|(i, (c, _))| [i as f64, *c as f64])
                        .collect();
                    let mem_pts: PlotPoints = hist
                        .iter()
                        .enumerate()
                        .map(|(i, (_, m))| [i as f64, *m as f64 / 1_048_576.0])
                        .collect();
                    Plot::new("proc_hist")
                        .height(120.0)
                        .legend(egui_plot::Legend::default())
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new("CPU %", cpu_pts));
                            plot_ui.line(Line::new("RAM MB", mem_pts));
                        });
                }
            }
        }
    }
}
