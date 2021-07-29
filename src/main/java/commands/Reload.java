package commands;

import download.watcher.DownloadWatcher;
import net.dv8tion.jda.api.entities.MessageChannel;
import net.dv8tion.jda.api.events.interaction.SlashCommandEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Reload {
	
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Reload Command");
	
	/**
	 * Will reload all Directories
	 *
	 * @param channel The Channel where the command was send from
	 */
	public static void reloadDirectories(MessageChannel channel) {
		logger.info("Reloading all Directories");
		DownloadWatcher.directories.clear();
		DownloadWatcher.mapAllKnownDirectories();
		DownloadWatcher.mapAllAlternativeDirectories();
		channel.sendMessage("Reloaded all Directories").queue();
		DownloadWatcher.checkDownloadFolder(true);
	}
	
	/**
	 * Will reload all Directories
	 *
	 * @param event The Event where the command was send from
	 */
	public static void reloadDirectories(SlashCommandEvent event) {
		event.deferReply().queue();
		logger.info("Reloading all Directories");
		DownloadWatcher.directories.clear();
		DownloadWatcher.mapAllKnownDirectories();
		DownloadWatcher.mapAllAlternativeDirectories();
		event.getHook().sendMessage("Reloaded all Directories").queue();
		DownloadWatcher.checkDownloadFolder(true);
	}
}
