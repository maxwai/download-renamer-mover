package bot;

import javax.security.auth.login.LoginException;
import net.dv8tion.jda.api.JDA;
import net.dv8tion.jda.api.JDABuilder;
import net.dv8tion.jda.api.entities.Activity;
import net.dv8tion.jda.api.hooks.AnnotatedEventManager;
import net.dv8tion.jda.api.requests.GatewayIntent;
import net.dv8tion.jda.api.utils.MemberCachePolicy;
import xml.XMLParser;

public class BotMain {
	
	/**
	 * Token of the Bot, Fetch with the Token.cfg file
	 */
	public static String TOKEN = XMLParser.getBotToken();
	/**
	 * {@link JDA} Instance of the Bot
	 */
	private static JDA jda;
	/**
	 * {@link JDABuilder} for the Bot
	 */
	private static JDABuilder jdaBuilder;
	
	public static void main(String[] args) throws LoginException {
		initializeJDABuilder();
		connectBot();
	}
	
	/**
	 * Will setup the JDA Builder with the necessary settings
	 */
	private static void initializeJDABuilder() {
		jdaBuilder = JDABuilder.createDefault(TOKEN)
				// set to Event Manager to use @Annotated Methods
				.setEventManager(new AnnotatedEventManager())
				// add the Event Listener Class
				.addEventListeners(new BotEvents())
				// set that the Bot is "listening to !help"
				.setActivity(Activity.watching("downloads"))
				// disable the Presences and typing Intents since not used
				.disableIntents(GatewayIntent.GUILD_PRESENCES, GatewayIntent.GUILD_MESSAGE_TYPING)
				// enable the Message reaction and guild members intents
				.enableIntents(GatewayIntent.GUILD_MESSAGE_REACTIONS, GatewayIntent.GUILD_MEMBERS)
				// cache all members, this is used for member fetching
				.setMemberCachePolicy(MemberCachePolicy.ALL);
	}
	
	/**
	 * Connect to the Bot and load the Countdowns
	 *
	 * @throws LoginException if the TOKEN of the Bot is wrong
	 */
	private static void connectBot() throws LoginException {
		jda = jdaBuilder.build();
		try {
			jda.awaitReady(); // wait that the Bot is fully connected
		} catch (InterruptedException ignored) {
		}
	}
	
	/**
	 * Will disconnect the Bot and save the Countdowns
	 */
	public static void disconnectBot() {
		jda.getRegisteredListeners().forEach(jda::removeEventListener);
		jda.shutdown();
	}
	
}
