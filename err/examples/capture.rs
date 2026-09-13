//! Capture a message and an error without panicking.
use airbug_err::{
    Options, Severity, add_breadcrumb, capture_error, capture_message, configure_scope,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _guard = airbug_err::init(
        Options::new()
            .endpoint(
                std::env::var("AIRBUG_ERR_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:8790/api/v1/errors".into()),
            )
            .release(env!("CARGO_PKG_VERSION"))
            .environment("dev")
            .service("airbug-err-example"),
    )?;

    configure_scope(|s| {
        s.set_tag("example", "capture");
        s.set_user(airbug_err::User {
            id: Some("demo".into()),
            email: None,
            username: Some("airbug".into()),
        });
    });

    add_breadcrumb("example", "about to capture", Severity::Info);
    let _ = capture_message_with_level_helper();
    let err = std::io::Error::other("demo failure");
    let _ = capture_error(&err);
    println!("events sent (if hub is listening on /api/v1/errors)");
    Ok(())
}

fn capture_message_with_level_helper() -> Option<String> {
    capture_message("hello from airbug-err capture example")
}
