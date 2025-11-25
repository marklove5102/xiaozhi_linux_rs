use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    // 音频进程配置
    pub audio_local_port: u16,
    pub audio_remote_port: u16,
    pub audio_local_ip: &'static str,
    pub audio_remote_ip: &'static str,
    pub audio_buffer_size: usize,

    // GUI进程配置
    pub gui_local_port: u16,
    pub gui_remote_port: u16,
    pub gui_local_ip: &'static str,
    pub gui_remote_ip: &'static str,
    pub gui_buffer_size: usize,

    // IoT进程配置
    pub iot_local_port: u16,
    pub iot_remote_port: u16,
    pub iot_local_ip: &'static str,
    pub iot_remote_ip: &'static str,
    pub iot_buffer_size: usize,

    // 网络配置（静态部分）
    pub ws_url: &'static str,
    pub ota_url: &'static str,
    pub ws_token: &'static str,

    // 设备标识（动态部分，可在运行时修改）
    pub device_id: String,
    pub client_id: String,

    // Hello消息参数
    pub hello_format: &'static str,
    pub hello_sample_rate: u32,
    pub hello_channels: u8,
    pub hello_frame_duration: u32,
}

impl Config {
    /// 从编译时设置的环境变量创建配置
    /// 所有参数都在编译时从 config.toml 中读取
    pub fn new() -> Result<Self, &'static str> {
        Ok(Self {
            // 音频进程配置
            audio_local_port: env!("AUDIO_LOCAL_PORT").parse()
                .map_err(|_| "Failed to parse AUDIO_LOCAL_PORT")?,
            audio_remote_port: env!("AUDIO_REMOTE_PORT").parse()
                .map_err(|_| "Failed to parse AUDIO_REMOTE_PORT")?,
            audio_local_ip: env!("AUDIO_LOCAL_IP"),
            audio_remote_ip: env!("AUDIO_REMOTE_IP"),
            audio_buffer_size: env!("AUDIO_BUFFER_SIZE").parse()
                .map_err(|_| "Failed to parse AUDIO_BUFFER_SIZE")?,

            // GUI进程配置
            gui_local_port: env!("GUI_LOCAL_PORT").parse()
                .map_err(|_| "Failed to parse GUI_LOCAL_PORT")?,
            gui_remote_port: env!("GUI_REMOTE_PORT").parse()
                .map_err(|_| "Failed to parse GUI_REMOTE_PORT")?,
            gui_local_ip: env!("GUI_LOCAL_IP"),
            gui_remote_ip: env!("GUI_REMOTE_IP"),
            gui_buffer_size: env!("GUI_BUFFER_SIZE").parse()
                .map_err(|_| "Failed to parse GUI_BUFFER_SIZE")?,

            // IoT进程配置
            iot_local_port: env!("IOT_LOCAL_PORT").parse()
                .map_err(|_| "Failed to parse IOT_LOCAL_PORT")?,
            iot_remote_port: env!("IOT_REMOTE_PORT").parse()
                .map_err(|_| "Failed to parse IOT_REMOTE_PORT")?,
            iot_local_ip: env!("IOT_LOCAL_IP"),
            iot_remote_ip: env!("IOT_REMOTE_IP"),
            iot_buffer_size: env!("IOT_BUFFER_SIZE").parse()
                .map_err(|_| "Failed to parse IOT_BUFFER_SIZE")?,

            // 网络配置
            ws_url: env!("WS_URL"),
            ota_url: env!("OTA_URL"),
            ws_token: env!("WS_TOKEN"),

            // 设备标识初始化为config.toml中的值
            device_id: env!("DEVICE_ID").to_string(),
            client_id: env!("CLIENT_ID").to_string(),

            // Hello消息参数
            hello_format: env!("HELLO_FORMAT"),
            hello_sample_rate: env!("HELLO_SAMPLE_RATE").parse()
                .map_err(|_| "Failed to parse HELLO_SAMPLE_RATE")?,
            hello_channels: env!("HELLO_CHANNELS").parse()
                .map_err(|_| "Failed to parse HELLO_CHANNELS")?,
            hello_frame_duration: env!("HELLO_FRAME_DURATION").parse()
                .map_err(|_| "Failed to parse HELLO_FRAME_DURATION")?,
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new().expect("Failed to create default Config from build-time environment variables")
    }
}
