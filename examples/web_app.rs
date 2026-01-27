use hnefatafl_arena::web::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🛡️  Hnefatafl Arena - Web Edition");
    println!("====================================");
    println!();

    run_server().await?;

    Ok(())
}
