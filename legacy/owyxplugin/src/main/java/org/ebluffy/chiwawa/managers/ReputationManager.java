package org.ebluffy.chiwawa.managers;

import org.bukkit.Bukkit;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.ebluffy.chiwawa.config.ConfigManager;

import java.util.Map;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Logger;

/**
 * Менеджер репутации для системы +rep/-rep
 */
public class ReputationManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;
    private final UserManager userManager;
    private final ConfigManager configManager;

    // Кулдауны на выдачу репутации (игрок -> время последней выдачи)
    private final Map<UUID, Long> reputationCooldowns = new ConcurrentHashMap<>();

    // Кулдаун на выдачу репутации (1 час)
    private static final long REPUTATION_COOLDOWN = 60 * 60 * 1000; // 1 час в миллисекундах

    public ReputationManager(JavaPlugin plugin, ApiClient apiClient, UserManager userManager, ConfigManager configManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
        this.userManager = userManager;
        this.configManager = configManager;
    }

    /**
     * Дать репутацию игроку
     */
    public CompletableFuture<ReputationResult> giveReputation(Player giver, String targetNickname) {
        UUID giverUuid = giver.getUniqueId();

        // Проверяем кулдаун
        if (isOnCooldown(giverUuid)) {
            long remainingTime = getRemainingCooldown(giverUuid);
            return CompletableFuture.completedFuture(
                ReputationResult.failure("Вы сможете дать репутацию через " + formatTime(remainingTime))
            );
        }

        // Нельзя давать репутацию самому себе
        if (giver.getName().equalsIgnoreCase(targetNickname)) {
            return CompletableFuture.completedFuture(
                ReputationResult.failure("Нельзя давать репутацию самому себе")
            );
        }

        // Проверяем, что дающий имеет достаточный trust level
        ChiwawaUser giverUser = userManager.getUser(giverUuid);
        if (giverUser == null || giverUser.getTrustLevel() < 1) {
            return CompletableFuture.completedFuture(
                ReputationResult.failure("Для выдачи репутации нужен Trust Level 1 или выше")
            );
        }

        // Сначала ищем игрока в кеше (если онлайн)
        ChiwawaUser targetUser = userManager.getUserByNickname(targetNickname);
        if (targetUser != null) {
            // Игрок найден в кеше - выдаем репутацию
            return giveReputationToUser(giver, targetUser);
        }

        // Игрок не в кеше - ищем через API
        return apiClient.getUserByNickname(targetNickname)
                .thenCompose(foundUser -> {
                    if (foundUser == null) {
                        return CompletableFuture.completedFuture(
                            ReputationResult.failure("Игрок " + targetNickname + " не найден")
                        );
                    }
                    
                    // Игрок найден через API - выдаем репутацию
                    return giveReputationToUser(giver, foundUser);
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка поиска игрока " + targetNickname + ": " + throwable.getMessage());
                    return ReputationResult.failure("Ошибка при поиске игрока. Попробуйте позже");
                });
    }

    /**
     * Выдать репутацию найденному пользователю
     */
    private CompletableFuture<ReputationResult> giveReputationToUser(Player giver, ChiwawaUser targetUser) {
        UUID giverUuid = giver.getUniqueId();
        String reason = "Репутация от игрока " + giver.getName();
        
        return apiClient.updateReputation(targetUser.getId(), 1, reason)
                .thenApply(success -> {
                    if (success) {
                        // Устанавливаем кулдаун
                        reputationCooldowns.put(giverUuid, System.currentTimeMillis());

                        // Записываем активность
                        // String metadata = String.format("{\"target_id\": %d, \"target_nickname\": \"%s\", \"reputation_change\": 1}",
                        //         targetUser.getId(), targetUser.getNickname());
                        // userManager.recordActivity(giverUuid, "reputation_given",
                        //         "Дал репутацию игроку " + targetUser.getNickname(), metadata);

                        logger.info(giver.getName() + " дал репутацию игроку " + targetUser.getNickname());

                        // Уведомляем получателя если он онлайн
                        Player targetPlayer = Bukkit.getPlayerExact(targetUser.getNickname());
                        if (targetPlayer != null) {
                            String message = configManager.getMessage("reputation_received")
                                    .replace("%player%", giver.getName());
                            targetPlayer.sendMessage(message);
                        }

                        return ReputationResult.success("Вы дали +1 репутацию игроку " + targetUser.getNickname());
                    } else {
                        return ReputationResult.failure("Ошибка при выдаче репутации. Попробуйте позже");
                    }
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка выдачи репутации от " + giver.getName() + " к " + targetUser.getNickname() + ": " + throwable.getMessage());
                    return ReputationResult.failure("Ошибка при выдаче репутации. Попробуйте позже");
                });
    }

    /**
     * Изменить репутацию игрока (для администраторов)
     */
    public CompletableFuture<ReputationResult> modifyReputation(Player admin, String targetNickname, int amount, String reason) {
        // Проверяем права администратора
        ChiwawaUser adminUser = userManager.getUser(admin.getUniqueId());
        if (adminUser == null || !adminUser.isModerator()) {
            return CompletableFuture.completedFuture(
                ReputationResult.failure("У вас нет прав для изменения репутации")
            );
        }

        // Сначала ищем игрока в кеше (если онлайн)
        ChiwawaUser targetUser = userManager.getUserByNickname(targetNickname);
        if (targetUser != null) {
            // Игрок найден в кеше - изменяем репутацию
            return modifyReputationForUser(admin, targetUser, amount, reason);
        }

        // Игрок не в кеше - ищем через API
        return apiClient.getUserByNickname(targetNickname)
                .thenCompose(foundUser -> {
                    if (foundUser == null) {
                        return CompletableFuture.completedFuture(
                            ReputationResult.failure("Игрок " + targetNickname + " не найден")
                        );
                    }
                    
                    // Игрок найден через API - изменяем репутацию
                    return modifyReputationForUser(admin, foundUser, amount, reason);
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка поиска игрока " + targetNickname + ": " + throwable.getMessage());
                    return ReputationResult.failure("Ошибка при поиске игрока. Попробуйте позже");
                });
    }

    /**
     * Изменить репутацию найденному пользователю
     */
    private CompletableFuture<ReputationResult> modifyReputationForUser(Player admin, ChiwawaUser targetUser, int amount, String reason) {
        String fullReason = reason != null ? reason : ("Изменение репутации администратором " + admin.getName());

        return apiClient.updateReputation(targetUser.getId(), amount, fullReason)
                .thenApply(success -> {
                    if (success) {
                        // Записываем активность
                        // String metadata = String.format("{\"target_id\": %d, \"target_nickname\": \"%s\", \"reputation_change\": %d, \"reason\": \"%s\"}",
                        //         targetUser.getId(), targetUser.getNickname(), amount, fullReason);
                        // userManager.recordActivity(admin.getUniqueId(), "reputation_modified",
                        //         "Изменил репутацию игрока " + targetUser.getNickname() + " на " + amount, metadata);

                        logger.info(admin.getName() + " изменил репутацию игрока " + targetUser.getNickname() + " на " + amount + " (" + fullReason + ")");

                        String sign = amount > 0 ? "+" : "";
                        return ReputationResult.success("Репутация игрока " + targetUser.getNickname() + " изменена на " + sign + amount);
                    } else {
                        return ReputationResult.failure("Ошибка при изменении репутации. Попробуйте позже");
                    }
                })
                .exceptionally(throwable -> {
                    logger.severe("Ошибка изменения репутации администратором " + admin.getName() + ": " + throwable.getMessage());
                    return ReputationResult.failure("Ошибка при изменении репутации. Попробуйте позже");
                });
    }

    /**
     * Проверить, может ли игрок дать репутацию (кулдаун)
     */
    public boolean canGiveReputation(UUID playerUuid) {
        return !isOnCooldown(playerUuid);
    }

    /**
     * Проверить, находится ли игрок на кулдауне
     */
    private boolean isOnCooldown(UUID playerUuid) {
        Long lastTime = reputationCooldowns.get(playerUuid);
        if (lastTime == null) {
            return false;
        }

        return (System.currentTimeMillis() - lastTime) < REPUTATION_COOLDOWN;
    }

    /**
     * Получить оставшееся время кулдауна в миллисекундах
     */
    public long getRemainingCooldown(UUID playerUuid) {
        Long lastTime = reputationCooldowns.get(playerUuid);
        if (lastTime == null) {
            return 0;
        }

        long elapsed = System.currentTimeMillis() - lastTime;
        return Math.max(0, REPUTATION_COOLDOWN - elapsed);
    }

    /**
     * Форматировать время в читаемый вид
     */
    private String formatTime(long milliseconds) {
        long seconds = milliseconds / 1000;
        long minutes = seconds / 60;
        long hours = minutes / 60;

        if (hours > 0) {
            return hours + "ч " + (minutes % 60) + "м";
        } else if (minutes > 0) {
            return minutes + "м " + (seconds % 60) + "с";
        } else {
            return seconds + "с";
        }
    }

    /**
     * Очистить кулдауны при выходе игрока (опционально)
     */
    public void onPlayerQuit(UUID playerUuid) {
        // Оставляем кулдауны даже после выхода игрока
        // reputationCooldowns.remove(playerUuid);
    }

    /**
     * Получить статистику репутации
     */
    public String getReputationStats() {
        long activePlayerCount = reputationCooldowns.entrySet().stream()
                .filter(entry -> (System.currentTimeMillis() - entry.getValue()) < REPUTATION_COOLDOWN)
                .count();

        return String.format("Игроков на кулдауне репутации: %d", activePlayerCount);
    }

    /**
     * Результат операции с репутацией
     */
    public static class ReputationResult {
        private final boolean success;
        private final String message;

        private ReputationResult(boolean success, String message) {
            this.success = success;
            this.message = message;
        }

        public static ReputationResult success(String message) {
            return new ReputationResult(true, message);
        }

        public static ReputationResult failure(String message) {
            return new ReputationResult(false, message);
        }

        public boolean isSuccess() {
            return success;
        }

        public String getMessage() {
            return message;
        }
    }
}
