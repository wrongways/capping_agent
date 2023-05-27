use clap::Parser;
use agent::server;

#[tokio::main]
async fn main() {
    let args = CLI::parse();
    let server = server::Server::new(&args.listen_address);
    server.run()
    .await;
}


#[allow(clippy::upper_case_acronyms)]
#[derive(Parser)]
#[command(author, version, about, long_about=None)]
struct CLI {
    #[arg(long, short, help="eg: '0.0.0.0:8000' or 'oahu10000.local:8080'")]
    listen_address: String,
}
