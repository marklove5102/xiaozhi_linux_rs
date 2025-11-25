mod audio_bridge;
mod config;
mod gui_bridge;
mod net_link;
mod state_machine;
mod activation;

use audio_bridge::{AudioBridge, AudioEvent};
use config::Config;
use gui_bridge::{GuiBridge, GuiEvent};
use net_link::{NetCommand, NetEvent, NetLink};
use state_machine::SystemState;
use std::sync::Arc;
use tokio::sync::mpsc;
use serde::Deserialize;
use mac_address::get_mac_address;
use uuid::Uuid;


// 服务器消息结构体
#[derive(Deserialize)]
struct ServerMessage {
    #[serde(rename = "type")]
    msg_type: String,
    command: Option<String>, // 用于IOT类型
    text: Option<String>, // 用于TTS文本
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    env_logger::init();

    // 加载配置
    let mut config = Config::new().unwrap_or_default();

    // 设备id和客户端id的处理
    if config.device_id == "unknown-device" {
        config.device_id = match get_mac_address() {
            Ok(Some(mac)) => mac.to_string().to_lowercase(),
            _ => Uuid::new_v4().to_string(),
        };
    }
    
    // 设备端UUID，先从本地文件读取以保持重启间身份一致，如果不存在则生成新的并保存
    let uuid_file_path = "xiaozhi_uuid.txt";
    if config.client_id == "unknown-client" {
        if let Ok(content) = std::fs::read_to_string(uuid_file_path) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                config.client_id = trimmed.to_string();
                println!("Loaded Client ID from file: {}", config.client_id);
            }
        }
    }

    // 生成新的UUID并保存
    if config.client_id == "unknown-client" {
        config.client_id = Uuid::new_v4().to_string();
        println!("Generated new Client ID: {}", config.client_id);
        // Save to file
        if let Err(e) = std::fs::write(uuid_file_path, &config.client_id) {
            eprintln!("Failed to save Client ID to file: {}", e);
        } else {
            println!("Saved Client ID to {}", uuid_file_path);
        }
    }

    // 创建通道，用于组件间通信
    // 事件通道
    let (tx_net_event, mut rx_net_event) = mpsc::channel::<NetEvent>(100);

    // 命令通道
    let (tx_net_cmd, rx_net_cmd) = mpsc::channel::<NetCommand>(100);

    // 音频通道
    let (tx_audio_event, mut rx_audio_event) = mpsc::channel::<AudioEvent>(100);

    // GUI通道
    let (tx_gui_event, mut rx_gui_event) = mpsc::channel::<GuiEvent>(100);

    // 启动GUI桥，与GUI进程通信，优先启动，用于播报激活状态或者激活码
    let gui_bridge = Arc::new(GuiBridge::new(&config, tx_gui_event).await?);
    // clone一份，用于异步任务，还要用原始的gui_bridge在主循环中发送消息
    let gui_bridge_clone = gui_bridge.clone();
    tokio::spawn(async move {
        if let Err(e) = gui_bridge_clone.run().await {
            eprintln!("GuiBridge error: {}", e);
        }
    });

    // 在启动 NetLink 前检查激活
    loop {
        match activation::check_device_activation(&config).await {
            activation::ActivationResult::Activated => {
                println!("Device is activated. Starting WebSocket...");
                let _ = gui_bridge.send_message(r#"{"type":"toast", "text":"设备已激活"}"#).await;
                break; // 跳出循环，继续下面的 NetLink 启动
            }
            activation::ActivationResult::NeedActivation(code) => {
                println!("Device NOT activated. Code: {}", code);
                
                // GUI 显示验证码
                let gui_msg = format!(r#"{{"type":"activation", "code":"{}"}}"#, code);
                let _ = gui_bridge.send_message(&gui_msg).await; // 通知 GUI 显示激活码
                
                // TTS 播报 
                // 简单做法：假设 sound_app 能播报数字
                // audio_bridge.speak_text(format!("请在手机输入验证码 {}", code)).await;
                
                // 等待几秒再轮询
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
            activation::ActivationResult::Error(e) => {
                eprintln!("Activation check error: {}. Retrying in 5s...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }

    // 启动网络链接，与小智服务器通信
    let net_link = NetLink::new(config.clone(), tx_net_event, rx_net_cmd);
    tokio::spawn(async move {
        net_link.run().await;
    });

    // 启动音频桥，与音频进程通信
    let audio_bridge = Arc::new(AudioBridge::new(&config, tx_audio_event).await?);
    let audio_bridge_clone = audio_bridge.clone();
    tokio::spawn(async move {
        if let Err(e) = audio_bridge_clone.run().await {
            eprintln!("AudioBridge error: {}", e);
        }
    });

    // 主事件循环，处理各组件事件
    // 监听来自NetLink、AudioBridge和GuiBridge的事件，并进行相应处理
    let mut current_state = SystemState::Idle;
    println!("Xiaozhi Core Started. State: {:?}", current_state);

    loop {
        tokio::select! {

            // 监听与服务器的网络事件
            Some(event) = rx_net_event.recv() => {
                match event {

                    // 如果接收到服务器的文本消息，就转发给GUI
                    NetEvent::Text(text) => {
                        println!("Received Text from Server: {}", text);
                        
                        // Try to parse as JSON to handle specific message types
                        if let Ok(msg) = serde_json::from_str::<ServerMessage>(&text) {
                            match msg.msg_type.as_str() {
                                "iot" => {
                                    if let Some(cmd) = msg.command {
                                        println!("Received IoT Command: {}", cmd);
                                        // 如果是IOT命令，在这里实现对应的操作，暂时留空
                                    }
                                }
                                "tts" => {
                                    // 转发给GUI显示TTS文本
                                    if let Some(t) = msg.text {
                                        println!("TTS: {}", t);
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let Err(e) = gui_bridge.send_message(&text).await {
                            eprintln!("Failed to send to GUI: {}", e);
                        }
                    }

                    // 如果接收到服务器的二进制音频数据，就转发给音频桥播放
                    NetEvent::Binary(data) => {
                        // println!("Received Audio from Server: {} bytes", data.len());
                        if current_state != SystemState::Speaking {
                            current_state = SystemState::Speaking;
                            // Notify GUI: kDeviceStateSpeaking = 6
                            let _ = gui_bridge.send_message(r#"{"state": 6}"#).await;
                        }
                        // Forward to Audio
                        if let Err(e) = audio_bridge.send_audio(&data).await {
                            eprintln!("Failed to send to Audio: {}", e);
                        }
                    }

                    // 连接状态变化
                    NetEvent::Connected => {
                        println!("WebSocket Connected");
                        // Notify GUI: kDeviceStateIdle = 3
                        let _ = gui_bridge.send_message(r#"{"state": 3}"#).await;
                    }
                    NetEvent::Disconnected => {
                        println!("WebSocket Disconnected");
                        current_state = SystemState::NetworkError;
                        // Notify GUI: kDeviceStateConnecting = 4 (or Error = 9)
                        let _ = gui_bridge.send_message(r#"{"state": 4}"#).await;
                        
                        // 清理音频缓冲区（如果有的话）
                    }
                }
            }

            // 监听来自音频桥的音频事件
            Some(event) = rx_audio_event.recv() => {
                match event {
                    AudioEvent::AudioData(data) => {
                        // 打印收到的音频数据长度
                        println!("Received Audio from Mic: {} bytes", data.len());
                        
                        // 无论何情况都转发音频到服务器以支持插入式对话（打断），服务端会进行VAD判断用户是否在说话并发送停止命令
                        if current_state != SystemState::Listening {
                             current_state = SystemState::Listening;
                             // Notify GUI: kDeviceStateListening = 5
                             let _ = gui_bridge.send_message(r#"{"state": 5}"#).await;
                        }
                        // 把音频数据转发给服务器
                        let _ = tx_net_cmd.send(NetCommand::SendBinary(data)).await;
                    }
                    AudioEvent::Command(cmd) => {
                        println!("Received Command from AudioBridge: {:?}", cmd);
                        // Handle commands from sound_app if any
                        // 处理音频命令，比如播放结束等
                    }
                }
            }

            // 监听来自GUI桥的GUI事件
            Some(event) = rx_gui_event.recv() => {
                match event {
                    GuiEvent::Message(msg) => {
                        println!("Received Message from GUI: {}", msg);
                        // Forward to Server
                        let _ = tx_net_cmd.send(NetCommand::SendText(msg)).await;
                    }
                }
            }
        }
    }
}
