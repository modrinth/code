package org.ebluffy.chiwawa.managers;

import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.api.dto.PlayerStats;

import java.util.HashMap;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Logger;

/**
 * Менеджер пользователей для кеширования и управления данными игроков
 */
public class UserManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;

    // Кеш пользователей по UUID
    private final Map<UUID, ChiwawaUser> userCache = new ConcurrentHashMap<>();
    // Кеш статистики по UUID
    private final Map<UUID, PlayerStats> statsCache = new ConcurrentHashMap<>();
    // Кеш соответствия UUID -> User ID
    private final Map<UUID, Integer> userIdCache = new ConcurrentHashMap<>();
    // Кеш соответствия никнейм -> UUID
    private final Map<String, UUID> nicknameCache = new ConcurrentHashMap<>();

    public UserManager(JavaPlugin plugin, ApiClient apiClient) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
    }

    /**
     * Загрузить данные пользователя при входе на сервер
     */
    public CompletableFuture<ChiwawaUser> loadUser(Player player) {
        UUID playerUuid = player.getUniqueId();
        String nickname = player.getName();

        // Проверяем кеш
        if (userCache.containsKey(playerUuid)) {
            return CompletableFuture.completedFuture(userCache.get(playerUuid));
        }

        return apiClient.getUserByNickname(nickname)
                .thenApply(user -> {
                    if (user != null) {
                        // Кешируем данные
                        userCache.put(playerUuid, user);
                        userIdCache.put(playerUuid, user.getId());
                        nicknameCache.put(nickname.toLowerCase(), playerUuid);

                        logger.info("Загружен пользователь: " + user.getNickname() + " (ID: " + user.getId() + ", Trust Level: " + user.getTrustLevel() + ")");

                        // Загружаем статистику асинхронно
                        loadPlayerStats(playerUuid, user.getId());
                    } else {
                        logger.warning("Пользователь не найден в базе данных: " + nickname);
                    }
                    return user;
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка загрузки пользователя " + nickname + ": " + throwable.getMessage());
                    return null;
                });
    }

    /**
     * Загрузить статистику игрока
     */
    public CompletableFuture<PlayerStats> loadPlayerStats(UUID playerUuid, int userId) {
        // Проверяем кеш
        if (statsCache.containsKey(playerUuid)) {
            return CompletableFuture.completedFuture(statsCache.get(playerUuid));
        }

        return apiClient.getPlayerStats(userId)
                .thenApply(stats -> {
                    if (stats != null) {
                        statsCache.put(playerUuid, stats);
                        logger.fine("Загружена статистика для игрока " + playerUuid + ": " + stats.getFormattedPlaytime());
                    }
                    return stats;
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка загрузки статистики игрока " + playerUuid + ": " + throwable.getMessage());
                    return null;
                });
    }

    /**
     * Получить пользователя из кеша
     */
    public ChiwawaUser getUser(UUID playerUuid) {
        return userCache.get(playerUuid);
    }

    /**
     * Получить пользователя по никнейму
     */
    public ChiwawaUser getUserByNickname(String nickname) {
        UUID uuid = nicknameCache.get(nickname.toLowerCase());
        return uuid != null ? userCache.get(uuid) : null;
    }

    /**
     * Получить статистику из кеша
     */
    public PlayerStats getPlayerStats(UUID playerUuid) {
        return statsCache.get(playerUuid);
    }

    /**
     * Получить User ID по UUID
     */
    public Integer getUserId(UUID playerUuid) {
        return userIdCache.get(playerUuid);
    }

    /**
     * Проверить доступ к серверу
     */
    public CompletableFuture<Boolean> checkServerAccess(Player player) {
        return loadUser(player)
                .thenCompose(user -> {
                    if (user == null) {
                        return CompletableFuture.completedFuture(false);
                    }

                    // Проверяем базовый доступ
                    if (!user.hasServerAccess()) {
                        return CompletableFuture.completedFuture(false);
                    }

                    // Дополнительная проверка через API
                    return apiClient.checkServerAccess(player.getName());
                });
    }

    /**
     * Обновить trust level пользователя
     */
    public CompletableFuture<Boolean> updateTrustLevel(UUID playerUuid, int newTrustLevel) {
        ChiwawaUser user = userCache.get(playerUuid);
        if (user == null) {
            return CompletableFuture.completedFuture(false);
        }

        // Используем метод по никнейму для поддержки оффлайн игроков
        return apiClient.updateTrustLevelByNickname(user.getNickname(), newTrustLevel)
                .thenApply(success -> {
                    if (success) {
                        user.setTrustLevel(newTrustLevel);
                        logger.info("Обновлен trust level для " + user.getNickname() + " до " + newTrustLevel);
                    }
                    return success;
                });
    }

    /**
     * Забанить игрока
     */
    public CompletableFuture<Boolean> banPlayer(UUID playerUuid, String reason) {
        ChiwawaUser user = userCache.get(playerUuid);
        if (user == null) {
            return CompletableFuture.completedFuture(false);
        }

        // Используем метод по никнейму для поддержки оффлайн игроков
        return apiClient.banPlayerByNickname(user.getNickname(), reason)
                .thenApply(success -> {
                    if (success) {
                        user.setBanned(true);
                        user.setBanReason(reason);
                        logger.info("Забанен игрок " + user.getNickname() + " по причине: " + reason);
                    }
                    return success;
                });
    }

    /**
     * Разбанить игрока
     */
    public CompletableFuture<Boolean> unbanPlayer(UUID playerUuid) {
        ChiwawaUser user = userCache.get(playerUuid);
        if (user == null) {
            return CompletableFuture.completedFuture(false);
        }

        // Используем метод по никнейму для поддержки оффлайн игроков
        return apiClient.unbanPlayerByNickname(user.getNickname())
                .thenApply(success -> {
                    if (success) {
                        user.setBanned(false);
                        user.setBanReason(null);
                        logger.info("Разбанен игрок " + user.getNickname());
                    }
                    return success;
                });
    }

    /**
     * Синхронизировать данные игрока с сервером
     */
    public CompletableFuture<Boolean> syncPlayer(UUID playerUuid) {
        ChiwawaUser user = userCache.get(playerUuid);
        if (user == null) {
            return CompletableFuture.completedFuture(false);
        }

        // Перезагружаем данные с сервера
        return apiClient.getUserByNickname(user.getNickname())
                .thenCompose(freshUser -> {
                    if (freshUser != null) {
                        userCache.put(playerUuid, freshUser);
                        logger.info("Синхронизированы данные игрока " + freshUser.getNickname());

                        // Также синхронизируем статистику
                        return loadPlayerStats(playerUuid, freshUser.getId())
                                .thenApply(stats -> true);
                    }
                    return CompletableFuture.completedFuture(false);
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка синхронизации игрока " + playerUuid + ": " + throwable.getMessage());
                    return false;
                });
    }

    /**
     * Очистить кеш игрока при выходе
     */
    public void unloadUser(UUID playerUuid) {
        ChiwawaUser user = userCache.remove(playerUuid);
        statsCache.remove(playerUuid);
        userIdCache.remove(playerUuid);

        if (user != null) {
            nicknameCache.remove(user.getNickname().toLowerCase());
            logger.fine("Выгружен из кеша игрок: " + user.getNickname());
        }
    }

    /**
     * Получить всех закешированных пользователей
     */
    public Map<UUID, ChiwawaUser> getAllCachedUsers() {
        return new HashMap<>(userCache);
    }

    /**
     * Очистить весь кеш
     */
    public void clearCache() {
        userCache.clear();
        statsCache.clear();
        userIdCache.clear();
        nicknameCache.clear();
        logger.info("Кеш пользователей очищен");
    }

    /**
     * Получить размер кеша
     */
    public int getCacheSize() {
        return userCache.size();
    }

    /**
     * Проверить, загружен ли пользователь
     */
    public boolean isUserLoaded(UUID playerUuid) {
        return userCache.containsKey(playerUuid);
    }

    /**
     * Записать активность игрока
     */
    public CompletableFuture<Boolean> recordActivity(UUID playerUuid, String activityType, String description, String metadata) {
        Integer userId = userIdCache.get(playerUuid);
        if (userId == null) {
            return CompletableFuture.completedFuture(false);
        }

        return apiClient.recordActivity(userId, activityType, description, metadata);
    }

    /**
     * Авторизовать игрока с помощью токена
     */
    public void authorizePlayer(Player player, int userId) {
        UUID playerUuid = player.getUniqueId();
        String nickname = player.getName();

        // Загружаем данные пользователя по ID
        apiClient.getUserByNickname(nickname)
            .thenAccept(user -> {
                if (user != null && user.getId() == userId) {
                    // Кешируем данные авторизованного пользователя
                    userCache.put(playerUuid, user);
                    userIdCache.put(playerUuid, user.getId());
                    nicknameCache.put(nickname.toLowerCase(), playerUuid);

                    // Загружаем статистику
                    loadPlayerStats(playerUuid, user.getId());

                    logger.info("Игрок " + nickname + " авторизован с токеном (ID: " + userId + ", Trust Level: " + user.getTrustLevel() + ")");
                } else {
                    logger.warning("Несоответствие данных при авторизации игрока " + nickname + " с токеном");
                }
            })
            .exceptionally(throwable -> {
                logger.severe("Ошибка авторизации игрока " + nickname + ": " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Проверить, находится ли игрок в кеше (авторизован)
     */
    public boolean isPlayerCached(UUID playerUuid) {
        return userCache.containsKey(playerUuid);
    }

    /**
     * Получить данные игрока из кеша
     */
    public ChiwawaUser getCachedUser(UUID playerUuid) {
        return userCache.get(playerUuid);
    }
}
