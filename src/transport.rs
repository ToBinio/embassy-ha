pub trait Transport: embedded_io_async::Read + embedded_io_async::Write {}

impl<T> Transport for T where T: embedded_io_async::Read + embedded_io_async::Write {}
