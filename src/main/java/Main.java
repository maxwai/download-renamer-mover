import bot.BotMain;
import download.watcher.DownloadWatcher;
import java.io.IOException;
import javax.security.auth.login.LoginException;

public class Main {
	
	public static void main(String[] args) throws LoginException, IOException {
		DownloadWatcher.startDownloadWatcher(args[0], BotMain.startBot());
	}
}
