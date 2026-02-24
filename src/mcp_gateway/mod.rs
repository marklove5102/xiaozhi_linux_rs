pub mod config;
pub mod protocol;
pub mod server;
pub mod tool;

pub use config::ExternalToolConfig;
pub use server::McpServer;

use tool::DynamicTool;

pub fn init_mcp_gateway(configs: Vec<ExternalToolConfig>) -> McpServer {
    let mut server = McpServer::new();
    for config in configs {
        let tool_name = config.name.clone();
        let tool = DynamicTool::new(config);
        server.register_tool(Box::new(tool));
        log::info!("Registered MCP Tool: {}", tool_name);
    }
    server
}
