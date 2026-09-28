//! Self-contained `index.html` for a test run: the same renderer the hub uses, with the report
//! embedded, so a CI artifact or a copied folder opens without a server.
use super::model::Report;

const TEMPLATE: &str = include_str!("../../static/report.html");
pub const TESTRUN_CSS: &str = include_str!("../../static/testrun.css");
pub const TESTRUN_JS: &str = include_str!("../../static/testrun.js");

pub fn render(report: &Report) -> String {
    let data = serde_json::to_string(&serde_json::json!({ "report": report }))
        .unwrap_or_else(|_| "{}".into())
        // `<` only occurs inside JSON strings, where `\u003c` is equivalent; this keeps
        // `</script>` and `<!--` in test output from ending the data block.
        .replace('<', "\\u003c");
    let title = escape_html(&report.title);
    TEMPLATE
        .replace("{{AIRBUG_TITLE}}", &title)
        .replace("/*{{AIRBUG_CSS}}*/", TESTRUN_CSS)
        .replace("/*{{AIRBUG_JS}}*/", TESTRUN_JS)
        .replace("{{AIRBUG_DATA}}", &data)
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_data_cannot_close_the_script() {
        let report = Report {
            title: "<b>t</b>".into(),
            build_log: Some("</script><script>alert(1)</script>".into()),
            ..Report::default()
        };
        let html = render(&report);
        assert!(!html.contains("</script><script>alert(1)"));
        assert!(html.contains("&lt;b&gt;t&lt;/b&gt;"));
        assert!(html.contains("AirbugTestRun"));
        assert!(!html.contains("{{AIRBUG_DATA}}"));
    }
}
