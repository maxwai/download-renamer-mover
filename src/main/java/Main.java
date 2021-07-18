import bot.BotMain;
import download.watcher.DownloadWatcher;
import javax.security.auth.login.LoginException;

public class Main {
	
	public static void main(String[] args) throws LoginException {
		DownloadWatcher.startDownloadWatcher(args[0], BotMain.startBot());
	}
}
