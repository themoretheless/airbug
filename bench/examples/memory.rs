use airbug_bench::memory::{self, Allocator};
#[global_allocator]
static ALLOC: Allocator<std::alloc::System> = Allocator(std::alloc::System);
#[inline(never)]
fn temporary() {
    let data = vec![42u8; 4096];
    std::hint::black_box(&data);
}
#[inline(never)]
fn retained() -> Vec<u8> {
    vec![7u8; 16384]
}
fn main() -> airbug_bench::Result<()> {
    let count = if std::env::args().any(|a| a == "--overflow") {
        5000
    } else {
        20
    };
    let data = memory::profile(|| {
        for _ in 0..count {
            temporary();
        }
        retained()
    })?;
    std::hint::black_box(data);
    Ok(())
}
