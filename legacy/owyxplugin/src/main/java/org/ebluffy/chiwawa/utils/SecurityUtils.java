package org.ebluffy.chiwawa.utils;

import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.dto.ChiwawaUser;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import javax.crypto.Cipher;
import javax.crypto.spec.SecretKeySpec;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SecureRandom;
import java.util.Base64;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicLong;
import java.util.regex.Pattern;

/**
 * Утилиты безопасности для валидации данных
 */
public class SecurityUtils {
    private static final Logger logger = LoggerFactory.getLogger(SecurityUtils.class);

    // Rate limiting
    private static final ConcurrentHashMap<String, AtomicLong> requestCounts = new ConcurrentHashMap<>();
    private static final ConcurrentHashMap<String, Long> lastRequestTime = new ConcurrentHashMap<>();

    // Константы для rate limiting
    private static final long RATE_LIMIT_WINDOW_MS = 60000; // 1 минута
    private static final int MAX_REQUESTS_PER_MINUTE = 60;

    // Паттерны для валидации
    private static final Pattern EMAIL_PATTERN = Pattern.compile(
        "^[A-Za-z0-9+_.-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}$"
    );

    private static final Pattern MINECRAFT_USERNAME_PATTERN = Pattern.compile(
        "^[a-zA-Z0-9_]{3,16}$"
    );

    private static final Pattern UUID_PATTERN = Pattern.compile(
        "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    );

    /**
     * Проверка rate limit для IP адреса
     */
    public static boolean checkRateLimit(String ipAddress) {
        long currentTime = System.currentTimeMillis();

        // Получаем или создаем счетчик для IP
        AtomicLong counter = requestCounts.computeIfAbsent(ipAddress, k -> new AtomicLong(0));
        Long lastTime = lastRequestTime.get(ipAddress);

        // Если прошла минута, сбрасываем счетчик
        if (lastTime == null || (currentTime - lastTime) > RATE_LIMIT_WINDOW_MS) {
            counter.set(1);
            lastRequestTime.put(ipAddress, currentTime);
            return true;
        }

        // Проверяем лимит
        long currentCount = counter.incrementAndGet();
        if (currentCount > MAX_REQUESTS_PER_MINUTE) {
            logger.warn("Rate limit exceeded for IP: {}", ipAddress);
            return false;
        }

        return true;
    }

    /**
     * Валидация email адреса
     */
    public static boolean isValidEmail(String email) {
        if (email == null || email.trim().isEmpty()) {
            return false;
        }
        return EMAIL_PATTERN.matcher(email.trim()).matches();
    }

    /**
     * Валидация UUID
     */
    public static boolean isValidUUID(String uuid) {
        if (uuid == null || uuid.trim().isEmpty()) {
            return false;
        }
        return UUID_PATTERN.matcher(uuid.trim()).matches();
    }

    /**
     * Валидация имени игрока Minecraft
     */
    public static boolean isValidMinecraftUsername(String username) {
        if (username == null || username.trim().isEmpty()) {
            return false;
        }
        return MINECRAFT_USERNAME_PATTERN.matcher(username.trim()).matches();
    }

    /**
     * Очистка чувствительных данных из строки для логирования
     */
    public static String sanitizeForLogging(String input) {
        if (input == null) {
            return "null";
        }

        // Удаляем потенциально опасные символы
        return input.replaceAll("[\\r\\n\\t]", "_")
                   .replaceAll("password", "***")
                   .replaceAll("token", "***")
                   .replaceAll("key", "***");
    }

    /**
     * Простое шифрование для конфигурационных данных
     */
    public static String encryptString(String plainText, String secretKey) {
        try {
            SecretKeySpec keySpec = new SecretKeySpec(secretKey.getBytes(), "AES");
            Cipher cipher = Cipher.getInstance("AES");
            cipher.init(Cipher.ENCRYPT_MODE, keySpec);

            byte[] encrypted = cipher.doFinal(plainText.getBytes());
            return Base64.getEncoder().encodeToString(encrypted);

        } catch (Exception e) {
            logger.error("Ошибка при шифровании: {}", e.getMessage());
            return plainText; // Возвращаем исходный текст в случае ошибки
        }
    }

    /**
     * Расшифровка строки
     */
    public static String decryptString(String encryptedText, String secretKey) {
        try {
            SecretKeySpec keySpec = new SecretKeySpec(secretKey.getBytes(), "AES");
            Cipher cipher = Cipher.getInstance("AES");
            cipher.init(Cipher.DECRYPT_MODE, keySpec);

            byte[] decoded = Base64.getDecoder().decode(encryptedText);
            byte[] decrypted = cipher.doFinal(decoded);
            return new String(decrypted);

        } catch (Exception e) {
            logger.error("Ошибка при расшифровке: {}", e.getMessage());
            return encryptedText; // Возвращаем исходный текст в случае ошибки
        }
    }

    /**
     * Генерация безопасного API ключа
     */
    public static String generateSecureApiKey() {
        SecureRandom random = new SecureRandom();
        byte[] bytes = new byte[32];
        random.nextBytes(bytes);
        return Base64.getUrlEncoder().withoutPadding().encodeToString(bytes);
    }

    /**
     * Валидация IP адреса
     */
    public static boolean isValidIpAddress(String ip) {
        if (ip == null) return false;

        String[] parts = ip.split("\\.");
        if (parts.length != 4) return false;

        try {
            for (String part : parts) {
                int num = Integer.parseInt(part);
                if (num < 0 || num > 255) return false;
            }
            return true;
        } catch (NumberFormatException e) {
            return false;
        }
    }

    /**
     * Проверка на подозрительную активность
     */
    public static boolean isSuspiciousActivity(String playerName, String action) {
        // Простые эвристики для детекции подозрительной активности
        if (playerName == null || action == null) return true;

        // Проверяем на слишком быстрые действия
        String key = playerName + ":" + action;
        Long lastActionTime = lastRequestTime.get(key);
        long currentTime = System.currentTimeMillis();

        if (lastActionTime != null && (currentTime - lastActionTime) < 1000) {
            logger.warn("Подозрительно быстрые действия от игрока: {}", playerName);
            return true;
        }

        lastRequestTime.put(key, currentTime);
        return false;
    }

    /**
     * Очистка старых записей rate limiting
     */
    public static void cleanupOldEntries() {
        long currentTime = System.currentTimeMillis();

        lastRequestTime.entrySet().removeIf(entry ->
            (currentTime - entry.getValue()) > RATE_LIMIT_WINDOW_MS * 2);

        // Также очищаем счетчики для удаленных IP
        requestCounts.entrySet().removeIf(entry ->
            !lastRequestTime.containsKey(entry.getKey()));
    }

    /**
     * Хеширование строки (для логирования или кеширования)
     */
    public static String hashString(String input) {
        if (input == null) return null;

        try {
            MessageDigest md = MessageDigest.getInstance("SHA-256");
            byte[] hash = md.digest(input.getBytes());
            StringBuilder hexString = new StringBuilder();

            for (byte b : hash) {
                String hex = Integer.toHexString(0xff & b);
                if (hex.length() == 1) {
                    hexString.append('0');
                }
                hexString.append(hex);
            }

            return hexString.toString();
        } catch (NoSuchAlgorithmException e) {
            throw new RuntimeException("SHA-256 algorithm not available", e);
        }
    }

    /**
     * Проверка прав доступа игрока
     */
    public static boolean hasPermission(Player player, ChiwawaUser user, String permission) {
        if (player == null || user == null) {
            return false;
        }

        // Операторы имеют все права
        if (player.isOp()) {
            return true;
        }

        // Проверка по trust level
        switch (permission) {
            case "admin":
                return user.isAdmin();
            case "moderator":
                return user.isModerator();
            case "user":
                return user.getTrustLevel() >= 1;
            default:
                return player.hasPermission(permission);
        }
    }

    /**
     * Санитизация текста для безопасного вывода в чат
     */
    public static String sanitizeText(String text) {
        if (text == null) return "";

        return text.trim()
                   .replaceAll("&[0-9a-fA-Fk-oK-OrR]", "") // Удаляем цветовые коды
                   .replaceAll("§[0-9a-fA-Fk-oK-OrR]", "") // Удаляем цветовые коды
                   .replaceAll("[\\r\\n\\t]", " ") // Заменяем переносы строк на пробелы
                   .replaceAll("\\s+", " "); // Убираем множественные пробелы
    }

    /**
     * Проверка на подозрительную активность (rate limiting)
     */
    public static boolean isRateLimited(UUID playerUuid, String action, long cooldownMs,
                                       java.util.Map<String, Long> lastActionTimes) {
        String key = playerUuid.toString() + ":" + action;
        Long lastTime = lastActionTimes.get(key);
        long currentTime = System.currentTimeMillis();

        if (lastTime == null || (currentTime - lastTime) >= cooldownMs) {
            lastActionTimes.put(key, currentTime);
            return false;
        }

        return true;
    }

    /**
     * Валидация Trust Level
     */
    public static boolean isValidTrustLevel(int trustLevel) {
        return trustLevel >= 0 && trustLevel <= 3;
    }

    /**
     * Валидация репутации (разумные пределы)
     */
    public static boolean isValidReputationChange(int change) {
        return change >= -100 && change <= 100;
    }
}
