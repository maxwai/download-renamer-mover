extern crate regex;
extern crate reqwest;

use std::{env, thread};
use std::collections::HashMap;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Duration;

use log::{error, info, warn};
use poise::serenity_prelude as serenity;
use regex::Regex;
use serenity::{ChannelId, Context};

use crate::xml;

/// The signal the Bots sends when a new mapping was added
pub const SIGNAL_NEW_MAPPING: u8 = 1;
/// The signal the Bots sends when the folders should be reloaded
pub const SIGNAL_RELOAD: u8 = 2;
/// The signal the Bots send to tell the Thread to Stop
pub const SIGNAL_STOP: u8 = 3;

/// The emoji to prepend when an error occurs
const ERROR_EMOJI: &str = ":x: ";

/// Struct containing shared Objects
pub struct ThreadInfos {
    /// The Mappings that need to be added
    pub missing_mappings: Vec<String>,
    /// The Directories that are present and known
    pub og_directories: HashMap<String, PathBuf>,
}

/// Will get the necessary Paths to start the Download Watcher or None
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

/// The main function that the Download Watcher runs on
#[tokio::main(flavor = "current_thread")]
async fn run(
    ctx: Context,
    anime_folder: PathBuf,
    series_folder: PathBuf,
    download_folder: PathBuf,
    rx: Receiver<u8>,
    shared_thread_infos: Arc<Mutex<ThreadInfos>>,
) {
    const WAIT_TIME_IN_SEC: u64 = 60;

    let channel = ChannelId(xml::get_main_channel());
    let mut directories: HashMap<String, PathBuf> = HashMap::new();
    let mut to_ignore: Vec<PathBuf> = Vec::new();
    get_known_directories(&anime_folder, &series_folder, &shared_thread_infos);
    get_xml_mappings(&mut directories, &shared_thread_infos);
    loop {
        if !check_download_folder(
            &directories,
            &mut to_ignore,
            &shared_thread_infos,
            &download_folder,
            &ctx,
            &channel,
        )
        .await
        {
            match rx.recv_timeout(Duration::from_secs(WAIT_TIME_IN_SEC)) {
                Ok(signal) => match signal {
                    SIGNAL_STOP => return,
                    SIGNAL_RELOAD => {
                        get_known_directories(&anime_folder, &series_folder, &shared_thread_infos);
                        get_xml_mappings(&mut directories, &shared_thread_infos);
                        shared_thread_infos.lock().unwrap().missing_mappings.clear();
                    }
                    SIGNAL_NEW_MAPPING => get_xml_mappings(&mut directories, &shared_thread_infos),
                    _ => error!("Got unknown signal code: {}", signal),
                },
                Err(_) => {} // timeout
            }
        }
    }
}

/// Gets all Directories that can be seen in the Anime and Serien directory
fn get_known_directories(
    anime_folder: &PathBuf,
    series_folder: &PathBuf,
    shared_thread_infos: &Arc<Mutex<ThreadInfos>>,
) {
    traverse_directory(anime_folder, shared_thread_infos);
    traverse_directory(series_folder, shared_thread_infos);
}

/// Gets all Directories that can be seen in the specified directory
fn traverse_directory(folder: &PathBuf, shared_thread_infos: &Arc<Mutex<ThreadInfos>>) {
    std::fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.as_ref().unwrap().path())
        .filter(|path| path.is_dir())
        .for_each(|dir| {
            shared_thread_infos.lock().unwrap().og_directories.insert(
                dir.file_name().unwrap().to_str().unwrap().to_lowercase(),
                dir,
            );
        });
}

/// Gets all the mapping from the config
fn get_xml_mappings(
    directories: &mut HashMap<String, PathBuf>,
    shared_thread_infos: &Arc<Mutex<ThreadInfos>>,
) {
    let new_mappings = xml::get_mappings();
    directories.clear();
    new_mappings.iter().for_each(|(&ref alt, og)| {
        let mutex_share = shared_thread_infos.lock().unwrap();
        match mutex_share.og_directories.get(og) {
            None => {}
            Some(path) => {
                directories.insert(alt.to_string(), path.to_path_buf());
            }
        }
    })
}

/// Will check the download Folder and move every File possible to the correct Folder
async fn check_download_folder(
    directories: &HashMap<String, PathBuf>,
    to_ignore: &mut Vec<PathBuf>,
    shared_thread_infos: &Arc<Mutex<ThreadInfos>>,
    download_folder: &PathBuf,
    ctx: &Context,
    channel: &ChannelId,
) -> bool {
    // gets the available files and also refreshed the to_ignore file vector
    let mut new_to_ignore: Vec<PathBuf> = Vec::new();
    let mut files: Vec<PathBuf> = Vec::new();
    for file in std::fs::read_dir(download_folder)
        .unwrap()
        .map(|entry| entry.as_ref().unwrap().path())
        .filter(|path| path.is_file())
        .filter(|file_path| match file_path.extension() {
            None => false,
            Some(extension) => extension == "mp4" || extension == "mkv" || extension == "avi",
        })
        .filter(|file_path| {
            if to_ignore.contains(file_path) {
                new_to_ignore.push((*file_path).clone());
                return false;
            }
            return true;
        })
    {
        files.push(file);
    }
    to_ignore.clear();
    to_ignore.append(&mut new_to_ignore);

    let pattern = Regex::new(r"(?i)^(.*?)((s\d+)[- ]?)?(e\d+).*?(.*)?\.([a-zA-Z0-9]*)").unwrap();

    // retrieves the video names once in advance to refresh the missing_mappings hashmap
    let mut local_files: Vec<String> = Vec::new();
    for file in files.clone() {
        let name = match file.file_name().unwrap().to_str() {
            None => continue,
            Some(string) => string,
        };
        match pattern.captures(name.to_lowercase().as_str()) {
            None => {}
            Some(captures) => {
                let temp_video_name = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .replace(".", " ")
                    .replace("-", " ");
                let video_name = temp_video_name.trim().to_string();
                local_files.push(video_name);
            }
        }
    }
    {
        shared_thread_infos
            .lock()
            .unwrap()
            .missing_mappings
            .retain(|name| local_files.contains(name));
    }

    // loop that goes through every file and tries to handle it
    let mut return_value = false;
    let mut reply = String::from("");
    'file_loop: for file in files {
        let name = match file.file_name().unwrap().to_str() {
            None => {
                error!("File name not UTF-8: {}", file.display());
                let message = format!("{} File name not UTF-8: {}", ERROR_EMOJI, file.display());
                if reply.len() + message.len() >= 1999 {
                    let _ = channel.say(ctx, reply.clone()).await;
                    reply.clear();
                }
                reply.push_str(message.as_str());
                reply.push('\n');
                break 'file_loop;
            }
            Some(string) => string,
        };
        if check_voe(name, &file, ctx, channel).await {
            return_value = true;
            break 'file_loop;
        }
        match pattern.captures(name.to_lowercase().as_str()) {
            None => {
                warn!("File did not contain regex");
                let message = format!(
                    "{} `{}` did not match regex. Please adjust regex to match file name",
                    ERROR_EMOJI, name);
                if reply.len() + message.len() >= 1999 {
                    let _ = channel.say(ctx, reply.clone()).await;
                    reply.clear();
                }
                reply.push_str(message.as_str());
                reply.push('\n');
                to_ignore.push(file);
            }
            Some(captures) => {
                let temp_video_name = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .replace(".", " ")
                    .replace("-", " ");
                let video_name = temp_video_name.trim();
                let season = match captures.get(3).map(|capture| capture.as_str()) {
                    None => {
                        let message = format!(
                            "{} `{}` does not have a Season. Please add a Season or move it manually",
                            ERROR_EMOJI, name);

                        if reply.len() + message.len() >= 1999 {
                            let _ = channel.say(ctx, reply.clone()).await;
                            reply.clear();
                        }
                        reply.push_str(message.as_str());
                        reply.push('\n');

                        to_ignore.push(file);
                        continue 'file_loop;
                    }
                    Some(season) => season[1..].parse::<i32>().unwrap(),
                };
                let episode = captures.get(4).unwrap().as_str()[1..]
                    .parse::<i32>()
                    .unwrap();
                // group 5 is all the not needed information between episode number and file ending
                let file_format = captures.get(6).unwrap().as_str();

                {
                    // if the video name is already known to be missing, don't prompt the user again
                    if shared_thread_infos
                        .lock()
                        .unwrap()
                        .missing_mappings
                        .contains(&video_name.to_string())
                    {
                        continue 'file_loop;
                    }
                }

                match directories.get(video_name) {
                    Some(video_path) => {
                        let message = move_video(
                            video_path,
                            &file,
                            season,
                            episode,
                            file_format,
                            shared_thread_infos,
                        )
                        .await;

                        if reply.len() + message.len() >= 1999 {
                            let _ = channel.say(ctx, reply.clone()).await;
                            reply.clear();
                        }
                        reply.push_str(message.as_str());
                        reply.push('\n');
                    }
                    None => {
                        let mut mutex_share = shared_thread_infos.lock().unwrap();
                        match mutex_share.og_directories.get(video_name) {
                            Some(video_path) => {
                                let message = move_video(
                                    video_path,
                                    &file,
                                    season,
                                    episode,
                                    file_format,
                                    shared_thread_infos,
                                )
                                .await;

                                if reply.len() + message.len() >= 1999 {
                            let _ = channel.say(ctx, reply.clone()).await;
                            reply.clear();
                        }
                        reply.push_str(message.as_str());
                                reply.push('\n');
                            }
                            None => {
                                warn!("File name \"{}\" is not known", video_name);
                                mutex_share.missing_mappings.push(video_name.to_string());
                                let _ = channel.send_message(ctx, |builder| {
                                    builder.embed(|embed_builder| {
                                        embed_builder.field(
                                            "Please add a Mapping with following command:",
                                            format!("`/map new alt:{} og:<series name on the server>`", video_name),
                                            false)
                                    })
                                }).await;
                            }
                        }
                    }
                }
            }
        };
    }
    if !reply.is_empty() {
        let _ = channel.say(ctx,reply).await;
    }
    return return_value;
}

/// checks if the file was downloaded from voe, in this case get the actual file name
async fn check_voe(name: &str, file: &PathBuf, ctx: &Context, channel: &ChannelId) -> bool {
    let voe_pattern = Regex::new(
        r"Watch (.*\.mp4) - VOE \| Content Delivery Network \(CDN\) & Video Cloud",
    )
    .unwrap();
    let mut local_name = name.clone();
    if !local_name.starts_with("voe_")
        || local_name.contains(" ")
        || local_name.chars().filter(|ch| *ch == '.').count() != 1
        || !local_name.ends_with(".mp4") {
        return false;
    }
    if local_name.starts_with("voe_") {
        local_name = &local_name[4..];
    }

    match reqwest::get(format!(
        "https://voe.sx/e/{}",
        local_name.replace(".mp4", "")
    ))
    .await
    {
        Ok(response) => {
            for line in response.text().await.unwrap().split("\n") {
                if line.contains("<title>") {
                    if let Some(captures) = voe_pattern.captures(line) {
                        let video_name = captures.get(1).unwrap().as_str().trim();
                        std::fs::rename(file, file.parent().unwrap().join(video_name)).unwrap();
                        return true;
                    }
                    break;
                }
            }
        }
        Err(why) => {
            error!("{:?}", why);
            let _ = channel
                .say(
                    ctx,
                    format!(
                        "{} Got some kind of error while trying to connect to voe, check the logs",
                        ERROR_EMOJI
                    ),
                )
                .await;
            return false;
        }
    };

    return false;
}

/// Will move a found video to the given destination with the correct name
async fn move_video(
    destination: &PathBuf,
    source: &PathBuf,
    season: i32,
    episode: i32,
    file_format: &str,
    shared_thread_infos: &Arc<Mutex<ThreadInfos>>,
) -> String {
    let season_destination = destination.join(format!("Staffel {:02}", season));
    if !season_destination.is_dir() {
        if let Err(why) = std::fs::create_dir(season_destination.clone()) {
            error!("{:?}", why);
            return format!(
                "{} Something went wrong while trying to create the directory `{}`. Please look at the logs",
                ERROR_EMOJI, season_destination.display());
        }
    }

    let target = season_destination.join(format!(
        "{} - s{:02}e{:02}.{}",
        destination.file_name().unwrap().to_str().unwrap(),
        season,
        episode,
        file_format
    ));
    if target.is_file() {
        warn!(
            "{} is a duplicate file",
            source.file_name().unwrap().to_str().unwrap()
        );
        shared_thread_infos.lock().unwrap().missing_mappings.push(
            destination
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
        );
        return format!("{} File already present: `{}`",
                       ERROR_EMOJI, source.file_name().unwrap().to_str().unwrap());
    }
    return match std::fs::rename(source, target.clone()) {
        Ok(_) => {
            info!(
                "Moved {} to {}",
                source.file_name().unwrap().to_str().unwrap(),
                target.file_name().unwrap().to_str().unwrap()
            );
            format!(
                "Moved `{}` as `{}` to known folder.",
                source.file_name().unwrap().to_str().unwrap(),
                target.file_name().unwrap().to_str().unwrap()
            )
        }
        Err(why) => {
            error!("{:?}", why);
            format!(
                "{} Something went wrong while trying to move the file `{}`. Please look at the logs",
                ERROR_EMOJI, source.file_name().unwrap().to_str().unwrap()
            )
        }
    };
}

/// The entrypoint to start the download watcher thread
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
        og_directories: HashMap::new(),
    }));

    let infos_for_thread = Arc::clone(&shared_thread_infos);
    let _ = thread::Builder::new()
        .name("download_watcher".into())
        .spawn(move || {
            run(
                ctx1,
                anime_folder,
                series_folder,
                download_folder,
                rx,
                infos_for_thread,
            );
        });
    return Some((tx, shared_thread_infos));
}
