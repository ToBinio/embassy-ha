use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

use embassy_executor::{Executor, Spawner};
use embassy_sync::waitqueue::AtomicWaker;
use embassy_time::Timer;
use static_cell::StaticCell;

static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static RESOURCES: StaticCell<embassy_ha::DeviceResources> = StaticCell::new();

struct AsyncTcp {
    write_handle: JoinHandle<()>,
    write_buffer: Arc<Mutex<Vec<u8>>>,
    read_buffer: Arc<Mutex<Vec<u8>>>,
    waker: Arc<AtomicWaker>,
}

impl AsyncTcp {
    fn connect(addr: impl ToSocketAddrs) -> Self {
        let stream = TcpStream::connect(addr).expect("failed to connect to remote");
        let mut read_stream = stream.try_clone().unwrap();
        let mut write_stream = stream;

        let read_buffer: Arc<Mutex<Vec<u8>>> = Default::default();
        let write_buffer: Arc<Mutex<Vec<u8>>> = Default::default();

        let waker = Arc::new(AtomicWaker::new());

        let write_handle = std::thread::spawn({
            let write_buffer = write_buffer.clone();
            move || {
                loop {
                    let buffer = {
                        let mut buffer = write_buffer.lock().unwrap();
                        std::mem::take(&mut *buffer)
                    };
                    if !buffer.is_empty() {
                        println!("writing {} bytes", buffer.len());
                        write_stream.write_all(&buffer).unwrap();
                        write_stream.flush().unwrap();
                    } else {
                        std::thread::park();
                    }
                }
            }
        });

        std::thread::spawn({
            let read_buffer = read_buffer.clone();
            let waker = waker.clone();
            move || {
                let mut scratch = [0u8; 1024];
                loop {
                    let n = read_stream.read(&mut scratch).unwrap();
                    if n == 0 {
                        panic!("EOF");
                    }

                    {
                        let mut buffer = read_buffer.lock().unwrap();
                        buffer.extend_from_slice(&scratch[..n]);
                        waker.wake();
                    }
                }
            }
        });

        Self {
            write_handle,
            write_buffer,
            read_buffer,
            waker,
        }
    }
}

impl embedded_io_async::ErrorType for AsyncTcp {
    type Error = std::io::Error;
}

impl embedded_io_async::Write for AsyncTcp {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        {
            let mut buffer = self.write_buffer.lock().unwrap();
            buffer.extend_from_slice(buf);
        }
        self.write_handle.thread().unpark();
        Ok(buf.len())
    }
}

impl embedded_io_async::Read for AsyncTcp {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        struct WaitForWaker<'a>(&'a AtomicWaker, bool);

        impl<'a> Future for WaitForWaker<'a> {
            type Output = ();

            fn poll(
                mut self: std::pin::Pin<&mut Self>,
                cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Self::Output> {
                if self.1 {
                    std::task::Poll::Ready(())
                } else {
                    self.as_mut().1 = true;
                    self.0.register(cx.waker());
                    std::task::Poll::Pending
                }
            }
        }

        loop {
            {
                let mut buffer = self.read_buffer.lock().unwrap();
                if !buffer.is_empty() {
                    let copy_n = buf.len().min(buffer.len());
                    buf[..copy_n].copy_from_slice(&buffer[..copy_n]);
                    buffer.drain(..copy_n);
                    return Ok(copy_n);
                }
            }
            WaitForWaker(&self.waker, false).await
        }
    }
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let mut stream = AsyncTcp::connect("mqtt.d464.sh:1883");

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
        sensor.publish(42.0);
        Timer::after_secs(1).await;
    }
}

fn main() {
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.must_spawn(main_task(spawner));
    });
}
