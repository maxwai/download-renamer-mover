use log::{error, info};
use sonarr::apis::api_info_api::api_get;

pub mod bot;
pub mod download_watcher;
pub mod xml;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    info!("booting up");
    match api_get(xml::get_sonarr_config()).await {
        Ok(_) => {
            info!("Connected to sonarr server");
        }
        Err(err) => {
            error!("Couldn't connect to sonarr server: {:?}", err);
            panic!();
        }
    };
    bot::entrypoint().await;
    return;
}
