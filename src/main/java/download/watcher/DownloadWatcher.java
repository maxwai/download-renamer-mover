package download.watcher;

import java.awt.Color;
import java.io.IOException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import net.dv8tion.jda.api.EmbedBuilder;
import net.dv8tion.jda.api.entities.TextChannel;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import xml.XMLParser;

public class DownloadWatcher {
	
	/**
	 * All Series name mapped to the respective Paths
	 */
	public static final Map<String, Path> directories = new HashMap<>();
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Download Watcher");
	/**
	 * The Regex to parse a new downloaded File
	 */
	private static final Pattern pattern = Pattern
			.compile("^(.*?)((s\\d+)[- ]?)?(e\\d+).*?\\.(.*)");
	/**
	 * The Preset for the Season Directory
	 */
	private static final String SEASON_PRESET = "Staffel %1$02d";
	/**
	 * The time the thread waits until it checks again if new files are present.
	 */
	private static final int WAIT_TIME = 60 * 1000;
	/**
	 * Set with all the mappings that still need to be done.
	 */
	private static final Set<String> mappingToDo = new HashSet<>();
	/**
	 * The TextChannel where to give the User Feedback and ask for mappings
	 */
	private static TextChannel textChannel;
	/**
	 * The Path of the Anime Folder
	 */
	private static Path anime_folder;
	/**
	 * The Path of the Series Folder.
	 */
	private static Path series_folder;
	/**
	 * The Path of the Download Folder.
	 */
	private static Path download_folder;
	
	/**
	 * Will start the Download Watcher
	 *
	 * @param rootFolder the path of the root Folder of the Server
	 * @param channel The TextChannel where to give the User Feedback and ask for mappings
	 */
	public static void startDownloadWatcher(String rootFolder, @Nonnull TextChannel channel) {
		textChannel = channel;
		final String DOWNLOAD_FOLDER_NAME = "Download";
		final String SHARED_VIDEO_FOLDER_NAME = "Shared Video";
		final String ANIME_FOLDER_NAME = "Anime";
		final String SERIES_FOLDER_NAME = "Serien";
		
		logger.info("Parsing programm Arguments");
		
		final Path root_folder = Path.of(rootFolder);
		if (!Files.isDirectory(root_folder))
			throw new IllegalArgumentException(
					"Did not give correct Path to Public Share of Server");
		
		anime_folder = Path.of(root_folder.toString(), SHARED_VIDEO_FOLDER_NAME, ANIME_FOLDER_NAME);
		if (!Files.isDirectory(anime_folder))
			throw new IllegalStateException("Could not find Anime Folder");
		
		series_folder = Path
				.of(root_folder.toString(), SHARED_VIDEO_FOLDER_NAME, SERIES_FOLDER_NAME);
		if (!Files.isDirectory(series_folder))
			throw new IllegalStateException("Could not find Series Folder");
		
		download_folder = Path.of(root_folder.toString(), DOWNLOAD_FOLDER_NAME);
		if (!Files.isDirectory(download_folder))
			throw new IllegalStateException("Could not find Download Folder");
		
		mapAllKnownDirectories();
		mapAllAlternativeDirectories();
		
		Thread thread = new Thread(() -> {
			while (true) {
				checkDownloadFolder(false);
				try {
					//noinspection BusyWait
					Thread.sleep(WAIT_TIME);
				} catch (InterruptedException ignored) {
				}
			}
		}, "Download-Watcher");
		thread.setDaemon(true);
		thread.start();
	}
	
	/**
	 * Gets all Directories that can be seen in the Anime and Serien directory
	 */
	public static void mapAllKnownDirectories() {
		logger.info("Getting all Subdirectories");
		try {
			Files.newDirectoryStream(anime_folder, Files::isDirectory)
					.forEach(subDir -> directories
							.put(subDir.getFileName().toString().toLowerCase(Locale.ROOT), subDir));
			Files.newDirectoryStream(series_folder, Files::isDirectory)
					.forEach(subDir -> directories
							.put(subDir.getFileName().toString().toLowerCase(Locale.ROOT), subDir));
		} catch (IOException e) {
			logger.error("Got some sort of IOException");
			e.printStackTrace();
			textChannel.sendMessage("Got some sort of IOException please check the logs").queue();
		}
	}
	
	/**
	 * Gets all the mapping from the config.
	 */
	public static void mapAllAlternativeDirectories() {
		logger.info("Getting all known mappings");
		Map<String, String> mappings = XMLParser.getMappings();
		mappings.forEach((s, s2) -> directories.put(s, directories.get(s2)));
	}
	
	/**
	 * Will check the download Folder and move every File possible to the correct Folder.
	 */
	public static void checkDownloadFolder(boolean checkTilde) {
		final List<Path> filesToDo = new ArrayList<>();
		try {
			Files.newDirectoryStream(download_folder, path -> {
				if (!Files.isDirectory(path)) {
					return (path.toString().endsWith(".mp4") ||
							path.toString().endsWith(".mkv") ||
							path.toString().endsWith(".avi")) &&
						   !path.getFileName().toString().startsWith("_") &&
						   (checkTilde || !path.getFileName().toString().startsWith("~"));
				}
				return false;
			}).forEach(filesToDo::add);
		} catch (IOException e) {
			logger.error("Got some sort of IOException");
			e.printStackTrace();
			textChannel.sendMessage("Got some sort of IOException please check the logs").queue();
		}
		if (checkTilde) {
			mappingToDo.clear();
			filesToDo.replaceAll(video -> {
				if (video.getFileName().toString().startsWith("~"))
					try {
						return Files.move(video, video.getParent()
								.resolve(video.getFileName().toString().substring(1)));
					} catch (IOException e) {
						logger.error("Got some sort of IOException");
						e.printStackTrace();
						textChannel
								.sendMessage("Got some sort of IOException please check the logs")
								.queue();
						return null;
					}
				else
					return video;
			});
			filesToDo.removeIf(Objects::isNull);
		}
		if (!filesToDo.isEmpty()) {
			logger.info("Found " + filesToDo.size() + " files in the download Folder");
			final Path video = filesToDo.get(0);
			String name = video.getFileName().toString();
			Matcher matcher = pattern.matcher(name.toLowerCase(Locale.ROOT));
			if (matcher.find()) {
				final String video_name = matcher.group(1).trim();
				final String season = matcher.group(3);
				final int episode = Integer.parseInt(matcher.group(4).substring(1));
				final String file_format = matcher.group(5);
				if (directories.containsKey(video_name)) {
					moveVideo(directories.get(video_name), video, season, episode,
							file_format);
					checkDownloadFolder(checkTilde);
				} else {
					logger.warn("File name \"" + video_name + "\" is not known");
					try {
						Files.move(video, video.getParent().resolve("~" + video.getFileName()));
					} catch (IOException e) {
						logger.error("Got some sort of IOException");
						e.printStackTrace();
						textChannel
								.sendMessage("Got some sort of IOException please check the logs")
								.queue();
					}
					if (mappingToDo.add(video_name)) {
						EmbedBuilder eb = new EmbedBuilder();
						eb.setColor(Color.RED);
						eb.setTitle("Unknown series");
						eb.setDescription(video_name);
						eb.addField("Please add a Mapping with following command:",
								"`/map new alt:" + video_name + " og:<series name on server>`", false);
						
						textChannel.sendMessage(eb.build()).queue();
					}
				}
			} else {
				logger.error("File did not contain regex");
				textChannel.sendMessage("`" + name
										+ "` did not match regex. Please adjust the regex to match fileName")
						.queue();
				try {
					Files.move(video, video.getParent().resolve("_" + video.getFileName()));
				} catch (IOException e) {
					logger.error("Got some sort of IOException");
					e.printStackTrace();
					textChannel.sendMessage("Got some sort of IOException please check the logs")
							.queue();
				}
			}
		}
	}
	
	/**
	 * Will move a found video to the given destination with the correct name.
	 *
	 * @param destination the destination of the video
	 * @param video the video to move
	 * @param season the season found if any, else null
	 * @param episode the episode of the video
	 * @param file_format the file ending of the video
	 */
	private static void moveVideo(@Nonnull Path destination, @Nonnull Path video,
			@Nullable String season, int episode, @Nonnull String file_format) {
		int season_number;
		if (season == null) {
			textChannel.sendMessage("`" + video.getFileName().toString()
									+ "` does not have Season. Please add Season or move it manually")
					.queue();
			try {
				Files.move(video, video.getParent().resolve("_" + video.getFileName()));
			} catch (IOException e) {
				logger.error("Got some sort of IOException");
				e.printStackTrace();
				textChannel.sendMessage("Got some sort of IOException please check the logs")
						.queue();
			}
			return;
		}
		season_number = Integer.parseInt(season.substring(1));
		
		destination = destination.resolve(SEASON_PRESET.formatted(season_number));
		if (!Files.exists(destination)) {
			try {
				Files.createDirectory(destination);
			} catch (IOException e) {
				logger.error("Error while trying to create new Folder.");
				textChannel.sendMessage(
						"Something went wrong while trying to create the directory `" + destination
						+ "`. Please look at the Bot Log.").queue();
				return;
			}
		}
		
		try {
			logger.info("Moving file to " + destination
					.resolve(destination.getParent().getFileName().toString()
							 + " - s%1$02de%2$02d.%3$s"
									 .formatted(season_number, episode, file_format)));
			textChannel
					.sendMessage("Moving `" + video.getFileName().toString() + "` to known folder.")
					.queue();
			Files.move(video, destination.resolve(
					destination.getParent().getFileName().toString() + " - s%1$02de%2$02d.%3$s"
							.formatted(season_number, episode, file_format)));
		} catch (FileAlreadyExistsException e) {
			logger.warn(video.getFileName().toString() + " is a duplicate File");
			textChannel.sendMessage(
					"`" + video.getFileName().toString() + "` is already present please check")
					.queue();
			try {
				Files.move(video, video.getParent().resolve("_" + video.getFileName()));
			} catch (IOException e1) {
				logger.error("Got some sort of IOException");
				e1.printStackTrace();
				textChannel.sendMessage("Got some sort of IOException please check the logs")
						.queue();
			}
		} catch (IOException e) {
			logger.error("Got some sort of IOException");
			e.printStackTrace();
			textChannel.sendMessage("Got some sort of IOException please check the logs").queue();
		}
	}
}
