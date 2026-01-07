extern crate regex;
extern crate reqwest;

use crate::xml;
use log::{error, info, warn};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{CreateEmbed, CreateMessage};
use regex::Regex;
use serenity::{ChannelId, Context};
use sonarr::apis::episode_api::api_v3_episode_get;
use sonarr::apis::series_api::api_v3_series_get;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::{env, thread};

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
    /// The files that are duplicates
    pub duplicate_files: Vec<String>,
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
        "./server"
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
#[tokio::main]
async fn run(
    ctx: Context,
    anime_folder: PathBuf,
    series_folder: PathBuf,
    download_folder: PathBuf,
    rx: Receiver<u8>,
    shared_thread_infos: Arc<Mutex<ThreadInfos>>,
) {
    const WAIT_TIME_IN_SEC: u64 = 15;

    let channel = ChannelId::new(xml::get_main_channel());
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
            if let Ok(signal) = rx.recv_timeout(Duration::from_secs(WAIT_TIME_IN_SEC)) {
                match signal {
                    SIGNAL_STOP => return,
                    SIGNAL_RELOAD => {
                        get_known_directories(&anime_folder, &series_folder, &shared_thread_infos);
                        get_xml_mappings(&mut directories, &shared_thread_infos);
                        shared_thread_infos.lock().unwrap().missing_mappings.clear();
                    }
                    SIGNAL_NEW_MAPPING => get_xml_mappings(&mut directories, &shared_thread_infos),
                    _ => error!("Got unknown signal code: {}", signal),
                }
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
    new_mappings.iter().for_each(|(alt, og)| {
        let mutex_share = shared_thread_infos.lock().unwrap();
        match mutex_share.og_directories.get(og) {
            None => {}
            Some(path) => {
                directories.insert(alt.to_string(), path.to_path_buf());
            }
        }
    })
}

/// Append string to reply and post reply in channel if too long
async fn append_to_reply(ctx: &Context, channel: &ChannelId, reply: &mut String, message: String) {
    if !message.is_empty() {
        if reply.len() + message.len() >= 1999 {
            let _ = channel.say(ctx, reply.clone()).await;
            reply.clear();
        }
        reply.push_str(message.as_str());
        reply.push('\n');
    }
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
            true
        })
    {
        files.push(file);
    }
    to_ignore.clear();
    to_ignore.append(&mut new_to_ignore);

    let pattern =
        Regex::new(r"(?i)^(?:\[.*] *)?(.*?)(?:[ (.]+20\d{2}[ ).]+)?(s\d+)?[- ]*e?(\d+).*?(?:.*)?\.([a-zA-Z0-9]*)").unwrap();

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
                    .replace(['.', '-'], " ")
                    .replace(", ", " ")
                    .replace(",", " ");
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
    let mut reply = String::from("");
    'file_loop: for file in files {
        let name = match file.file_name().unwrap().to_str() {
            None => {
                error!("File name not UTF-8: {}", file.display());
                let message = format!("{} File name not UTF-8: {}", ERROR_EMOJI, file.display());
                append_to_reply(ctx, channel, &mut reply, message).await;
                break 'file_loop;
            }
            Some(string) => string,
        };
        match pattern.captures(name.to_lowercase().as_str()) {
            None => {
                warn!("File did not contain regex");
                let message = format!(
                    "{} `{}` did not match regex. Please adjust regex to match file name",
                    ERROR_EMOJI, name
                );
                append_to_reply(ctx, channel, &mut reply, message).await;
                to_ignore.push(file);
            }
            Some(captures) => {
                let temp_video_name = captures
                    .get(1)
                    .unwrap()
                    .as_str()
                    .replace(['.', '-'], " ")
                    .replace("  ", " ");
                let video_name = temp_video_name.trim();
                let season = captures.get(2);
                let episode = captures.get(3).unwrap().as_str().parse::<i32>().unwrap();
                // group 5 is all the not needed information between episode number and file ending
                let file_format = captures.get(4).unwrap().as_str();

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
                        let (season, episode) = match season {
                            None => match get_only_missing_episode(video_path).await {
                                None => {
                                    warn!("File didn't contain season and there isn't exactly one episode missing");
                                    let message = format!(
                                        "{} `{}` didn't contain season and there isn't exactly one episode missing. Add season to name.",
                                        ERROR_EMOJI, name
                                    );
                                    append_to_reply(ctx, channel, &mut reply, message).await;
                                    to_ignore.push(file);
                                    continue 'file_loop;
                                }
                                Some(tuple) => tuple,
                            },
                            Some(season) => (season.as_str()[1..].parse::<i32>().unwrap(), episode),
                        };
                        let message = move_video(
                            video_path,
                            &file,
                            season,
                            episode,
                            file_format,
                            shared_thread_infos,
                        )
                        .await;
                        append_to_reply(ctx, channel, &mut reply, message).await;
                    }
                    None => {
                        let path = shared_thread_infos
                            .lock()
                            .unwrap()
                            .og_directories
                            .get(video_name)
                            .cloned();
                        match path {
                            Some(video_path) => {
                                let (season, episode) = match season {
                                    None => match get_only_missing_episode(&video_path).await {
                                        None => {
                                            warn!("File didn't contain season and there isn't exactly one episode missing");
                                            let message = format!(
                                                "{} `{}` didn't contain season and there isn't exactly one episode missing. Add season to name.",
                                                ERROR_EMOJI, name
                                            );
                                            append_to_reply(ctx, channel, &mut reply, message)
                                                .await;
                                            to_ignore.push(file);
                                            continue 'file_loop;
                                        }
                                        Some(tuple) => tuple,
                                    },
                                    Some(season) => {
                                        (season.as_str()[1..].parse::<i32>().unwrap(), episode)
                                    }
                                };
                                let message = move_video(
                                    &video_path,
                                    &file,
                                    season,
                                    episode,
                                    file_format,
                                    shared_thread_infos,
                                )
                                .await;
                                append_to_reply(ctx, channel, &mut reply, message).await;
                            }
                            None => {
                                warn!("File name \"{}\" is not known", video_name);
                                shared_thread_infos
                                    .lock()
                                    .unwrap()
                                    .missing_mappings
                                    .push(video_name.to_string());
                                let _ = channel
                                    .send_message(
                                        ctx,
                                        CreateMessage::default().embed(
                                            CreateEmbed::default().field(
                                                "Please add a Mapping with following command:",
                                                format!(
                                                "`/map new alt:{} og:<series name on the server>`",
                                                video_name
                                            ),
                                                false,
                                            ),
                                        ),
                                    )
                                    .await;
                            }
                        }
                    }
                }
            }
        };
    }
    if !reply.is_empty() {
        let _ = channel.say(ctx, reply).await;
    }
    false
}

/// Will move a found video to the given destination with the correct name
async fn move_video(
    destination: &Path,
    source: &Path,
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
        let file_name = source.file_name().unwrap().to_str().unwrap().to_string();
        return if !shared_thread_infos
            .lock()
            .unwrap()
            .duplicate_files
            .contains(&file_name)
        {
            warn!(
                "{} is a duplicate file",
                source.file_name().unwrap().to_str().unwrap()
            );
            shared_thread_infos
                .lock()
                .unwrap()
                .duplicate_files
                .push(file_name);
            format!(
                "{} File already present: `{}`",
                ERROR_EMOJI,
                source.file_name().unwrap().to_str().unwrap()
            )
        } else {
            String::new()
        };
    }
    match std::fs::rename(source, target.clone()) {
        Ok(_) => {
            info!(
                "Moved {} to {}",
                source.file_name().unwrap().to_str().unwrap(),
                target.file_name().unwrap().to_str().unwrap()
            );
            format!(
                "Moved `{}` as `{}` to known folder.",
                source
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('`', "\\`"),
                target
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('`', "\\`")
            )
        }
        Err(why) => {
            error!("{:?}", why);
            format!(
                "{} Something went wrong while trying to move the file `{}`. Please look at the logs",
                ERROR_EMOJI, source.file_name().unwrap().to_str().unwrap()
            )
        }
    }
}

/// Will fetch the series and check if the newest episode is the only episode missing
async fn get_only_missing_episode(path: &Path) -> Option<(i32, i32)> {
    match api_v3_series_get(xml::get_sonarr_config(), None, None).await {
        Ok(series_vec) => {
            for series in series_vec {
                if series.path.is_some_and(|sonarr_path| {
                    sonarr_path.is_some_and(|sonarr_path| {
                        Path::new(&sonarr_path).ends_with(path.file_name().unwrap_or("".as_ref()))
                    })
                }) {
                    let mut seasons = series.seasons??;
                    seasons.sort_by_key(|season| season.season_number);
                    if seasons.last()?.statistics.clone()?.episode_count? == 0 {
                        seasons.pop();
                    }
                    for season in seasons.clone() {
                        let statistics = season.statistics.clone()?;
                        if statistics.episode_count? == statistics.episode_file_count? {
                            continue;
                        }
                        if statistics.episode_count? == statistics.episode_file_count? + 1 {
                            if seasons.iter().position(|n| n == &season)? == seasons.len() - 1 {
                                let season_number = season.season_number?;
                                let series_id = series.id?;
                                match api_v3_episode_get(
                                    xml::get_sonarr_config(),
                                    Some(series_id),
                                    Some(season_number),
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                )
                                .await
                                {
                                    Ok(episodes) => {
                                        for episode in episodes {
                                            if episode.has_file? {
                                                continue;
                                            }
                                            if episode.episode_number? == statistics.episode_count?
                                            {
                                                return Some((
                                                    season_number,
                                                    episode.episode_number?,
                                                ));
                                            }
                                            return None;
                                        }
                                    }
                                    Err(err) => {
                                        error!("{:?}", err);
                                        return None;
                                    }
                                }
                                return None;
                            }
                        }
                    }
                    return None;
                }
            }
            None
        }
        Err(err) => {
            error!("{:?}", err);
            None
        }
    }
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
        duplicate_files: Vec::new(),
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
    Some((tx, shared_thread_infos))
}
