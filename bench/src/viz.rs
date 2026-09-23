//! Offline SVG charts and diagrams: no dependencies, no JavaScript, no network fonts.
//!
//! Coordinates are emitted as fixed-point numbers and every label goes through [`esc`],
//! so plots stay safe to embed in a report a person opens as a local file. [`Plot`] is the
//! canvas with primitives; [`charts`] builds the report and live views on top of it.
use crate::analysis::Decision;

/// Escape XML text content and attribute values.
pub fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Format a measured value for a label: no `-0`, no exponent for ordinary ranges,
/// trailing zeros removed.
pub fn fmt(v: f64) -> String {
    if !v.is_finite() {
        return "n/a".into();
    }
    let a = v.abs();
    let raw = if a >= 1e9 || (a > 0. && a < 1e-4) {
        format!("{v:.2e}")
    } else if a >= 100. {
        format!("{v:.0}")
    } else if a >= 1. {
        format!("{v:.2}")
    } else {
        format!("{v:.4}")
    };
    let trimmed = if raw.contains('.') && !raw.contains('e') {
        raw.trim_end_matches('0').trim_end_matches('.')
    } else {
        raw.as_str()
    };
    if trimmed == "-0" {
        "0".into()
    } else {
        trimmed.to_string()
    }
}

/// Colors shared by the report, the live page and every diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub ink: &'static str,
    pub grid: &'static str,
    pub good: &'static str,
    pub bad: &'static str,
    pub unresolved: &'static str,
    pub neutral: &'static str,
    pub band: &'static str,
}

impl Palette {
    pub const LIGHT: Palette = Palette {
        ink: "#192b35",
        grid: "#a4b4b2",
        good: "#09695d",
        bad: "#b32c34",
        unresolved: "#685286",
        neutral: "#5c6b78",
        band: "#0e2a3a0f",
    };

    /// Series colors for variants and lanes, cycled by index.
    pub const SERIES: [&'static str; 6] = [
        "#09695d", "#2f6f9e", "#b45309", "#685286", "#7d8b3c", "#a33a5e",
    ];

    /// Verdict colors need no direction parameter: [`Decision`] already accounts for the
    /// metric's declared direction, so a regression is red in every chart in the product.
    pub fn decision(&self, d: &Decision) -> &'static str {
        match d {
            Decision::Regression => self.bad,
            Decision::Improvement | Decision::WithinMargin => self.good,
            Decision::Inconclusive | Decision::Unavailable => self.unresolved,
            Decision::Neutral => self.neutral,
        }
    }

    pub fn series(&self, index: usize) -> &'static str {
        Self::SERIES[index % Self::SERIES.len()]
    }
}

/// A continuous or categorical scale, mapped to pixels when the plot renders.
#[derive(Clone, Debug, PartialEq)]
pub enum Domain {
    /// Bounds may be equal; padding keeps a single value from collapsing the axis.
    Linear { min: f64, max: f64 },
    /// Positive bounds; for values spanning orders of magnitude, e.g. throughput.
    Log { min: f64, max: f64 },
    /// Ordered labels; value `i` plots at the center of lane `i`.
    Categories { labels: Vec<String> },
}

impl Domain {
    /// Tight finite bounds over `values`, padded for headroom. Empty input yields `0..1`.
    pub fn fit(values: impl IntoIterator<Item = f64>) -> Self {
        let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
        for v in values {
            if v.is_finite() {
                min = min.min(v);
                max = max.max(v);
            }
        }
        if min.is_finite() && max.is_finite() {
            Self::Linear { min, max }.padded()
        } else {
            Self::Linear { min: 0., max: 1. }
        }
    }

    /// Widen to cover `v` (used so zero or a threshold always stays on canvas).
    pub fn include(mut self, v: f64) -> Self {
        if !v.is_finite() {
            return self;
        }
        match &mut self {
            Self::Linear { min, max } | Self::Log { min, max } => {
                *min = (*min).min(v);
                *max = (*max).max(v);
            }
            Self::Categories { .. } => (),
        }
        self
    }

    fn padded(self) -> Self {
        match self {
            Self::Linear { min, max } => {
                let span = max - min;
                let (lo, hi) = if span > 0. && span.is_finite() {
                    (min - span * 0.06, max + span * 0.06)
                } else {
                    let pad = min.abs().max(1.) * 0.1;
                    (min - pad, max + pad)
                };
                Self::Linear { min: lo, max: hi }
            }
            other => other,
        }
    }

    /// Value pair mapping to the two ends of a pixel span; for [`Domain::Categories`] a
    /// value is a lane index, so the span covers half-lanes of padding on each side.
    pub fn ends(&self) -> (f64, f64) {
        match self {
            Self::Linear { min, max } if max > min && min.is_finite() && max.is_finite() => {
                (*min, *max)
            }
            Self::Linear { min, .. } => (*min, *min + 1.),
            Self::Log { min, max } => {
                let lo = min.max(f64::MIN_POSITIVE);
                let hi = max.max(lo * 10.);
                (lo.log10(), hi.log10())
            }
            Self::Categories { labels } => (-0.5, labels.len().max(1) as f64 - 0.5),
        }
    }

    fn ticks(&self, target: usize) -> Vec<f64> {
        match self {
            Self::Log { min, max } => decade_ticks(*min, *max),
            Self::Categories { .. } => vec![],
            Self::Linear { min, max } => nice_steps(*min, *max, target),
        }
    }

    fn label(&self, v: f64) -> String {
        match self {
            Self::Categories { labels } => labels
                .get(v.round().max(0.) as usize)
                .cloned()
                .unwrap_or_default(),
            _ => fmt(v),
        }
    }
}

fn nice_steps(min: f64, max: f64, target: usize) -> Vec<f64> {
    let target = target.clamp(2, 12);
    let span = max - min;
    if !span.is_finite() || span <= 0. {
        return vec![min];
    }
    let raw = span / target as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let step = match raw / mag {
        n if n < 1.5 => 1.,
        n if n < 3. => 2.,
        n if n < 7. => 5.,
        _ => 10.,
    } * mag;
    let mut out = vec![];
    let mut v = (min / step).ceil() * step;
    while v <= max + step * 1e-9 && out.len() <= target * 3 {
        out.push(if v.abs() < step * 1e-9 { 0. } else { v });
        v += step;
    }
    out
}

/// Decade and 1/2/5 subdivisions inside a logarithmic domain.
fn decade_ticks(min: f64, max: f64) -> Vec<f64> {
    let lo = min.max(f64::MIN_POSITIVE).log10().floor() as i32;
    let hi = max.max(f64::MIN_POSITIVE).log10().ceil() as i32;
    let mut out = vec![];
    for decade in lo..=hi {
        for m in [1., 2., 5.] {
            let v = m * 10f64.powi(decade);
            if v >= min && v <= max && out.len() < 24 {
                out.push(v);
            }
        }
    }
    out
}

/// Canvas with axes, grid, bands and marks. Primitives accumulate SVG; [`Plot::figure`] renders it.
pub struct Plot {
    title: String,
    notes: String,
    palette: Palette,
    width: f64,
    height: f64,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    x: Domain,
    y: Domain,
    x_ticks: usize,
    y_ticks: usize,
    x_suffix: String,
    y_suffix: String,
    grid: bool,
    axes: bool,
    background: bool,
    structure: String,
    marks: String,
    overlay: String,
    legend: String,
}

impl Plot {
    /// 720x240 canvas with room for tick labels.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            notes: String::new(),
            palette: Palette::LIGHT,
            width: 720.,
            height: 240.,
            left: 64.,
            right: 16.,
            top: 16.,
            bottom: 34.,
            x: Domain::Linear { min: 0., max: 1. },
            y: Domain::Linear { min: 0., max: 1. },
            x_ticks: 5,
            y_ticks: 5,
            x_suffix: String::new(),
            y_suffix: String::new(),
            grid: true,
            axes: true,
            background: true,
            structure: String::new(),
            marks: String::new(),
            overlay: String::new(),
            legend: String::new(),
        }
    }

    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn margins(mut self, left: f64, right: f64, top: f64, bottom: f64) -> Self {
        (self.left, self.right, self.top, self.bottom) = (left, right, top, bottom);
        self
    }

    pub fn x(mut self, domain: Domain) -> Self {
        self.x = domain;
        self
    }

    pub fn y(mut self, domain: Domain) -> Self {
        self.y = domain;
        self
    }

    pub fn ticks(mut self, x: usize, y: usize) -> Self {
        (self.x_ticks, self.y_ticks) = (x, y);
        self
    }

    /// Text appended to tick labels, e.g. `%` or ` ms`.
    pub fn suffixes(mut self, x: impl Into<String>, y: impl Into<String>) -> Self {
        (self.x_suffix, self.y_suffix) = (x.into(), y.into());
        self
    }

    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = notes.into();
        self
    }

    /// Drop axes, ticks and the white card: for inline and live diagrams.
    pub fn bare(mut self) -> Self {
        (self.grid, self.axes, self.background) = (false, false, false);
        self
    }

    fn span(&self, horizontal: bool) -> (f64, f64) {
        if horizontal {
            (self.left, self.width - self.right)
        } else {
            (self.height - self.bottom, self.top)
        }
    }

    /// Pixel position of a value along a span; `None` for non-finite input.
    /// [`Plot::span`] maps the vertical axis bottom-to-top, so `t = 0` is the bottom edge.
    pub fn at(&self, v: f64, domain: &Domain, span: (f64, f64)) -> Option<f64> {
        if !v.is_finite() {
            return None;
        }
        let (lo, hi) = domain.ends();
        let t = (v - lo) / (hi - lo);
        if !t.is_finite() {
            return None;
        }
        Some(span.0 + t * (span.1 - span.0))
    }

    pub fn xpx(&self, v: f64) -> Option<f64> {
        self.at(v, &self.x, self.span(true))
    }

    pub fn ypx(&self, v: f64) -> Option<f64> {
        self.at(v, &self.y, self.span(false))
    }

    /// Extent of one categorical lane along an axis.
    pub fn lane(&self, horizontal: bool, lanes: usize) -> f64 {
        let (a, b) = self.span(horizontal);
        ((b - a).abs() / lanes.max(1) as f64).clamp(2., (b - a).abs())
    }

    pub fn plot_left(&self) -> f64 {
        self.left
    }

    pub fn plot_right(&self) -> f64 {
        self.width - self.right
    }

    pub fn colors(&self) -> Palette {
        self.palette
    }

    #[allow(clippy::too_many_arguments)]
    fn line(
        &self,
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        color: &str,
        width: f64,
        dash: &str,
    ) -> String {
        let dash = if dash.is_empty() {
            String::new()
        } else {
            format!(" stroke-dasharray=\"{dash}\"")
        };
        format!(
            "<line x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" stroke=\"{}\" stroke-width=\"{width:.2}\"{dash}/>",
            esc(color)
        )
    }

    fn text(&self, x: f64, y: f64, content: &str, anchor: &str, size: f64) -> String {
        format!(
            "<text x=\"{x:.2}\" y=\"{y:.2}\" text-anchor=\"{anchor}\" font-size=\"{size:.0}\" fill=\"{}\">{}</text>",
            self.palette.ink,
            esc(content)
        )
    }

    /// Shaded horizontal band, e.g. the practical margin around zero change.
    pub fn band_y(&mut self, from: f64, to: f64) -> &mut Self {
        if let (Some(a), Some(b)) = (self.ypx(from), self.ypx(to)) {
            let (top, height) = (a.min(b), (a - b).abs());
            self.structure.push_str(&format!(
                "<rect x=\"{:.2}\" y=\"{top:.2}\" width=\"{:.2}\" height=\"{height:.2}\" fill=\"{}\"/>",
                self.left,
                self.width - self.right - self.left,
                self.palette.band
            ));
        }
        self
    }

    /// Dashed reference line across the plot at a y value.
    pub fn guide_y(&mut self, v: f64, label: Option<&str>, color: &str) -> &mut Self {
        if let Some(y) = self.ypx(v) {
            self.overlay.push_str(&self.line(
                self.left,
                y,
                self.width - self.right,
                y,
                color,
                1.,
                "4 4",
            ));
            if let Some(label) = label {
                let x = self.width - self.right - 2.;
                self.overlay.push_str(&format!(
                    "<text x=\"{x:.2}\" y=\"{:.2}\" text-anchor=\"end\" font-size=\"11\" fill=\"{}\">{}</text>",
                    y - 4.,
                    esc(color),
                    esc(label)
                ));
            }
        }
        self
    }

    /// Dots with a per-point tooltip. Non-finite coordinates are skipped, never plotted at zero.
    pub fn points(&mut self, points: &[(f64, f64)], color: &str, radius: f64) -> &mut Self {
        for (x, y) in points {
            let (Some(cx), Some(cy)) = (self.xpx(*x), self.ypx(*y)) else {
                continue;
            };
            self.marks.push_str(&format!(
                "<circle cx=\"{cx:.2}\" cy=\"{cy:.2}\" r=\"{radius:.2}\" fill=\"{}\"><title>x = {}, y = {}</title></circle>",
                esc(color),
                fmt(*x),
                fmt(*y)
            ));
        }
        self
    }

    /// Polyline through the points in the order given.
    pub fn path(&mut self, points: &[(f64, f64)], color: &str, width: f64) -> &mut Self {
        let mut d = String::new();
        for (x, y) in points {
            let (Some(px), Some(py)) = (self.xpx(*x), self.ypx(*y)) else {
                continue;
            };
            let verb = if d.is_empty() { "M" } else { "L" };
            d.push_str(&format!("{verb}{px:.2} {py:.2}"));
        }
        if !d.is_empty() {
            self.marks.push_str(&format!(
                "<path d=\"{d}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linejoin=\"round\"/>",
                esc(color),
                width
            ));
        }
        self
    }

    /// Filled rectangle with a tooltip: a segment in a timeline, a median tick, a bar.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: &str, tip: &str) {
        self.marks.push_str(&format!(
            "<g><title>{}</title><rect x=\"{x:.2}\" y=\"{y:.2}\" width=\"{:.2}\" height=\"{:.2}\" fill=\"{}\"/></g>",
            esc(tip),
            width.max(1.),
            height.max(1.),
            esc(color)
        ));
    }

    /// Interval with a point estimate on the given row, labeled above it.
    #[allow(clippy::too_many_arguments)]
    pub fn interval(
        &mut self,
        row: f64,
        point: f64,
        low: f64,
        high: f64,
        color: &str,
        label: &str,
        tip: &str,
    ) -> &mut Self {
        let Some(y) = self.ypx(row) else { return self };
        self.marks
            .push_str(&self.text(self.left + 4., y - 8., label, "start", 12.));
        let (Some(a), Some(b), Some(m)) = (self.xpx(low), self.xpx(high), self.xpx(point)) else {
            return self;
        };
        self.marks.push_str(&format!(
            "<g><title>{}</title>{}<circle cx=\"{m:.2}\" cy=\"{y:.2}\" r=\"5\" fill=\"{}\"/></g>",
            esc(tip),
            self.line(a, y, b, y, color, 5., ""),
            esc(color)
        ));
        self
    }

    /// Horizontal bar from zero to `value` on the row, with its label in the left margin.
    pub fn bar(&mut self, row: f64, value: f64, color: &str, label: &str, lane: f64) {
        let (Some(y), Some(x)) = (self.ypx(row), self.xpx(value)) else {
            return;
        };
        let zero = self.xpx(0.).unwrap_or(self.left);
        let (left, width) = (x.min(zero), (x - zero).abs().max(1.));
        let thickness = (lane * 0.55).clamp(2., 18.);
        let short: String = label.chars().take(60).collect();
        self.marks.push_str(&format!(
            "<g><title>{}</title>{}{}</g>",
            esc(label),
            self.line(left, y, left + width, y, color, thickness, ""),
            self.text((self.left - 6.).max(0.), y + 4., &short, "end", 11.)
        ));
    }

    /// Text in the left margin at a lane, e.g. a variant or timeline row name.
    pub fn lane_label(&mut self, row: f64, text: &str, size: f64) -> &mut Self {
        let Some(y) = self.ypx(row) else { return self };
        let short: String = text.chars().take(90).collect();
        let x = (self.left - 6.).max(0.);
        self.marks
            .push_str(&self.text(x, y + 4., &short, "end", size));
        self
    }

    /// Text under a lane on the x axis, for categorical series that the axis itself does not label.
    pub fn bottom_label(&mut self, x: f64, text: &str) -> &mut Self {
        let Some(px) = self.xpx(x) else { return self };
        let short: String = text.chars().take(48).collect();
        let y = self.height - 14.;
        self.marks
            .push_str(&self.text(px, y, &short, "middle", 11.));
        self
    }

    /// Color key along the top of the canvas.
    pub fn legend(&mut self, entries: &[(String, &str)]) -> &mut Self {
        let mut x = self.left;
        for (label, color) in entries {
            self.legend.push_str(&format!(
                "<rect x=\"{x:.2}\" y=\"4\" width=\"10\" height=\"10\" fill=\"{}\"/><text x=\"{:.2}\" y=\"13\" font-size=\"11\">{}</text>",
                esc(color),
                x + 14.,
                esc(label)
            ));
            x += 34. + label.chars().count() as f64 * 6.6;
        }
        self
    }

    fn write_axes(&mut self) {
        if !self.axes {
            return;
        }
        let (left, right) = (self.left, self.width - self.right);
        let (top, bottom) = (self.top, self.height - self.bottom);
        self.structure.push_str(&format!(
            "<path d=\"M{left:.2} {top:.2}V{bottom:.2}H{right:.2}\" stroke=\"{}\" fill=\"none\"/>",
            self.palette.grid
        ));
        for v in self.y.ticks(self.y_ticks) {
            let Some(y) = self.ypx(v) else { continue };
            if self.grid {
                self.structure.push_str(&self.line(
                    left,
                    y,
                    right,
                    y,
                    self.palette.grid,
                    1.,
                    "2 3",
                ));
            }
            self.structure.push_str(&format!(
                "<text x=\"4\" y=\"{:.2}\" font-size=\"11\" fill=\"{}\">{}{}</text>",
                y + 4.,
                self.palette.ink,
                self.y.label(v),
                self.y_suffix
            ));
        }
        for v in self.x.ticks(self.x_ticks) {
            let Some(x) = self.xpx(v) else { continue };
            if self.grid {
                self.structure.push_str(&self.line(
                    x,
                    top,
                    x,
                    bottom,
                    self.palette.grid,
                    1.,
                    "2 3",
                ));
            }
            self.structure.push_str(&format!(
                "<text x=\"{x:.2}\" y=\"{:.2}\" text-anchor=\"middle\" font-size=\"11\" fill=\"{}\">{}{}</text>",
                self.height - 14.,
                self.palette.ink,
                self.x.label(v),
                self.x_suffix
            ));
        }
    }

    /// `<svg>` with grid, marks, guides and legend in that z-order.
    pub fn svg(&mut self) -> String {
        self.write_axes();
        let mut body = String::new();
        if self.background {
            body.push_str(&format!(
                "<rect width=\"{:.0}\" height=\"{:.0}\" fill=\"#fff\"/>",
                self.width, self.height
            ));
        }
        body.push_str(&std::mem::take(&mut self.structure));
        body.push_str(&std::mem::take(&mut self.marks));
        body.push_str(&std::mem::take(&mut self.overlay));
        body.push_str(&std::mem::take(&mut self.legend));
        format!(
            "<svg role=\"img\" aria-label=\"{}\" viewBox=\"0 0 {:.0} {:.0}\" style=\"width:100%;max-width:1000px\" font-family=\"ui-monospace,monospace\">{}</svg>",
            esc(&self.title),
            self.width,
            self.height,
            body
        )
    }

    /// Accessible figure: caption, SVG, then the caveat paragraph.
    pub fn figure(mut self) -> String {
        let svg = self.svg();
        let mut out = format!(
            "<figure class=\"viz\"><figcaption>{}</figcaption>{svg}",
            esc(&self.title)
        );
        if !self.notes.trim().is_empty() {
            out.push_str(&format!(
                "<p class=\"viz-notes\">{}</p>",
                esc(self.notes.trim())
            ));
        }
        out.push_str("</figure>");
        out
    }

    /// Compact inline figure without a caption, for the live banner.
    pub fn inline(mut self) -> String {
        let svg = self.svg();
        format!("<span class=\"viz viz-inline\">{svg}</span>")
    }
}

/// Report and live diagrams composed from [`Plot`] primitives.
pub mod charts {
    use super::{Domain, Palette, Plot, esc, fmt};
    use crate::analysis::Decision;

    /// One plotted cloud or line.
    pub struct Series {
        pub label: String,
        pub points: Vec<(f64, f64)>,
    }

    impl Series {
        pub fn new(label: impl Into<String>, points: Vec<(f64, f64)>) -> Self {
            Self {
                label: label.into(),
                points,
            }
        }
    }

    /// Shared-scale scatter of one or more series, thinned to `max_points` each.
    pub fn scatter(title: &str, series: &[Series], max_points: usize, notes: &str) -> String {
        let xs = series.iter().flat_map(|s| s.points.iter().map(|p| p.0));
        let ys = series.iter().flat_map(|s| s.points.iter().map(|p| p.1));
        let mut plot = Plot::new(title)
            .x(Domain::fit(xs))
            .y(Domain::fit(ys))
            .notes(notes);
        if series.len() > 1 {
            plot = plot.margins(64., 16., 26., 34.);
            let entries: Vec<_> = series
                .iter()
                .enumerate()
                .map(|(i, s)| (s.label.clone(), Palette::LIGHT.series(i)))
                .collect();
            plot.legend(&entries);
        }
        for (i, s) in series.iter().enumerate() {
            let stride = s.points.len().div_ceil(max_points.max(1)).max(1);
            let points: Vec<_> = s.points.iter().step_by(stride).copied().collect();
            plot.points(&points, Palette::LIGHT.series(i), 2.3);
        }
        plot.figure()
    }

    /// Single-series scatter for callers plotting one process or series at a time.
    pub fn dot_plot(title: &str, points: &[(f64, f64)], max_points: usize, notes: &str) -> String {
        scatter(
            title,
            &[Series::new(title, points.to_vec())],
            max_points,
            notes,
        )
    }

    /// A labeled interval estimate for the forest plot.
    pub struct Interval {
        pub label: String,
        pub point: f64,
        pub low: f64,
        pub high: f64,
        pub decision: Decision,
    }

    /// Point estimates and intervals per row, with zero and the practical margin as guides.
    pub fn forest(title: &str, rows: &[Interval], threshold: f64, notes: &str) -> String {
        let colors = Palette::LIGHT;
        let values = rows.iter().flat_map(|r| [r.point, r.low, r.high]);
        let mut domain = Domain::fit(values).include(0.);
        let usable = threshold.is_finite() && threshold > 0.;
        if usable {
            domain = domain.include(threshold).include(-threshold);
        }
        let height = 75. + rows.len() as f64 * 66.;
        let mut plot = Plot::new(title)
            .size(800., height)
            .margins(20., 20., 15., 40.)
            .x(domain)
            .y(Domain::Categories {
                labels: (0..rows.len()).map(|i| i.to_string()).collect(),
            })
            .ticks(6, 0)
            .suffixes("%", "")
            .notes(notes);
        if usable {
            plot.band_y(-threshold, threshold);
            plot.guide_y(threshold, None, colors.grid);
            plot.guide_y(-threshold, None, colors.grid);
        }
        plot.guide_y(0., Some("0"), colors.grid);
        for (i, r) in rows.iter().enumerate() {
            let tip = format!(
                "{}: {}% [{}, {}]",
                r.label,
                fmt(r.point),
                fmt(r.low),
                fmt(r.high)
            );
            plot.interval(
                i as f64,
                r.point,
                r.low,
                r.high,
                colors.decision(&r.decision),
                &r.label,
                &tip,
            );
        }
        plot.figure()
    }

    /// A categorical lane of independent-unit values plus its summary.
    pub struct Lane {
        pub label: String,
        pub values: Vec<f64>,
        /// Typically the median of the lane; drawn as a short thick tick.
        pub median: Option<f64>,
        /// Force a color, e.g. to keep a variant's hue across charts.
        pub color: Option<&'static str>,
    }

    /// Dots per independent unit inside categorical lanes, with a median tick per lane.
    pub fn strip(title: &str, lanes: &[Lane], unit: &str, notes: &str) -> String {
        let colors = Palette::LIGHT;
        let all = lanes.iter().flat_map(|l| l.values.iter().copied());
        let labels: Vec<_> = lanes
            .iter()
            .map(|l| format!("{} · n={}", l.label, l.values.len()))
            .collect();
        let mut plot = Plot::new(title)
            .size(720., 220.)
            .margins(190., 24., 16., 34.)
            .x(Domain::Categories { labels })
            .y(Domain::fit(all))
            .ticks(0, 5)
            .suffixes("", format!(" {unit}"))
            .notes(notes);
        let lane = plot.lane(true, lanes.len().max(1));
        for (i, l) in lanes.iter().enumerate() {
            let color = l.color.unwrap_or_else(|| colors.series(i));
            let n = l.values.len();
            // Deterministic spread across the lane instead of overlapping dots.
            let points: Vec<_> = l
                .values
                .iter()
                .enumerate()
                .map(|(k, v)| {
                    let offset = if n < 2 {
                        0.
                    } else {
                        (k as f64 / (n - 1) as f64 - 0.5) * 0.5
                    };
                    (i as f64 + offset, *v)
                })
                .collect();
            plot.points(&points, color, 3.2);
            plot.bottom_label(i as f64, &format!("{} · n={}", l.label, n));
            if let Some(median) = l.median
                && let (Some(y), Some(x)) = (plot.ypx(median), plot.xpx(i as f64))
            {
                let half = lane * 0.22;
                plot.rect(
                    x - half,
                    y - 2.,
                    half * 2.,
                    4.,
                    color,
                    &format!("{} median {}", l.label, fmt(median)),
                );
            }
        }
        plot.figure()
    }

    /// Diverging horizontal bars of a signed per-row effect, colored by verdict.
    pub fn bars(title: &str, rows: &[(String, f64, Decision)], unit: &str, notes: &str) -> String {
        let colors = Palette::LIGHT;
        let values = rows.iter().map(|(_, v, _)| *v);
        let height = 46. + rows.len() as f64 * 30.;
        let mut plot = Plot::new(title)
            .size(720., height)
            .margins(250., 24., 12., 30.)
            .x(Domain::fit(values).include(0.))
            .y(Domain::Categories {
                labels: (0..rows.len()).map(|i| i.to_string()).collect(),
            })
            .ticks(5, 0)
            .suffixes(format!(" {unit}"), "")
            .notes(notes);
        plot.guide_y(0., None, colors.grid);
        let lane = plot.lane(false, rows.len().max(1));
        for (i, (label, value, decision)) in rows.iter().enumerate() {
            plot.bar(i as f64, *value, colors.decision(decision), label, lane);
        }
        plot.figure()
    }

    /// A time span inside a timeline lane.
    pub struct Segment {
        pub start: f64,
        pub stop: f64,
        pub detail: String,
    }

    /// A lane of segments, e.g. one variant's processes over elapsed seconds.
    pub struct TimelineLane {
        pub label: String,
        pub segments: Vec<Segment>,
    }

    /// Gantt-style lane diagram over elapsed seconds.
    pub fn timeline(title: &str, lanes: &[TimelineLane], notes: &str) -> String {
        let colors = Palette::LIGHT;
        let end = lanes
            .iter()
            .flat_map(|l| l.segments.iter().map(|s| s.stop))
            .fold(1., f64::max);
        let height = 34. + lanes.len().max(1) as f64 * 24.;
        let mut plot = Plot::new(title)
            .size(720., height)
            .margins(150., 16., 12., 26.)
            .x(Domain::Linear {
                min: 0.,
                max: end.max(1.),
            })
            .y(Domain::Categories {
                labels: lanes.iter().map(|l| l.label.clone()).collect(),
            })
            .ticks(5, 0)
            .suffixes("s", "")
            .notes(notes);
        let lane = plot.lane(false, lanes.len().max(1));
        for (i, l) in lanes.iter().enumerate() {
            plot.lane_label(i as f64, &l.label, 11.);
            let Some(y) = plot.ypx(i as f64) else {
                continue;
            };
            let height = (lane * 0.5).clamp(3., 14.);
            for s in &l.segments {
                let (Some(a), Some(b)) = (plot.xpx(s.start), plot.xpx(s.stop)) else {
                    continue;
                };
                let detail = if s.detail.is_empty() {
                    String::new()
                } else {
                    format!(" · {}", s.detail)
                };
                plot.rect(
                    a,
                    y - height / 2.,
                    b - a,
                    height,
                    colors.series(i),
                    &format!("{} · {}–{}s{detail}", l.label, fmt(s.start), fmt(s.stop)),
                );
            }
        }
        plot.figure()
    }

    /// Axis-free trend line for a status banner.
    pub fn sparkline(title: &str, values: &[f64], color: Option<&str>) -> String {
        let points: Vec<_> = values
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v))
            .collect();
        let mut plot = Plot::new(title)
            .size(320., 46.)
            .margins(3., 3., 5., 5.)
            .x(Domain::Linear {
                min: 0.,
                max: (values.len().max(2) - 1) as f64,
            })
            .y(Domain::fit(values.iter().copied()))
            .bare();
        let color = color.unwrap_or(Palette::LIGHT.good);
        plot.path(&points, color, 2.);
        if let Some(last) = points.last() {
            plot.points(&[*last], color, 3.);
        }
        plot.inline()
    }

    /// Escape a caption for callers that assemble their own markup.
    pub fn label(text: &str) -> String {
        esc(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup_in_labels_and_titles() {
        let s = esc("<script>alert('x')</script>");
        assert!(!s.contains("<script>"));
        assert!(s.contains("&lt;script&gt;"));
        assert!(s.contains("&#39;"));
        let svg = charts::dot_plot("<b>t</b>", &[(0., 1.)], 100, "note");
        assert!(!svg.contains("<b>t</b>"));
        assert!(svg.contains("&lt;b&gt;t&lt;/b&gt;"));
        assert!(svg.contains("aria-label="));
        assert!(svg.contains("role=\"img\""));
    }

    #[test]
    fn formats_values_without_exponent_surprises() {
        assert_eq!(fmt(0.), "0");
        assert_eq!(fmt(-0.0), "0");
        assert_eq!(fmt(1.5), "1.5");
        assert_eq!(fmt(1234.56), "1235");
        assert_eq!(fmt(0.25), "0.25");
        assert_eq!(fmt(f64::NAN), "n/a");
        assert_eq!(fmt(f64::INFINITY), "n/a");
        assert_eq!(fmt(1e12), "1.00e12");
    }

    #[test]
    fn domains_handle_empty_single_and_non_finite_input() {
        assert_eq!(Domain::fit([]), Domain::Linear { min: 0., max: 1. });
        let Domain::Linear { min, max } = Domain::fit([5.]) else {
            panic!("expected a linear domain");
        };
        assert!(min < 5. && max > 5., "{min}..{max}");
        assert_eq!(
            Domain::fit([1., f64::NAN, 3., f64::INFINITY]),
            Domain::fit([1., 3.])
        );
        let widened = Domain::fit([1., 2.]).include(9.);
        let (lo, hi) = widened.ends();
        assert!(lo <= 1. && hi == 9., "{lo}..{hi}");
    }

    #[test]
    fn linear_ticks_are_nice_and_bounded() {
        let ticks = nice_steps(0., 100., 5);
        assert!(ticks.contains(&0.) && ticks.contains(&100.));
        assert!(ticks.windows(2).all(|w| w[0] < w[1]));
        assert!(ticks.len() <= 15);
        assert_eq!(nice_steps(5., 5., 5), vec![5.]);
    }

    #[test]
    fn log_ticks_and_mapping_span_decades() {
        let ticks = decade_ticks(1., 1000.);
        assert!(ticks.contains(&1.) && ticks.contains(&10.) && ticks.contains(&1000.));
        let plot = Plot::new("t")
            .x(Domain::Log {
                min: 1.,
                max: 1000.,
            })
            .y(Domain::Linear { min: 0., max: 1. });
        let left = plot.xpx(1.).unwrap();
        let right = plot.xpx(1000.).unwrap();
        let middle = plot.xpx(31.6).unwrap();
        assert!(left < middle && middle < right, "{left} {middle} {right}");
    }

    #[test]
    fn non_finite_values_are_dropped_not_plotted_at_zero() {
        let svg = charts::dot_plot("t", &[(0., 1.), (2., f64::NAN)], 100, "");
        assert_eq!(svg.matches("<circle").count(), 1);
        assert!(!svg.contains("NaN"));
    }

    #[test]
    fn rendering_is_deterministic() {
        let a = charts::dot_plot("t", &[(0., 1.), (1., 2.)], 100, "n");
        let b = charts::dot_plot("t", &[(0., 1.), (1., 2.)], 100, "n");
        assert_eq!(a, b);
    }

    #[test]
    fn forest_draws_margin_band_zero_guide_and_verdict_colors() {
        let rows = [
            charts::Interval {
                label: "parse / wall".into(),
                point: 12.,
                low: 8.,
                high: 16.,
                decision: Decision::Regression,
            },
            charts::Interval {
                label: "parse / cpu".into(),
                point: -1.,
                low: -1.5,
                high: -0.5,
                decision: Decision::Improvement,
            },
        ];
        let svg = charts::forest("Effects", &rows, 5., "caveat");
        assert!(svg.contains(Palette::LIGHT.band), "{svg}");
        assert!(svg.contains(Palette::LIGHT.bad));
        assert!(svg.contains(Palette::LIGHT.good));
        assert!(svg.matches("stroke-dasharray=\"4 4\"").count() >= 3);
        assert!(svg.contains("caveat"));
        assert!(svg.contains("%</text>"));
        assert!(svg.contains("parse / wall"));
    }

    #[test]
    fn forest_without_rows_or_threshold_stays_well_formed() {
        let svg = charts::forest("Empty", &[], 5., "none");
        assert!(svg.contains("<svg") && svg.contains("none"));
        let no_margin = charts::forest(
            "Single",
            &[charts::Interval {
                label: "a".into(),
                point: 0.,
                low: 0.,
                high: 0.,
                decision: Decision::Neutral,
            }],
            0.,
            "",
        );
        assert_eq!(no_margin.matches("stroke-dasharray=\"4 4\"").count(), 1);
    }

    #[test]
    fn strip_labels_lane_sizes_and_medians() {
        let lanes = [charts::Lane {
            label: "baseline".into(),
            values: vec![1., 2., 3.],
            median: Some(2.),
            color: None,
        }];
        let svg = charts::strip("Spread", &lanes, "ms", "note");
        assert!(svg.contains("baseline · n=3"));
        assert!(svg.contains("median"));
        assert_eq!(svg.matches("<circle").count(), 3);
        assert!(svg.contains("ms</text>"));
    }

    #[test]
    fn bars_plot_signed_extents_on_both_sides_of_zero() {
        let rows = vec![
            ("first".to_string(), 20., Decision::Regression),
            ("second".to_string(), -20., Decision::Improvement),
        ];
        let svg = charts::bars("Deltas", &rows, "%", "note");
        assert!(svg.matches("<line").count() >= 3);
        assert!(svg.contains("first") && svg.contains("second"));
        assert!(!svg.contains("width=\"-"));
    }

    #[test]
    fn timeline_clamps_segments_and_escapes_details() {
        let lanes = [charts::TimelineLane {
            label: "candidate".into(),
            segments: vec![
                charts::Segment {
                    start: 0.,
                    stop: 4.,
                    detail: "<i>pid 1</i>".into(),
                },
                charts::Segment {
                    start: 4.,
                    stop: 12.,
                    detail: String::new(),
                },
            ],
        }];
        let svg = charts::timeline("Schedule", &lanes, "seconds");
        assert!(svg.contains("&lt;i&gt;pid 1&lt;/i&gt;"));
        assert_eq!(svg.matches("<rect").count(), 3);
        assert!(!svg.contains("width=\"-"));
    }

    #[test]
    fn sparkline_has_no_axis_chrome() {
        let svg = charts::sparkline("progress", &[0., 1., 3., 6.], None);
        assert!(!svg.contains("<text"));
        assert!(svg.contains("<path"));
        assert!(svg.contains("viz-inline"));
    }

    #[test]
    fn empty_charts_stay_well_formed() {
        for out in [
            charts::sparkline("t", &[], None),
            charts::strip("t", &[], "ms", ""),
            charts::timeline("t", &[], ""),
            charts::bars("t", &[], "%", ""),
            charts::scatter("t", &[], 10, ""),
        ] {
            assert!(out.contains("<svg") && out.contains("</svg>"), "{out}");
            assert!(!out.contains("NaN") && !out.contains("inf"), "{out}");
        }
    }

    #[test]
    fn pixel_mapping_respects_orientation_and_finiteness() {
        let plot = Plot::new("t")
            .x(Domain::Linear { min: 0., max: 10. })
            .y(Domain::Linear { min: 0., max: 10. });
        assert_eq!(plot.xpx(0.).unwrap().round(), plot.plot_left());
        assert_eq!(plot.xpx(10.).unwrap().round(), plot.plot_right());
        assert!(plot.xpx(f64::NAN).is_none());
        assert!(plot.ypx(0.).unwrap() > plot.ypx(10.).unwrap());
    }

    #[test]
    fn categories_map_to_lane_centers() {
        let plot = Plot::new("t").x(Domain::Categories {
            labels: vec!["a".into(), "b".into(), "c".into()],
        });
        let a = plot.xpx(0.).unwrap();
        let b = plot.xpx(1.).unwrap();
        let c = plot.xpx(2.).unwrap();
        assert!((b - a - (c - b)).abs() < 0.01, "{a} {b} {c}");
        assert!(a > plot.plot_left() && c < plot.plot_right());
    }
}
