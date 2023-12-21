mod api;
mod console;

use std::io::{self, BufWriter, Write};
use std::net::Ipv4Addr;
use std::process::ExitCode;
use std::time::Duration;
use std::{env, thread};

use crate::api::ApiClient;

const USAGE: &str = "Usage: ro [OPTIONS]

Options:
  --scan        Scan and display all the available wifi
  --reboot      Reboot the router
  --reset       Reset the router
  --help        Print this help
";

fn main() -> ExitCode {
    let mut api = match ApiClient::new(Ipv4Addr::new(192, 168, 16, 1)) {
        Ok(api) => api,
        Err(err) => {
            eprintln!("error: Failed to initilize controller, {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(e) = api.login("admin", "admin") {
        eprintln!("error: Failed to login, {e}");
        return ExitCode::FAILURE;
    };
    let mut args = env::args();

    match args.nth(1).as_deref() {
        Some("--scan") => {
            show_wifi_list(&mut api).unwrap();
            return ExitCode::SUCCESS;
        }
        Some("--reboot") => {
            api.reboot().unwrap();
            return ExitCode::SUCCESS;
        }
        Some("--reset") => {
            api.reset().unwrap();
            return ExitCode::SUCCESS;
        }
        Some("--help") => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Some(unknown) => {
            println!("error: Unknown option, '{unknown}'");
            println!("{USAGE}");
            return ExitCode::FAILURE;
        }
        None => (),
    }
    show_wifi_status(&mut api).unwrap();

    ExitCode::SUCCESS
}

fn show_wifi_list(api: &mut ApiClient) -> io::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock());

    writeln!(stdout, "{:<30} {:<5}", "SSID", "SIGNAL")?;
    stdout.flush()?;

    while let Ok(ref list) = api.scan_wifi() {
        for wifi in list {
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
    Ok(())
}

fn show_wifi_status(api: &mut ApiClient) -> io::Result<()> {
    let mut stdout = BufWriter::new(io::stdout().lock());

    let ssid = api.connected_ssid();
    let ssid = match ssid.as_ref() {
        Ok(Some(ref s)) => s,
        Ok(None) | Err(_) => "FAILED TO RETRIVE",
    };
    writeln!(stdout, "{:>8}: {ssid}", "SSID")?;
    writeln!(stdout, "{:>8}: 0", "Signal")?;

    while let (Ok(router_info), Ok(wifi_list)) = (api.router_info(), api.scan_wifi()) {
        if let Some(info) = wifi_list.iter().find(|w| w.ssid == ssid) {
            cursor_up!(stdout)?;
            clear_line!(stdout)?;
            writeln!(stdout, "{:>8}: {}", "Signal", info.signal)?;
        }
        clear_line!(stdout)?;
        write!(stdout, "{:>8}: {router_info}", "Speed")?;
        stdout.flush()?;
    }
    writeln!(stdout)?;
    writeln!(stdout, "error: API request failed, Closing app...")?;
    stdout.flush()?;

    Ok(())
}
