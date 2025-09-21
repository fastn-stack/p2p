use clap::Parser;

#[derive(Parser)]
#[command(name = "expose_http")]
#[command(about = "Expose local HTTP service over P2P")]
struct Args {
    /// Local host to forward to
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// Local port to forward to  
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Bridge server domain
    #[arg(long, default_value = "kulfi.site")]
    bridge: String,
}

#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();
    
    println!("expose_http: {}:{} via {}", args.host, args.port, args.bridge);
    
    Ok(())
}