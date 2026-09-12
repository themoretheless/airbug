#![cfg(feature = "macros")]
//! Exercise diagnostics using an actual consumer crate, including a renamed dependency.
use std::{fs, process::Command};
struct Cleanup(std::path::PathBuf);
impl Drop for Cleanup {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn consumer_macros_and_compile_fail_diagnostics() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("airbug-ui-{}-{stamp}", std::process::id()));
    fs::create_dir(&directory).unwrap();
    let _cleanup = Cleanup(directory.clone());
    fs::create_dir(directory.join("src")).unwrap();
    fs::write(directory.join("Cargo.toml"), format!(
        "[package]\nname = \"airbug-ui\"\nversion = \"0.0.0\"\nedition = \"2024\"\n[workspace]\n[dependencies]\nhelpers = {{ package = \"airbug\", path = {:?}, features = [\"macros\"] }}\n",
        env!("CARGO_MANIFEST_DIR")
    )).unwrap();
    let cases = [
        (
            "#[derive(helpers::Generate)] enum Invalid { A }",
            Some("Generate supports structs"),
        ),
        (
            "#[derive(helpers::Generate)] struct Invalid<'a> { value: &'a str }",
            Some("lifetime parameters are not supported"),
        ),
        (
            "#[derive(helpers::Generate)] struct Invalid { #[fixture(default, with = value)] value: u8 }",
            Some("exactly one fixture strategy"),
        ),
        (
            "#[helpers::mock] trait Invalid { type Value; }",
            Some("associated types and constants"),
        ),
        (
            "#[helpers::mock] trait Invalid { fn get(&self) -> &str; }",
            Some("owned returns"),
        ),
        (
            "#[helpers::mock] trait Invalid { fn set(&self, value: &mut u8); }",
            Some("mutable argument references"),
        ),
        (
            "#[helpers::mock] trait Invalid { fn get<T>(&self, value: T); }",
            Some("without generics"),
        ),
        (
            "#[helpers::cases(one(1), one(2))] fn sample(v: u8) {}",
            Some("duplicate case name"),
        ),
        (
            "#[helpers::cases(one(1, 2))] fn sample(v: u8) {}",
            Some("argument count"),
        ),
        (
            "#[helpers::cases(one(1))] async fn sample(v: u8) {}",
            Some("synchronous safe function"),
        ),
        (
            r#"
            #[derive(helpers::Generate)] pub struct Data { pub value: u64 }
            #[helpers::mock] pub trait Store { fn get(&self, name: &str) -> u64; }
            pub fn smoke() {
                let mut ctx = helpers::FixtureContext::new();
                assert_eq!(ctx.builder::<Data>().with_value(7).build().value, 7);
                let mock = MockStore::default();
                mock.get.expect("all", |_| true).returns(1);
                assert_eq!(mock.get("name"), 1);
            }
        "#,
            None,
        ),
    ];
    for (source, expected) in cases {
        fs::write(directory.join("src/lib.rs"), source).unwrap();
        let output = Command::new(env!("CARGO"))
            .args(["check", "--offline", "--quiet", "--tests"])
            .current_dir(&directory)
            .env("CARGO_TARGET_DIR", directory.join("target"))
            .output()
            .expect("run consumer cargo check");
        let stderr = String::from_utf8_lossy(&output.stderr);
        match expected {
            Some(message) => {
                assert!(
                    !output.status.success(),
                    "expected compile failure for {source}"
                );
                assert!(
                    stderr.contains(message),
                    "missing diagnostic {message:?}: {stderr}"
                );
            }
            None => assert!(output.status.success(), "renamed consumer failed: {stderr}"),
        }
    }
}
