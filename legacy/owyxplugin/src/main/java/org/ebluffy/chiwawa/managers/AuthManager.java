package org.ebluffy.chiwawa.managers;

import at.favre.lib.crypto.bcrypt.BCrypt;
import org.bukkit.entity.Player;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.dto.AuthPlayer;
import org.ebluffy.chiwawa.database.DatabaseManager;

import java.security.SecureRandom;
import java.util.Map;
import java.util.Optional;
import java.util.UUID;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentHashMap;
import java.util.logging.Logger;

/**
 * Менеджер для управления простой авторизацией игроков
 */
public class AuthManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final DatabaseManager databaseManager;
    
    // Кеш авторизованных игроков (UUID -> AuthPlayer)
    private final Map<UUID, AuthPlayer> authorizedPlayers = new ConcurrentHashMap<>();
    
    // Кеш кодов подтверждения для смены режима (UUID -> код)
    private final Map<UUID, String> pendingVerifications = new ConcurrentHashMap<>();
    
    // BCrypt параметры
    private static final int BCRYPT_COST = 12;
    
    public AuthManager(JavaPlugin plugin, DatabaseManager databaseManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.databaseManager = databaseManager;
    }

    /**
     * Зарегистрировать игрока с паролем
     */
    public CompletableFuture<Boolean> registerPlayer(Player player, String password) {
        String username = player.getName();
        String uuid = player.getUniqueId().toString();
        String ipAddress = player.getAddress().getAddress().getHostAddress();
        
        // Проверяем, не зарегистрирован ли игрок
        return databaseManager.getPlayerByUsername(username)
            .thenCompose(existingPlayer -> {
                if (existingPlayer.isPresent()) {
                    logger.warning("Попытка регистрации уже существующего игрока: " + username);
                    return CompletableFuture.completedFuture(false);
                }
                
                // Хешируем пароль
                String passwordHash = hashPassword(password);
                
                // Регистрируем в БД
                return databaseManager.registerPlayer(username, passwordHash, uuid, ipAddress)
                    .thenCompose(success -> {
                        if (success) {
                            // Загружаем данные игрока
                            return databaseManager.getPlayerByUsername(username)
                                .thenApply(authPlayer -> {
                                    authPlayer.ifPresent(ap -> {
                                        authorizedPlayers.put(player.getUniqueId(), ap);
                                        // Логирование убрано - уже в RegisterCommand
                                    });
                                    return authPlayer.isPresent();
                                });
                        }
                        return CompletableFuture.completedFuture(false);
                    });
            });
    }

    /**
     * Авторизовать игрока с паролем
     */
    public CompletableFuture<Boolean> loginPlayer(Player player, String password) {
        String username = player.getName();
        String ipAddress = player.getAddress().getAddress().getHostAddress();
        
        // Сначала проверяем блокировку
        return databaseManager.isPlayerLocked(username)
            .thenCompose(isLocked -> {
                if (isLocked) {
                    // Получаем время до разблокировки
                    return databaseManager.getMinutesUntilUnlock(username)
                        .thenApply(minutesRemaining -> {
                            logger.warning("Попытка входа заблокированного игрока: " + username + 
                                          " (осталось " + minutesRemaining + " минут)");
                            
                            // Логируем попытку входа заблокированного игрока
                            logAuthAttempt(player, false, "Аккаунт временно заблокирован (" + minutesRemaining + " мин)");
                            
                            // Возвращаем -1 как признак блокировки
                            return -1;
                        })
                        .thenCompose(result -> CompletableFuture.completedFuture(false));
                }
                
                // Продолжаем обычную авторизацию
                return databaseManager.getPlayerByUsername(username)
                    .thenCompose(authPlayerOpt -> {
                        if (authPlayerOpt.isEmpty()) {
                            logger.warning("Попытка входа незарегистрированного игрока: " + username);
                            logAuthAttempt(player, false, "Игрок не зарегистрирован");
                            return CompletableFuture.completedFuture(false);
                        }
                        
                        AuthPlayer authPlayer = authPlayerOpt.get();
                        
                        // Проверяем тип авторизации
                        if (authPlayer.isWebsiteAuth()) {
                            logger.warning("Игрок " + username + " использует авторизацию через сайт");
                            logAuthAttempt(player, false, "Использует авторизацию через сайт");
                            return CompletableFuture.completedFuture(false);
                        }
                        
                        // Проверяем пароль
                        if (!verifyPassword(password, authPlayer.getPasswordHash())) {
                            logger.warning("Неверный пароль для игрока: " + username);
                            
                            // Увеличиваем счетчик неудачных попыток
                            databaseManager.incrementFailedAttempts(username);
                            
                            // Проверяем количество попыток
                            return databaseManager.getFailedLoginAttempts(username)
                                .thenApply(attempts -> {
                                    if (attempts >= 5) {
                                        logger.warning("Игрок " + username + " превысил лимит попыток входа (5)");
                                        logAuthAttempt(player, false, "Неверный пароль (попытка " + attempts + "/5) - заблокирован на 5 минут");
                                    } else {
                                        logAuthAttempt(player, false, "Неверный пароль (попытка " + attempts + "/5)");
                                    }
                                    return false;
                                });
                        }
                        
                        // Пароль верный - сбрасываем счетчик попыток
                        databaseManager.resetFailedAttempts(username);
                        
                        // Обновляем последний вход
                        return databaseManager.updateLastLogin(username, ipAddress)
                            .thenApply(success -> {
                                if (success) {
                                    authorizedPlayers.put(player.getUniqueId(), authPlayer);
                                    logger.info("Игрок " + username + " успешно авторизовался");
                                    logAuthAttempt(player, true, "Успешная авторизация");
                                    return true;
                                }
                                return false;
                            });
                    });
            });
    }

    /**
     * Изменить пароль игрока
     */
    public CompletableFuture<Boolean> changePassword(Player player, String oldPassword, String newPassword) {
        String username = player.getName();
        
        return databaseManager.getPlayerByUsername(username)
            .thenCompose(authPlayerOpt -> {
                if (authPlayerOpt.isEmpty()) {
                    return CompletableFuture.completedFuture(false);
                }
                
                AuthPlayer authPlayer = authPlayerOpt.get();
                
                // Проверяем старый пароль
                if (!verifyPassword(oldPassword, authPlayer.getPasswordHash())) {
                    logger.warning("Неверный старый пароль при смене для игрока: " + username);
                    return CompletableFuture.completedFuture(false);
                }
                
                // Хешируем новый пароль
                String newPasswordHash = hashPassword(newPassword);
                
                // Обновляем в БД
                return databaseManager.changePassword(username, newPasswordHash)
                    .thenApply(success -> {
                        if (success) {
                            logger.info("Пароль игрока " + username + " успешно изменен");
                        }
                        return success;
                    });
            });
    }

    /**
     * Начать процесс смены режима авторизации на сайт
     */
    public CompletableFuture<String> requestAuthModeChange(Player player) {
        String username = player.getName();
        
        // Получаем email из таблицы users
        return databaseManager.getEmailByNickname(username)
            .thenCompose(emailOpt -> {
                if (emailOpt.isEmpty()) {
                    logger.warning("Email не найден для игрока " + username + " в таблице users");
                    return CompletableFuture.completedFuture("EMAIL_NOT_FOUND");
                }
                
                String email = emailOpt.get();
                
                // Генерируем код подтверждения
                String verificationCode = generateVerificationCode();
                
                // Сохраняем код в БД
                return databaseManager.createVerificationCode(username, verificationCode)
                    .thenCompose(success -> {
                        if (!success) {
                            return CompletableFuture.completedFuture("DB_ERROR");
                        }
                        
                        // Сохраняем код в кеше
                        pendingVerifications.put(player.getUniqueId(), verificationCode);
                        
                        // Отправляем email через API
                        return sendVerificationEmail(username, email, verificationCode)
                            .thenApply(emailSent -> {
                                if (emailSent) {
                                    logger.info("Код подтверждения отправлен на email для " + username);
                                    return "SUCCESS:" + email;
                                } else {
                                    logger.warning("Не удалось отправить email для " + username);
                                    return "EMAIL_SEND_ERROR";
                                }
                            });
                    });
            });
    }

    /**
     * Подтвердить смену режима авторизации
     */
    public CompletableFuture<Boolean> confirmAuthModeChange(Player player, String code) {
        String username = player.getName();
        
        return databaseManager.verifyAndChangeLoginType(username, code, 1)
            .thenApply(success -> {
                if (success) {
                    pendingVerifications.remove(player.getUniqueId());
                    
                    // Обновляем кеш
                    AuthPlayer authPlayer = authorizedPlayers.get(player.getUniqueId());
                    if (authPlayer != null) {
                        authPlayer.setLoginType(1);
                    }
                    
                    logger.info("Игрок " + username + " успешно переключился на авторизацию через сайт");
                    return true;
                }
                
                logger.warning("Неверный или истекший код подтверждения для " + username);
                return false;
            });
    }

    /**
     * Проверить, авторизован ли игрок
     */
    public boolean isPlayerAuthorized(UUID playerUuid) {
        return authorizedPlayers.containsKey(playerUuid);
    }

    /**
     * Получить данные авторизованного игрока
     */
    public Optional<AuthPlayer> getAuthorizedPlayer(UUID playerUuid) {
        return Optional.ofNullable(authorizedPlayers.get(playerUuid));
    }

    /**
     * Выход игрока (очистка кеша)
     */
    public void logoutPlayer(UUID playerUuid) {
        AuthPlayer removed = authorizedPlayers.remove(playerUuid);
        pendingVerifications.remove(playerUuid);
        
        if (removed != null) {
            logger.info("Игрок " + removed.getUsername() + " вышел из системы");
        }
    }

    /**
     * Очистить все кеши
     */
    public void clearCaches() {
        authorizedPlayers.clear();
        pendingVerifications.clear();
        logger.info("Кеши авторизации очищены");
    }

    /**
     * Хешировать пароль с использованием BCrypt
     */
    private String hashPassword(String password) {
        return BCrypt.withDefaults().hashToString(BCRYPT_COST, password.toCharArray());
    }

    /**
     * Проверить пароль
     */
    private boolean verifyPassword(String password, String passwordHash) {
        BCrypt.Result result = BCrypt.verifyer().verify(password.toCharArray(), passwordHash);
        return result.verified;
    }

    /**
     * Генерировать случайный код подтверждения
     */
    private String generateVerificationCode() {
        SecureRandom random = new SecureRandom();
        int code = 100000 + random.nextInt(900000); // 6-значный код
        return String.valueOf(code);
    }

    /**
     * Отправить email с кодом подтверждения через API сайта
     */
    private CompletableFuture<Boolean> sendVerificationEmail(String username, String email, String code) {
        return CompletableFuture.supplyAsync(() -> {
            try {
                // Получаем API клиент из главного плагина
                org.ebluffy.chiwawa.ChiwawaPlugin plugin = 
                    (org.ebluffy.chiwawa.ChiwawaPlugin) org.bukkit.Bukkit.getPluginManager().getPlugin("ChiwawaPlugin");
                
                if (plugin == null) {
                    logger.severe("ChiwawaPlugin не найден!");
                    return false;
                }
                
                org.ebluffy.chiwawa.api.ApiClient apiClient = plugin.getApiClient();
                
                // Отправляем запрос на отправку email через API
                com.google.gson.JsonObject requestBody = new com.google.gson.JsonObject();
                requestBody.addProperty("email", email);
                requestBody.addProperty("username", username);
                requestBody.addProperty("code", code);
                requestBody.addProperty("type", "login_change");
                requestBody.addProperty("subject", "Подтверждение смены режима авторизации");
                
                // Формируем текст письма
                String messageText = String.format(
                    "Здравствуйте, %s!\n\n" +
                    "Вы запросили смену режима авторизации на сервере ChiwawaMine.\n\n" +
                    "Ваш код подтверждения: %s\n\n" +
                    "Используйте команду в игре: /changelogin confirm %s\n\n" +
                    "Код действителен 15 минут.\n\n" +
                    "Если вы не запрашивали смену режима, проигнорируйте это письмо.\n\n" +
                    "С уважением,\n" +
                    "Команда ChiwawaMine",
                    username, code, code
                );
                
                requestBody.addProperty("message", messageText);
                
                // Используем существующий метод API для отправки через backend
                Boolean result = apiClient.sendCustomEmail(email, 
                    "Подтверждение смены режима авторизации", 
                    messageText).get();
                
                if (result) {
                    logger.info("Код подтверждения успешно отправлен на email " + email + " для игрока " + username);
                } else {
                    logger.warning("Не удалось отправить код подтверждения на email " + email);
                }
                
                return result;
                
            } catch (Exception e) {
                logger.severe("Ошибка отправки email: " + e.getMessage());
                e.printStackTrace();
                return false;
            }
        });
    }

    /**
     * Получить количество авторизованных игроков
     */
    public int getAuthorizedPlayersCount() {
        return authorizedPlayers.size();
    }

    /**
     * Логирование попыток авторизации в отдельный файл
     */
    private void logAuthAttempt(Player player, boolean success, String reason) {
        String logMessage = String.format(
            "[%s] Player: %s, UUID: %s, IP: %s, Success: %s, Reason: %s, Time: %s",
            success ? "AUTH_SUCCESS" : "AUTH_FAILED",
            player.getName(),
            player.getUniqueId(),
            player.getAddress().getAddress().getHostAddress(),
            success,
            reason,
            new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss").format(new java.util.Date())
        );
        
        // Используем отдельный логгер для аудита авторизаций
        java.util.logging.Logger authLogger = java.util.logging.Logger.getLogger("ChiwawaAuth");
        
        try {
            // Создаем обработчик для записи в отдельный файл auth.log
            java.util.logging.FileHandler fileHandler = new java.util.logging.FileHandler(
                "plugins/ChiwawaPlugin/logs/auth.log", true);
            fileHandler.setFormatter(new java.util.logging.SimpleFormatter() {
                private static final String format = "%1$s%n";
                
                @Override
                public synchronized String format(java.util.logging.LogRecord lr) {
                    return String.format(format, lr.getMessage());
                }
            });
            
            authLogger.addHandler(fileHandler);
            authLogger.setUseParentHandlers(false);
            
            if (success) {
                authLogger.info(logMessage);
            } else {
                authLogger.warning(logMessage);
            }
            
            fileHandler.close();
            
        } catch (Exception e) {
            logger.warning("Не удалось записать лог авторизации: " + e.getMessage());
        }
    }

    /**
     * Валидация пароля
     */
    public static boolean isPasswordValid(String password) {
        if (password == null || password.length() < 6) {
            return false;
        }
        if (password.length() > 32) {
            return false;
        }
        // Проверка на запрещенные символы
        return password.matches("^[a-zA-Z0-9!@#$%^&*()_+\\-=\\[\\]{}|;:,.<>?]+$");
    }

    /**
     * Получить силу пароля
     */
    public static String getPasswordStrength(String password) {
        if (password == null || password.length() < 6) {
            return "Слабый";
        }
        
        int strength = 0;
        
        // Длина
        if (password.length() >= 8) strength++;
        if (password.length() >= 12) strength++;
        
        // Содержит цифры
        if (password.matches(".*\\d.*")) strength++;
        
        // Содержит заглавные буквы
        if (password.matches(".*[A-Z].*")) strength++;
        
        // Содержит специальные символы
        if (password.matches(".*[!@#$%^&*()_+\\-=\\[\\]{}|;:,.<>?].*")) strength++;
        
        if (strength <= 1) return "Слабый";
        if (strength <= 3) return "Средний";
        return "Сильный";
    }
}
