package commands;

import bot.BotMain;
import net.dv8tion.jda.api.entities.MessageChannel;
import net.dv8tion.jda.api.events.interaction.SlashCommandEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Stop {
	
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Bot Status");
	
	/**
	 * Will Stop the Bot
	 *
	 * @param channel The Channel where the Message was send.
	 */
	public static void stopBot(MessageChannel channel) {
		logger.warn("Stopping Bot");
		channel.sendMessage("stopping the Bot. Bye...").queue();
		BotMain.disconnectBot(); // stop the Bot
	}
	
	/**
	 * Will Stop the Bot
	 *
	 * @param event The Event where the Message was send.
	 */
	public static void stopBot(SlashCommandEvent event) {
		logger.warn("Stopping Bot");
		event.reply("stopping the Bot. Bye...").queue();
		BotMain.disconnectBot(); // stop the Bot
	}
}
