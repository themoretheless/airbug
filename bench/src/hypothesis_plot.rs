//! Standalone SVG for a retained Welch null distribution.
use crate::{
    Result,
    hypothesis::{WelchDistribution, histogram},
    report::escape,
};
use std::fmt::Write;

pub fn svg(distribution: &WelchDistribution, title: &str) -> Result<String> {
    let data = histogram(distribution, 40)?;
    let mut output = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 360" role="img" style="max-width:900px;width:100%;background:white;font:12px system-ui,sans-serif;color:#243247"><title>{}</title><desc>Pooled null distribution of Welch t statistics. Blue bars count null draws; orange segments count draws at least as extreme as the observed statistic.</desc><text x="60" y="27" font-size="16" font-weight="600">Welch null distribution</text>"#,
        escape(title)
    );
    if data.total == 0 {
        write!(
            output,
            "<text x=\"60\" y=\"85\">{}</text></svg>",
            escape(
                distribution
                    .test
                    .unavailable_reason
                    .as_deref()
                    .unwrap_or("No retained null draws")
            )
        )
        .unwrap();
        return Ok(output);
    }
    let peak = data
        .bins
        .iter()
        .map(|bin| bin.count)
        .max()
        .unwrap_or(0)
        .max(1) as f64;
    let extent = data.bins.last().unwrap().upper;
    let x = |value: f64| 60.0 + (value / extent + 1.0) * 320.0;
    let y = |count: f64| 255.0 - count / peak * 180.0;
    for fraction in [0.0, 0.5, 1.0] {
        let value = peak * fraction;
        let yy = y(value);
        write!(output, r##"<path d="M60 {yy:.2}H700" stroke="#e4e9f0"/><text x="50" y="{:.2}" text-anchor="end">{value:.0}</text>"##, yy + 4.0).unwrap();
    }
    for (index, bin) in data.bins.iter().enumerate() {
        let xx = 60.0 + index as f64 * 16.0;
        let height = bin.count as f64 / peak * 180.0;
        let tail = bin.extreme_count as f64 / peak * 180.0;
        write!(output, r##"<g><title>{:.4e} to {:.4e}: {} draws, {} extreme</title><rect x="{:.2}" y="{:.2}" width="15" height="{height:.2}" fill="#397ec0"/><rect x="{:.2}" y="{:.2}" width="15" height="{tail:.2}" fill="#e38c36"/></g>"##, bin.lower, bin.upper, bin.count, bin.extreme_count, xx + 0.5, y(bin.count as f64), xx + 0.5, y(bin.count as f64)).unwrap();
    }
    for fraction in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        let value = extent * fraction;
        write!(
            output,
            "<text x=\"{:.2}\" y=\"276\" text-anchor=\"middle\">{value:.2e}</text>",
            x(value)
        )
        .unwrap();
    }
    if let Some(observed) = data.observed {
        for boundary in [-observed.abs(), observed.abs()] {
            write!(
                output,
                r##"<path d="M{:.2} 64V255" stroke="#a75b0c" stroke-dasharray="4 4"/>"##,
                x(boundary)
            )
            .unwrap();
        }
        write!(output, r##"<path d="M{:.2} 56V255" stroke="#ad2845" stroke-width="2"/><text x="60" y="49">Observed t = {observed:.4}; two-sided p = {:.6}</text>"##, x(observed), distribution.test.p_value.unwrap_or(1.0)).unwrap();
    }
    write!(output, r##"<text x="380" y="299" text-anchor="middle">Welch t statistic</text><text x="60" y="322">{} null draws · {} extreme · infinities: −{} / +{}</text><text x="60" y="343">Blue: null draws · orange: extreme draws · red line: observed t</text></svg>"##, data.total, distribution.test.extreme_resamples, data.negative_infinity, data.positive_infinity).unwrap();
    Ok(output)
}
/// Render comparison rows as a portable HTML document with inline SVG charts.
/// Missing captures are explicit; no new statistical samples are generated.
pub fn html(rows: &[crate::analysis::Comparison], title: &str) -> Result<String> {
    let mut output = format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{0}</title><body><main><h1>{0}</h1>",
        escape(title)
    );
    output.push_str(&fragment(rows)?);
    output.push_str("</main></body></html>");
    Ok(output)
}

/// Embeddable comparison charts without a second HTML document.
pub fn fragment(rows: &[crate::analysis::Comparison]) -> Result<String> {
    let mut output = String::new();
    if rows.is_empty() {
        output.push_str("<p>No comparisons available.</p>");
    }
    for row in rows {
        let label = format!("{} · {} ({})", row.case, row.metric, row.unit);
        write!(output, "<section><h2>{}</h2>", escape(&label)).unwrap();
        match &row.hypothesis {
            Some(result) => {
                if let Some(draws) = &result.null_distribution {
                    output.push_str(&svg(
                        &WelchDistribution {
                            test: result.test.clone(),
                            null_statistics: draws.clone(),
                        },
                        &label,
                    )?);
                } else {
                    output.push_str("<p>Null distribution was not retained.</p>");
                }
                write!(output, "<p>Significance level: {:.6}. Independent units: baseline {}, candidate {}.</p>", result.significance_level, result.test.baseline_units, result.test.candidate_units).unwrap();
            }
            None => output.push_str("<p>No independent-sample Welch test for this comparison.</p>"),
        }
        write!(output, "<p>{}</p></section>", escape(&row.note)).unwrap();
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn document_distinguishes_uncaptured_paired_and_retained_rows() {
        let distribution = crate::hypothesis::welch_distribution(
            &[1., 2., 3.],
            &[2., 3., 4.],
            crate::hypothesis::Config {
                resamples: 128,
                seed: 1,
            },
        )
        .unwrap();
        let mut row = crate::analysis::Comparison {
            case: "<case>".into(),
            metric: "<metric>".into(),
            unit: "ns".into(),
            scope: "process".into(),
            baseline: None,
            candidate: None,
            change_percent: None,
            interval_percent: None,
            independent_units: 3,
            hypothesis: None,
            decision: crate::analysis::Decision::Inconclusive,
            note: "<script>bad()</script>".into(),
        };
        let paired = row.clone();
        row.hypothesis = Some(crate::analysis::HypothesisResult {
            method: "welch".into(),
            test: distribution.test,
            null_distribution: None,
            significance_level: 0.05,
            rejects_zero_effect: Some(false),
        });
        let uncaptured = row.clone();
        row.hypothesis.as_mut().unwrap().null_distribution = Some(distribution.null_statistics);
        let document = html(&[paired, uncaptured, row], "<title>").unwrap();
        assert_eq!(document.matches("<section>").count(), 3);
        assert_eq!(document.matches("<svg ").count(), 1);
        assert!(document.contains("No independent-sample Welch test"));
        assert!(document.contains("was not retained"));
        assert!(document.contains("128 null draws"));
        assert!(document.contains("&lt;case&gt;"));
        assert!(!document.contains("<script>"));
        assert!(
            html(&[], "Empty")
                .unwrap()
                .contains("No comparisons available")
        );
    }
    #[test]
    fn escaped_accessible_svg_handles_available_and_unavailable_distributions() {
        let distribution = crate::hypothesis::welch_distribution(
            &[1., 2., 3.],
            &[2., 3., 4.],
            crate::hypothesis::Config {
                resamples: 128,
                seed: 1,
            },
        )
        .unwrap();
        let image = svg(&distribution, "<script>alert(1)</script>").unwrap();
        assert!(image.contains("&lt;script&gt;"));
        assert!(!image.contains("<script>"));
        assert!(image.contains("role=\"img\""));
        assert!(image.contains("128 null draws"));
        assert!(!image.contains("NaN"));
        let unavailable = crate::hypothesis::welch_distribution(
            &[1.],
            &[2.],
            crate::hypothesis::Config::default(),
        )
        .unwrap();
        assert!(
            svg(&unavailable, "Unavailable")
                .unwrap()
                .contains("at least two independent units")
        );
    }
}
