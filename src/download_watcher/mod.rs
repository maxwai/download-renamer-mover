use std::{env, thread};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender};

use log::{error, info};
use poise::serenity_prelude as serenity;
use serenity::{ChannelId, Context};

use crate::xml;

pub const SIGNAL_NEW_MAPPING: u8 = 1;
pub const SIGNAL_RELOAD: u8 = 2;

pub struct ThreadInfos {
    pub missing_mappings: Vec<String>,
    pub directories: HashMap<String, PathBuf>,
    pub og_directories: HashMap<String, PathBuf>
}

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

pub fn run(_ctx: Context, _anime_folder: PathBuf, _series_folder: PathBuf, _download_folder: PathBuf, _rx: Receiver<u8>, _shared_thread_infos: Arc<Mutex<ThreadInfos>>) {
    let _channel = ChannelId(xml::get_main_channel());
    // TODO: rcv(SIGNAL_NEW_MAPPING) == new mapping -> reload mappings from xml
    // TODO: rcv(SIGNAL_RELOAD) == reload directories
}

pub fn entrypoint(ctx: &Context) -> Option<(SyncSender<u8>, Arc<Mutex<ThreadInfos>>)> {
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

    let shared_thread_infos = Arc::new(Mutex::new(ThreadInfos {
        missing_mappings: Vec::new(),
        directories: HashMap::new(),
        og_directories: HashMap::new()
    }));

    let infos_for_thread = Arc::clone(&shared_thread_infos);
    thread::spawn(move || {
        run(ctx1, anime_folder, series_folder, download_folder, rx, infos_for_thread);
    });
    return Some((tx, shared_thread_infos));
}
