# err

Error / panic reporting for Rust apps in the airbug monorepo
(**Sentry ∪ Bugsnag ∪ Rollbar** capabilities that matter for Rust).

Package: `airbug-err` — client SDK. Hub stores grouped **issues** at
`POST /api/v1/errors` → `dash/hub/data/issues.sqlite`.

## Quick start

```bash
# terminal A
cargo run -p airbug-hub -- serve --root .

# terminal B
cargo test -p airbug-err
cargo run -p airbug-err --example capture
# optional: cargo run -p airbug-err --example panic
```

```rust
use airbug_err::{add_breadcrumb, capture_error, capture_message, configure_scope, Options, Severity};

let _guard = airbug_err::init(
    Options::new()
        .endpoint("http://127.0.0.1:8790/api/v1/errors")
        .release(env!("CARGO_PKG_VERSION"))
        .environment("dev")
        .service("checkout"),
)?;

configure_scope(|s| s.set_tag("tenant", "acme"));
add_breadcrumb("auth", "login ok", Severity::Info);
capture_message("something odd");
capture_error(&err);
```

Env override: `AIRBUG_ERR_ENDPOINT`. Hub webhook: `AIRBUG_ISSUES_WEBHOOK` or `--webhook`.

## Features

| Feature | Default | Role |
|---------|---------|------|
| `panic` | yes | Panic hook → event, then previous hook |
| `anyhow` | no | `capture_anyhow` |

## Coverage (Sentry / Bugsnag / Rollbar → Rust)

| Capability | Implemented |
|------------|-------------|
| `init` + guard | yes |
| Panic hook (forward previous) | yes (`panic`) |
| `capture_error` / `capture_message` | yes |
| Severity levels | yes (`fatal`/`error`/`warning`/`info`) |
| Breadcrumbs ring buffer | yes |
| Scope: tags, user, extra, fingerprint | yes |
| Release / environment / service | yes |
| Contexts (OS, hostname, rust) | yes |
| Backtrace frames | yes |
| Default + custom fingerprint | yes |
| `before_send` | yes |
| Sample rate | yes |
| `anyhow` helper | feature `anyhow` |
| Issues grouping + counts | hub |
| Issue states resolve / ignore / reopen | hub |
| New-issue webhook | hub |
| Event detail in UI | hub Issues panel |

## Skipped (and why)

These exist in Sentry / Bugsnag / Rollbar product surfaces but are **not** needed
for a Rust-first monorepo SDK:

1. **Source maps / JS minification** — no JS bundle; Rust symbols come from backtraces.
2. **Session Replay / screenshots** — browser/mobile UX; server Rust has no DOM session.
3. **Mobile crash reporters (ANR, NDK, PLCrashReporter)** — other platforms; Rust panic hook covers in-process unwind.
4. **CSP / browser handlers / Web Vitals** — web client only.
5. **Suspect commits / code owners / PR linking** — VCS SaaS product layer, not SDK core.
6. **Quota / billing / multi-tenant org UI** — cloud product; local hub + `sample_rate` is enough.
7. **Full Performance APM / transactions UI** — owned by `airbug-otel` + Jaeger.
8. **Release Health session protocol** — deferred; release is a tag on events for MVP.
9. **PII scrubbing rules engine / Relay** — start with `before_send`; full scrubbing later.
10. **Multi-language SDKs / JVM local-var agents** — Rust-only goal.
11. **Debug-image upload / Breakpad symbol server** — userspace backtrace is enough for MVP.
12. **Actix / Tower / Tracing deep integrations** — core API first; framework glue as follow-ups.
