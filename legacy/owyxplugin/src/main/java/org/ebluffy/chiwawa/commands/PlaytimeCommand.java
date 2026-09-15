package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.PlaytimeManager;
import org.ebluffy.chiwawa.managers.UserManager;

/**
 * Команда /playtime для просмотра времени игры
 */
public class PlaytimeCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final PlaytimeManager playtimeManager;
    private final UserManager userManager;

    public PlaytimeCommand(ConfigManager configManager, PlaytimeManager playtimeManager, UserManager userManager) {
        this.configManager = configManager;
        this.playtimeManager = playtimeManager;
        this.userManager = userManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;
        showPlaytime(player);
        return true;
    }

    private void showPlaytime(Player player) {
        ChiwawaUser user = userManager.getUser(player.getUniqueId());

        if (user == null) {
            player.sendMessage("§cВаши данные не загружены. Попробуйте позже.");
            return;
        }

        int totalMinutes = playtimeManager.getTotalPlaytime(player.getUniqueId());
        int sessionMinutes = playtimeManager.getSessionPlaytime(player.getUniqueId());

        player.sendMessage("§e═══════ ВРЕМЯ ИГРЫ ═══════");

        // Общее время
        player.sendMessage(configManager.getMessage("playtime_total", "%time%", playtimeManager.formatTime(totalMinutes)));

        // Текущая сессия
        player.sendMessage(configManager.getMessage("playtime_current", "%time%", playtimeManager.formatTime(sessionMinutes)));

        // Лимит времени (только для проходимцев)
        if (user.hasTimeLimit()) {
            int remaining = playtimeManager.getRemainingTime(player.getUniqueId());
            player.sendMessage(configManager.getMessage("playtime_remaining", "%time%", playtimeManager.formatTime(remaining)));
            player.sendMessage("§7Всего доступно: §c10 часов");
            player.sendMessage("§7Для снятия лимита подтвердите email на сайте:");
            player.sendMessage("§b§nhttps://chiwawa.site");
        } else {
            player.sendMessage(configManager.getMessage("playtime_unlimited"));
        }

        player.sendMessage("§e═══════════════════════");
    }
}
