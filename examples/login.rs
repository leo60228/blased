use blased::*;
use std::env::args;

#[async_std::main]
async fn main() -> Result<()> {
    let mut args = args().skip(1);
    let user = args.next().unwrap();
    let pass = args.next().unwrap();
    let _client = AuthenticatedClient::user_pass(&user, &pass).await?;

    Ok(())
}
