package bot;

import commands.Mapping;
import commands.Ping;
import commands.Reload;
import commands.Stop;
import java.util.Locale;
import net.dv8tion.jda.api.entities.MessageChannel;
import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import net.dv8tion.jda.api.hooks.SubscribeEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class BotEvents {
	
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
				case "stop" -> Stop.stopBot(channel); // stops the Bot, this takes a while
				case "map" -> Mapping
						.addMapping(channel, content); // add a Mapping or prints all known Mappings
				case "reload" -> Reload.reloadDirectories(channel); // reload Directories
			}
		}
	}
}
