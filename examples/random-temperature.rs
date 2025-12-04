mod common;

use common::AsyncTcp;
use embassy_executor::{Executor, Spawner};
use embassy_time::Timer;
use static_cell::StaticCell;

static RESOURCES: StaticCell<embassy_ha::DeviceResources> = StaticCell::new();

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let mut stream = AsyncTcp::connect(std::env!("MQTT_ADDRESS"));

    let mut device = embassy_ha::Device::new(
        RESOURCES.init(Default::default()),
        embassy_ha::DeviceConfig {
            device_id: "example-device-id",
            device_name: "Example Device Name",
            manufacturer: "Example Device Manufacturer",
            model: "Example Device Model",
        },
    );

    let temperature_sensor = device.create_temperature_sensor(
        "temperature-sensor-id",
        "Temperature Sensor Name",
        embassy_ha::TemperatureUnit::Celcius,
    );

    spawner.must_spawn(temperature(temperature_sensor));

    device.run(&mut stream).await;
}

#[embassy_executor::task]
async fn temperature(mut sensor: embassy_ha::TemperatureSensor<'static>) {
    loop {
        sensor.publish(rand::random_range(0.0..50.0));
        Timer::after_secs(1).await;
    }
}

example_main!();
