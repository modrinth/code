package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.command.TabCompleter;
import org.bukkit.entity.Player;
import org.bukkit.Bukkit;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.ReputationManager;
import org.ebluffy.chiwawa.managers.UserManager;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

/**
 * Команда /rep для выдачи репутации игрокам
 */
public class ReputationCommand implements CommandExecutor, TabCompleter {
    private final ConfigManager configManager;
    private final ReputationManager reputationManager;
    private final UserManager userManager;

    public ReputationCommand(ConfigManager configManager, ReputationManager reputationManager, UserManager userManager) {
        this.configManager = configManager;
        this.reputationManager = reputationManager;
        this.userManager = userManager;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;

        // Проверяем, включена ли система репутации
        if (!configManager.isReputationEnabled()) {
            player.sendMessage("§cСистема репутации отключена на сервере!");
            return true;
        }

        if (args.length == 0) {
            showReputationInfo(player);
            return true;
        }

        if (args.length < 1) {
            player.sendMessage("§cИспользование: /rep <игрок>");
            return true;
        }

        String targetName = args[0];

        // Выдаем репутацию
        reputationManager.giveReputation(player, targetName)
            .thenAccept(result -> {
                if (result.isSuccess()) {
                    player.sendMessage("§a" + result.getMessage());
                } else {
                    player.sendMessage("§c" + result.getMessage());
                }
            });

        return true;
    }

    private void showReputationInfo(Player player) {
        player.sendMessage("§e═══════ СИСТЕМА РЕПУТАЦИИ ═══════");
        player.sendMessage("§7Использование: §f/rep <игрок>");
        player.sendMessage("§7Описание: §fДать +1 репутацию игроку");
        player.sendMessage("§7Кулдаун: §f1 час между выдачами");
        player.sendMessage("§7Требования: §fTrust Level 1+");

        // Показываем статус кулдауна
        if (!reputationManager.canGiveReputation(player.getUniqueId())) {
            long remaining = reputationManager.getRemainingCooldown(player.getUniqueId());
            long minutes = remaining / (60 * 1000);
            long seconds = (remaining / 1000) % 60;
            player.sendMessage("§cКулдаун: §f" + minutes + "м " + seconds + "с");
        } else {
            player.sendMessage("§aВы можете дать репутацию!");
        }

        player.sendMessage("§e═══════════════════════════════");
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        if (args.length == 1) {
            return Bukkit.getOnlinePlayers().stream()
                .map(Player::getName)
                .filter(name -> !name.equals(sender.getName())) // Исключаем самого себя
                .filter(name -> name.toLowerCase().startsWith(args[0].toLowerCase()))
                .collect(Collectors.toList());
        }
        return new ArrayList<>();
    }
}
