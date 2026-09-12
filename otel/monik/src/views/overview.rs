use egui_plot::{Legend, Line, Plot, PlotPoints};

use super::{fmt_bytes, SharedState};

fn series<F: Fn(&crate::sampler::Sample) -> f64>(state: &SharedState, f: F) -> PlotPoints<'_> {
    state
        .ring
        .iter()
        .enumerate()
        .map(|(i, s)| [i as f64, f(s)])
        .collect()
}

fn plot_lines<'a>(
    ui: &mut egui::Ui,
    id: &str,
    title: &str,
    lines: Vec<(&str, PlotPoints<'a>)>,
    y_fmt: impl Fn(f64) -> String + 'static,
) {
    ui.label(title);
    Plot::new(id)
        .height(140.0)
        .legend(Legend::default())
        .allow_scroll(false)
        .allow_zoom(false)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            for (name, pts) in lines {
                plot_ui.line(Line::new(name, pts));
            }
        });
    let _ = y_fmt;
}

pub fn show(ui: &mut egui::Ui, state: &SharedState) {
    let latest = match state.latest() {
        Some(s) => s.clone(),
        None => {
            ui.centered_and_justified(|ui| {
                ui.label("Ожидание данных…");
            });
            return;
        }
    };

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label(format!("CPU: {:.1}%", latest.cpu_total));
            ui.separator();
            ui.label(format!(
                "RAM: {} / {}",
                fmt_bytes(latest.mem_used as f64),
                fmt_bytes(latest.mem_total as f64)
            ));
            ui.separator();
            match &latest.gpu {
                Some(g) => ui.label(format!("GPU: {:.1}%", g.util)),
                None => ui.label("GPU недоступен (нужен sudo)"),
            };
            ui.separator();
            ui.label(format!(
                "Сеть: ↓{}/s ↑{}/s",
                fmt_bytes(latest.net_rx_rate),
                fmt_bytes(latest.net_tx_rate)
            ));
            ui.separator();
            ui.label(format!(
                "Диск: R {}/s W {}/s",
                fmt_bytes(latest.disk_read_rate),
                fmt_bytes(latest.disk_write_rate)
            ));
        });
        ui.separator();

        plot_lines(
            ui,
            "cpu_total",
            "CPU общий, %",
            vec![("total", series(state, |s| s.cpu_total as f64))],
            |v| format!("{v:.0}%"),
        );

        let cores = latest.cpu_per_core.len();
        let core_lines: Vec<(String, PlotPoints)> = (0..cores)
            .map(|c| {
                (
                    format!("core {c}"),
                    state
                        .ring
                        .iter()
                        .enumerate()
                        .map(|(i, s)| [i as f64, s.cpu_per_core.get(c).copied().unwrap_or(0.0) as f64])
                        .collect::<PlotPoints>(),
                )
            })
            .collect();
        ui.label("CPU по ядрам, %");
        Plot::new("cpu_cores")
            .height(140.0)
            .legend(Legend::default())
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .show(ui, |plot_ui| {
                for (name, pts) in core_lines {
                    plot_ui.line(Line::new(name, pts));
                }
            });

        plot_lines(
            ui,
            "mem",
            "Память, %",
            vec![
                (
                    "RAM",
                    series(state, |s| {
                        if s.mem_total > 0 {
                            100.0 * s.mem_used as f64 / s.mem_total as f64
                        } else {
                            0.0
                        }
                    }),
                ),
                (
                    "swap",
                    series(state, |s| {
                        if s.swap_total > 0 {
                            100.0 * s.swap_used as f64 / s.swap_total as f64
                        } else {
                            0.0
                        }
                    }),
                ),
            ],
            |v| format!("{v:.0}%"),
        );

        if latest.gpu.is_some() {
            plot_lines(
                ui,
                "gpu",
                "GPU, %",
                vec![(
                    "util",
                    series(state, |s| s.gpu.map(|g| g.util as f64).unwrap_or(0.0)),
                )],
                |v| format!("{v:.0}%"),
            );
        }

        plot_lines(
            ui,
            "net",
            "Сеть, байт/с",
            vec![
                ("rx", series(state, |s| s.net_rx_rate)),
                ("tx", series(state, |s| s.net_tx_rate)),
            ],
            fmt_bytes,
        );

        plot_lines(
            ui,
            "disk",
            "Диск, байт/с",
            vec![
                ("read", series(state, |s| s.disk_read_rate)),
                ("write", series(state, |s| s.disk_write_rate)),
            ],
            fmt_bytes,
        );
    });
}
