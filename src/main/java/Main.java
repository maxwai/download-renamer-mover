import bot.BotMain;
import download.watcher.DownloadWatcher;
import javax.security.auth.login.LoginException;

public class Main {
	
	public static void main(String[] args) throws LoginException {
		String path = args.length == 0 ? "/server" : args[0];
		DownloadWatcher.startDownloadWatcher(path, BotMain.startBot());
	}
}
