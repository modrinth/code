package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.dto.AuthPlayer;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.AuthManager;

import java.util.Optional;
import java.util.logging.Logger;

/**
 * Команда для смены пароля (/changepass <старый_пароль> <новый_пароль> <повтор>)
 */
public class ChangePasswordCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final AuthManager authManager;
    private final Logger logger;

    public ChangePasswordCommand(ConfigManager configManager, AuthManager authManager, Logger logger) {
        this.configManager = configManager;
        this.authManager = authManager;
        this.logger = logger;
    }

    @Override
    public boolean onCommand(CommandSender sender, Command command, String label, String[] args) {
        if (!(sender instanceof Player)) {
            sender.sendMessage("§cЭта команда доступна только игрокам!");
            return true;
        }

        Player player = (Player) sender;

        // Проверка авторизации
        if (!authManager.isPlayerAuthorized(player.getUniqueId())) {
            player.sendMessage("§cВы должны быть авторизованы для использования этой команды!");
            return true;
        }

        // Получаем данные игрока
        Optional<AuthPlayer> authPlayerOpt = authManager.getAuthorizedPlayer(player.getUniqueId());
        if (authPlayerOpt.isEmpty()) {
            player.sendMessage("§cОшибка получения данных игрока");
            return true;
        }

        AuthPlayer authPlayer = authPlayerOpt.get();

        // Проверяем, использует ли игрок простую авторизацию
        if (authPlayer.isWebsiteAuth()) {
            player.sendMessage("§c§lВы используете авторизацию через сайт!");
            player.sendMessage("§7Для смены пароля используйте сайт:");
            player.sendMessage("§b§nhttps://chiwawa.site/profile/security");
            return true;
        }

        if (args.length != 3) {
            player.sendMessage("§cИспользование: /changepass <старый_пароль> <новый_пароль> <повтор>");
            player.sendMessage("§7Новый пароль должен быть от 6 до 32 символов");
            return true;
        }

        String oldPassword = args[0];
        String newPassword = args[1];
        String newPasswordRepeat = args[2];

        // Проверка совпадения новых паролей
        if (!newPassword.equals(newPasswordRepeat)) {
            player.sendMessage("§c§l✗ Новые пароли не совпадают!");
            player.sendMessage("§7Пожалуйста, введите одинаковые пароли");
            return true;
        }

        // Проверка, что новый пароль отличается от старого
        if (oldPassword.equals(newPassword)) {
            player.sendMessage("§c§l✗ Новый пароль должен отличаться от старого!");
            return true;
        }

        // Валидация нового пароля
        if (!AuthManager.isPasswordValid(newPassword)) {
            player.sendMessage("§c§l✗ Недопустимый новый пароль!");
            player.sendMessage("§7Пароль должен:");
            player.sendMessage("§7• Быть от 6 до 32 символов");
            player.sendMessage("§7• Содержать только буквы, цифры и спецсимволы");
            return true;
        }

        // Показываем силу нового пароля
        String strength = AuthManager.getPasswordStrength(newPassword);
        String strengthColor = strength.equals("Сильный") ? "§a" : strength.equals("Средний") ? "§e" : "§c";
        
        player.sendMessage("§7Сила нового пароля: " + strengthColor + strength);
        player.sendMessage("§7Смена пароля...");

        // Меняем пароль
        authManager.changePassword(player, oldPassword, newPassword)
            .thenAccept(success -> {
                if (success) {
                    player.sendMessage("§a§l✓ Пароль успешно изменен!");
                    player.sendMessage("§7При следующем входе используйте новый пароль");
                    player.sendMessage("§7Запомните его или сохраните в надежном месте");
                    
                    // Логируем изменение пароля
                    logger.info("Игрок " + player.getName() + " успешно изменил пароль");
                    
                    // Записываем в лог аудита
                    logPasswordChange(player, true, null);
                } else {
                    player.sendMessage("§c§l✗ Неверный старый пароль!");
                    player.sendMessage("§7Пожалуйста, проверьте правильность ввода");
                    
                    // Логируем неудачную попытку
                    logger.warning("Неудачная попытка смены пароля для игрока " + player.getName());
                    logPasswordChange(player, false, "Неверный старый пароль");
                }
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка смены пароля!");
                player.sendMessage("§7Попробуйте позже или обратитесь к администрации");
                
                logger.severe("Ошибка смены пароля для игрока " + player.getName() + ": " + throwable.getMessage());
                logPasswordChange(player, false, "Ошибка: " + throwable.getMessage());
                return null;
            });

        return true;
    }

    /**
     * Логирование изменения пароля
     */
    private void logPasswordChange(Player player, boolean success, String reason) {
        String logMessage = String.format(
            "[PASSWORD_CHANGE] Player: %s, UUID: %s, IP: %s, Success: %s%s",
            player.getName(),
            player.getUniqueId(),
            player.getAddress().getAddress().getHostAddress(),
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
