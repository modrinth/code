package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.api.dto.PlayerStats;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.PlaytimeManager;
import org.ebluffy.chiwawa.managers.UserManager;

/**
 * Команда /profile для просмотра профиля игрока
 */
public class ProfileCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final UserManager userManager;
    private final PlaytimeManager playtimeManager;

    public ProfileCommand(ConfigManager configManager, UserManager userManager, PlaytimeManager playtimeManager) {
        this.configManager = configManager;
        this.userManager = userManager;
        this.playtimeManager = playtimeManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;
        ChiwawaUser user = userManager.getUser(player.getUniqueId());
        PlayerStats stats = userManager.getPlayerStats(player.getUniqueId());

        if (user == null) {
            sender.sendMessage("§cВаши данные не загружены. Попробуйте позже.");
            return true;
        }

        // Показываем профиль
        showProfile(player, user, stats);
        return true;
    }

    private void showProfile(Player player, ChiwawaUser user, PlayerStats stats) {
        player.sendMessage(configManager.getMessage("profile_header"));

        // Основная информация
        player.sendMessage(configManager.getMessage("profile_nickname", "%nickname%", user.getNickname()));
        player.sendMessage(configManager.getMessage("profile_trust_level",
            "%level%", String.valueOf(user.getTrustLevel()),
            "%description%", user.getTrustLevelDescription()));

        // Репутация
        if (stats != null) {
            player.sendMessage(configManager.getMessage("profile_reputation", "%reputation%", String.valueOf(stats.getReputation())));
        }

        // Время игры
        int totalMinutes = playtimeManager.getTotalPlaytime(player.getUniqueId());
        player.sendMessage(configManager.getMessage("profile_playtime", "%playtime%", playtimeManager.formatTime(totalMinutes)));

        // Статус email
        String emailStatus = user.isEmailVerified() ? "§aПодтвержден" : "§cНе подтвержден";
        player.sendMessage(configManager.getMessage("profile_email_status", "%status%", emailStatus));

        // Дата регистрации
        if (user.getRegisteredAt() != null) {
            player.sendMessage(configManager.getMessage("profile_registration", "%date%", user.getRegisteredAt()));
        }

        // Дополнительная информация для проходимцев
        if (user.hasTimeLimit()) {
            int remaining = playtimeManager.getRemainingTime(player.getUniqueId());
            player.sendMessage("§7Лимит времени: §c" + playtimeManager.formatTime(remaining) + " из 10 часов");
            player.sendMessage("§7Для снятия лимита подтвердите email на сайте!");
        }

        player.sendMessage(configManager.getMessage("profile_footer"));
    }
}
