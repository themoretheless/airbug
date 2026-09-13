//! Trigger a panic that airbug-err should report (then re-raise).
use airbug_err::Options;

fn main() {
    let _guard = airbug_err::init(
        Options::new()
            .endpoint(
                std::env::var("AIRBUG_ERR_ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:8790/api/errors".into()),
            )
            .release(env!("CARGO_PKG_VERSION"))
            .environment("dev")
            .service("airbug-err-panic-example"),
    )
    .expect("init");

    panic!("airbug-err panic example");
}
