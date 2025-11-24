use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub audio_local_port: u16,
    pub audio_remote_port: u16,
    pub gui_local_port: u16,
    pub gui_remote_port: u16,
    pub ws_url: String,
    pub ws_token: String,
    pub device_id: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            audio_local_port: 5676,
            audio_remote_port: 5677,
            gui_local_port: 5678,
            gui_remote_port: 5679,
            // Default values, should be overridden by config file or env vars
            ws_url: "wss://api.xiaozhi.me/v1/ws".to_string(),
            ws_token: "test-token".to_string(),
            device_id: "unknown-device".to_string(),
        }
    }
}
