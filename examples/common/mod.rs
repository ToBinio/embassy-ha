mod std_async_tcp;
use embassy_executor::Executor;
use static_cell::StaticCell;
pub use std_async_tcp::AsyncTcp;

pub static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[macro_export]
macro_rules! example_main {
    () => {
        fn main() {
            let executor = common::EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                spawner.must_spawn(main_task(spawner));
            });
        }
    };
}
