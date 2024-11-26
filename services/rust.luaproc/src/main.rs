use clap::Parser;
use std::sync::Arc;
use templating::LuaVmAction;

#[derive(Parser, Debug, Clone)]
pub struct CmdArgs {
    #[clap(long)]
    pub guild_id: String,
    #[clap(long)]
    pub server_name: String,
}

#[tokio::main]
async fn main() {
    // Setup logging
    let cmd_args = Arc::new(CmdArgs::parse());
}
