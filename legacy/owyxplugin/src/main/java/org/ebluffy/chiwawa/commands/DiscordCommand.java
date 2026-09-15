package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.config.ConfigManager;

/**
 * Команда /discord для информации о Discord сервере
 */
public class DiscordCommand implements CommandExecutor {
    private final ConfigManager configManager;

    public DiscordCommand(ConfigManager configManager) {
        this.configManager = configManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;
        showDiscordInfo(player);
        return true;
    }

    private void showDiscordInfo(Player player) {
        player.sendMessage("§e═════ DISCORD СЕРВЕР ═════");
        player.sendMessage("§7Присоединяйтесь к нашему Discord серверу:");
        player.sendMessage("§9§nhttps://discord.gg/chiwawa");
        player.sendMessage("");
        player.sendMessage("§7В Discord вы можете:");
        player.sendMessage("§7• Общаться с другими игроками");
        player.sendMessage("§7• Получать новости сервера");
        player.sendMessage("§7• Связать Discord аккаунт для повышения Trust Level");
        player.sendMessage("§7• Обращаться в поддержку");
        player.sendMessage("§e═══════════════════════");
    }
}
