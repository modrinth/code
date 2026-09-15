package org.ebluffy.chiwawa.config;

import org.bukkit.ChatColor;
import org.bukkit.configuration.file.FileConfiguration;
import org.bukkit.plugin.java.JavaPlugin;

import java.util.logging.Logger;

/**
 * Менеджер конфигурации плагина
 */
public class ConfigManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private FileConfiguration config;

    public ConfigManager(JavaPlugin plugin) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        loadConfig();
    }

    /**
     * Загрузить конфигурацию
     */
    public void loadConfig() {
        plugin.saveDefaultConfig();
        plugin.reloadConfig();
        config = plugin.getConfig();
        // Логирование убрано - выводится в главном классе
    }

    /**
     * Перезагрузить конфигурацию
     */
    public void reloadConfig() {
        plugin.reloadConfig();
        config = plugin.getConfig();
        logger.info("Конфигурация перезагружена");
    }

    // API настройки
    public String getApiBaseUrl() {
        return config.getString("api.base_url", "http://localhost:3000/api");
    }

    public int getApiTimeout() {
        return config.getInt("api.timeout", 5000);
    }

    public String getApiKey() {
        return config.getString("api.admin_token", "");
    }

    public int getRetryAttempts() {
        return config.getInt("api.retry_attempts", 3);
    }

    // Настройки сервера
    public boolean isWhitelistEnabled() {
        return config.getBoolean("server.enable_whitelist", true);
    }

    public boolean isTimeLimitEnabled() {
        return config.getBoolean("server.enable_time_limit", true);
    }

    public boolean isTrustSystemEnabled() {
        return config.getBoolean("server.enable_trust_system", true);
    }

    public boolean isReputationEnabled() {
        return config.getBoolean("server.enable_reputation", true);
    }
    
    public boolean isAuthenticationEnabled() {
        return config.getBoolean("server.enable_authentication", true);
    }

    public int getSyncInterval() {
        return config.getInt("server.sync_interval", 300);
    }

    // Trust Level настройки
    public int getTrustLevelMaxHours(int trustLevel) {
        return config.getInt("trust_levels." + trustLevel + ".max_total_hours", -1);
    }

    public String getTrustLevelDescription(int trustLevel) {
        return config.getString("trust_levels." + trustLevel + ".description", "Неизвестный уровень");
    }

    // Сообщения
    public String getMessage(String key) {
        String message = config.getString("messages." + key, "Сообщение не найдено: " + key);
        return ChatColor.translateAlternateColorCodes('&', message);
    }

    public String getMessage(String key, String... replacements) {
        String message = getMessage(key);
        for (int i = 0; i < replacements.length; i += 2) {
            if (i + 1 < replacements.length) {
                message = message.replace(replacements[i], replacements[i + 1]);
            }
        }
        return message;
    }

    // Проверка корректности конфигурации
    public boolean validateConfig() {
        boolean valid = true;

        if (getApiKey().isEmpty()) {
            logger.severe("Admin token не настроен! Плагин не будет работать корректно.");
            valid = false;
        }

        if (getApiBaseUrl().isEmpty()) {
            logger.severe("API base URL не настроен!");
            valid = false;
        }

        return valid;
    }

    /**
     * Получить полную конфигурацию
     */
    public FileConfiguration getConfig() {
        return config;
    }
    
    /**
     * Получить IP сервера из конфигурации
     */
    public String getServerIp() {
        return config.getString("server.ip", "localhost");
    }
    
    /**
     * Получить порт сервера из конфигурации
     */
    public int getServerPort() {
        return config.getInt("server.port", 25565);
    }
}
