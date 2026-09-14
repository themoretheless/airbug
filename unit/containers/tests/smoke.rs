//! Integration smoke tests. Skipped (soft) when Docker/Podman is unavailable.
use airbug_containers::prelude::*;

fn docker_available() -> bool {
    airbug_containers::Runtime::detect().is_ok()
}

#[test]
fn redis_module_starts_and_maps_port() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let redis = RedisBuilder::new().start().expect("redis should start");
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
    let container = ContainerBuilder::new("nginx:1.27-alpine")
        .with_port_binding(80, true)
        .with_wait_strategy(Wait::tcp_port(80))
        .build()
        .expect("build")
        .start()
        .expect("start nginx");
    let port = container.get_mapped_public_port(80).expect("port");
    assert!(port > 0);
}

#[test]
fn mysql_module_connection_string() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let mysql = MySqlBuilder::new().start().expect("mysql");
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
    let mongo = MongoDbBuilder::new().start().expect("mongo");
    let uri = mongo.get_connection_string().expect("uri");
    assert!(uri.starts_with("mongodb://"));
}

#[test]
fn rabbitmq_module_connection_string() {
    if !docker_available() {
        eprintln!("skip: no docker/podman");
        return;
    }
    let rabbit = RabbitMqBuilder::new().start().expect("rabbit");
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
