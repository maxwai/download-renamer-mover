package commands;

import static bot.BotEvents.MAP_ALL_SUBCOMMAND;
import static bot.BotEvents.MAP_ALT_OPTION;
import static bot.BotEvents.MAP_COMMAND;
import static bot.BotEvents.MAP_NEW_SUBCOMMAND;
import static bot.BotEvents.MAP_OG_OPTION;
import static bot.BotEvents.PING_COMMAND;
import static bot.BotEvents.RELOAD_COMMAND;
import static bot.BotEvents.STOP_COMMAND;

import net.dv8tion.jda.api.events.message.MessageReceivedEvent;
import net.dv8tion.jda.api.interactions.commands.OptionType;
import net.dv8tion.jda.api.interactions.commands.build.CommandData;
import net.dv8tion.jda.api.interactions.commands.build.OptionData;
import net.dv8tion.jda.api.interactions.commands.build.SubcommandData;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

public class Slash {
	
	/**
	 * The Logger for Log Messages
	 */
	private static final Logger logger = LoggerFactory.getLogger("Ping Command");
	
	/**
	 * Will reload all Slash Commands in the guild
	 *
	 * @param event The event where the command was sent from
	 */
	public static void reloadSlashCommands(MessageReceivedEvent event) {
		if (event.isFromGuild()) {
			logger.info("Reloading all slash Commands");
			event.getGuild().updateCommands()
					.addCommands(new CommandData(PING_COMMAND, "Gives the current ping"))
					.addCommands(new CommandData(RELOAD_COMMAND, "Will reload all Directories"))
					.addCommands(new CommandData(STOP_COMMAND, "Will stop the bot"))
					.addCommands(new CommandData(MAP_COMMAND, "mapping command")
							.addSubcommands(
									new SubcommandData(MAP_ALL_SUBCOMMAND,
											"Will give back all known mappings"),
									new SubcommandData(MAP_NEW_SUBCOMMAND, "Creates a new mapping")
											.addOptions(new OptionData(OptionType.STRING,
															MAP_ALT_OPTION,
															"alternative name", true),
													new OptionData(OptionType.STRING, MAP_OG_OPTION,
															"series name on server", true))))
					.queue();
			event.getChannel().sendMessage("Reloaded all Slash Commands for this Guild").queue();
		} else
			event.getChannel().sendMessage("Must send this Command from a Guild").queue();
	}
}
