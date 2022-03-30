use crate::opts::DeviceOpts;
use tokio_serial::{ClearBuffer, Error, SerialPort, SerialPortBuilderExt, SerialStream};
use tracing::info;

pub fn new(opts: &DeviceOpts) -> Result<SerialStream, Error> {
    info!(
        "Opening '{}', baud_rate={}, data_bits={:?}, parity={:?}, stop_bits={:?}",
        opts.device, opts.baud_rate, opts.data_bits.0, opts.parity.0, opts.stop_bits.0
    );

    let mut port = tokio_serial::new(&opts.device, opts.baud_rate)
        .data_bits(opts.data_bits.0)
        .flow_control(opts.flow_control.0)
        .parity(opts.parity.0)
        .stop_bits(opts.stop_bits.0)
        .open_native_async()?;
    port.clear(ClearBuffer::All)?;

    #[cfg(unix)]
    port.set_exclusive(false)?;

    Ok(port)
}
