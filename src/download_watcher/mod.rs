use std::{env, thread};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};

use log::{error, info};
use poise::serenity_prelude as serenity;
use serenity::{ChannelId, Context};

use crate::xml;

pub const SIGNAL_NEW_MAPPING: u8 = 1;
pub const SIGNAL_RELOAD: u8 = 2;

pub fn get_paths() -> Option<(PathBuf, PathBuf, PathBuf)> {
    const DOWNLOAD_FOLDER_NAME: &str = "Download";
    const SHARED_VIDEO_FOLDER_NAME: &str = "Shared Video";
    const ANIME_FOLDER_NAME: &str = "Anime";
    const SERIES_FOLDER_NAME: &str = "Serien";

    info!("Parsing programm Arguments");

    let args: Vec<String> = env::args().collect();
    let root_path = Path::new(if args.len() != 2 {
        "./server".deref()
    } else {
        &args[1]
    });
    if !root_path.is_dir() {
        error!("Did not give correct Path to Public Share of Server");
        return None;
    }

    let video_folder = root_path.join(Path::new(SHARED_VIDEO_FOLDER_NAME));

    let anime_folder = video_folder.join(Path::new(ANIME_FOLDER_NAME));
    if !anime_folder.is_dir() {
        error!("Could not find Anime Folder");
        return None;
    }

    let series_folder = video_folder.join(Path::new(SERIES_FOLDER_NAME));
    if !series_folder.is_dir() {
        error!("Could not find Series Folder");
        return None;
    }

    let download_folder = root_path.join(Path::new(DOWNLOAD_FOLDER_NAME));
    if !download_folder.is_dir() {
        error!("Could not find Download Folder");
        return None;
    }
    Some((anime_folder, series_folder, download_folder))
}

pub fn get_current_missing_mappings() -> Vec<String> {
    vec![]
}

pub fn get_directories() -> Vec<String> {
    vec![]
}

pub fn run(_ctx: Context, _anime_folder: PathBuf, _series_folder: PathBuf, _download_folder: PathBuf, _rx: Receiver<u8>) {
    let _channel = ChannelId(xml::get_main_channel());
    // TODO: rcv(SIGNAL_NEW_MAPPING) == new mapping
    // TODO: rcv(SIGNAL_RELOAD) == new mapping
}

pub fn entrypoint(ctx: &Context) -> Option<SyncSender<u8>> {
    let ctx1 = ctx.clone();
    let (anime_folder, series_folder, download_folder): (PathBuf, PathBuf, PathBuf);
    match get_paths() {
        None => {
            return None;
        }
        Some((anime, series, download)) => {
            anime_folder = anime;
            series_folder = series;
            download_folder = download;
        }
    }

    let (tx, rx) = mpsc::sync_channel(16);

    thread::spawn(move || {
        run(ctx1, anime_folder, series_folder, download_folder, rx);
    });
    return Some(tx)
}
