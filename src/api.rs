use std::fmt;
use std::net::Ipv4Addr;

use crate::{bold, dim};

use reqwest::blocking::{Client, Response};
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;

type ApiResult<T> = reqwest::Result<T>;

pub struct ApiClient<'cred> {
    user: &'cred str,
    pwd: &'cred str,
    url: Url,
    client: Client,
}

impl<'cred> ApiClient<'cred> {
    pub fn new(addr: Ipv4Addr, user: &'cred str, pwd: &'cred str) -> ApiResult<ApiClient<'cred>> {
        let url = Url::parse(&format!("http://{addr}")).unwrap();
        let client = Client::builder().build()?;

        Ok(Self {
            user,
            pwd,
            url,
            client,
        })
    }

    pub fn login(&mut self) -> ApiResult<Response> {
        self.url.set_path("login/auth");
        self.client
            .post(self.url.as_str())
            .form(&[("user", self.user), ("pass", self.pwd)])
            .send()
    }

    pub fn router_info(&mut self) -> ApiResult<RouterInfo> {
        self.url.set_path("goform/get_router_info");
        let res = self.client.get(self.url.as_str()).send()?.json::<Value>()?;

        let upspeed = res["upspeed"]
            .as_str()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default();
        let downspeed = res["downspeed"]
            .as_str()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default();

        Ok(RouterInfo { upspeed, downspeed })
    }

    pub fn wlan_info(&mut self) -> ApiResult<WlanInfo> {
        self.url.set_path("/goform/get_wifi_info");
        self.client.get(self.url.as_str()).send()?.json()
    }

    pub fn connect(&mut self, ssid: &str, pwd: &str) -> ApiResult<Response> {
        loop {
            if let Some(wifi) = self.scan_wifi()?.iter().find(|wifi| wifi.ssid == ssid) {
                let (security, arithmetic) = wifi
                    .security
                    .split_once('/')
                    .unwrap_or((&wifi.security, "undefined"));
                let wlan = self.wlan_info()?;
                let data = &[
                    ("type", "seteasycfg"),
                    ("opermode", "repeater"),
                    ("channel", &wifi.channel),
                    ("bssid", &wifi.bssid),
                    ("ssid", &wifi.ssid),
                    ("security", security),
                    ("arithmetic", arithmetic),
                    ("key", pwd),
                    ("ext_ch", &wifi.ext_ch),
                    ("wlan2ssid", &wlan.wlanssid),
                    ("wlan2psw", wlan.wlanpsw.as_deref().unwrap_or("123456789")),
                    ("routepwd", self.pwd),
                ];
                self.url.set_path("/goform/set_EasyCfg");
                return self.client.post(self.url.as_str()).form(&data).send();
            };
        }
    }

    pub fn connected_ssid(&mut self) -> ApiResult<Option<String>> {
        self.url.set_path("goform/get_connetsta_cfg");
        let res = self.client.get(self.url.as_str()).send()?.json::<Value>()?;
        let ssid = res["ssid"].as_str();

        Ok(ssid.and_then(|s| (s != "NULL").then(|| s.to_owned())))
    }

    pub fn scan_wifi(&mut self) -> ApiResult<Vec<ScannedWifi>> {
        self.url.set_path("goform/get_RepeaterScan_cfg");
        self.client
            .get(self.url.as_str())
            .send()?
            .json::<Value>()
            .map(|mut res| serde_json::from_value(res["list"].take()).unwrap_or_default())
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
    pub upspeed: f64,
    pub downspeed: f64,
}

#[derive(Deserialize)]
pub struct WlanInfo {
    pub wlanssid: String,
    pub wlanpswmode: String,
    pub wlanpswencry: String,
    pub wlanpsw: Option<String>,
}

#[derive(Deserialize)]
pub struct ScannedWifi {
    pub channel: String,
    pub ssid: String,
    pub bssid: String,
    pub security: String,
    pub signal: String,
    pub ext_ch: String,
}

impl fmt::Display for RouterInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "↑ {} {} \t ↓ {} {}",
            bold!(self.upspeed),
            dim!(speed_unit(self.upspeed)),
            bold!(self.downspeed),
            dim!(speed_unit(self.downspeed)),
        )
    }
}

fn speed_unit(speed: f64) -> &'static str {
    if speed < 1024.0 {
        "B/s"
    } else if (speed / 1024.0).round() > 1024.0 {
        "MB/s"
    } else {
        "KB/s"
    }
}
