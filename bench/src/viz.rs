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

    /// Continuous cell color for heatmaps; `t` is the position on the ramp, `0..=1`.
    /// A signed ramp runs `bad -> light -> good`, so a change-% map reads the same way as
    /// [`Palette::decision`] does: red is a regression whichever direction the axis points.
    pub fn heat(&self, t: f64, signed: bool) -> String {
        let light = "#eef2f1";
        let t = if t.is_finite() { t.clamp(0., 1.) } else { 0.5 };
        if signed {
            if t < 0.5 {
                mix(light, self.bad, 1. - t * 2.)
            } else {
                mix(light, self.good, (t - 0.5) * 2.)
            }
        } else {
            mix(light, self.ink, t)
        }
    }
}

/// Linear blend of two `#rrggbb` colors; an unparsable operand is returned unchanged.
fn mix(from: &str, to: &str, t: f64) -> String {
    let (Some(a), Some(b)) = (rgb(from), rgb(to)) else {
        return if t < 0.5 {
            from.to_string()
        } else {
            to.to_string()
        };
    };
    let t = t.clamp(0., 1.);
    let ch = |i| (a[i] as f64 + (b[i] as f64 - a[i] as f64) * t).round() as u32;
    format!("#{:02x}{:02x}{:02x}", ch(0), ch(1), ch(2))
}

fn rgb(hex: &str) -> Option<[u8; 3]> {
    let s = hex.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
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

/// CSS that runs the staggered reveal emitted by [`Plot::figure`]; embed it once per page.
///
/// It is the only half of the mechanism that lives outside the diagram, and it is deliberately
/// optional: a page without it shows the finished frame, which is also what print and
/// `prefers-reduced-motion` force. The live UI omits it on purpose — its diagrams are replaced
/// every 1.5 s, so a reveal that lasts about two seconds would never settle.
///
/// The selectors must stay in sync with [`Plot::reveal_groups`]: a `.rv` wrapper carrying
/// `--s` (step in ms) around delay groups carrying `--d` (an index). A mark that should draw
/// itself instead of fading carries `class="dv"` and `pathLength="100"` ([`Plot::path`],
/// [`Plot::bar`]), which is what lets the dash arithmetic work without a measured length.
pub const REVEAL_CSS: &str = concat!(
    ".rv>g{opacity:0;animation:airbug-reveal 320ms cubic-bezier(.2,.7,.3,1) forwards;",
    "animation-delay:calc(var(--d,0)*var(--s,24ms))}",
    "@keyframes airbug-reveal{from{opacity:0;transform:translateY(5px)}",
    "to{opacity:1;transform:none}}",
    ".rv .dv{stroke-dasharray:100;stroke-dashoffset:100;",
    "animation:airbug-draw 520ms ease-out forwards;",
    "animation-delay:calc(var(--d,0)*var(--s,24ms))}",
    "@keyframes airbug-draw{to{stroke-dashoffset:0}}",
    "@media print{.rv>g{animation:none;opacity:1;transform:none}",
    ".rv .dv{animation:none;stroke-dasharray:none;stroke-dashoffset:0}}",
    "@media (prefers-reduced-motion:reduce){.rv>g{animation:none;opacity:1;transform:none}",
    ".rv .dv{animation:none;stroke-dasharray:none;stroke-dashoffset:0}}",
);

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
    marks: Vec<String>,
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
            marks: Vec::new(),
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
    /// The private `span` helper maps the vertical axis bottom-to-top, so `t = 0` is the bottom edge.
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
            self.marks.push(format!(
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
            self.marks.push(format!(
                "<path d=\"{d}\" pathLength=\"100\" class=\"dv\" fill=\"none\" stroke=\"{}\" stroke-width=\"{:.2}\" stroke-linejoin=\"round\"/>",
                esc(color),
                width
            ));
        }
        self
    }

    /// Filled rectangle with a tooltip: a segment in a timeline, a median tick, a bar.
    pub fn rect(&mut self, x: f64, y: f64, width: f64, height: f64, color: &str, tip: &str) {
        self.marks.push(format!(
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
        let caption = self.text(self.left + 4., y - 8., label, "start", 12.);
        let (Some(a), Some(b), Some(m)) = (self.xpx(low), self.xpx(high), self.xpx(point)) else {
            self.marks.push(caption);
            return self;
        };
        self.marks.push(format!(
            "{caption}<g><title>{}</title>{}<circle cx=\"{m:.2}\" cy=\"{y:.2}\" r=\"5\" fill=\"{}\"/></g>",
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
        // Drawn from zero outwards, so the reveal grows the bar; a 1px stub keeps a zero value visible.
        let end = if x >= zero {
            zero + (x - zero).max(1.)
        } else {
            zero - (zero - x).max(1.)
        };
        let thickness = (lane * 0.55).clamp(2., 18.);
        let short: String = label.chars().take(60).collect();
        self.marks.push(format!(
            "<g><title>{}</title><path d=\"M{zero:.2} {y:.2}L{end:.2} {y:.2}\" pathLength=\"100\" class=\"dv\" fill=\"none\" stroke=\"{}\" stroke-width=\"{thickness:.2}\"/>{}</g>",
            esc(label),
            esc(color),
            self.text((self.left - 6.).max(0.), y + 4., &short, "end", 11.)
        ));
    }

    /// Text in the left margin at a lane, e.g. a variant or timeline row name.
    pub fn lane_label(&mut self, row: f64, text: &str, size: f64) -> &mut Self {
        let Some(y) = self.ypx(row) else { return self };
        let short: String = text.chars().take(90).collect();
        let x = (self.left - 6.).max(0.);
        self.marks.push(self.text(x, y + 4., &short, "end", size));
        self
    }

    /// Text under a lane on the x axis, for categorical series that the axis itself does not label.
    pub fn bottom_label(&mut self, x: f64, text: &str) -> &mut Self {
        let Some(px) = self.xpx(x) else { return self };
        let short: String = text.chars().take(48).collect();
        let y = self.height - 14.;
        self.marks.push(self.text(px, y, &short, "middle", 11.));
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

    /// Marks in reading order, in at most 64 groups that [`REVEAL_CSS`] reveals one after
    /// another. The step lands on the wrapper as `--s` and the group index as `--d`, so the
    /// whole figure builds in about two seconds whatever its mark count, and the per-mark
    /// markup stays unchanged: a page without the CSS renders exactly the same pixels.
    pub fn reveal_groups(marks: &[String]) -> String {
        const GROUPS: usize = 64;
        let chunk = marks.len().div_ceil(GROUPS).max(1);
        let groups = marks.len().div_ceil(chunk).max(1);
        let step = (1800 / groups).clamp(10, 60);
        let mut out = format!("<g class=\"rv\" style=\"--s:{step}ms\">");
        for (i, part) in marks.chunks(chunk).enumerate() {
            out.push_str(&format!("<g style=\"--d:{i}\">"));
            for mark in part {
                out.push_str(mark);
            }
            out.push_str("</g>");
        }
        out.push_str("</g>");
        out
    }

    /// `<svg>` with grid, marks, guides and legend in that z-order; static.
    pub fn svg(&mut self) -> String {
        self.render(false)
    }

    fn render(&mut self, reveal: bool) -> String {
        self.write_axes();
        let mut body = String::new();
        if self.background {
            body.push_str(&format!(
                "<rect width=\"{:.0}\" height=\"{:.0}\" fill=\"#fff\"/>",
                self.width, self.height
            ));
        }
        body.push_str(&std::mem::take(&mut self.structure));
        let marks = std::mem::take(&mut self.marks);
        if reveal {
            body.push_str(&Self::reveal_groups(&marks));
        } else {
            for mark in marks {
                body.push_str(&mark);
            }
        }
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

    /// Accessible figure: caption, then the SVG building itself in, then the caveat paragraph.
    ///
    /// The reveal needs [`REVEAL_CSS`] on the page; without it this is the finished frame.
    pub fn figure(mut self) -> String {
        let svg = self.render(true);
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

    /// Cumulative share of each lane's units at or below a value, one line per lane.
    ///
    /// Where [`strip`] shows which values occurred, this answers how much of a lane sits under a
    /// threshold, which is what a tail question needs. Order statistics are joined by a line
    /// rather than drawn as steps: below a few hundred units the two read the same, and the line
    /// costs a fraction of the markup. A lane with no finite value contributes nothing but its
    /// legend entry, so an unmeasured variant stays visibly empty.
    pub fn ecdf(title: &str, lanes: &[Lane], unit: &str, notes: &str) -> String {
        let colors = Palette::LIGHT;
        let sorted: Vec<Vec<f64>> = lanes
            .iter()
            .map(|l| {
                let mut v: Vec<f64> = l.values.iter().copied().filter(|v| v.is_finite()).collect();
                v.sort_by(|a, b| a.total_cmp(b));
                v
            })
            .collect();
        let mut plot = Plot::new(title)
            .size(720., 260.)
            .margins(64., 24., 16., 34.)
            .x(Domain::fit(sorted.iter().flatten().copied()))
            .y(Domain::Linear { min: 0., max: 100. })
            .ticks(5, 4)
            .suffixes(format!(" {unit}"), " %")
            .notes(notes);
        for (i, values) in sorted.iter().enumerate() {
            if values.is_empty() {
                continue;
            }
            let color = lanes[i].color.unwrap_or_else(|| colors.series(i));
            let n = values.len();
            let points: Vec<(f64, f64)> = values
                .iter()
                .enumerate()
                .map(|(k, v)| (*v, 100. * (k as f64 + 1.) / n as f64))
                .collect();
            plot.path(&points, color, 2.);
        }
        let legend: Vec<(String, &'static str)> = lanes
            .iter()
            .enumerate()
            .map(|(i, l)| {
                let color = l.color.unwrap_or_else(|| colors.series(i));
                (format!("{} \u{00b7} n={}", l.label, sorted[i].len()), color)
            })
            .collect();
        plot.legend(&legend);
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

    /// How cell values map onto the color ramp in [`heatmap`].
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Heat {
        /// One ramp across the whole matrix: any two cells are directly comparable.
        Global,
        /// One ramp per row, for a matrix whose rows are measured on different scales.
        RowLocal,
        /// Signed ramp with `center` at the neutral stop, e.g. `0` for a change in percent.
        Signed { center: f64 },
    }

    /// One row of a heatmap: a label plus one optional value per column.
    pub struct HeatRow {
        pub label: String,
        /// Missing data is a blank cell, never a zero-colored one. Short rows read as blanks.
        pub cells: Vec<Option<f64>>,
    }

    /// Matrix of values as colored cells: case × metric, process × case, variant × case.
    ///
    /// Every drawn cell carries a tooltip with its full label and value, so the figure stays
    /// readable when column names are truncated to fit. Each cell costs about 100 bytes of
    /// markup, hence `max_cells`: whole rows are drawn until the budget is used and the rest
    /// are counted in the caption rather than drawn half.
    pub fn heatmap(
        title: &str,
        cols: &[String],
        rows: &[HeatRow],
        scale: Heat,
        unit: &str,
        max_cells: usize,
        notes: &str,
    ) -> String {
        let colors = Palette::LIGHT;
        let ncols = cols.len().max(1);
        let cell_h = 22.;
        let mut plot = Plot::new(title)
            .size(800., 62. + rows.len() as f64 * cell_h)
            .margins(240., 16., 28., 34.)
            .x(Domain::Categories {
                labels: cols.to_vec(),
            })
            .y(Domain::Categories {
                labels: rows.iter().map(|r| r.label.clone()).collect(),
            })
            .ticks(0, 0);
        let cell_w = plot.lane(true, ncols);
        let budget = max_cells.min(rows.len() * ncols);
        let all: Vec<f64> = rows.iter().flat_map(|r| extent_cells(&r.cells)).collect();
        let (gmin, gmax) = extent(&all);
        let half = match scale {
            Heat::Signed { center } => all
                .iter()
                .map(|v| (v - center).abs())
                .fold(0., f64::max)
                .max(1e-9),
            _ => 0.,
        };
        let norm = |v: f64, lo: f64, hi: f64| match scale {
            Heat::Signed { center } => 0.5 + (v - center) / (2. * half),
            _ if hi > lo => (v - lo) / (hi - lo),
            _ => 0.5,
        };
        let signed = matches!(scale, Heat::Signed { .. });
        let mut slots = 0usize;
        let mut skipped = 0usize;
        for (i, r) in rows.iter().enumerate() {
            if slots + ncols > budget {
                skipped += rows.len() - i;
                break;
            }
            slots += ncols;
            let row: Vec<f64> = extent_cells(&r.cells).collect();
            let (rlo, rhi) = match scale {
                Heat::RowLocal => extent(&row),
                _ => (gmin, gmax),
            };
            plot.lane_label(i as f64, &r.label, 11.);
            for (j, cell) in r.cells.iter().enumerate().take(ncols) {
                let Some(v) = cell.filter(|v| v.is_finite()) else {
                    continue;
                };
                let (Some(cx), Some(cy)) = (plot.xpx(j as f64), plot.ypx(i as f64)) else {
                    continue;
                };
                let t = norm(v, rlo, rhi);
                plot.rect(
                    cx - cell_w / 2. + 0.5,
                    cy - cell_h / 2. + 0.5,
                    cell_w - 1.,
                    cell_h - 1.,
                    &colors.heat(t, signed),
                    &format!("{} · {} = {}", r.label, cols[j], with_unit(v, unit)),
                );
            }
        }
        for (j, col) in cols.iter().enumerate().take(ncols) {
            plot.bottom_label(j as f64, &clip(col, (cell_w / 6.6).floor() as usize));
        }
        let key = |t: f64, label: String| (label, colors.heat(t, signed));
        let legend: Vec<(String, String)> = match scale {
            Heat::Signed { center } => vec![
                key(0., with_unit(-half, unit)),
                key(0.5, with_unit(center, unit)),
                key(1., with_unit(half, unit)),
            ],
            Heat::Global => vec![
                key(0., with_unit(gmin, unit)),
                key(0.5, with_unit((gmin + gmax) / 2., unit)),
                key(1., with_unit(gmax, unit)),
            ],
            Heat::RowLocal => vec![
                (String::from("row min"), colors.heat(0.25, false)),
                (String::from("row max"), colors.heat(1., false)),
            ],
        };
        let legend: Vec<_> = legend
            .iter()
            .map(|(t, c)| (t.clone(), c.as_str()))
            .collect();
        plot.legend(&legend);
        let mut notes = notes.trim().to_string();
        notes.push_str(" Blank cells have no measurement; a blank is not zero.");
        if skipped > 0 {
            notes.push_str(&format!(
                " {skipped} of {} rows are not drawn to keep the figure readable; their values are in the JSON report.",
                rows.len()
            ));
        }
        plot.notes(notes).figure()
    }

    /// Finite values of a heat row, for the scale extent.
    fn extent_cells(cells: &[Option<f64>]) -> impl Iterator<Item = f64> + '_ {
        cells.iter().flatten().copied().filter(|v| v.is_finite())
    }

    /// Tight finite bounds over a row or the whole matrix; empty input yields `0..1`.
    fn extent(values: &[f64]) -> (f64, f64) {
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for v in values {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
        if lo.is_finite() && hi.is_finite() {
            (lo, hi)
        } else {
            (0., 1.)
        }
    }

    /// A value with its unit, or the bare value when the matrix mixes units.
    fn with_unit(v: f64, unit: &str) -> String {
        let unit = unit.trim();
        if unit.is_empty() {
            fmt(v)
        } else {
            format!("{} {unit}", fmt(v))
        }
    }

    /// Clip to `chars` with an ellipsis, never below one visible character.
    fn clip(text: &str, chars: usize) -> String {
        let chars = chars.max(1);
        let count = text.chars().count();
        if count <= chars {
            return text.to_string();
        }
        let mut out: String = text.chars().take(chars.saturating_sub(1)).collect();
        out.push('…');
        out
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

    fn delays(svg: &str) -> Vec<usize> {
        svg.split("--d:")
            .skip(1)
            .map(|part| {
                part.chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .unwrap()
            })
            .collect()
    }

    fn rows(n: usize) -> Vec<charts::Interval> {
        (0..n)
            .map(|i| charts::Interval {
                label: format!("case {i}"),
                point: i as f64,
                low: i as f64 - 1.,
                high: i as f64 + 1.,
                decision: Decision::Neutral,
            })
            .collect()
    }

    #[test]
    fn figure_reveals_its_marks_one_group_after_another() {
        let figure = charts::forest("Effects", &rows(5), 5., "note");
        assert!(figure.contains("<g class=\"rv\" style=\"--s:"), "{figure}");
        assert_eq!(delays(&figure), vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn svg_and_inline_render_the_finished_frame() {
        let mut plot = Plot::new("t");
        plot.rect(1., 1., 4., 4., "#000", "cell");
        assert!(!plot.svg().contains("--d:"));
        let mut plot = Plot::new("t");
        plot.rect(1., 1., 4., 4., "#000", "cell");
        assert!(!plot.inline().contains("class=\"rv\""));
    }

    #[test]
    fn reveal_stays_bounded_whatever_the_mark_count() {
        let marks = vec!["<rect/>".to_string(); 500];
        let wrapped = Plot::reveal_groups(&marks);
        let groups = delays(&wrapped);
        assert!(groups.len() <= 64 && groups.len() > 48, "{}", groups.len());
        assert_eq!(wrapped.matches("<rect/>").count(), 500);
        assert!(Plot::reveal_groups(&[]).contains("class=\"rv\""));
    }

    /// `d` of the marks the reveal CSS draws in, so the axis path never counts as one.
    fn drawn(markup: &str) -> Vec<String> {
        markup
            .split("d=\"")
            .skip(1)
            .filter(|rest| rest.contains("pathLength=\"100\" class=\"dv\""))
            .map(|rest| rest[..rest.find('"').unwrap_or(rest.len())].to_string())
            .collect()
    }

    #[test]
    fn lines_and_bars_are_marked_so_they_draw_instead_of_fading() {
        let scale = Plot::new("t")
            .x(Domain::Linear {
                min: -20.,
                max: 20.,
            })
            .y(Domain::Linear { min: 0., max: 10. });
        let (zero, left, right) = (
            scale.xpx(0.).unwrap(),
            scale.xpx(-12.).unwrap(),
            scale.xpx(7.).unwrap(),
        );
        let mut plot = Plot::new("t")
            .x(Domain::Linear {
                min: -20.,
                max: 20.,
            })
            .y(Domain::Linear { min: 0., max: 10. });
        plot.path(&[(0., 1.), (1., 2.)], "#000", 2.);
        plot.bar(3., -12., "#000", "negative", 20.);
        plot.bar(4., 7., "#000", "positive", 20.);
        let figure = plot.figure();
        assert_eq!(
            figure.matches("pathLength=\"100\" class=\"dv\"").count(),
            3,
            "{figure}"
        );
        let bars = drawn(&figure);
        assert_eq!(bars.len(), 3, "{figure}");
        // Both bars start on the zero axis and run outwards, so the reveal grows them.
        assert!(
            bars[1].starts_with(&format!("M{zero:.2} ")),
            "{:?}",
            bars[1]
        );
        assert!(bars[1].contains(&format!("L{left:.2} ")), "{:?}", bars[1]);
        assert!(bars[2].contains(&format!("L{right:.2} ")), "{:?}", bars[2]);
    }

    #[test]
    fn reveal_css_draws_the_lines_and_forces_the_finished_frame() {
        assert!(
            REVEAL_CSS.contains(".rv .dv{stroke-dasharray:100;stroke-dashoffset:100"),
            "{REVEAL_CSS}"
        );
        assert!(REVEAL_CSS.contains("@keyframes airbug-draw{to{stroke-dashoffset:0}}"));
        assert_eq!(REVEAL_CSS.matches("@keyframes ").count(), 2);
        for context in ["@media print", "@media (prefers-reduced-motion:reduce)"] {
            let block = REVEAL_CSS.split(context).nth(1).unwrap_or_default();
            assert!(
                block.contains("stroke-dasharray:none"),
                "{context}: {block}"
            );
        }
    }

    #[test]
    fn ecdf_rises_monotonically_to_the_top_of_the_axis() {
        let lanes = [charts::Lane {
            label: "candidate".into(),
            values: vec![30., 10., 20., 10.],
            median: None,
            color: None,
        }];
        let figure = charts::ecdf("Cumulative share", &lanes, "ms", "note");
        assert_eq!(drawn(&figure).len(), 1, "{figure}");
        assert!(figure.contains("candidate \u{00b7} n=4"), "{figure}");
        let vertices: Vec<(f64, f64)> = drawn(&figure)[0]
            .trim_start_matches('M')
            .split(['L', 'M'])
            .map(|pair| {
                let mut it = pair.split(' ');
                (
                    it.next().unwrap().parse().unwrap(),
                    it.next().unwrap().parse().unwrap(),
                )
            })
            .collect();
        assert_eq!(vertices.len(), 4);
        assert!(
            vertices.windows(2).all(|w| w[0].0 <= w[1].0),
            "{vertices:?}"
        );
        // Screen y runs downwards, so a growing share is a shrinking pixel.
        assert!(
            vertices.windows(2).all(|w| w[0].1 >= w[1].1),
            "{vertices:?}"
        );
        assert!((vertices[3].1 - 16.).abs() < 0.01, "{vertices:?}");
    }

    #[test]
    fn ecdf_drops_non_finite_units_and_an_unmeasured_lane_stays_visible() {
        let lanes = [
            charts::Lane {
                label: "<script>a</script>".into(),
                values: vec![1., f64::NAN, 2., f64::INFINITY],
                median: None,
                color: None,
            },
            charts::Lane {
                label: "unmeasured".into(),
                values: vec![f64::NAN],
                median: None,
                color: Some(Palette::LIGHT.series(1)),
            },
        ];
        let figure = charts::ecdf("t", &lanes, "ms", "note");
        assert_eq!(drawn(&figure).len(), 1, "{figure}");
        assert!(
            figure.contains("&lt;script&gt;a&lt;/script&gt; \u{00b7} n=2"),
            "{figure}"
        );
        assert!(figure.contains("unmeasured \u{00b7} n=0"), "{figure}");
        assert!(!figure.contains("<script"), "{figure}");
    }

    #[test]
    fn heat_ramp_centers_on_zero_and_clamps_out_of_range_scales() {
        let colors = Palette::LIGHT;
        assert_eq!(colors.heat(0., false), colors.heat(0.5, true));
        assert_eq!(colors.heat(0.5, true), colors.heat(f64::NAN, true));
        assert_eq!(colors.heat(0., true), colors.heat(-9., true));
        assert_eq!(colors.heat(1., true), colors.heat(9., true));
        assert_ne!(colors.heat(0.25, true), colors.heat(0.75, true));
    }

    #[test]
    fn heatmap_labels_every_cell_escapes_them_and_leaves_blanks_empty() {
        let cols = vec!["<b>rss</b>".into(), "cpu".into(), "wall".into()];
        let heat = vec![
            charts::HeatRow {
                label: "parse".into(),
                cells: vec![Some(2.), None, Some(-1.)],
            },
            charts::HeatRow {
                label: "load".into(),
                cells: vec![Some(1.)],
            },
        ];
        let svg = charts::heatmap(
            "H",
            &cols,
            &heat,
            charts::Heat::Signed { center: 0. },
            "%",
            240,
            "note",
        );
        assert!(svg.contains("&lt;b&gt;rss&lt;/b&gt;"), "{svg}");
        assert!(!svg.contains("<b>rss</b>"));
        assert!(svg.contains("parse · wall = -1 %"));
        assert!(svg.contains("load · &lt;b&gt;rss&lt;/b&gt; = 1 %"));
        assert!(!svg.contains("parse · cpu ="), "missing must stay blank");
        assert!(svg.contains("-2 %"));
        assert!(svg.contains("0 %"));
        assert!(svg.contains("2 %"));
        assert!(svg.contains("Blank cells have no measurement"));
    }

    #[test]
    fn heatmap_scales_and_caps_rows_it_does_not_draw() {
        let cols: Vec<String> = (0..8).map(|i| format!("c{i}")).collect();
        let heat: Vec<_> = (0..40)
            .map(|r| charts::HeatRow {
                label: format!("r{r}"),
                cells: cols.iter().map(|_| Some(1. + r as f64)).collect(),
            })
            .collect();
        let svg = charts::heatmap("H", &cols, &heat, charts::Heat::Global, "ms", 24, "");
        assert!(svg.contains("37 of 40 rows are not drawn"), "{svg}");
        assert!(svg.contains("r2</text>"));
        assert!(!svg.contains("r3</text>"));
        assert!(svg.contains("20.5 ms"));
        let local = charts::heatmap("H", &cols, &heat, charts::Heat::RowLocal, "", 400, "");
        assert!(local.contains("row min") && local.contains("row max"));
        assert!(!local.contains("rows are not drawn"));
    }
}
