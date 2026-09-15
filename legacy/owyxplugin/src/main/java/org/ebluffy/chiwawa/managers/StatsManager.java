package org.ebluffy.chiwawa.managers;

import org.bukkit.Bukkit;
import org.bukkit.Statistic;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.scheduler.BukkitTask;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.config.ConfigManager;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Logger;

/**
 * Менеджер для сбора и отправки детальной статистики игроков на сайт
 */
public class StatsManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;
    private final UserManager userManager;
    private final ConfigManager configManager;

    // Кеш последней отправленной статистики для каждого игрока
    private final Map<UUID, Map<String, Integer>> lastSentStats = new ConcurrentHashMap<>();
    
    // Время последней отправки статистики
    private final Map<UUID, Long> lastStatsUpdate = new ConcurrentHashMap<>();

    // Задача для периодической отправки статистики
    private BukkitTask statsTask;

    // Интервал отправки статистики (1 минута = 1200 тиков)
    private static final int STATS_UPDATE_INTERVAL = 1200;
    
    // Минимальный интервал между отправками для одного игрока (30 секунд)
    private static final long MIN_UPDATE_INTERVAL = 30 * 1000;

    public StatsManager(JavaPlugin plugin, ApiClient apiClient, UserManager userManager, ConfigManager configManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
        this.userManager = userManager;
        this.configManager = configManager;
    }

    /**
     * Запуск менеджера статистики
     */
    public void start() {
        // Запускаем периодическую отправку статистики
        statsTask = Bukkit.getScheduler().runTaskTimerAsynchronously(plugin, 
            this::updateAllPlayersStats, 
            STATS_UPDATE_INTERVAL, 
            STATS_UPDATE_INTERVAL
        );
    }

    /**
     * Остановка менеджера статистики
     */
    public void stop() {
        if (statsTask != null) {
            statsTask.cancel();
            statsTask = null;
        }
        
        // Отправляем финальную статистику для всех онлайн игроков
        for (Player player : Bukkit.getOnlinePlayers()) {
            updatePlayerStats(player, true);
        }
    }

    /**
     * Регистрация входа игрока
     */
    public void onPlayerJoin(UUID playerUuid) {
        lastStatsUpdate.put(playerUuid, System.currentTimeMillis());
        logger.fine("Зарегистрирован вход игрока для статистики: " + playerUuid);
    }

    /**
     * Регистрация выхода игрока
     */
    public void onPlayerQuit(UUID playerUuid) {
        Player player = Bukkit.getPlayer(playerUuid);
        if (player != null) {
            // Отправляем финальную статистику
            updatePlayerStats(player, true);
        }
        
        // Очищаем кеш
        lastSentStats.remove(playerUuid);
        lastStatsUpdate.remove(playerUuid);
        
        logger.fine("Зарегистрирован выход игрока из статистики: " + playerUuid);
    }

    /**
     * Обновление статистики всех онлайн игроков
     */
    private void updateAllPlayersStats() {
        for (Player player : Bukkit.getOnlinePlayers()) {
            updatePlayerStats(player, false);
        }
    }

    /**
     * Обновление статистики конкретного игрока
     */
    public void updatePlayerStats(Player player, boolean forceUpdate) {
        UUID playerUuid = player.getUniqueId();
        
        // Проверяем минимальный интервал
        Long lastUpdate = lastStatsUpdate.get(playerUuid);
        long currentTime = System.currentTimeMillis();
        
        if (!forceUpdate && lastUpdate != null && 
            (currentTime - lastUpdate) < MIN_UPDATE_INTERVAL) {
            return; // Слишком рано для обновления
        }

        // Проверяем что игрок авторизован
        ChiwawaUser user = userManager.getCachedUser(playerUuid);
        if (user == null) {
            return; // Игрок не авторизован
        }

        try {
            // Собираем статистику
            Map<String, Object> stats = collectPlayerStats(player);
            
            // Проверяем нужно ли отправлять (изменились ли данные значительно)
            if (!forceUpdate && !shouldUpdateStats(playerUuid, stats)) {
                return;
            }

            // Отправляем статистику на сервер
            sendStatsToServer(player.getName(), stats);
            
            // Обновляем кеш
            updateStatsCache(playerUuid, stats);
            lastStatsUpdate.put(playerUuid, currentTime);
            
            logger.fine("Статистика игрока " + player.getName() + " отправлена на сервер");
            
        } catch (Exception e) {
            logger.warning("Ошибка обновления статистики игрока " + player.getName() + ": " + e.getMessage());
        }
    }

    /**
     * Сбор статистики игрока
     */
    private Map<String, Object> collectPlayerStats(Player player) {
        Map<String, Object> stats = new HashMap<>();
        
        try {
            // Основная статистика времени
            stats.put("total_logins", getPlayerStatistic(player, Statistic.LEAVE_GAME) + 1); // +1 для текущей сессии
            
            // Блоки
            stats.put("blocks_broken", getPlayerStatistic(player, Statistic.MINE_BLOCK));
            stats.put("blocks_placed", getPlayerStatistic(player, Statistic.USE_ITEM));
            
            // Движение
            stats.put("distance_walked", 
                getPlayerStatistic(player, Statistic.WALK_ONE_CM) + 
                getPlayerStatistic(player, Statistic.SPRINT_ONE_CM) +
                getPlayerStatistic(player, Statistic.CROUCH_ONE_CM)
            );
            
            // Бой и смерти
            stats.put("deaths_count", getPlayerStatistic(player, Statistic.DEATHS));
            stats.put("mobs_killed", getPlayerStatistic(player, Statistic.MOB_KILLS));
            stats.put("damage_dealt", getPlayerStatistic(player, Statistic.DAMAGE_DEALT));
            stats.put("damage_taken", getPlayerStatistic(player, Statistic.DAMAGE_TAKEN));
            
            // Крафт и еда
            stats.put("items_crafted", getPlayerStatistic(player, Statistic.CRAFT_ITEM));
            stats.put("food_eaten", getPlayerStatistic(player, Statistic.ANIMALS_BRED)); // Приблизительно
            
            // Прыжки
            stats.put("jumps_count", getPlayerStatistic(player, Statistic.JUMP));
            
            // Время последнего входа (в правильном формате для PostgreSQL)
            stats.put("last_seen", java.time.LocalDateTime.now().format(java.time.format.DateTimeFormatter.ISO_LOCAL_DATE_TIME));
            
            // IP адрес
            if (player.getAddress() != null) {
                stats.put("last_ip_address", player.getAddress().getAddress().getHostAddress());
            }
            
            // Детальная статистика в JSON формате
            Map<String, Object> minecraftStats = new HashMap<>();
            minecraftStats.put("play_time_ticks", getPlayerStatistic(player, Statistic.PLAY_ONE_MINUTE));
            minecraftStats.put("sneak_time", getPlayerStatistic(player, Statistic.SNEAK_TIME));
            minecraftStats.put("time_since_death", getPlayerStatistic(player, Statistic.TIME_SINCE_DEATH));
            minecraftStats.put("player_kills", getPlayerStatistic(player, Statistic.PLAYER_KILLS));
            
            stats.put("minecraft_stats", minecraftStats);
            
            // Ежедневная статистика (для сегодняшнего дня)
            Map<String, Object> dailyStats = new HashMap<>();
            dailyStats.put("playtime_minutes", 5); // Будет обновлено PlaytimeManager
            dailyStats.put("logins_count", 1);
            dailyStats.put("blocks_broken", 0); // Разница от последней отправки
            dailyStats.put("blocks_placed", 0);
            dailyStats.put("distance_walked", 0);
            dailyStats.put("deaths_count", 0);
            dailyStats.put("mobs_killed", 0);
            
            stats.put("daily_stats", dailyStats);
            
        } catch (Exception e) {
            logger.warning("Ошибка сбора статистики для " + player.getName() + ": " + e.getMessage());
        }
        
        return stats;
    }

    /**
     * Получить статистику игрока безопасно
     */
    private int getPlayerStatistic(Player player, Statistic statistic) {
        try {
            return player.getStatistic(statistic);
        } catch (Exception e) {
            return 0;
        }
    }

    /**
     * Проверка нужно ли отправлять статистику (изменилась ли она значительно)
     */
    private boolean shouldUpdateStats(UUID playerUuid, Map<String, Object> currentStats) {
        Map<String, Integer> lastStats = lastSentStats.get(playerUuid);
        if (lastStats == null) {
            return true; // Первая отправка
        }
        
        // Проверяем значительные изменения
        String[] significantStats = {
            "blocks_broken", "blocks_placed", "distance_walked", 
            "deaths_count", "mobs_killed", "total_logins"
        };
        
        for (String statName : significantStats) {
            Integer current = (Integer) currentStats.get(statName);
            Integer last = lastStats.get(statName);
            
            if (current == null) current = 0;
            if (last == null) last = 0;
            
            // Если изменение больше 10 или прошло много времени
            if (Math.abs(current - last) > 10) {
                return true;
            }
        }
        
        return false;
    }

    /**
     * Обновление кеша статистики
     */
    private void updateStatsCache(UUID playerUuid, Map<String, Object> stats) {
        Map<String, Integer> intStats = new HashMap<>();
        
        for (Map.Entry<String, Object> entry : stats.entrySet()) {
            if (entry.getValue() instanceof Integer) {
                intStats.put(entry.getKey(), (Integer) entry.getValue());
            }
        }
        
        lastSentStats.put(playerUuid, intStats);
    }

    /**
     * Отправка статистики на сервер
     */
    private void sendStatsToServer(String playerName, Map<String, Object> stats) {
        // Используем ApiClient для отправки статистики
        apiClient.updatePlayerStats(playerName, stats)
            .thenAccept(success -> {
                if (success) {
                    logger.fine("Статистика игрока " + playerName + " успешно отправлена");
                } else {
                    logger.warning("Не удалось отправить статистику игрока " + playerName);
                }
            })
            .exceptionally(throwable -> {
                logger.severe("Ошибка отправки статистики для " + playerName + ": " + throwable.getMessage());
                return null;
            });
    }
}
