use anyhow::Result;
use clap::Parser;
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use linux_embedded_hal::I2cdev;
use ssd1306_i2c::{prelude::*, Builder};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(long, default_value = "127.0.0.1")]
    ip: String,

    #[arg(long, default_value = "/dev/i2c-0")]
    i2c_port: String,

    #[arg(long, default_value_t = 500)]
    refresh_rate: u64,
}

#[derive(Debug)]
struct ScreenInfo {
    status: String,
    ip: String,
    power: f64,
    hr: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let dev = I2cdev::new(args.i2c_port)?;

    let mut display: GraphicsMode<_> = Builder::new()
        .with_size(DisplaySize::Display128x64NoOffset)
        .with_i2c_addr(0x3c)
        .with_rotation(DisplayRotation::Rotate0)
        .connect_i2c(dev)
        .into();

    display.init().expect("Failed to initialize");

    display.clear();
    display.flush().expect("Couldn't flush display");

    let _text_style_small = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(BinaryColor::On)
        .build();

    let text_style_large = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        loop {
            let mut status = "Couldn't Get Status".to_string();
            let mut hr = 0.0;
            let mut power = 0.0;
            let mut ip = "N/A".to_string();
            tokio::time::sleep(Duration::from_millis(args.refresh_rate)).await;
            let Ok(client) = reqwest::Client::builder()
                .timeout(Duration::from_millis(500))
                .build()
            else {
                continue;
            };
            let Ok(summary) = client
                .get(format!("http://{}:4028/summary", args.ip))
                .send()
                .await
            else {
                continue;
            };

            let Ok(network) = client
                .get(format!("http://{}:4028/network", args.ip))
                .send()
                .await
            else {
                continue;
            };

            if let Ok(result) = summary.json::<serde_json::Value>().await {
                if let Some(status_api) = result.get("Status") {
                    if let Some(state_api) = status_api.get("Operating State") {
                        status = state_api.to_string();
                    }
                }
                if let Some(power_api) = result.get("Power Supply Stats") {
                    if let Some(ipower_api) = power_api.get("Input Power") {
                        power = ipower_api.as_f64().unwrap_or(0.0);
                    }
                }

                if let Some(session_api) = result.get("Session") {
                    if let Some(hr_api) = session_api.get("Average MHs") {
                        hr = hr_api.as_f64().unwrap_or(0.0);
                    }
                }
            }
            if let Ok(result) = network.json::<serde_json::Value>().await {
                if let Some(dhcp_api) = result.get("dhcp") {
                    if let Some(address_api) = dhcp_api.get("address") {
                        ip = address_api.to_string();
                    }
                }
            }

            let send_res = ScreenInfo {
                status,
                power,
                hr,
                ip,
            };

            if let Err(_) = tx.send(send_res).await {
                println!("receiver dropped");
                return;
            }
        }
    });

    while let Some(i) = rx.recv().await {
        display.clear();
        Text::with_baseline(
            &format!("IP: {}", i.ip),
            Point::zero(),
            text_style_large,
            Baseline::Top,
        )
        .draw(&mut display)?;

        Text::with_baseline(
            &format!("S: {}", i.status),
            Point::new(0, 20),
            text_style_large,
            Baseline::Top,
        )
        .draw(&mut display)?;

        Text::with_baseline(
            &format!("Power: {} W", i.power),
            Point::new(0, 30),
            text_style_large,
            Baseline::Top,
        )
        .draw(&mut display)?;

        Text::with_baseline(
            &format!("HR: {} TH/s", i.hr / 1000000.0),
            Point::new(0, 40),
            text_style_large,
            Baseline::Top,
        )
        .draw(&mut display)?;

        display.flush().expect("Couldn't flush display");
        //    println!("got = {:?}", i)
    }

    Ok(())
}
