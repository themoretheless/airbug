//! Integration smoke tests. Skipped (soft) when Docker/Podman is unavailable or runs the
//! other container OS, so a host without Linux images is not reported as a broken module.
use airbug_containers::prelude::*;

fn docker_available() -> bool {
    airbug_containers::Runtime::detect().is_ok()
}

/// Start a container, or give up on the host rather than on the module.
///
/// A daemon running the other container OS answers `docker version` just fine and then cannot
/// pull a Linux image at all, which is an environment fact every runner keeps the right to
/// refuse. Any other failure still panics with the original message.
fn start_or_skip<T>(started: Result<T, ContainerError>, what: &str) -> Option<T> {
    match started {
        Ok(container) => Some(container),
        Err(err) if manifest_missing(&err) => {
            eprintln!("skip: {what} needs another container OS: {err}");
            None
        }
        Err(err) => panic!("{what} should start: {err}"),
    }
}

/// Whether the CLI said the image has no manifest for the daemon's container OS.
fn manifest_missing(err: &ContainerError) -> bool {
    let ContainerError::Command { message, .. } = err else {
        return false;
    };
    message.contains("no matching manifest")
}

#[test]
fn redis_module_starts_and_maps_port() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let Some(redis) = start_or_skip(RedisBuilder::new().start(), "redis") else {
        return;
    };
    let port = redis
        .container()
        .get_mapped_public_port(6379)
        .expect("mapped port");
    assert!(port > 0);
    let uri = redis.get_connection_string().expect("uri");
    assert!(uri.starts_with("redis://"));
}

#[test]
fn generic_builder_nginx_style_port() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let built = ContainerBuilder::new("nginx:1.27-alpine")
        .with_port_binding(80, true)
        .with_wait_strategy(Wait::tcp_port(80))
        .build()
        .expect("build");
    let Some(container) = start_or_skip(built.start(), "nginx") else {
        return;
    };
    let port = container.get_mapped_public_port(80).expect("port");
    assert!(port > 0);
}

#[test]
fn mysql_module_connection_string() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let Some(mysql) = start_or_skip(MySqlBuilder::new().start(), "mysql") else {
        return;
    };
    let uri = mysql.get_connection_string().expect("uri");
    assert!(uri.starts_with("mysql://"));
    assert!(mysql.container().get_mapped_public_port(3306).unwrap() > 0);
}

#[test]
fn mongodb_module_connection_string() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let Some(mongo) = start_or_skip(MongoDbBuilder::new().start(), "mongo") else {
        return;
    };
    let uri = mongo.get_connection_string().expect("uri");
    assert!(uri.starts_with("mongodb://"));
}

#[test]
fn rabbitmq_module_connection_string() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let Some(rabbit) = start_or_skip(RabbitMqBuilder::new().start(), "rabbit") else {
        return;
    };
    let uri = rabbit.get_connection_string().expect("uri");
    assert!(uri.starts_with("amqp://"));
    assert!(rabbit.get_management_port().unwrap() > 0);
}

#[test]
fn http_wait_polls_local_server() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        thread,
        time::Duration,
    };

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    let ready = Arc::new(AtomicBool::new(false));
    let flag = ready.clone();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let body = r#"{"status":"ok"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        flag.store(true, Ordering::SeqCst);
    });

    let url = format!("http://127.0.0.1:{port}/health");
    Wait::http(&url)
        .poll(Duration::from_secs(2))
        .expect("http wait");
    assert!(ready.load(Ordering::SeqCst));

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 1024];
        let _ = stream.read(&mut buf);
        let body = r#"{"ready":true,"status":"ok"}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
    });
    let url = format!("http://127.0.0.1:{port}/ready");
    Wait::http_json(&url, "status", "ok")
        .poll(Duration::from_secs(2))
        .expect("http_json wait");
}

#[test]
fn soft_skip_documented_when_no_docker() {
    // Mirrors README / CI policy: absence of docker must not hard-fail.
    if !docker_available() {
        eprintln!("skip: no docker/podman");
    }
}

#[test]
fn only_a_missing_manifest_earns_a_skip() {
    // The one message a wrong-container-OS host produces, from a real Windows-mode runner.
    let foreign_os = ContainerError::Command {
        program: "docker".into(),
        args: vec!["run".into(), "-d".into(), "redis:7.2.4".into()],
        message: "Unable to find image 'redis:7.2.4' locally\n7.2.4: Pulling from library/redis\n\
                  docker: no matching manifest for windows(10.0.26100)/amd64 in the manifest list entries"
            .into(),
    };
    assert!(manifest_missing(&foreign_os));
    // Any other runtime trouble stays a failure a host may not quietly skip past.
    let refused = ContainerError::Command {
        program: "docker".into(),
        args: vec!["run".into()],
        message: "docker: error during connect: the pipe is being closed".into(),
    };
    assert!(!manifest_missing(&refused));
    assert!(!manifest_missing(&ContainerError::Timeout("redis".into())));
    assert!(!manifest_missing(&ContainerError::RuntimeUnavailable(
        "gone".into()
    )));
}
