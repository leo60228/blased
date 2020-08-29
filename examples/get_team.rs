use blased::*;

#[async_std::main]
async fn main() -> Result<()> {
    let client = BlaseballClient::new();
    let best_team = "8d87c468-699a-47a8-b40d-cfb73a5660ad";
    let team = client.get_team(best_team).await?;
    println!("{:#?}", team);

    Ok(())
}
