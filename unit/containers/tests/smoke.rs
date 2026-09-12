//! Integration smoke tests. Ignored unless Docker/Podman is available.
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
    let redis = RedisBuilder::new()
        .start()
        .expect("redis should start");
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
