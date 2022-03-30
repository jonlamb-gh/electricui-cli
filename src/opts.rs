use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, StructOpt)]
#[structopt(about = "The ElectricUI CLI")]
#[structopt(name = "electricui")]
#[structopt(help_message = "Prints help information. Use --help for more details.")]
pub struct Opts {
    #[structopt(flatten)]
    pub subcommand: Subcommand,
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub enum Subcommand {
    /// TODO
    Check(DeviceOpts),
}

#[derive(Debug, Clone, PartialEq, StructOpt)]
pub struct DeviceOpts {
    /// Serial device baud rate
    #[structopt(short = "b", long, default_value = "115200")]
    pub baud_rate: u32,

    /// Serial device data bits
    #[structopt(long, default_value = "8")]
    pub data_bits: DataBits,

    /// Serial device flow control
    #[structopt(long, default_value = "none")]
    pub flow_control: FlowControl,

    /// Serial device parity checking mode.
    #[structopt(long, default_value = "none")]
    pub parity: Parity,

    /// Serial device stop bits
    #[structopt(long, default_value = "1")]
    pub stop_bits: StopBits,

    /// Serial device path
    #[structopt(name = "device")]
    pub device: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DataBits(pub tokio_serial::DataBits);

impl FromStr for DataBits {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(match s.trim().to_lowercase().as_str() {
            "5" | "five" => tokio_serial::DataBits::Five,
            "6" | "six" => tokio_serial::DataBits::Six,
            "7" | "seven" => tokio_serial::DataBits::Seven,
            "8" | "eight" => tokio_serial::DataBits::Eight,
            _ => return Err("Invalid data bits".to_string()),
        }))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FlowControl(pub tokio_serial::FlowControl);

impl FromStr for FlowControl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(match s.trim().to_lowercase().as_str() {
            "none" => tokio_serial::FlowControl::None,
            "software" | "sw" => tokio_serial::FlowControl::Software,
            "hardware" | "hw" => tokio_serial::FlowControl::Hardware,
            _ => return Err("Invalid flow control".to_string()),
        }))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Parity(pub tokio_serial::Parity);

impl FromStr for Parity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(match s.trim().to_lowercase().as_str() {
            "none" => tokio_serial::Parity::None,
            "odd" => tokio_serial::Parity::Odd,
            "even" => tokio_serial::Parity::Even,
            _ => return Err("Invalid parity".to_string()),
        }))
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StopBits(pub tokio_serial::StopBits);

impl FromStr for StopBits {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(match s.trim().to_lowercase().as_str() {
            "1" | "one" => tokio_serial::StopBits::One,
            "2" | "two" => tokio_serial::StopBits::Two,
            _ => return Err("Invalid stop bits".to_string()),
        }))
    }
}
