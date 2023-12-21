mod api;
mod console;

use std::io::{self, BufWriter, Write};
use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::time::Duration;
use std::{env, thread};

use crate::api::ApiClient;

use anyhow::Context;

const USAGE: &str = "Usage: ro [OPTIONS]

Options:
  --scan        Scan and display all the available wifi
  --reboot      Reboot the router
  --reset       Reset the router
  --help        Print this help
";

fn main() -> anyhow::Result<ExitCode> {
    let mut api = ApiClient::new(Ipv4Addr::new(192, 168, 16, 1))
        .context("Failed to initilize the api client")?;
    api.login("admin", "admin")
        .context("Failed to logged into the router")?;

    let mut args = env::args();
    match args.nth(1).as_deref() {
        Some("--scan") => {
            show_wifi_list(&mut api)?;
            return Ok(ExitCode::SUCCESS);
        }
        Some("--reboot") => {
            api.reboot().context("Failed to reboot the router")?;
            return Ok(ExitCode::SUCCESS);
        }
        Some("--reset") => {
            api.reset().context("Failed to reset the router")?;
            return Ok(ExitCode::SUCCESS);
        }
        Some("--help") => {
            println!("{USAGE}");
            return Ok(ExitCode::SUCCESS);
        }
        Some(unknown) => {
            println!("error: Unknown option, '{unknown}'");
            println!("{USAGE}");
            return Ok(ExitCode::FAILURE);
        }
        None => (),
    }
    show_wifi_status(&mut api)?;
    Ok(ExitCode::SUCCESS)
}

fn show_wifi_list(api: &mut ApiClient) -> anyhow::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock());
    writeln!(stdout, "{:<30} {:<5}", "SSID", "SIGNAL")?;
    stdout.flush()?;

    loop {
        let list = api.scan_wifi().context("Failed to fetch wifi list")?;

        for wifi in &list {
            writeln!(stdout, "{:<30} {:<5}", wifi.ssid, wifi.signal)?;
        }
        stdout.flush()?;
        thread::sleep(Duration::from_secs(8));

        let num_lines = list.len();
        let mut num_cleared_lines = 0;

        // Clear all the printed list.
        while num_cleared_lines < num_lines {
            cursor_up!(stdout)?;
            clear_line!(stdout)?;
            num_cleared_lines += 1;
        }
    }
}

fn show_wifi_status(api: &mut ApiClient) -> anyhow::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock());

    let ssid = api.connected_ssid();
    let ssid = match ssid {
        Ok(Some(ref s)) => s,
        Ok(None) | Err(_) => "FAILED TO RETRIVE",
    };
    writeln!(stdout, "{:>8}: {ssid}", "SSID")?;
    writeln!(stdout, "{:>8}: 0", "Signal")?;

    loop {
        let router_info = api.router_info().context("Failed to fetch router info")?;
        let wifi_list = api.scan_wifi().context("Failed to fetch wifi list")?;

        if let Some(info) = wifi_list.iter().find(|wifi| wifi.ssid == ssid) {
            cursor_up!(stdout)?;
            clear_line!(stdout)?;
            writeln!(stdout, "{:>8}: {}", "Signal", info.signal)?;
        }
        clear_line!(stdout)?;
        write!(stdout, "{:>8}: {router_info}", "Speed")?;
        stdout.flush()?;
    }
}
