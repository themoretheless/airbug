use std::time::Duration;

use crossbeam_channel::Receiver;

use crate::sampler::Sample;
use crate::views::{
    self, SharedState, alerts::AlertsView, history::HistoryView, processes::ProcessesView,
};

#[derive(PartialEq)]
enum Tab {
    Overview,
    Processes,
    History,
    Alerts,
}

pub struct MonApp {
    rx: Receiver<Sample>,
    state: SharedState,
    tab: Tab,
    processes: ProcessesView,
    history: HistoryView,
    alerts: AlertsView,
}

impl MonApp {
    pub fn new(rx: Receiver<Sample>) -> Self {
        Self {
            rx,
            state: SharedState::new(),
            tab: Tab::Overview,
            processes: ProcessesView::default(),
            history: HistoryView::default(),
            alerts: AlertsView::default(),
        }
    }
}

impl eframe::App for MonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(sample) = self.rx.try_recv() {
            self.state.push(sample);
        }
        self.alerts.evaluate(&self.state);

        egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Overview, "Обзор");
                ui.selectable_value(&mut self.tab, Tab::Processes, "Процессы");
                ui.selectable_value(&mut self.tab, Tab::History, "История");
                ui.selectable_value(&mut self.tab, Tab::Alerts, "Алерты");
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Overview => views::overview::show(ui, &self.state),
            Tab::Processes => self.processes.show(ui, &self.state),
            Tab::History => self.history.show(ui),
            Tab::Alerts => self.alerts.show(ui),
        });

        ctx.request_repaint_after(Duration::from_millis(250));
    }
}
