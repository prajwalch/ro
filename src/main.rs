mod api;
mod console;

use std::io::{self, Write};
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
            let wifi_list = api.scan_wifi().unwrap();

            for wifi in wifi_list {
                println!("{} \t {} \t {}", wifi.channel, wifi.ssid, wifi.signal);
            }
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

    if let Ok(Some(ssid)) = api.connected_ssid() {
        println!("Connected to: {ssid}");
    }
    let mut stdout = io::stdout().lock();

    while let Ok(router_info) = api.router_info() {
        // Clear the line.
        write!(stdout, "\x1b[1K").unwrap();
        // Move cursor to start of a line.
        write!(stdout, "\r").unwrap();
        stdout.flush().unwrap();

        write!(stdout, "{router_info}").unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_secs(2));
    }

    ExitCode::SUCCESS
}
