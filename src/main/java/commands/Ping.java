package commands;

import net.dv8tion.jda.api.entities.MessageChannel;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Ping {
	
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Ping Command");
	
	/**
	 * Will send a "Pong!" Message and then edit this message to know the current ping of the Bot
	 *
	 * @param channel The Channel where the ping was send from
	 */
	public static void makePing(MessageChannel channel) {
		long time = System.currentTimeMillis();
		logger.info("Sending Ping Message");
		channel.sendMessage("Pong!").queue(message ->
				message.editMessageFormat("Pong: %d ms", System.currentTimeMillis() - time)
						.queue());
	}
}
