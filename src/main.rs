// sense.rs: Monitoring environment with LPS25H and HTS221 via I2C
// Requires: i2cdev, clap, chrono crates

use chrono::Utc;
use clap::Parser;
use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};
use std::fs::OpenOptions;
use std::io::{Write, stdout};
use std::thread::sleep;
use std::time::Duration;

#[derive(Parser)]
#[command(about = "write sensor value to file")]
struct Args {
    /// initialize sensors. Data are discarded.
    #[arg(long)]
    init: bool,
    /// I2C bus device (e.g. /dev/i2c-1)
    #[arg(long, default_value = "/dev/i2c-1")]
    i2c_bus: String,
    /// LPS25H I2C address (default 0x5c)
    #[arg(long, default_value_t = 0x5c)]
    lps25h_addr: u16,
    /// HTS221 I2C address (default 0x5f)
    #[arg(long, default_value_t = 0x5f)]
    hts221_addr: u16,
    /// Output file (default: stdout)
    #[arg(long)]
    output: Option<String>,
}

fn read_lps25h(dev: &mut LinuxI2CDevice) -> Result<(i32, i32), LinuxI2CError> {
    let mut data = [0u8; 5];
    // Power on.
    dev.smbus_write_byte_data(0x20, 0x80)?;
    sleep(Duration::from_millis(50));

    // Read raw data
    dev.write(&[0x28 | 0x80])?;
    dev.read(&mut data[..5])?;
    let press_raw = ((data[2] as u32) << 16 | (data[1] as u32) << 8 | (data[0] as u32)) as i32;
    let temp_raw = (((data[4] as u16) << 8) | (data[3] as u16)) as i16;

    let pressure = press_raw / 4096; // hPa
    let temperature = 425 + temp_raw as i32 / 48; // 0.1 C

    Ok((pressure, temperature))
}

fn read_hts221(dev: &mut LinuxI2CDevice) -> Result<(i32, i32), LinuxI2CError> {
    let mut calib = [0u8; 16];
    // Power on.
    dev.smbus_write_byte_data(0x20, 0x80)?;
    sleep(Duration::from_millis(50));

    // Read calibration data
    dev.write(&[0x30 | 0x80])?;
    dev.read(&mut calib)?;

    let t0_deg_c_x8 = (calib[2] as u16) | (((calib[5] & 0x03) as u16) << 8);
    let t1_deg_c_x8 = (calib[3] as u16) | (((calib[5] & 0x0C) as u16) << 6);
    let t0_deg_c = t0_deg_c_x8 / 8;
    let t1_deg_c = t1_deg_c_x8 / 8;
    let t0_out = (calib[12] as u16 | ((calib[13] as u16) << 8)) as i16;
    let t1_out = (calib[14] as u16 | ((calib[15] as u16) << 8)) as i16;

    let h0_rh_x2 = calib[0];
    let h1_rh_x2 = calib[1];
    let h0_t0_out = (calib[6] as u16 | ((calib[7] as u16) << 8)) as i16;
    let h1_t0_out = (calib[10] as u16 | ((calib[11] as u16) << 8)) as i16;
    let h0_rh = h0_rh_x2 / 2;
    let h1_rh = h1_rh_x2 / 2;

    // Read raw data
    let mut data = [0u8; 4];
    dev.write(&[0x28 | 0x80])?;
    dev.read(&mut data)?;
    let t_out = ((data[3] as u16) << 8 | data[2] as u16) as i16;
    let h_out = ((data[1] as u16) << 8 | data[0] as u16) as i16;

    let temp = if t1_out != t0_out {
        let tmp32 = (t_out - t0_out) as i32 * ((t1_deg_c - t0_deg_c) as i32 * 10);
        tmp32 / ((t1_out - t0_out) as i32) + (t0_deg_c as i32 * 10)
    } else {
        t0_deg_c as i32 * 10
    }; // 0.1 C

    let tmp = (h_out - h0_t0_out) as i32 * (h1_rh - h0_rh) as i32;
    let mut hum = if h1_t0_out != h0_t0_out {
        tmp / ((h1_t0_out - h0_t0_out) as i32) + h0_rh as i32
    } else {
        h0_rh as i32
    }; // 0.1%
    hum = (hum * 10).clamp(0, 1000);
    Ok((hum, temp))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut lps25h = LinuxI2CDevice::new(&args.i2c_bus, args.lps25h_addr)?;
    let mut hts221 = LinuxI2CDevice::new(&args.i2c_bus, args.hts221_addr)?;

    let (pressure, temp_press) = read_lps25h(&mut lps25h)?;
    let (humidity, temp_hum) = read_hts221(&mut hts221)?;
    let timestamp = Utc::now().timestamp();

    if !args.init {
        let output_line = format!(
            "{}\t{:.2}\t{:.2}\t{:.2}\t{:.2}",
            timestamp, pressure, temp_press, humidity, temp_hum
        );
        match &args.output {
            Some(filename) => {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filename)?;
                writeln!(file, "{}", output_line)?;
            }
            None => {
                let mut out = stdout();
                writeln!(out, "{}", output_line)?;
            }
        }
    }
    Ok(())
}
