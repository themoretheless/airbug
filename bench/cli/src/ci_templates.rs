//! Multi-CI & edge-runtime pipeline template generator (`#302–315`).
use airbug_bench::{Result, error};

pub const PROVIDERS: &[&str] = &[
    "github",
    "gitlab",
    "jenkins",
    "circleci",
    "travis",
    "appveyor",
    "azure",
    "bitbucket",
    "buildkite",
    "drone",
    "cloudflare-workers",
    "fastly-compute",
    "aws-lambda",
    "vercel-edge",
];

pub fn render(provider: &str) -> Result<&'static str> {
    match provider.trim().to_ascii_lowercase().as_str() {
        "github" | "github-actions" => Ok(include_str!("ci-template.yml")),
        "gitlab" | "gitlab-ci" => Ok(r#"stages:
  - bench

airbug_bench:
  stage: bench
  image: rust:1.96
  when: manual
  script:
    - cargo build --locked --offline --benches
    - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
  artifacts:
    paths:
      - bench-out/
"#),
        "jenkins" => Ok(r#"pipeline {
  agent any
  stages {
    stage('Benchmark') {
      steps {
        sh 'cargo build --locked --offline --benches'
        sh 'cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out'
      }
    }
  }
}
"#),
        "circleci" => Ok(r#"version: 2.1
jobs:
  bench:
    docker:
      - image: cimg/rust:1.96
    steps:
      - checkout
      - run: cargo build --locked --offline --benches
      - run: cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "travis" => Ok(r#"language: rust
rust: 1.96.0
script:
  - cargo build --locked --offline --benches
  - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "appveyor" => Ok(r#"image: Visual Studio 2022
build_script:
  - cargo build --locked --offline --benches
test_script:
  - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "azure" | "azure-pipelines" => Ok(r#"trigger: none
pool:
  vmImage: ubuntu-latest
steps:
  - script: |
      cargo build --locked --offline --benches
      cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
    displayName: 'Run airbug-bench suite'
"#),
        "bitbucket" => Ok(r#"image: rust:1.96
pipelines:
  custom:
    bench:
      - step:
          name: Run airbug-bench
          script:
            - cargo build --locked --offline --benches
            - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "buildkite" => Ok(r#"steps:
  - label: ":stopwatch: airbug-bench"
    command:
      - cargo build --locked --offline --benches
      - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "drone" => Ok(r#"kind: pipeline
type: docker
name: airbug-bench
steps:
  - name: bench
    image: rust:1.96
    commands:
      - cargo build --locked --offline --benches
      - cargo run --locked --offline -p cargo-airbug-bench -- bench --output bench-out
"#),
        "cloudflare-workers" => Ok(r#"name = "airbug-bench-worker"
main = "build/worker/shim.mjs"
compatibility_date = "2026-01-01"
[build]
command = "cargo build --locked --target wasm32-unknown-unknown --release"
"#),
        "fastly-compute" => Ok(r#"manifest_version = 3
name = "airbug-bench-compute"
language = "rust"
[scripts]
build = "cargo build --locked --target wasm32-wasip1 --release"
"#),
        "aws-lambda" => Ok(r#"AWSTemplateFormatVersion: '2010-09-09'
Description: airbug-bench Lambda custom runtime smoke template
Resources:
  BenchFn:
    Type: AWS::Lambda::Function
    Properties:
      Runtime: provided.al2023
      Handler: bootstrap
      MemorySize: 1769
"#),
        "vercel-edge" => Ok(r#"{
  "$schema": "https://openapi.vercel.sh/vercel.json",
  "functions": {
    "api/bench.rs": { "runtime": "vercel-rust@4.0.9" }
  }
}
"#),
        other => Err(error(format!(
            "unknown CI provider '{other}'; supported: {}",
            PROVIDERS.join(", ")
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_providers_render_nonempty_templates() {
        for provider in PROVIDERS {
            let tpl = render(provider).unwrap();
            assert!(!tpl.trim().is_empty(), "{provider}");
        }
        assert!(render("unknown-ci").is_err());
    }
}
