use log::info;
use log4rs;

pub mod bot;
pub mod xml;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    info!("booting up");

    let bot_instance = bot::entrypoint();

    bot_instance.await;

    return;
}
