use blased::*;
use std::env::args;

#[async_std::main]
async fn main() -> Result<()> {
    let mut args = args().skip(1);
    let user = args.next().unwrap();
    let pass = args.next().unwrap();
    let client = AuthenticatedClient::user_pass(&user, &pass).await?;
    let user = client.get_user().await?;
    println!("{:#?}", user);
    let team = client.get_team(&user.favorite_team).await?;
    println!("{:#?}", team);

    Ok(())
}
