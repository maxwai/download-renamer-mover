package commands;

import download.watcher.DownloadWatcher;
import java.awt.Color;
import java.util.AbstractMap.SimpleEntry;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import net.dv8tion.jda.api.EmbedBuilder;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import xml.XMLParser;

public class Mapping {
	
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Mapping Command");
	
	/**
	 * Will add a Mapping to the Bot
	 *
	 * @param event The event where the Message was send.
	 * @param content The Content of the Message that was send.
	 */
	public static void addMapping(MessageReceivedEvent event, String content) {
		if (content.equals("map")) {
			EmbedBuilder eb = new EmbedBuilder();
			eb.setColor(Color.YELLOW);
			eb.setDescription("Known Mappings:");
			Map<String, List<String>> mappings = new HashMap<>();
			DownloadWatcher.directories.forEach((s, path) -> {
				if (!path.getFileName().toString().toLowerCase(Locale.ROOT).equals(s)) {
					if (mappings.containsKey(path.getFileName().toString()))
						mappings.get(path.getFileName().toString()).add(s);
					else {
						final List<String> list = new ArrayList<>();
						list.add(s);
						mappings.put(path.getFileName().toString(), list);
					}
				}
			});
			mappings.forEach((s, strings) -> eb
					.addField(s, "`" + String.join("`\n`", strings) + "`", true));
			logger.info("Sending mappings");
			event.getChannel().sendMessage(eb.build()).queue();
		} else {
			if (content.contains("->")) {
				final String alt = content.substring(4, content.indexOf("->")).trim()
						.toLowerCase(Locale.ROOT);
				final String OG = content.substring(content.indexOf("->") + 2).trim()
						.toLowerCase(Locale.ROOT);
				if (DownloadWatcher.directories.containsKey(OG)) {
					logger.info("Adding new Mapping");
					event.getMessage().addReaction("\u2705").queue();
					DownloadWatcher.directories.put(alt, DownloadWatcher.directories.get(OG));
					XMLParser.addMapping(new SimpleEntry<>(alt, OG));
					DownloadWatcher.checkDownloadFolder(true);
				} else {
					logger.warn("Mapping could not be added");
					event.getChannel().sendMessage("Don't know `" + OG + "` please try again.").queue();
				}
			} else
				event.getChannel().sendMessage("""
						Not correct command format.
						Correct format is:
						`!map <alternative name> -> <series name on server>`""").queue();
		}
	}
}
