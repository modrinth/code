package org.ebluffy.chiwawa.managers;

import org.bukkit.Bukkit;
import org.bukkit.NamespacedKey;
import org.bukkit.OfflinePlayer;
import org.bukkit.Statistic;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.scheduler.BukkitRunnable;
import org.ebluffy.chiwawa.api.ApiClient;

import java.time.LocalDateTime;
import java.time.LocalTime;
import java.time.format.DateTimeFormatter;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Logger;

/**
 * Менеджер для сбора и отправки расширенной статистики игроков
 */
public class PlayerStatsManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;
    
    // Хранение данных сессий игроков
    private final Map<UUID, PlayerSessionData> playerSessions = new ConcurrentHashMap<>();
    private final Map<UUID, List<SessionHistory>> playerSessionHistory = new ConcurrentHashMap<>();
    
    private BukkitRunnable statsUpdateTask;
    
    public PlayerStatsManager(JavaPlugin plugin, ApiClient apiClient) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
    }
    
    /**
     * Запуск автоматического обновления статистики
     */
    public void startStatsUpdates() {
        if (statsUpdateTask != null) {
            statsUpdateTask.cancel();
        }
        
        // Обновляем статистику каждые 5 минут
        statsUpdateTask = new BukkitRunnable() {
            @Override
            public void run() {
                for (Player player : Bukkit.getOnlinePlayers()) {
                    updatePlayerStats(player);
                }
            }
        };
        
        // Первое обновление через 1 минуту, затем каждые 5 минут
        statsUpdateTask.runTaskTimerAsynchronously(plugin, 1200L, 6000L); // 1min, 5min
    }
    
    /**
     * Остановка обновления статистики
     */
    public void stopStatsUpdates() {
        if (statsUpdateTask != null) {
            statsUpdateTask.cancel();
            statsUpdateTask = null;
        }
    }
    
    /**
     * Начало игровой сессии
     */
    public void startPlayerSession(Player player) {
        UUID playerId = player.getUniqueId();
        PlayerSessionData sessionData = new PlayerSessionData();
        sessionData.sessionStart = System.currentTimeMillis();
        sessionData.loginTime = LocalTime.now();
        sessionData.startStats = collectPlayerStats(player);
        
        playerSessions.put(playerId, sessionData);
        
        // Увеличиваем счетчик входов сразу при старте сессии
        incrementPlayerLogins(player);
        
        logger.fine("Начата сессия для игрока " + player.getName());
    }
    
    /**
     * Увеличение счетчика входов игрока
     */
    private void incrementPlayerLogins(Player player) {
        Map<String, Object> loginStats = new HashMap<>();
        loginStats.put("increment_login", true); // Флаг для инкремента входов
        loginStats.put("last_seen", LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME));
        
        // Отправляем обновление счетчика входов
        apiClient.updatePlayerStats(player.getName(), loginStats)
            .thenAccept(success -> {
                if (success) {
                    logger.fine("Счетчик входов обновлен для " + player.getName());
                } else {
                    logger.warning("Не удалось обновить счетчик входов для " + player.getName());
                }
            })
            .exceptionally(throwable -> {
                logger.warning("Ошибка обновления счетчика входов для " + player.getName() + ": " + throwable.getMessage());
                return null;
            });
    }
    
    /**
     * Завершение игровой сессии
     */
    public void endPlayerSession(Player player) {
        UUID playerId = player.getUniqueId();
        PlayerSessionData sessionData = playerSessions.remove(playerId);
        
        if (sessionData != null) {
            long sessionDuration = System.currentTimeMillis() - sessionData.sessionStart;
            
            // Сохраняем в историю сессий
            SessionHistory history = new SessionHistory();
            history.date = LocalDateTime.now();
            history.duration = sessionDuration;
            history.loginTime = sessionData.loginTime;
            
            playerSessionHistory.computeIfAbsent(playerId, k -> new ArrayList<>()).add(history);
            
            // Обновляем статистику на сайте
            updatePlayerStats(player);
            
            logger.fine("Завершена сессия для игрока " + player.getName() + 
                       " (длительность: " + (sessionDuration / 1000 / 60) + " минут)");
        }
    }
    
    /**
     * Обновление статистики игрока
     */
    public CompletableFuture<Boolean> updatePlayerStats(Player player) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                Map<String, Object> stats = new HashMap<>();
                
                // Базовые данные
                stats.put("minecraft_nick", player.getName());
                stats.put("last_seen", LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME));
                stats.put("last_ip_address", player.getAddress().getAddress().getHostAddress());
                
                // Время игры
                int totalPlaytimeMinutes = player.getStatistic(Statistic.PLAY_ONE_MINUTE) / 20 / 60; // тики -> минуты
                stats.put("time_played_minutes", totalPlaytimeMinutes);
                
                // Статистика сессий
                UUID playerId = player.getUniqueId();
                List<SessionHistory> sessions = playerSessionHistory.getOrDefault(playerId, new ArrayList<>());
                
                if (!sessions.isEmpty()) {
                    // Количество сессий (входов)
                    stats.put("session_count", sessions.size());
                    stats.put("total_logins", sessions.size()); // Исправлено: количество входов
                    
                    // Средняя длительность сессии (в минутах)
                    long totalSessionTime = sessions.stream()
                        .mapToLong(s -> s.duration)
                        .sum();
                    long averageSessionDuration = (totalSessionTime / sessions.size()) / 1000 / 60;
                    stats.put("average_session_duration", averageSessionDuration);
                    
                    // Самая длинная сессия (в минутах)
                    long longestSession = sessions.stream()
                        .mapToLong(s -> s.duration)
                        .max()
                        .orElse(0L) / 1000 / 60;
                    stats.put("longest_session_duration", longestSession);
                    
                    // Активные дни за последние 30 дней
                    LocalDateTime thirtyDaysAgo = LocalDateTime.now().minusDays(30);
                    long activeDays = sessions.stream()
                        .filter(s -> s.date.isAfter(thirtyDaysAgo))
                        .map(s -> s.date.toLocalDate())
                        .distinct()
                        .count();
                    stats.put("active_days_last_month", (int)activeDays);
                    
                    // Общее время игры за последние 30 дней (в минутах)
                    long totalTimeLastMonth = sessions.stream()
                        .filter(s -> s.date.isAfter(thirtyDaysAgo))
                        .mapToLong(s -> s.duration)
                        .sum() / 1000 / 60; // миллисекунды -> минуты
                    stats.put("time_played_last_month", (int)totalTimeLastMonth);
                    
                    // Любимое время игры (час дня, когда чаще всего заходит)
                    Map<Integer, Long> hourFrequency = new HashMap<>();
                    sessions.stream()
                        .filter(s -> s.date.isAfter(thirtyDaysAgo))
                        .forEach(s -> {
                            int hour = s.loginTime.getHour();
                            hourFrequency.put(hour, hourFrequency.getOrDefault(hour, 0L) + 1);
                        });
                    
                    if (!hourFrequency.isEmpty()) {
                        int favoriteHour = hourFrequency.entrySet().stream()
                            .max(Map.Entry.comparingByValue())
                            .map(Map.Entry::getKey)
                            .orElse(12);
                        stats.put("favorite_play_hour", favoriteHour);
                    }
                } else {
                    // Значения по умолчанию если нет истории сессий
                    stats.put("session_count", 1);
                    stats.put("total_logins", 1);
                    stats.put("average_session_duration", 0);
                    stats.put("longest_session_duration", 0);
                    stats.put("active_days_last_month", 1);
                    stats.put("time_played_last_month", 0); // Нет истории сессий = 0 времени за 30 дней
                    stats.put("favorite_play_hour", LocalDateTime.now().getHour());
                }
                
                // Убираем достижения полностью - слишком сложно реализовать корректно
                // Достижения в Minecraft очень сложны для правильного подсчета через API
                
                // Упрощенная статистика - только необходимое
                stats.put("deaths_count", player.getStatistic(Statistic.DEATHS));
                stats.put("mobs_killed", player.getStatistic(Statistic.MOB_KILLS));
                
                // Отправляем статистику
                return apiClient.updatePlayerStats(player.getName(), stats).join();
                
            } catch (Exception e) {
                logger.severe("Ошибка обновления статистики игрока " + player.getName() + ": " + e.getMessage());
                return false;
            }
        });
    }
    
    /**
     * Получение статистики игрока для команд
     */
    public String getPlayerStatsString(Player player) {
        try {
            UUID playerId = player.getUniqueId();
            List<SessionHistory> sessions = playerSessionHistory.getOrDefault(playerId, new ArrayList<>());
            
            int totalPlaytime = player.getStatistic(Statistic.PLAY_ONE_MINUTE) / 20 / 60; // минуты
            int achievements = Math.max(0, totalPlaytime / 60); // примерно
            
            long averageSession = 0;
            long longestSession = 0;
            int activeDays = 0;
            
            if (!sessions.isEmpty()) {
                long totalSessionTime = sessions.stream().mapToLong(s -> s.duration).sum();
                averageSession = (totalSessionTime / sessions.size()) / 1000 / 60;
                longestSession = sessions.stream().mapToLong(s -> s.duration).max().orElse(0L) / 1000 / 60;
                
                LocalDateTime thirtyDaysAgo = LocalDateTime.now().minusDays(30);
                activeDays = (int) sessions.stream()
                    .filter(s -> s.date.isAfter(thirtyDaysAgo))
                    .map(s -> s.date.toLocalDate())
                    .distinct()
                    .count();
            }
            
            return String.format(
                "§e§l════════ СТАТИСТИКА ИГРОКА ════════\n" +
                "§7Игрок: §f%s\n" +
                "§7Общее время игры: §b%d часов %d минут\n" +
                "§7Сессий сыграно: §a%d\n" +
                "§7Средняя сессия: §e%d минут\n" +
                "§7Самая длинная сессия: §6%d минут\n" +
                "§7Активных дней (30д): §a%d\n" +
                "§7Достижений: §d%d\n" +
                "§7Блоков сломано: §c%d\n" +
                "§7Мобов убито: §4%d\n" +
                "§7Пройдено метров: §b%.1f км\n" +
                "§e§l══════════════════════════════════",
                player.getName(),
                totalPlaytime / 60, totalPlaytime % 60,
                sessions.size(),
                averageSession, longestSession,
                activeDays, achievements,
                player.getStatistic(Statistic.MINE_BLOCK),
                player.getStatistic(Statistic.MOB_KILLS),
                player.getStatistic(Statistic.WALK_ONE_CM) / 100000.0 // км
            );
        } catch (Exception e) {
            return "§cОшибка получения статистики: " + e.getMessage();
        }
    }
    
    /**
     * Сбор базовой статистики игрока
     */
    private Map<String, Integer> collectPlayerStats(Player player) {
        Map<String, Integer> stats = new HashMap<>();
        
        try {
            // Безопасный сбор статистики с обработкой ошибок
            
            // Убитые мобы - простая статистика
            stats.put("mobs_killed", player.getStatistic(Statistic.MOB_KILLS));
            
            // Пройденное расстояние - простая статистика  
            stats.put("distance_walked", player.getStatistic(Statistic.WALK_ONE_CM));
            
            // Время игры - простая статистика
            stats.put("play_time", player.getStatistic(Statistic.PLAY_ONE_MINUTE));
            
            // Прыжки - простая статистика
            stats.put("jumps", player.getStatistic(Statistic.JUMP));
            
            // Урон нанесенный - простая статистика  
            stats.put("damage_dealt", player.getStatistic(Statistic.DAMAGE_DEALT));
            
            // Урон получен - простая статистика
            stats.put("damage_taken", player.getStatistic(Statistic.DAMAGE_TAKEN));
            
            // Количество смертей - простая статистика
            stats.put("deaths", player.getStatistic(Statistic.DEATHS));
            
            // Количество достижений (упрощенный подсчет)
            int achievements = 0;
            try {
                // Пытаемся получить основные достижения
                if (player.hasPlayedBefore()) {
                    achievements = 1; // Базовое достижение за подключение
                }
                // Можно добавить другие простые проверки достижений
            } catch (Exception e) {
                logger.fine("Не удалось получить информацию о достижениях для " + player.getName());
            }
            stats.put("achievements", achievements);
            
        } catch (Exception e) {
            logger.warning("Ошибка при сборе статистики игрока " + player.getName() + ": " + e.getMessage());
        }
        
        return stats;
    }
    
    /**
     * Данные игровой сессии
     */
    private static class PlayerSessionData {
        long sessionStart;
        LocalTime loginTime;
        Map<String, Integer> startStats;
    }
    
    /**
     * История сессий игрока
     */
    private static class SessionHistory {
        LocalDateTime date;
        long duration; // в миллисекундах
        LocalTime loginTime;
    }
}
