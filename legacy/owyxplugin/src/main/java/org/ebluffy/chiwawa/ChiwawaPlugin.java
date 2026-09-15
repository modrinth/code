package org.ebluffy.chiwawa;

import org.bukkit.Bukkit;
import org.bukkit.plugin.PluginManager;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.commands.*;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.database.DatabaseManager;
import org.ebluffy.chiwawa.listeners.AuthenticationListener;
import org.ebluffy.chiwawa.listeners.PlayerListener;
import org.ebluffy.chiwawa.managers.*;

import java.util.logging.Logger;

/**
 * Главный класс плагина ChiwawaPlugin для интеграции с сайтом ChiwawaMine
 */
public class ChiwawaPlugin extends JavaPlugin {
    private static ChiwawaPlugin instance;

    // Менеджеры
    private ConfigManager configManager;
    private ApiClient apiClient;
    private DatabaseManager databaseManager;
    private UserManager userManager;
    private AuthManager authManager;
    private PlaytimeManager playtimeManager;
    private ReputationManager reputationManager;
    private StatsManager statsManager;
    private ServerStatsManager serverStatsManager;
    private PlayerStatsManager playerStatsManager;

    // Логгер
    private Logger logger;

    @Override
    public void onEnable() {
        instance = this;
        logger = getLogger();

        try {
            logger.info("════════════════════════════════════");
            logger.info("   ChiwawaPlugin v1.0.0 - Запуск");
            logger.info("════════════════════════════════════");

            boolean hasErrors = false;
            StringBuilder errorMessages = new StringBuilder();

            // 1. Загрузка конфигурации
            configManager = new ConfigManager(this);
            if (!configManager.validateConfig()) {
                logger.severe("❌ Конфигурация содержит критические ошибки!");
                getServer().getPluginManager().disablePlugin(this);
                return;
            }

            // 2. Инициализация API клиента
            apiClient = new ApiClient(this,
                configManager.getApiBaseUrl(),
                configManager.getApiKey(),
                configManager.getApiTimeout(),
                configManager.getRetryAttempts());

            // 3. Инициализация базы данных
            String dbHost = getConfig().getString("database.host", "127.0.0.1");
            int dbPort = getConfig().getInt("database.port", 5432);
            String dbName = getConfig().getString("database.name", "chiwawa");
            String dbUser = getConfig().getString("database.username", "postgres");
            String dbPassword = getConfig().getString("database.password", "");
            
            databaseManager = new DatabaseManager(this, dbHost, dbPort, dbName, dbUser, dbPassword);
            
            if (!databaseManager.testConnection()) {
                hasErrors = true;
                errorMessages.append("  • База данных: подключение не установлено\n");
            }

            // 4. Инициализация менеджеров
            userManager = new UserManager(this, apiClient);
            authManager = new AuthManager(this, databaseManager);
            playtimeManager = new PlaytimeManager(this, apiClient, userManager, configManager);
            reputationManager = new ReputationManager(this, apiClient, userManager, configManager);
            statsManager = new StatsManager(this, apiClient, userManager, configManager);
            serverStatsManager = new ServerStatsManager(this, apiClient, configManager);
            playerStatsManager = new PlayerStatsManager(this, apiClient);

            // 5. Регистрация команд и слушателей
            registerCommands();
            registerListeners();

            // 6. Запуск менеджеров
            startManagers();

            // 7. Вывод итоговой информации
            logger.info("────────────────────────────────────");
            if (hasErrors) {
                logger.warning("⚠ Плагин запущен с ошибками:");
                logger.warning(errorMessages.toString());
            } else {
                logger.info("✓ Плагин успешно запущен!");
            }
            
            // Активные функции
            logger.info("Активные функции:");
            logger.info("  • Whitelist (заявки): " + (configManager.isWhitelistEnabled() ? "✓" : "✗"));
            logger.info("  • Авторизация (login/register): " + (configManager.isAuthenticationEnabled() ? "✓" : "✗"));
            logger.info("  • Лимит времени: " + (configManager.isTimeLimitEnabled() ? "✓" : "✗"));
            logger.info("  • Trust система: " + (configManager.isTrustSystemEnabled() ? "✓" : "✗"));
            logger.info("  • Репутация: " + (configManager.isReputationEnabled() ? "✓" : "✗"));
            logger.info("════════════════════════════════════");

        } catch (Exception e) {
            logger.severe("════════════════════════════════════");
            logger.severe("❌ КРИТИЧЕСКАЯ ОШИБКА ЗАПУСКА");
            logger.severe("════════════════════════════════════");
            logger.severe("Ошибка: " + e.getMessage());
            e.printStackTrace();
            getServer().getPluginManager().disablePlugin(this);
        }
    }

    @Override
    public void onDisable() {
        try {
            logger.info("Отключение ChiwawaPlugin...");

            // Остановка менеджеров
            if (playtimeManager != null) {
                playtimeManager.stop();
            }
            
            if (statsManager != null) {
                statsManager.stop();
            }
            
            if (serverStatsManager != null) {
                serverStatsManager.stopStatsCollection();
            }
            
            if (playerStatsManager != null) {
                playerStatsManager.stopStatsUpdates();
            }

            // Сохранение данных всех онлайн игроков
            if (playtimeManager != null && userManager != null) {
                playtimeManager.saveAllPlaytimes().join();
                logger.info("Сохранено время игры всех игроков");
            }

            // Очистка кешей
            if (userManager != null) {
                userManager.clearCache();
            }
            
            if (authManager != null) {
                authManager.clearCaches();
            }
            
            // Закрытие подключения к БД
            if (databaseManager != null) {
                databaseManager.close();
            }

            logger.info("ChiwawaPlugin отключен!");

        } catch (Exception e) {
            logger.severe("Ошибка при отключении плагина: " + e.getMessage());
            e.printStackTrace();
        } finally {
            instance = null;
        }
    }

    /**
     * Регистрация команд плагина
     */
    private void registerCommands() {
        // Главная админ команда
        ChiwawaCommand chiwawaCommand = new ChiwawaCommand(this, configManager, userManager, playtimeManager, reputationManager, apiClient);
        getCommand("chiwawa").setExecutor(chiwawaCommand);
        getCommand("chiwawa").setTabCompleter(chiwawaCommand);

        // Команды для игроков
        getCommand("profile").setExecutor(new ProfileCommand(configManager, userManager, playtimeManager));
        getCommand("playtime").setExecutor(new PlaytimeCommand(configManager, playtimeManager, userManager));
        getCommand("applications").setExecutor(new ApplicationsCommand(configManager, userManager));
        getCommand("discord").setExecutor(new DiscordCommand(configManager));
        
        // Команды авторизации (обновленные)
        getCommand("login").setExecutor(new LoginCommand(configManager, apiClient, userManager, authManager, logger));
        getCommand("register").setExecutor(new RegisterCommand(configManager, authManager, logger));
        getCommand("changelogin").setExecutor(new ChangeLoginCommand(configManager, authManager, logger));
        getCommand("changepass").setExecutor(new ChangePasswordCommand(configManager, authManager, logger));
        getCommand("unregister").setExecutor(new UnregisterCommand(configManager, databaseManager, logger));

        ReputationCommand repCommand = new ReputationCommand(configManager, reputationManager, userManager);
        getCommand("rep").setExecutor(repCommand);
        getCommand("rep").setTabCompleter(repCommand);

        // Логирование убрано
    }

    /**
     * Регистрация слушателей событий
     */
    private void registerListeners() {
        getServer().getPluginManager().registerEvents(
            new PlayerListener(this, configManager, userManager, playtimeManager, playerStatsManager), this);
        
        getServer().getPluginManager().registerEvents(
            new AuthenticationListener(this, userManager), this);

        // Логирование убрано
    }

    /**
     * Запуск менеджеров
     */
    private void startManagers() {
        if (configManager.isTimeLimitEnabled()) {
            playtimeManager.start();
        }
        
        statsManager.start();
        serverStatsManager.startStatsCollection();
        playerStatsManager.startStatsUpdates();
        
        // Логирование убрано - выводится общая информация
    }

    /**
     * Тест соединения с API (не используется - убрано из onEnable)
     */
    private void testApiConnection() {
        // Метод оставлен для возможного использования в будущем
    }

    /**
     * Перезагрузить плагин
     */
    public void reloadPlugin() {
        logger.info("Перезагрузка плагина...");

        // Остановка менеджеров
        if (playtimeManager != null) {
            playtimeManager.stop();
        }

        // Перезагрузка конфигурации
        configManager.reloadConfig();

        // Перезапуск менеджеров
        if (configManager.isTimeLimitEnabled()) {
            playtimeManager.start();
        }

        logger.info("Плагин перезагружен!");
    }

    // Геттеры для доступа к менеджерам
    public static ChiwawaPlugin getInstance() {
        return instance;
    }

    public ConfigManager getConfigManager() {
        return configManager;
    }

    public ApiClient getApiClient() {
        return apiClient;
    }
    
    public StatsManager getStatsManager() {
        return statsManager;
    }

    public UserManager getUserManager() {
        return userManager;
    }

    public PlaytimeManager getPlaytimeManager() {
        return playtimeManager;
    }

    public ReputationManager getReputationManager() {
        return reputationManager;
    }
    
    public AuthManager getAuthManager() {
        return authManager;
    }
    
    public DatabaseManager getDatabaseManager() {
        return databaseManager;
    }
}
