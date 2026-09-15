package org.ebluffy.chiwawa.commands;

import org.bukkit.Bukkit;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.command.TabCompleter;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import com.google.gson.JsonObject;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.api.dto.PlayerStats;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.PlaytimeManager;
import org.ebluffy.chiwawa.managers.ReputationManager;
import org.ebluffy.chiwawa.managers.UserManager;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.UUID;
import java.util.stream.Collectors;

/**
 * Главная команда /chiwawa для администрирования плагина
 */
public class ChiwawaCommand implements CommandExecutor, TabCompleter {
    private final JavaPlugin plugin;
    private final ConfigManager configManager;
    private final UserManager userManager;
    private final PlaytimeManager playtimeManager;
    private final ReputationManager reputationManager;
    private final ApiClient apiClient;

    public ChiwawaCommand(JavaPlugin plugin, ConfigManager configManager, UserManager userManager,
                         PlaytimeManager playtimeManager, ReputationManager reputationManager, ApiClient apiClient) {
        this.plugin = plugin;
        this.configManager = configManager;
        this.userManager = userManager;
        this.playtimeManager = playtimeManager;
        this.reputationManager = reputationManager;
        this.apiClient = apiClient;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (args.length == 0) {
            sendHelp(sender);
            return true;
        }

        String subCommand = args[0].toLowerCase();
        switch (subCommand) {
            case "reload":
                return handleReload(sender);
            case "sync":
                return handleSync(sender, args);
            case "ban":
                return handleBan(sender, args);
            case "unban":
                return handleUnban(sender, args);
            case "trust":
                return handleTrust(sender, args);
            case "playtime":
                return handlePlaytime(sender, args);
            case "rep":
                return handleReputation(sender, args);
            case "stats":
                return handleStats(sender);
            default:
                sender.sendMessage("§cНеизвестная команда! Используйте /chiwawa для справки.");
                return true;
        }
    }

    /**
     * Показать справку по командам
     */
    private void sendHelp(CommandSender sender) {
        sender.sendMessage("§e§l════════ CHIWAWA КОМАНДЫ ════════");

        if (hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§7/chiwawa reload §f- Перезагрузить конфиг");
            sender.sendMessage("§7/chiwawa sync <игрок> §f- Синхронизировать данные");
            sender.sendMessage("§7/chiwawa ban <игрок> <причина> §f- Забанить игрока");
            sender.sendMessage("§7/chiwawa unban <игрок> §f- Разбанить игрока");
            sender.sendMessage("§7/chiwawa trust <игрок> <0-3> §f- Изменить Trust Level");
            sender.sendMessage("§7/chiwawa playtime <игрок> §f- Показать время игры");
            sender.sendMessage("§7/chiwawa rep <игрок> <кол-во> [причина] §f- Изменить репутацию");
            sender.sendMessage("§7/chiwawa stats §f- Статистика сервера");
        } else {
            sender.sendMessage("§cУ вас нет прав для использования этих команд!");
        }

        sender.sendMessage("§e§l═══════════════════════════════");
    }

    /**
     * Перезагрузить конфигурацию и переподключиться к API
     */
    private boolean handleReload(CommandSender sender) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        sender.sendMessage("§e§l[RELOAD] §7Начинаем перезагрузку плагина...");
        
        // 1. Перезагружаем конфигурацию
        sender.sendMessage("§7[1/2] Перезагрузка конфигурации...");
        configManager.reloadConfig();
        
        // 2. Тестируем подключение к API
        sender.sendMessage("§7[2/2] Проверка подключения к API...");
        apiClient.testConnection().thenAccept(connected -> {
            if (connected) {
                sender.sendMessage("§a[✓] API соединение восстановлено!");
            } else {
                sender.sendMessage("§c[✗] Ошибка подключения к API!");
            }
        });
        
        // // 3. Очищаем кеш пользователей
        // sender.sendMessage("§7[3/4] Очистка кеша пользователей...");
        // userManager.clearCache();
        // sender.sendMessage("§a[✓] Кеш пользователей очищен!");
        
        // // 4. Завершаем перезагрузку
        // sender.sendMessage("§7[3/3] Завершение...");
        
        sender.sendMessage("§a§l[SUCCESS] §fПлагин успешно перезагружен!");
        return true;
    }

    /**
     * Синхронизировать данные игрока
     */
    private boolean handleSync(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 2) {
            sender.sendMessage("§cИспользование: /chiwawa sync <игрок>");
            return true;
        }

        String playerName = args[1];
        Player targetPlayer = Bukkit.getPlayerExact(playerName);

        if (targetPlayer == null) {
            sender.sendMessage("§cИгрок не найден или не в сети!");
            return true;
        }

        sender.sendMessage("§7Синхронизация данных игрока " + playerName + "...");

        userManager.syncPlayer(targetPlayer.getUniqueId())
            .thenAccept(success -> {
                if (success) {
                    sender.sendMessage("§aСинхронизация завершена успешно!");
                } else {
                    sender.sendMessage("§cОшибка синхронизации данных!");
                }
            });

        return true;
    }

    /**
     * Забанить игрока
     */
    private boolean handleBan(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 3) {
            sender.sendMessage("§cИспользование: /chiwawa ban <игрок> <причина> [дни]");
            sender.sendMessage("§7Пример: /chiwawa ban Player Читы 7");
            sender.sendMessage("§7Для постоянного бана не указывайте дни или укажите 0");
            return true;
        }

        String playerName = args[1];
        
        // Определяем где заканчивается причина и начинается количество дней
        int daysIndex = -1;
        int daysValue = 0; // 0 означает постоянный бан
        
        // Проверяем последний аргумент - если это число, то это дни
        if (args.length > 3) {
            try {
                String lastArg = args[args.length - 1];
                daysValue = Integer.parseInt(lastArg);
                daysIndex = args.length - 1;
            } catch (NumberFormatException e) {
                // Последний аргумент не число, значит вся строка после имени - это причина
                daysValue = 0;
            }
        }
        
        final int days = daysValue; // Делаем final для использования в lambda
        
        // Формируем причину
        String reason;
        if (daysIndex > 0) {
            // Есть дни в конце, причина до них
            reason = String.join(" ", Arrays.copyOfRange(args, 2, daysIndex));
        } else {
            // Нет дней, вся строка после имени - причина
            reason = String.join(" ", Arrays.copyOfRange(args, 2, args.length));
        }

        // Используем nickname-based бан для поддержки оффлайн игроков
        apiClient.banPlayerByNickname(playerName, reason, days)
            .thenAccept(success -> {
                if (success) {
                    if (days > 0) {
                        sender.sendMessage("§aИгрок " + playerName + " забанен на " + days + " дней!");
                        sender.sendMessage("§7Причина: " + reason);
                    } else {
                        sender.sendMessage("§aИгрок " + playerName + " забанен навсегда!");
                        sender.sendMessage("§7Причина: " + reason);
                    }

                    // Кикаем игрока если он онлайн
                    Player targetPlayer = Bukkit.getPlayerExact(playerName);
                    if (targetPlayer != null) {
                        String kickMessage = "§c§lВЫ ЗАБЛОКИРОВАНЫ НА СЕРВЕРЕ\n\n";
                        kickMessage += "§fПричина: §c" + reason + "\n\n";
                        if (days > 0) {
                            kickMessage += "§fТип бана: §eВременный (" + days + " дней)\n\n";
                        } else {
                            kickMessage += "§fТип бана: §4Постоянный\n\n";
                        }
                        kickMessage += "§7Обратитесь к администрации для разбана";
                        
                        final String finalKickMessage = kickMessage;
                        // ВАЖНО: Кик должен происходить в главном потоке!
                        plugin.getServer().getScheduler().runTask(plugin, () -> {
                            targetPlayer.kick(net.kyori.adventure.text.Component.text(finalKickMessage));
                        });
                        
                        // Уведомляем всех администраторов и игроков о бане
                        String banAnnouncement = "§c[БАН] §fИгрок §c" + playerName + " §fзабанен";
                        if (days > 0) {
                            banAnnouncement += " на §e" + days + " дней";
                        } else {
                            banAnnouncement += " §4навсегда";
                        }
                        banAnnouncement += "§f. Причина: §7" + reason;
                        
                        final String finalBanAnnouncement = banAnnouncement;
                        plugin.getServer().getScheduler().runTask(plugin, () -> {
                            for (Player onlinePlayer : Bukkit.getOnlinePlayers()) {
                                onlinePlayer.sendMessage(finalBanAnnouncement);
                            }
                        });
                    }
                } else {
                    sender.sendMessage("§cОшибка при бане игрока! Возможно игрок не найден в базе данных.");
                }
            })
            .exceptionally(throwable -> {
                sender.sendMessage("§cОшибка при бане игрока: " + throwable.getMessage());
                return null;
            });

        return true;
    }

    /**
     * Разбанить игрока
     */
    private boolean handleUnban(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 2) {
            sender.sendMessage("§cИспользование: /chiwawa unban <игрок>");
            return true;
        }

        String playerName = args[1];

        // Используем nickname-based разбан для поддержки оффлайн игроков
        apiClient.unbanPlayerByNickname(playerName)
            .thenAccept(success -> {
                if (success) {
                    sender.sendMessage("§aИгрок " + playerName + " успешно разбанен!");
                    
                    // Уведомляем всех игроков о разбане
                    String unbanAnnouncement = "§a[РАЗБАН] §fИгрок §a" + playerName + " §fразбанен администратором §a" + sender.getName();
                    plugin.getServer().getScheduler().runTask(plugin, () -> {
                        for (Player onlinePlayer : Bukkit.getOnlinePlayers()) {
                            onlinePlayer.sendMessage(unbanAnnouncement);
                        }
                    });
                } else {
                    sender.sendMessage("§cОшибка при разбане игрока! Возможно игрок не найден в базе данных.");
                }
            })
            .exceptionally(throwable -> {
                sender.sendMessage("§cОшибка при разбане игрока: " + throwable.getMessage());
                return null;
            });

        return true;
    }

    /**
     * Изменить Trust Level
     */
    private boolean handleTrust(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 3) {
            sender.sendMessage("§cИспользование: /chiwawa trust <игрок> <уровень>");
            sender.sendMessage("§7Уровни: 0-Проходимец, 1-Новичок, 2-Проверенный, 3-Ветеран");
            return true;
        }

        String playerName = args[1];
        int trustLevel;

        try {
            trustLevel = Integer.parseInt(args[2]);
        } catch (NumberFormatException e) {
            sender.sendMessage("§cНеверный формат уровня доверия!");
            return true;
        }

        if (trustLevel < 0 || trustLevel > 3) {
            sender.sendMessage("§cУровень доверия должен быть от 0 до 3!");
            return true;
        }

        UUID playerUuid = getPlayerUuid(playerName);
        if (playerUuid == null) {
            sender.sendMessage("§cИгрок не найден!");
            return true;
        }

        userManager.updateTrustLevel(playerUuid, trustLevel)
            .thenAccept(success -> {
                if (success) {
                    String levelName = getTrustLevelName(trustLevel);
                    sender.sendMessage("§aТраст уровень игрока " + playerName + " изменен на " + trustLevel + " (" + levelName + ")");

                    // Уведомляем игрока если он онлайн
                    Player targetPlayer = Bukkit.getPlayerExact(playerName);
                    if (targetPlayer != null) {
                        targetPlayer.sendMessage("§a§l[★] §fВаш Trust Level повышен до §a§l" + trustLevel + " §7(" + levelName + ")§f!");
                    }
                } else {
                    sender.sendMessage("§cОшибка при изменении Trust Level!");
                }
            });

        return true;
    }

    /**
     * Показать время игры
     */
    private boolean handlePlaytime(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 2) {
            sender.sendMessage("§cИспользование: /chiwawa playtime <игрок>");
            return true;
        }

        String playerName = args[1];
        UUID playerUuid = getPlayerUuid(playerName);

        if (playerUuid == null) {
            sender.sendMessage("§cИгрок не найден!");
            return true;
        }

        PlayerStats stats = userManager.getPlayerStats(playerUuid);
        if (stats == null) {
            sender.sendMessage("§cСтатистика игрока не найдена!");
            return true;
        }

        int totalMinutes = playtimeManager.getTotalPlaytime(playerUuid);
        int sessionMinutes = playtimeManager.getSessionPlaytime(playerUuid);

        sender.sendMessage("§e═════ ВРЕМЯ ИГРЫ: " + playerName + " ═════");
        sender.sendMessage("§7Общее время: §b" + playtimeManager.formatTime(totalMinutes));
        sender.sendMessage("§7Текущая сессия: §b" + playtimeManager.formatTime(sessionMinutes));

        ChiwawaUser user = userManager.getUser(playerUuid);
        if (user != null && user.hasTimeLimit()) {
            int remaining = playtimeManager.getRemainingTime(playerUuid);
            sender.sendMessage("§7Осталось времени: §c" + playtimeManager.formatTime(remaining));
        } else {
            sender.sendMessage("§7Лимит времени: §aОтсутствует");
        }

        return true;
    }

    /**
     * Изменить репутацию (админ команда)
     */
    private boolean handleReputation(CommandSender sender, String[] args) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        if (args.length < 3) {
            sender.sendMessage("§cИспользование: /chiwawa rep <игрок> <количество> [причина]");
            return true;
        }

        String playerName = args[1];
        int amount;

        try {
            amount = Integer.parseInt(args[2]);
        } catch (NumberFormatException e) {
            sender.sendMessage("§cНеверный формат количества!");
            return true;
        }

        String reason = args.length > 3 ? String.join(" ", Arrays.copyOfRange(args, 3, args.length)) : "Изменение администратором";

        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        reputationManager.modifyReputation((Player) sender, playerName, amount, reason)
            .thenAccept(result -> sender.sendMessage(result.getMessage()));

        return true;
    }

    /**
     * Показать статистику сервера
     */
    private boolean handleStats(CommandSender sender) {
        if (!hasPermission(sender, "chiwawa.admin")) {
            sender.sendMessage("§cУ вас нет прав для выполнения этой команды!");
            return true;
        }

        sender.sendMessage("§e§l════════ СТАТИСТИКА СЕРВЕРА ════════");
        sender.sendMessage("§7Игроков онлайн: §a" + Bukkit.getOnlinePlayers().size());
        sender.sendMessage("§7Загружено пользователей: §a" + userManager.getCacheSize());
        sender.sendMessage("§7Отслеживается времени: §a" + playtimeManager.getPlaytimeStats());
        sender.sendMessage("§7Репутация: §a" + reputationManager.getReputationStats());
        
        // Получаем информацию с сайта
        sender.sendMessage("§7Получение данных с сайта...");
        apiClient.getServerInfo().thenAccept(serverInfo -> {
            if (serverInfo != null && serverInfo.has("success") && serverInfo.get("success").getAsBoolean()) {
                sender.sendMessage("§a§l──── ДАННЫЕ С САЙТА ────");
                
                if (serverInfo.has("server") && !serverInfo.get("server").isJsonNull()) {
                    JsonObject server = serverInfo.getAsJsonObject("server");
                    sender.sendMessage("§7Статус сервера: §a" + (serverInfo.has("online") && !serverInfo.get("online").isJsonNull() ? 
                        (serverInfo.get("online").getAsBoolean() ? "онлайн" : "офлайн") : "неизвестно"));
                    
                    if (server.has("version") && !server.get("version").isJsonNull()) {
                        sender.sendMessage("§7Версия: §f" + server.get("version").getAsString());
                    }
                    
                    if (server.has("motd") && !server.get("motd").isJsonNull()) {
                        sender.sendMessage("§7MOTD: §f" + server.get("motd").getAsString());
                    }
                }
                
                if (serverInfo.has("performance") && !serverInfo.get("performance").isJsonNull()) {
                    JsonObject performance = serverInfo.getAsJsonObject("performance");
                    
                    if (performance.has("ping") && !performance.get("ping").isJsonNull()) {
                        sender.sendMessage("§7Ping: §e" + performance.get("ping").getAsInt() + "ms");
                    }
                    
                    if (performance.has("tps") && !performance.get("tps").isJsonNull()) {
                        sender.sendMessage("§7TPS: §e" + performance.get("tps").getAsDouble());
                    }
                }
                
                if (serverInfo.has("players") && !serverInfo.get("players").isJsonNull()) {
                    JsonObject players = serverInfo.getAsJsonObject("players");
                    
                    int online = players.has("online") && !players.get("online").isJsonNull() ? players.get("online").getAsInt() : 0;
                    int max = players.has("max") && !players.get("max").isJsonNull() ? players.get("max").getAsInt() : 0;
                    
                    sender.sendMessage("§7Игроков в базе: §a" + online + "/" + max);
                }
            } else {
                sender.sendMessage("§cОшибка получения данных с сайта!");
            }
        }).exceptionally(throwable -> {
            sender.sendMessage("§cОшибка подключения к API сайта: " + throwable.getMessage());
            return null;
        });
        
        sender.sendMessage("§e§l══════════════════════════════════");

        return true;
    }

    /**
     * Проверить права доступа
     */
    private boolean hasPermission(CommandSender sender, String permission) {
        if (sender.isOp()) return true;
        if (!(sender instanceof Player)) return true; // Консоль имеет все права

        Player player = (Player) sender;
        ChiwawaUser user = userManager.getUser(player.getUniqueId());

        if (user == null) return false;

        // Проверяем по РОЛИ, а не trust level
        if (permission.equals("chiwawa.admin")) {
            return user.isAdmin(); // Проверяет role == "admin"
        }

        if (permission.equals("chiwawa.moderator")) {
            return user.isModerator(); // Проверяет role == "moderator" || role == "admin"
        }

        return player.hasPermission(permission);
    }

    /**
     * Получить UUID игрока по имени
     */
    private UUID getPlayerUuid(String playerName) {
        Player player = Bukkit.getPlayerExact(playerName);
        if (player != null) {
            return player.getUniqueId();
        }

        // Ищем в кеше пользователей
        for (var entry : userManager.getAllCachedUsers().entrySet()) {
            if (entry.getValue().getNickname().equalsIgnoreCase(playerName)) {
                return entry.getKey();
            }
        }

        return null;
    }

    /**
     * Получить название trust level
     */
    private String getTrustLevelName(int trustLevel) {
        switch (trustLevel) {
            case 0: return "Проходимец";
            case 1: return "Новичок";
            case 2: return "Проверенный";
            case 3: return "Ветеран";
            default: return "Неизвестный";
        }
    }

    @Override
    public List<String> onTabComplete(CommandSender sender, Command command, String alias, String[] args) {
        if (args.length == 1) {
            // Первый аргумент - подкоманды
            List<String> subCommands = Arrays.asList("reload", "sync", "ban", "unban", "trust", "playtime", "rep", "stats");
            return subCommands.stream()
                .filter(cmd -> cmd.toLowerCase().startsWith(args[0].toLowerCase()))
                .collect(Collectors.toList());
        }

        if (args.length == 2) {
            // Второй аргумент - имена игроков для большинства команд
            String subCommand = args[0].toLowerCase();
            if (Arrays.asList("sync", "ban", "unban", "trust", "playtime", "rep").contains(subCommand)) {
                return Bukkit.getOnlinePlayers().stream()
                    .map(Player::getName)
                    .filter(name -> name.toLowerCase().startsWith(args[1].toLowerCase()))
                    .collect(Collectors.toList());
            }
        }

        if (args.length == 3 && args[0].equalsIgnoreCase("trust")) {
            // Trust level от 0 до 3
            return Arrays.asList("0", "1", "2", "3").stream()
                .filter(level -> level.startsWith(args[2]))
                .collect(Collectors.toList());
        }

        return new ArrayList<>();
    }
}
