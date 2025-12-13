use std::io::Result;

fn main() -> Result<()> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/pb")
        .compile(
            &["proto/hyperliquid.proto"],
            &["proto"],
        )?;
    Ok(())
}