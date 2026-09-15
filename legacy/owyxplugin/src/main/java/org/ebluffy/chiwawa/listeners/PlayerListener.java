package org.ebluffy.chiwawa.listeners;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.player.AsyncPlayerPreLoginEvent;
import org.bukkit.event.player.PlayerJoinEvent;
import org.bukkit.event.player.PlayerQuitEvent;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.PlaytimeManager;
import org.ebluffy.chiwawa.managers.PlayerStatsManager;
import org.ebluffy.chiwawa.managers.UserManager;

import java.util.logging.Logger;

/**
 * Слушатель событий игроков
 */
public class PlayerListener implements Listener {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ConfigManager configManager;
    private final UserManager userManager;
    private final PlaytimeManager playtimeManager;
    private final PlayerStatsManager playerStatsManager;
    private final ApiClient apiClient;
    private final org.ebluffy.chiwawa.managers.StatsManager statsManager;

    public PlayerListener(JavaPlugin plugin, ConfigManager configManager, UserManager userManager, PlaytimeManager playtimeManager, PlayerStatsManager playerStatsManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.configManager = configManager;
        this.userManager = userManager;
        this.playtimeManager = playtimeManager;
        this.playerStatsManager = playerStatsManager;
        // Получаем ApiClient и StatsManager из главного плагина
        this.apiClient = ((org.ebluffy.chiwawa.ChiwawaPlugin) plugin).getApiClient();
        this.statsManager = ((org.ebluffy.chiwawa.ChiwawaPlugin) plugin).getStatsManager();
    }

    /**
     * Обработка входа игрока (асинхронно, до входа на сервер)
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onAsyncPlayerPreLogin(AsyncPlayerPreLoginEvent event) {
        String playerName = event.getName();
        logger.info("Проверка доступа для игрока: " + playerName);

        try {
            // Сначала проверяем соединение с API
            Boolean apiConnected = apiClient.testConnection().get();
            
            if (!apiConnected) {
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_OTHER,
                    "§cОшибка подключения к серверу!\n§fПопробуйте зайти позже.\n§7Ошибка: API недоступен");
                logger.severe("Игрок " + playerName + " не допущен - API недоступен");
                return;
            }

            // Если whitelist включен, проверяем доступ
            if (configManager.isWhitelistEnabled()) {
                Boolean hasAccess = apiClient.checkServerAccess(playerName).get();

                if (!hasAccess) {
                    event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_WHITELIST,
                        configManager.getMessage("no_access_kick"));
                    logger.info("Игрок " + playerName + " не допущен на сервер - нет доступа");
                    return;
                }
            }

            // ВАЖНО: Проверяем бан игрока ВСЕГДА (независимо от whitelist)
            ChiwawaUser user = apiClient.getUserByNickname(playerName).get();
            if (user != null && user.isBanned()) {
                String banMessage = "§c§lВЫ ЗАБЛОКИРОВАНЫ НА СЕРВЕРЕ\n\n";
                banMessage += "§fПричина: §c" + (user.getBanReason() != null ? user.getBanReason() : "Не указана") + "\n\n";
                
                if (user.getBanUntil() != null && !user.getBanUntil().isEmpty()) {
                    // Временный бан
                    banMessage += "§fТип бана: §eВременный\n";
                    banMessage += "§fОстается времени: §e" + user.getFormattedBanTimeRemaining() + "\n\n";
                } else {
                    // Постоянный бан
                    banMessage += "§fТип бана: §4Постоянный\n\n";
                }
                
                banMessage += "§7Обратитесь к администрации для разбана\n";
                banMessage += "§7Discord: §bhttps://discord.gg/chiwawa";
                
                event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_BANNED, banMessage);
                logger.info("Игрок " + playerName + " не допущен на сервер - заблокирован: " + user.getBanReason());
                return;
            }

            logger.info("Игрок " + playerName + " допущен на сервер");

        } catch (Exception e) {
            logger.severe("Ошибка проверки доступа для " + playerName + ": " + e.getMessage());

            // В случае ошибки API - НЕ пускаем игрока (fail-secure)
            event.disallow(AsyncPlayerPreLoginEvent.Result.KICK_OTHER,
                "§cОшибка подключения к серверу!\n§fПопробуйте зайти позже.\n§7Ошибка: " + e.getMessage());
            logger.warning("Игрок " + playerName + " не допущен из-за ошибки API");
        }
    }

    /**
     * Обработка входа игрока на сервер
     */
    @EventHandler(priority = EventPriority.MONITOR)
    public void onPlayerJoin(PlayerJoinEvent event) {
        Player player = event.getPlayer();
        String playerName = player.getName();
        String playerUuid = player.getUniqueId().toString();
        String ipAddress = player.getAddress().getAddress().getHostAddress();

        // Сначала проверяем активную игровую сессию
        apiClient.checkGameSession(playerName, playerUuid, ipAddress)
            .thenAccept(sessionResult -> {
                if (sessionResult.isValid()) {
                    // Сессия найдена, авторизуем игрока автоматически
                    plugin.getServer().getScheduler().runTask(plugin, () -> {
                        userManager.authorizePlayer(player, sessionResult.getUserId());
                        
                        // Уведомляем StatsManager о входе игрока
                        statsManager.onPlayerJoin(player.getUniqueId());
                        
                        // Запускаем отслеживание сессии игрока
                        playerStatsManager.startPlayerSession(player);
                        
                        plugin.getServer().getScheduler().runTaskLater(plugin, () -> {
                            player.sendMessage("§a§l✓ Добро пожаловать!");
                            player.sendMessage("§7Вы автоматически авторизованы по сохраненной сессии");
                            player.sendMessage("§7Роль: §a" + getRoleDisplayName(sessionResult.getRole()));
                            player.sendMessage("§7Trust Level: §a" + sessionResult.getTrustLevel());
                        }, 20L);
                        
                        logger.info("Игрок " + playerName + " автоматически авторизован по сессии");
                        
                        // Загружаем данные пользователя
                        loadUserDataAfterAuth(player);
                    });
                } else {
                    // Сессия не найдена, требуется авторизация через токен
                    plugin.getServer().getScheduler().runTask(plugin, () -> {
                        showAuthorizationMessage(player);
                        
                        // Даем игроку 5 минут на авторизацию
                        plugin.getServer().getScheduler().runTaskLater(plugin, () -> {
                            if (!userManager.isPlayerCached(player.getUniqueId())) {
                                player.kickPlayer("§cВремя авторизации истекло");
                                logger.info("Игрок " + playerName + " кикнут за неавторизованность");
                            }
                        }, 20L * 60 * 5); // 5 минут
                    });
                }
            })
            .exceptionally(throwable -> {
                logger.warning("Ошибка проверки сессии для " + playerName + ": " + throwable.getMessage());
                
                // В случае ошибки требуем авторизацию через токен
                plugin.getServer().getScheduler().runTask(plugin, () -> {
                    showAuthorizationMessage(player);
                });
                
                return null;
            });
    }

    /**
     * Показать сообщение о необходимости авторизации
     */
    private void showAuthorizationMessage(Player player) {
        plugin.getServer().getScheduler().runTaskLater(plugin, () -> {
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            player.sendMessage("§c§l            ТРЕБУЕТСЯ АВТОРИЗАЦИЯ");
            player.sendMessage("");
            player.sendMessage("§7Выберите способ авторизации:");
            player.sendMessage("");
            player.sendMessage("§a§l1. ПРОСТАЯ АВТОРИЗАЦИЯ");
            player.sendMessage("§7   Новый игрок? §f/register <пароль> <повтор>");
            player.sendMessage("§7   Уже есть аккаунт? §f/login <пароль>");
            player.sendMessage("");
            player.sendMessage("§b§l2. ЧЕРЕЗ САЙТ §7(рекомендуется)");
            player.sendMessage("§7   • Зайдите на: §b§nhttps://chiwawa.site");
            player.sendMessage("§7   • Получите токен в личном кабинете");
            player.sendMessage("§7   • Используйте: §f/login <токен>");
            player.sendMessage("");
            player.sendMessage("§7У вас есть §c5 минут §7для авторизации");
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }, 20L);
    }

    /**
     * Загрузить данные пользователя после авторизации
     */
    private void loadUserDataAfterAuth(Player player) {
        // Регистрируем время входа
        playtimeManager.onPlayerJoin(player.getUniqueId());

        // Получаем данные из кеша (они уже должны быть загружены через userManager.authorizePlayer)
        plugin.getServer().getScheduler().runTaskLater(plugin, () -> {
            ChiwawaUser user = userManager.getCachedUser(player.getUniqueId());
            if (user != null) {
                showWelcomeMessage(player, user);
                logger.info("Игрок " + player.getName() + " загружен с данными (Trust Level: " + user.getTrustLevel() + ", Role: " + user.getRole() + ")");
            }
        }, 40L); // Небольшая задержка чтобы данные точно загрузились
    }

    /**
     * Обработка выхода игрока
     */
    @EventHandler(priority = EventPriority.MONITOR)
    public void onPlayerQuit(PlayerQuitEvent event) {
        Player player = event.getPlayer();

        // Уведомляем StatsManager о выходе игрока
        statsManager.onPlayerQuit(player.getUniqueId());

        // Завершаем отслеживание сессии игрока
        playerStatsManager.endPlayerSession(player);

        // Сохраняем время игры
        playtimeManager.onPlayerQuit(player.getUniqueId())
            .thenRun(() -> {
                // Выгружаем данные из кеша
                userManager.unloadUser(player.getUniqueId());
                logger.info("Игрок " + player.getName() + " покинул сервер");
            })
            .exceptionally(throwable -> {
                logger.severe("Ошибка при выходе игрока " + player.getName() + ": " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Показать приветственное сообщение игроку
     */
    private void showWelcomeMessage(Player player, ChiwawaUser user) {
        plugin.getServer().getScheduler().runTaskLater(plugin, () -> {
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            player.sendMessage("§f§l            Добро пожаловать на §bChiwawaMine§f!");
            player.sendMessage("");
            player.sendMessage("§7Ваша роль: §a" + getRoleDisplayName(user.getRole()));
            player.sendMessage("§7Trust Level: §a" + user.getTrustLevel() + " §7(" + user.getTrustLevelDescription() + ")");

            if (user.hasTimeLimit()) {
                int remaining = playtimeManager.getRemainingTime(player.getUniqueId());
                player.sendMessage("§7Лимит времени: §c" + playtimeManager.formatTime(remaining) + " §7из 10 часов");
                player.sendMessage("§7Подтвердите email на сайте для снятия лимита!");
                player.sendMessage("§7Сайт: §b§nhttps://chiwawa.site");
            }

            player.sendMessage("");
            player.sendMessage("§7Полезные команды:");
            player.sendMessage("§7• §f/profile §7- Ваш профиль");
            player.sendMessage("§7• §f/playtime §7- Время игры");
            player.sendMessage("§7• §f/rep <игрок> §7- Дать репутацию");
            player.sendMessage("§7• §f/discord §7- Discord сервер");
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }, 20L); // Задержка 1 секунда
    }

    /**
     * Получить отображаемое название роли
     */
    private String getRoleDisplayName(String role) {
        switch (role) {
            case "admin": return "Администратор";
            case "moderator": return "Модератор";
            case "user": return "Пользователь";
            default: return "Неизвестная роль";
        }
    }
}
