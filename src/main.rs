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
  --connect <SSID> <PWD> Connect to the given ssid and display status
  --scan                 Scan and display all the available wifi
  --reboot               Reboot the router
  --reset                Reset the router
  --help                 Print this help
";

fn main() -> anyhow::Result<ExitCode> {
    let mut api = ApiClient::new(Ipv4Addr::new(192, 168, 16, 1), "admin", "admin")
        .context("Failed to initilize the api client")?;
    api.login().context("Failed to logged into the router")?;

    let mut args = env::args();
    match args.nth(1).as_deref() {
        Some("--connect") => {
            if let (Some(ssid), Some(pwd)) = (args.next(), args.next()) {
                connect_wifi(&mut api, &ssid, &pwd)?;
            } else {
                eprintln!("error: Incomplete credentials\n{USAGE}");
                return Ok(ExitCode::FAILURE);
            }
        }
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

fn connect_wifi(api: &mut ApiClient, ssid: &str, pwd: &str) -> anyhow::Result<()> {
    println!("Hunting and connecting'{ssid}', be patience ;)");
    api.connect(ssid, pwd).context("Failed to connect ssid")?;

    println!("Done, now waiting 60s for router to be reboot XD");
    thread::sleep(Duration::from_secs(60));

    // Relogin into the router.
    api.login().context("Failed to logged into the router")?;
    Ok(())
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

    let ssid = api.connected_ssid().context("Failed to fetch ssid")?;
    let ssid = ssid.as_deref().unwrap_or("UNKNOWN");
    writeln!(stdout, "{:>8}: {ssid}", "SSID",)?;
    writeln!(stdout, "{:>8}: 0", "Signal")?;

    // Always append the newline at the bottom so that if API functions fail
    // the error message can appear from the newline; otherwise move the cursor
    // up by checking this variable after the call to API functions succeed.
    let mut is_cursor_at_bottom = false;

    loop {
        let router_info = api.router_info().context("Failed to fetch router info")?;
        let wifi_list = api.scan_wifi().context("Failed to fetch wifi list")?;

        if is_cursor_at_bottom {
            cursor_up!(stdout)?;
        }
        if let Some(info) = wifi_list.iter().find(|wifi| wifi.ssid == ssid) {
            // From the speed printed line, move to the previous line.
            cursor_up!(stdout)?;
            // Clear the old printed data.
            clear_line!(stdout)?;
            // And reprint the updated one.
            writeln!(stdout, "{:>8}: {}", "Signal", info.signal)?;
        }
        // We are already at the right line, clear the old printed data.
        clear_line!(stdout)?;
        // And reprint the updated one.
        writeln!(stdout, "{:>8}: {router_info}", "Speed")?;
        stdout.flush()?;
        // Final `writeln` moves the cursor at the newline.
        is_cursor_at_bottom = true;
    }
}
