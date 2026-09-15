package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.dto.Application;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.UserManager;

/**
 * Команда /applications для просмотра статуса заявки
 */
public class ApplicationsCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final UserManager userManager;

    public ApplicationsCommand(ConfigManager configManager, UserManager userManager) {
        this.configManager = configManager;
        this.userManager = userManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;

        // Показываем информацию о заявках
        showApplicationInfo(player);
        return true;
    }

    private void showApplicationInfo(Player player) {
        player.sendMessage("§e═════ СТАТУС ЗАЯВКИ ═════");
        player.sendMessage("§7Ваша заявка уже одобрена, раз вы находитесь на сервере!");
        player.sendMessage("§7Для получения дополнительной информации посетите:");
        player.sendMessage("§b§nhttps://chiwawa.site");
        player.sendMessage("§e═══════════════════════");
    }
}
