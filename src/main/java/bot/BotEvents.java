package bot;

import commands.Mapping;
import commands.Ping;
import commands.Reload;
import commands.Slash;
import commands.Stop;
import java.util.Locale;
import net.dv8tion.jda.api.entities.MessageChannel;
import net.dv8tion.jda.api.events.interaction.SlashCommandEvent;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import net.dv8tion.jda.api.hooks.SubscribeEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class BotEvents {
	
	public static final String
			PING_COMMAND = "ping",
			RELOAD_COMMAND = "reload",
			STOP_COMMAND = "stop",
			MAP_COMMAND = "map",
			SLASH_COMMAND = "slash",
			MAP_ALL_SUBCOMMAND = "all",
			MAP_NEW_SUBCOMMAND = "new",
			MAP_ALT_OPTION = "alt",
			MAP_OG_OPTION = "og";
	
	/**
	 * Is triggered when a Message is written
	 *
	 * @param event Message Receive Event
	 */
	@SubscribeEvent
	public void onReceiveMessage(MessageReceivedEvent event) {
		if (event.getAuthor().isBot()) return;
		Logger logger = LoggerFactory.getLogger("ReceivedMessage");
		String content = event.getMessage().getContentRaw().toLowerCase(Locale.ROOT);
		MessageChannel channel = event.getChannel();
		
		if (content.length() != 0 && content.charAt(0) == '!') {
			logger.info("Received Message from " + event.getAuthor().getName() + " in channel "
						+ channel.getName() + ": " + event.getMessage().getContentRaw());
			String command;
			content = content.substring(1);
			if (content.indexOf(' ') != -1)
				command = content.substring(0, content.indexOf(' '));
			else
				command = content;
			switch (command) {
				case PING_COMMAND -> Ping.makePing(channel); // make a ping test
				case STOP_COMMAND -> Stop.stopBot(channel); // stops the Bot, this takes a while
				case MAP_COMMAND -> Mapping
						.addMapping(event, content); // add a Mapping or prints all known Mappings
				case RELOAD_COMMAND -> Reload.reloadDirectories(channel); // reload Directories
				case SLASH_COMMAND -> Slash.reloadSlashCommands(event); // reload slash Commands
			}
		}
	}
	
	/**
	 * Is triggered when a SlashCommand is sent to the bot.
	 *
	 * @param event Slash Command Event
	 */
	@SubscribeEvent
	public void onSlashCommand(SlashCommandEvent event) {
		Logger logger = LoggerFactory.getLogger("ReceivedMessage");
		logger.info("Received Slash Command from " + event.getUser().getName() + " in channel "
					+ event.getChannel().getName() + ": " + event.getCommandPath());
		switch (event.getName()) {
			case PING_COMMAND -> Ping.makePing(event); // make a ping test
			case RELOAD_COMMAND -> Reload.reloadDirectories(event); // reload Directories
			case STOP_COMMAND -> Stop.stopBot(event); // stops the Bot, this takes a while
			case MAP_COMMAND -> {
				if (MAP_ALL_SUBCOMMAND.equals(event.getSubcommandName()))
					Mapping.sendMappings(event);
				else if (MAP_NEW_SUBCOMMAND.equals(event.getSubcommandName()))
					Mapping.addNewMapping(event);
				else {
					event.reply("Got unknown slash command: " + event.getCommandPath()).queue();
					logger.warn("Unknown Slash Command: " + event.getCommandPath());
				}
			}
			default -> {
				event.reply("Got unknown slash command: " + event.getCommandPath()).queue();
				logger.warn("Unknown Slash Command: " + event.getCommandPath());
			}
		}
	}
}
