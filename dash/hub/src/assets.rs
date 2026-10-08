//! The dashboard's static files, embedded at build time and served from one table.
//!
//! Adding a file to `static/` means adding one row to [`ASSETS`]; nothing else changes.

use crate::testrun::html::{TESTRUN_CSS, TESTRUN_JS};

const HTML: &str = "text/html; charset=utf-8";
const CSS: &str = "text/css; charset=utf-8";
const JS: &str = "text/javascript; charset=utf-8";

/// The dashboard page itself, served at `/` and `/index.html`.
pub const PAGE: &str = include_str!("../static/dashboard.html");

macro_rules! asset {
    ($path:literal, $mime:expr) => {
        (
            concat!("/static/", $path),
            $mime,
            include_str!(concat!("../static/", $path)),
        )
    };
}

/// `(url path, content type, body)`.
const ASSETS: &[(&str, &str, &str)] = &[
    asset!("dashboard.css", CSS),
    ("/static/testrun.css", CSS, TESTRUN_CSS),
    ("/static/testrun.js", JS, TESTRUN_JS),
    asset!("js/main.js", JS),
    asset!("js/shell.js", JS),
    asset!("js/api.js", JS),
    asset!("js/dom.js", JS),
    asset!("js/format.js", JS),
    asset!("js/charts.js", JS),
    asset!("js/runs.js", JS),
    asset!("js/sections/now.js", JS),
    asset!("js/sections/runs.js", JS),
    asset!("js/sections/tests.js", JS),
    asset!("js/sections/bench.js", JS),
    asset!("js/sections/issues.js", JS),
    asset!("js/sections/logs.js", JS),
    asset!("js/sections/metrics.js", JS),
    asset!("js/sections/health.js", JS),
];

/// `(content type, body)` for a URL path, or `None` when it is not a static asset.
pub fn lookup(path: &str) -> Option<(&'static str, &'static str)> {
    match path {
        "/" | "/index.html" => Some((HTML, PAGE)),
        _ => ASSETS
            .iter()
            .find(|(p, ..)| *p == path)
            .map(|(_, mime, body)| (*mime, *body)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{collections::BTreeSet, fs, path::Path};

    fn files_under(dir: &Path, out: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                files_under(&path, out);
            } else if path.extension().is_some_and(|e| e == "js") {
                out.insert(path.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    /// Every module the page can import is served; a forgotten row would 404 in the browser.
    #[test]
    fn every_js_module_is_served() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("static");
        let mut found = BTreeSet::new();
        files_under(&root.join("js"), &mut found);
        let prefix = root.to_string_lossy().replace('\\', "/");
        for file in found {
            let url = format!("/static{}", &file[prefix.len()..]);
            assert!(lookup(&url).is_some(), "{url} is not in ASSETS");
        }
    }

    /// Relative imports resolve to served modules.
    #[test]
    fn imports_resolve() {
        for (path, _, body) in ASSETS.iter().filter(|(p, ..)| p.starts_with("/static/js/")) {
            let dir = &path[..path.rfind('/').unwrap()];
            for spec in body
                .lines()
                .filter_map(|l| l.trim().strip_prefix("import "))
                .filter_map(|l| l.split('"').nth(1))
            {
                let mut parts: Vec<&str> = dir.split('/').collect();
                for seg in spec.split('/') {
                    match seg {
                        "." => {}
                        ".." => {
                            parts.pop();
                        }
                        s => parts.push(s),
                    }
                }
                let url = parts.join("/");
                assert!(
                    lookup(&url).is_some(),
                    "{path} imports {spec} → {url}, not served"
                );
            }
        }
    }

    #[test]
    fn page_loads_the_entry_module() {
        assert!(PAGE.contains(r#"src="/static/js/main.js""#));
        assert_eq!(lookup("/static/nope.js"), None);
    }
}
