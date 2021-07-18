package bot;

import commands.BotStatus;
import commands.Ping;
import java.util.Locale;
import net.dv8tion.jda.api.entities.MessageChannel;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import net.dv8tion.jda.api.hooks.SubscribeEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class BotEvents {
	
	private static final Logger logger = LoggerFactory.getLogger("BotStatus");
	
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
				case "ping" -> Ping.makePing(channel); // make a ping test
				case "stop" -> BotStatus.stopBot(channel); // stops the Bot, this takes a while
			}
		}
	}
	
}
