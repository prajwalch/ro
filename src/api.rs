use std::fmt;
use std::net::Ipv4Addr;

use crate::{bold, dim};

use reqwest::blocking::{Client, Response};
use reqwest::Url;
use serde::Deserialize;

type ApiResult<T> = reqwest::Result<T>;

pub struct ApiClient {
    url: Url,
    client: Client,
}

impl ApiClient {
    pub fn new(addr: Ipv4Addr) -> ApiResult<ApiClient> {
        let url = Url::parse(&format!("http://{addr}")).unwrap();
        let client = Client::builder().build()?;

        Ok(Self { url, client })
    }

    pub fn login(&mut self, user: &str, pwd: &str) -> ApiResult<Response> {
        self.url.set_path("login/auth");
        self.client
            .post(self.url.as_str())
            .form(&[("user", user), ("pass", pwd)])
            .send()
    }

    pub fn router_info(&mut self) -> ApiResult<RouterInfo> {
        self.url.set_path("goform/get_router_info");
        self.client.get(self.url.as_str()).send()?.json()
    }

    pub fn connected_ssid(&mut self) -> ApiResult<Option<String>> {
        // This structure is used only to get value of a `ssid` field from the
        // json which we will get as a response.
        #[derive(Deserialize)]
        struct ConnectStatus {
            ssid: String,
        }

        self.url.set_path("goform/get_connetsta_cfg");
        self.client
            .get(self.url.as_str())
            .send()?
            .json::<ConnectStatus>()
            .map(|a| (a.ssid != "NULL").then_some(a.ssid))
    }

    pub fn scan_wifi(&mut self) -> ApiResult<Vec<ScannedWifi>> {
        // This structure is used only to get value of a `list` field from the
        // json which we will get as a response.
        #[derive(Deserialize)]
        struct RepeaterScanResult {
            list: Vec<ScannedWifi>,
        }

        self.url.set_path("goform/get_RepeaterScan_cfg");
        self.client
            .get(self.url.as_str())
            .send()?
            .json::<RepeaterScanResult>()
            .map(|info| info.list)
    }

    pub fn reboot(&mut self) -> ApiResult<Response> {
        self.url.set_path("goform/set_reboot");
        self.client
            .post(self.url.as_str())
            .form(&[("mode", "reboot")])
            .send()
    }

    pub fn reset(&mut self) -> ApiResult<Response> {
        self.url.set_path("goform/set_restore");
        self.client
            .post(self.url.as_str())
            .form(&[("type", "restore")])
            .send()
    }
}

#[derive(Deserialize)]
pub struct RouterInfo {
    pub upspeed: String,
    pub downspeed: String,
}

#[derive(Deserialize)]
pub struct ScannedWifi {
    pub channel: String,
    pub ssid: String,
    pub signal: String,
}

impl fmt::Display for RouterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "↑ {} {} \t ↓ {} {}",
            bold!(self.upspeed),
            dim!(speed_unit(&self.upspeed)),
            bold!(self.downspeed),
            dim!(speed_unit(&self.downspeed)),
        )
    }
}

fn speed_unit(speed: &str) -> &'static str {
    let speed = speed.parse::<f64>().unwrap_or_default();

    if speed < 1024.0 {
        "B/s"
    } else if (speed / 1024.0).round() > 1024.0 {
        "MB/s"
    } else {
        "KB/s"
    }
}
