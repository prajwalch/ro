mod console;
mod controller;

use std::io::{self, Write};
use std::net::Ipv4Addr;
use std::thread;
use std::time::Duration;

use crate::controller::Controller;

fn main() {
    let mut ctlr = match Controller::new(Ipv4Addr::new(192, 168, 16, 1)) {
        Ok(ctlr) => ctlr,
        Err(err) => {
            eprintln!("error: Failed to initilize controller, {err}");
            return;
        }
    };

    if let Err(e) = ctlr.login("admin", "admin") {
        eprintln!("error: Failed to login, {e}")
    };

    if let Ok(Some(ssid)) = ctlr.connected_ssid() {
        println!("Connected to: {ssid}");
    }

    let mut stdout = io::stdout().lock();

    while let Ok(router_info) = ctlr.router_info() {
        // Clear the line.
        write!(stdout, "\x1b[1K").unwrap();
        // Move cursor to start of a line.
        write!(stdout, "\r").unwrap();
        stdout.flush().unwrap();

        write!(stdout, "{router_info}").unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_secs(2));
    }
}
