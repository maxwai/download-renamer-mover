package commands;

import static bot.BotEvents.MAP_ALT_OPTION;
import static bot.BotEvents.MAP_OG_OPTION;

import download.watcher.DownloadWatcher;
import java.awt.Color;
import java.util.AbstractMap.SimpleEntry;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import net.dv8tion.jda.api.EmbedBuilder;
import net.dv8tion.jda.api.entities.MessageEmbed;
import net.dv8tion.jda.api.events.interaction.SlashCommandEvent;
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
	 * Will add a Mapping to the Bot or show all Mappings
	 *
	 * @param event The event where the Message was sent.
	 * @param content The Content of the Message that was sent.
	 */
	public static void addMapping(MessageReceivedEvent event, String content) {
		if (content.equals("map")) {
			
			logger.info("Sending mappings");
			event.getChannel().sendMessageEmbeds(getMappingEmbed()).queue();
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
					event.getChannel().sendMessage("Don't know `" + OG + "` please try again.")
							.queue();
				}
			} else
				event.getChannel().sendMessage("""
						Not correct command format.
						Correct format is:
						`!map <alternative name> -> <series name on server>`""").queue();
		}
	}
	
	/**
	 * Will show all Mappings
	 *
	 * @param event The event where the Message was sent.
	 */
	public static void sendMappings(SlashCommandEvent event) {
		event.replyEmbeds(getMappingEmbed()).queue();
	}
	
	/**
	 * Will add a Mapping to the Bot
	 *
	 * @param event The event where the Message was sent.
	 */
	public static void addNewMapping(SlashCommandEvent event) {
		event.deferReply().queue();
		final String alt = Objects.requireNonNull(event.getOption(MAP_ALT_OPTION))
				.getAsString()
				.toLowerCase(Locale.ROOT);
		final String OG = Objects.requireNonNull(event.getOption(MAP_OG_OPTION))
				.getAsString()
				.toLowerCase(Locale.ROOT);
		if (DownloadWatcher.directories.containsKey(OG)) {
			logger.info("Adding new Mapping");
			event.getHook().sendMessage("Done").queue();
			DownloadWatcher.directories.put(alt, DownloadWatcher.directories.get(OG));
			XMLParser.addMapping(new SimpleEntry<>(alt, OG));
			DownloadWatcher.checkDownloadFolder(true);
		} else {
			logger.warn("Mapping could not be added");
			event.getHook().sendMessage("Don't know `" + OG + "` please try again.").queue();
		}
	}
	
	private static MessageEmbed getMappingEmbed() {
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
		return eb.build();
	}
}
