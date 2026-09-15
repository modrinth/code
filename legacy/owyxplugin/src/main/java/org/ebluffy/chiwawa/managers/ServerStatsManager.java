package org.ebluffy.chiwawa.managers;

import org.bukkit.Bukkit;
import org.bukkit.plugin.java.JavaPlugin;
import org.bukkit.scheduler.BukkitRunnable;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.config.ConfigManager;

import java.lang.management.ManagementFactory;
import java.util.concurrent.CompletableFuture;
import java.util.logging.Logger;

/**
 * Менеджер для сбора и отправки статистики сервера
 */
public class ServerStatsManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private final ApiClient apiClient;
    private final ConfigManager configManager;
    private BukkitRunnable statsTask;
    
    public ServerStatsManager(JavaPlugin plugin, ApiClient apiClient, ConfigManager configManager) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        this.apiClient = apiClient;
        this.configManager = configManager;
    }
    
    /**
     * Запуск автоматической отправки статистики сервера
     */
    public void startStatsCollection() {
        if (statsTask != null) {
            statsTask.cancel();
        }
        
        // Отправляем статистику каждые 30 секунд
        statsTask = new BukkitRunnable() {
            @Override
            public void run() {
                collectAndSendServerStats();
            }
        };
        
        // Первая отправка через 10 секунд, затем каждые 30 секунд
        statsTask.runTaskTimerAsynchronously(plugin, 200L, 600L); // 10s, 30s
    }
    
    /**
     * Остановка отправки статистики
     */
    public void stopStatsCollection() {
        if (statsTask != null) {
            statsTask.cancel();
            statsTask = null;
        }
    }
    
    /**
     * Сбор и отправка статистики сервера
     */
    public CompletableFuture<Boolean> collectAndSendServerStats() {
        return CompletableFuture.supplyAsync(() -> {
            try {
                // Получаем IP и порт сервера из конфигурации
                String serverIp = configManager.getServerIp();
                int serverPort = configManager.getServerPort();
                
                if (serverIp == null || serverIp.isEmpty() || serverPort <= 0) {
                    logger.warning("Не удалось получить IP или порт сервера из конфигурации!");
                    return false;
                }
                
                // logger.info("Отправка статистики сервера для " + serverIp + ":" + serverPort);
                // TPS (берём среднее за последнюю минуту)
                double tps = 0.0;
                try {
                    double[] tpsArray = Bukkit.getServer().getTPS();
                    if (tpsArray != null && tpsArray.length > 0) {
                        tps = Math.min(tpsArray[0], 20.0); // Ограничиваем 20.0
                    }
                } catch (Exception e) {
                    logger.warning("Не удалось получить TPS: " + e.getMessage());
                }
                
                // Время работы сервера в секундах
                long uptimeSeconds = ManagementFactory.getRuntimeMXBean().getUptime() / 1000;
                
                // Информация о памяти (в мегабайтах)
                Runtime runtime = Runtime.getRuntime();
                long maxMemory = runtime.maxMemory() / 1024 / 1024; // MB
                long totalMemory = runtime.totalMemory() / 1024 / 1024; // MB
                long freeMemory = runtime.freeMemory() / 1024 / 1024; // MB
                long usedMemory = totalMemory - freeMemory; // MB
                
                // Игроки
                int onlinePlayers = Bukkit.getOnlinePlayers().size();
                int maxPlayers = Bukkit.getMaxPlayers();
                
                // Версия сервера
                String serverVersion = Bukkit.getVersion();
                
                // Количество плагинов
                int pluginsCount = Bukkit.getPluginManager().getPlugins().length;
                
                // Количество загруженных миров
                int loadedWorlds = Bukkit.getWorlds().size();
                
                // Отправляем данные
                return apiClient.updateServerData(
                    serverIp, serverPort, tps, uptimeSeconds,
                    maxMemory, usedMemory, freeMemory,
                    onlinePlayers, maxPlayers, serverVersion,
                    pluginsCount, loadedWorlds
                ).join();
                
            } catch (Exception e) {
                logger.severe("Ошибка сбора статистики сервера: " + e.getMessage());
                return false;
            }
        });
    }
    
    /**
     * Получение текущей статистики сервера (для команд)
     */
    public String getServerStatsString() {
        try {
            double[] tpsArray = Bukkit.getServer().getTPS();
            double tps = (tpsArray != null && tpsArray.length > 0) ? tpsArray[0] : 0.0;
            
            long uptimeSeconds = ManagementFactory.getRuntimeMXBean().getUptime() / 1000;
            long uptimeMinutes = uptimeSeconds / 60;
            long uptimeHours = uptimeMinutes / 60;
            long uptimeDays = uptimeHours / 24;
            
            Runtime runtime = Runtime.getRuntime();
            long maxMemory = runtime.maxMemory() / 1024 / 1024;
            long totalMemory = runtime.totalMemory() / 1024 / 1024;
            long freeMemory = runtime.freeMemory() / 1024 / 1024;
            long usedMemory = totalMemory - freeMemory;
            
            String uptimeFormatted;
            if (uptimeDays > 0) {
                uptimeFormatted = String.format("%dд %dч %dм", uptimeDays, uptimeHours % 24, uptimeMinutes % 60);
            } else if (uptimeHours > 0) {
                uptimeFormatted = String.format("%dч %dм", uptimeHours, uptimeMinutes % 60);
            } else {
                uptimeFormatted = String.format("%dм", uptimeMinutes);
            }
            
            return String.format(
                "§e§l════════ СТАТИСТИКА СЕРВЕРА ════════\n" +
                "§7TPS: §a%.1f §7/ §a20.0\n" +
                "§7Время работы: §b%s\n" +
                "§7Игроков онлайн: §a%d §7/ §a%d\n" +
                "§7Память: §c%d§7/§a%d §7MB (свободно: §b%d §7MB)\n" +
                "§7Плагинов: §a%d\n" +
                "§7Миров: §a%d\n" +
                "§7Версия: §f%s\n" +
                "§e§l══════════════════════════════════",
                Math.min(tps, 20.0), uptimeFormatted,
                Bukkit.getOnlinePlayers().size(), Bukkit.getMaxPlayers(),
                usedMemory, maxMemory, freeMemory,
                Bukkit.getPluginManager().getPlugins().length,
                Bukkit.getWorlds().size(),
                Bukkit.getVersion()
            );
        } catch (Exception e) {
            return "§cОшибка получения статистики сервера: " + e.getMessage();
        }
    }
}
