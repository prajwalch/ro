use std::net::Ipv4Addr;

use reqwest::blocking::{Client, Response};
use reqwest::Url;

const APP_USER_ARGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(serde::Deserialize)]
struct ConnectStatus {
    ssid: String,
}

struct Controller {
    url: Url,
    client: Client,
}

impl Controller {
    pub fn new(addr: Ipv4Addr) -> reqwest::Result<Controller> {
        let url = Url::parse(&format!("http://{addr}")).unwrap();
        let client = Client::builder().user_agent(APP_USER_ARGENT).build()?;

        Ok(Self { url, client })
    }

    pub fn login(&mut self, user: &str, pwd: &str) -> reqwest::Result<Response> {
        self.url.set_path("login/auth");
        self.client
            .post(self.url.as_str())
            .form(&[("user", user), ("pass", pwd)])
            .send()
    }

    pub fn connected_ssid(&mut self) -> reqwest::Result<Option<String>> {
        self.url.set_path("goform/get_connetsta_cfg");
        self.client
            .get(self.url.as_str())
            .send()?
            .json::<ConnectStatus>()
            .map(|a| (a.ssid != "NULL").then_some(a.ssid))
    }
}

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
}
