use log::info;

pub mod bot;
pub mod download_watcher;
pub mod xml;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("booting up");
    bot::entrypoint().await;
    return;
}
