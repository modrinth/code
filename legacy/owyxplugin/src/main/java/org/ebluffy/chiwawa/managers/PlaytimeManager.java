package org.ebluffy.chiwawa.managers;

import org.bukkit.Bukkit;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.scheduler.BukkitTask;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.api.dto.PlayerStats;
import org.ebluffy.chiwawa.config.ConfigManager;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.CompletableFuture;
import java.util.logging.Logger;

/**
 * Менеджер времени игры для отслеживания лимитов и обновления статистики
 */
public class PlaytimeManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;
    private final UserManager userManager;
    private final ConfigManager configManager;

    // Время входа игроков
    private final Map<UUID, Long> loginTimes = new ConcurrentHashMap<>();
    // Накопленное время текущей сессии (в минутах)
    private final Map<UUID, Integer> sessionPlaytime = new ConcurrentHashMap<>();
    // Последнее сохранение времени
    private final Map<UUID, Long> lastSaveTime = new ConcurrentHashMap<>();

    // Задача для периодического обновления
    private BukkitTask updateTask;

    // Интервал обновления в тиках (5 минут = 6000 тиков)
    private static final int UPDATE_INTERVAL = 6000;
    // Интервал сохранения в минутах
    private static final int SAVE_INTERVAL = 5;

    public PlaytimeManager(JavaPlugin plugin, ApiClient apiClient, UserManager userManager, ConfigManager configManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
        this.userManager = userManager;
        this.configManager = configManager;
    }

    /**
     * Запустить менеджер времени игры
     */
    public void start() {
        updateTask = Bukkit.getScheduler().runTaskTimerAsynchronously(plugin, this::updatePlaytimes, UPDATE_INTERVAL, UPDATE_INTERVAL);
    }

    /**
     * Остановить менеджер времени игры
     */
    public void stop() {
        if (updateTask != null) {
            updateTask.cancel();
            updateTask = null;
        }

        // Сохраняем время всех онлайн игроков
        for (Player player : Bukkit.getOnlinePlayers()) {
            savePlaytime(player.getUniqueId()).join();
        }
    }

    /**
     * Зарегистрировать вход игрока
     */
    public void onPlayerJoin(UUID playerUuid) {
        long currentTime = System.currentTimeMillis();
        loginTimes.put(playerUuid, currentTime);
        sessionPlaytime.put(playerUuid, 0);
        lastSaveTime.put(playerUuid, currentTime);

        // Записываем активность входа
        userManager.recordActivity(playerUuid, "join_server", "Игрок зашел на сервер", null);

        logger.fine("Зарегистрирован вход игрока: " + playerUuid);
    }

    /**
     * Зарегистрировать выход игрока
     */
    public CompletableFuture<Void> onPlayerQuit(UUID playerUuid) {
        return savePlaytime(playerUuid)
                .thenRun(() -> {
                    // Записываем активность выхода
                    int sessionMinutes = sessionPlaytime.getOrDefault(playerUuid, 0);
                    String metadata = "{\"session_minutes\": " + sessionMinutes + "}";
                    userManager.recordActivity(playerUuid, "leave_server", "Игрок вышел с сервера", metadata);

                    // Очищаем данные
                    loginTimes.remove(playerUuid);
                    sessionPlaytime.remove(playerUuid);
                    lastSaveTime.remove(playerUuid);

                    logger.fine("Зарегистрирован выход игрока: " + playerUuid + " (сессия: " + sessionMinutes + " мин)");
                });
    }

    /**
     * Периодическое обновление времени игры
     */
    private void updatePlaytimes() {
        long currentTime = System.currentTimeMillis();

        for (Player player : Bukkit.getOnlinePlayers()) {
            UUID playerUuid = player.getUniqueId();
            updatePlayerPlaytime(playerUuid, currentTime);

            // Проверяем лимит времени для проходимцев
            checkTimeLimit(player);
        }
    }

    /**
     * Обновить время игры конкретного игрока
     */
    private void updatePlayerPlaytime(UUID playerUuid, long currentTime) {
        Long loginTime = loginTimes.get(playerUuid);
        Long lastSave = lastSaveTime.get(playerUuid);

        if (loginTime == null || lastSave == null) {
            return;
        }

        // Вычисляем прошедшее время с последнего сохранения
        long elapsedMillis = currentTime - lastSave;
        int elapsedMinutes = (int) (elapsedMillis / 60000); // Конвертируем в минуты

        if (elapsedMinutes > 0) {
            // Добавляем к времени сессии
            sessionPlaytime.merge(playerUuid, elapsedMinutes, Integer::sum);
            lastSaveTime.put(playerUuid, currentTime);

            // Сохраняем каждые SAVE_INTERVAL минут
            int totalSessionMinutes = sessionPlaytime.getOrDefault(playerUuid, 0);
            if (totalSessionMinutes >= SAVE_INTERVAL) {
                savePlaytime(playerUuid);
            }
        }
    }

    /**
     * Проверить лимит времени для проходимцев
     */
    private void checkTimeLimit(Player player) {
        UUID playerUuid = player.getUniqueId();
        ChiwawaUser user = userManager.getUser(playerUuid);
        PlayerStats stats = userManager.getPlayerStats(playerUuid);

        if (user == null || stats == null) {
            return;
        }

        // Проверяем только для проходимцев (trust level 0) без подтвержденного email
        if (!user.hasTimeLimit()) {
            return;
        }

        // Вычисляем общее время с учетом текущей сессии
        int currentSessionMinutes = sessionPlaytime.getOrDefault(playerUuid, 0);
        int totalMinutes = stats.getTimePlayedMinutes() + currentSessionMinutes;

        // Лимит 10 часов = 600 минут
        if (totalMinutes >= 600) {
            // Кикаем игрока с сообщением о превышении лимита
            Bukkit.getScheduler().runTask(plugin, () -> {
                String kickMessage = configManager.getMessage("time_limit_kick")
                        .replace("%hours%", "10")
                        .replace("%remaining%", "0");

                player.kickPlayer(kickMessage);
                logger.info("Игрок " + player.getName() + " исключен за превышение лимита времени (10 часов)");
            });

            // Записываем активность
            userManager.recordActivity(playerUuid, "time_limit_exceeded",
                "Игрок исключен за превышение лимита времени",
                "{\"total_minutes\": " + totalMinutes + ", \"limit_minutes\": 600}");
        }
    }

    /**
     * Сохранить время игры на сервер
     */
    public CompletableFuture<Boolean> savePlaytime(UUID playerUuid) {
        Integer userId = userManager.getUserId(playerUuid);
        PlayerStats stats = userManager.getPlayerStats(playerUuid);
        Integer sessionMinutes = sessionPlaytime.get(playerUuid);

        if (userId == null || stats == null || sessionMinutes == null || sessionMinutes == 0) {
            return CompletableFuture.completedFuture(false);
        }

        int newTotalMinutes = stats.getTimePlayedMinutes() + sessionMinutes;

        return apiClient.updatePlaytime(userId, newTotalMinutes)
                .thenApply(success -> {
                    if (success) {
                        // Обновляем кеш
                        stats.setTimePlayedMinutes(newTotalMinutes);
                        sessionPlaytime.put(playerUuid, 0); // Сбрасываем накопленное время сессии

                        logger.fine("Сохранено время игры для игрока " + playerUuid + ": " + newTotalMinutes + " минут");

                        // Записываем активность обновления времени
                        String metadata = "{\"session_minutes\": " + sessionMinutes + ", \"total_minutes\": " + newTotalMinutes + "}";
                        userManager.recordActivity(playerUuid, "playtime_update", "Обновлено время игры", metadata);
                    } else {
                        logger.warning("Не удалось сохранить время игры для игрока " + playerUuid);
                    }
                    return success;
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка сохранения времени игры для " + playerUuid + ": " + throwable.getMessage());
                    return false;
                });
    }

    /**
     * Получить время текущей сессии
     */
    public int getSessionPlaytime(UUID playerUuid) {
        Long loginTime = loginTimes.get(playerUuid);
        if (loginTime == null) {
            return 0;
        }

        long currentTime = System.currentTimeMillis();
        long sessionMillis = currentTime - loginTime;
        int sessionMinutes = (int) (sessionMillis / 60000);

        return sessionMinutes + sessionPlaytime.getOrDefault(playerUuid, 0);
    }

    /**
     * Получить общее время игры (сохраненное + текущая сессия)
     */
    public int getTotalPlaytime(UUID playerUuid) {
        PlayerStats stats = userManager.getPlayerStats(playerUuid);
        if (stats == null) {
            return getSessionPlaytime(playerUuid);
        }

        return stats.getTimePlayedMinutes() + getSessionPlaytime(playerUuid);
    }

    /**
     * Получить оставшееся время для проходимцев
     */
    public int getRemainingTime(UUID playerUuid) {
        ChiwawaUser user = userManager.getUser(playerUuid);
        if (user == null || !user.hasTimeLimit()) {
            return -1; // Нет лимита
        }

        int totalMinutes = getTotalPlaytime(playerUuid);
        return Math.max(0, 600 - totalMinutes); // 600 минут = 10 часов
    }

    /**
     * Форматировать время в читаемый вид
     */
    public String formatTime(int minutes) {
        if (minutes < 60) {
            return minutes + " мин";
        }

        int hours = minutes / 60;
        int remainingMinutes = minutes % 60;

        if (hours < 24) {
            return hours + "ч " + remainingMinutes + "м";
        }

        int days = hours / 24;
        int remainingHours = hours % 24;
        return days + "д " + remainingHours + "ч " + remainingMinutes + "м";
    }

    /**
     * Принудительно сохранить время всех онлайн игроков
     */
    public CompletableFuture<Void> saveAllPlaytimes() {
        CompletableFuture<?>[] futures = Bukkit.getOnlinePlayers().stream()
                .map(player -> savePlaytime(player.getUniqueId()))
                .toArray(CompletableFuture[]::new);

        return CompletableFuture.allOf(futures)
                .thenRun(() -> logger.info("Сохранено время игры для всех онлайн игроков"));
    }

    /**
     * Получить статистику по времени игры
     */
    public String getPlaytimeStats() {
        int onlinePlayers = Bukkit.getOnlinePlayers().size();
        int trackedPlayers = loginTimes.size();

        return String.format("Онлайн игроков: %d, Отслеживается: %d", onlinePlayers, trackedPlayers);
    }

    /**
     * Проверить, находится ли игрок онлайн и отслеживается
     */
    public boolean isPlayerTracked(UUID playerUuid) {
        return loginTimes.containsKey(playerUuid);
    }
}
