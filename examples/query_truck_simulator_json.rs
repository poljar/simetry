#[cfg(windows)]
#[tokio::main]
async fn main() {
    use simetry::truck_simulator;
    use std::time::Duration;

    let data = truck_simulator::json_client::Client::connect(
        truck_simulator::json_client::DEFAULT_URI,
        Duration::from_secs(1),
    )
    .await
    .query()
    .await
    .unwrap();
    println!("{:#?}", data);
}

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    anyhow::bail!("This example only works on Windows")
}
