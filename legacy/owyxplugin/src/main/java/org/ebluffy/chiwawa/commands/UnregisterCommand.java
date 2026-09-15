package org.ebluffy.chiwawa.commands;

import org.bukkit.Bukkit;
import org.bukkit.OfflinePlayer;
import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.database.DatabaseManager;

import java.util.UUID;
import java.util.logging.Logger;

/**
 * Команда для удаления регистрации игрока (только для админов)
 * /unregister <nickname>
 */
public class UnregisterCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final DatabaseManager databaseManager;
    private final Logger logger;

    public UnregisterCommand(ConfigManager configManager, DatabaseManager databaseManager, Logger logger) {
        this.configManager = configManager;
        this.databaseManager = databaseManager;
        this.logger = logger;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        // Проверка прав доступа
        if (!sender.hasPermission("chiwawa.command.unregister")) {
            sender.sendMessage("§cУ вас нет прав для использования этой команды!");
            return true;
        }

        if (args.length != 1) {
            sender.sendMessage("§cИспользование: /unregister <nickname>");
            sender.sendMessage("§7Удаляет регистрацию игрока из БД authplugin");
            sender.sendMessage("§7Игрок сможет зарегистрироваться заново");
            return true;
        }

        String targetNickname = args[0];

        // Проверяем, что администратор подтверждает действие
        sender.sendMessage("§e§l⚠ ВНИМАНИЕ!");
        sender.sendMessage("§7Вы собираетесь удалить регистрацию игрока §c" + targetNickname);
        sender.sendMessage("§7Это действие:");
        sender.sendMessage("§7• Удалит учетную запись из БД authplugin");
        sender.sendMessage("§7• Игрок потеряет доступ с текущим паролем");
        sender.sendMessage("§7• Игрок сможет зарегистрироваться заново");
        sender.sendMessage("");
        sender.sendMessage("§7Проверяю данные...");

        // Проверяем существование игрока
        databaseManager.getPlayerByUsername(targetNickname)
            .thenAccept(authPlayerOpt -> {
                if (authPlayerOpt.isEmpty()) {
                    sender.sendMessage("§c§l✗ Игрок " + targetNickname + " не зарегистрирован!");
                    sender.sendMessage("§7В БД authplugin нет записи для этого игрока");
                    return;
                }

                // Удаляем регистрацию
                databaseManager.deletePlayer(targetNickname)
                    .thenAccept(success -> {
                        if (success) {
                            sender.sendMessage("§a§l✓ Регистрация игрока " + targetNickname + " успешно удалена!");
                            sender.sendMessage("§7Игрок может зарегистрироваться заново: §a/register <пароль> <повтор>");
                            
                            // Логируем действие
                            logger.info("Администратор " + sender.getName() + " удалил регистрацию игрока " + targetNickname);
                            logUnregister(sender, targetNickname, true, null);
                            
                            // Если игрок онлайн - кикаем его
                            Player targetPlayer = Bukkit.getPlayerExact(targetNickname);
                            if (targetPlayer != null && targetPlayer.isOnline()) {
                                targetPlayer.kick(net.kyori.adventure.text.Component.text(
                                    "§c§lВаша регистрация удалена администратором\n\n" +
                                    "§7Вы можете зарегистрироваться заново:\n" +
                                    "§a/register <пароль> <повтор>"
                                ));
                                sender.sendMessage("§7Игрок " + targetNickname + " был кикнут с сервера");
                            }
                        } else {
                            sender.sendMessage("§c§l✗ Ошибка удаления регистрации!");
                            sender.sendMessage("§7Проверьте логи сервера для подробностей");
                            
                            logger.warning("Не удалось удалить регистрацию игрока " + targetNickname);
                            logUnregister(sender, targetNickname, false, "Ошибка БД");
                        }
                    })
                    .exceptionally(throwable -> {
                        sender.sendMessage("§c§l✗ Ошибка удаления регистрации!");
                        sender.sendMessage("§7" + throwable.getMessage());
                        
                        logger.severe("Ошибка удаления регистрации игрока " + targetNickname + ": " + throwable.getMessage());
                        logUnregister(sender, targetNickname, false, throwable.getMessage());
                        return null;
                    });
            })
            .exceptionally(throwable -> {
                sender.sendMessage("§c§l✗ Ошибка проверки игрока!");
                sender.sendMessage("§7" + throwable.getMessage());
                
                logger.severe("Ошибка проверки игрока " + targetNickname + ": " + throwable.getMessage());
                return null;
            });

        return true;
    }

    /**
     * Логирование удаления регистрации
     */
    private void logUnregister(CommandSender admin, String targetPlayer, boolean success, String reason) {
        String adminName = admin instanceof Player ? 
            ((Player) admin).getName() + " (" + ((Player) admin).getUniqueId() + ")" : 
            admin.getName();
        
        String logMessage = String.format(
            "[UNREGISTER] Admin: %s, Target: %s, Success: %s%s",
            adminName,
            targetPlayer,
            success,
            reason != null ? ", Reason: " + reason : ""
        );
        
        // Используем отдельный логгер для аудита
        java.util.logging.Logger auditLogger = java.util.logging.Logger.getLogger("ChiwawaAudit");
        
        if (success) {
            auditLogger.info(logMessage);
        } else {
            auditLogger.warning(logMessage);
        }
    }
}
