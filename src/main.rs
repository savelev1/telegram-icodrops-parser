use telegram_icodrops_parser::synchronizer::Synchronizer;

#[tokio::main]
async fn main() {
    let mut synchronizer: Synchronizer = Synchronizer::new();
    synchronizer.run().await;
}