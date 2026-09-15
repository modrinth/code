package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.AuthManager;

import java.util.logging.Logger;

/**
 * Команда для регистрации игрока (/register <пароль> <повтор_пароля>)
 */
public class RegisterCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final AuthManager authManager;
    private final Logger logger;

    public RegisterCommand(ConfigManager configManager, AuthManager authManager, Logger logger) {
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

        // Проверка, не авторизован ли уже игрок
        if (authManager.isPlayerAuthorized(player.getUniqueId())) {
            player.sendMessage("§cВы уже авторизованы!");
            return true;
        }

        if (args.length != 2) {
            player.sendMessage("§cИспользование: /register <пароль> <повтор_пароля>");
            player.sendMessage("§7Пароль должен быть от 6 до 32 символов");
            return true;
        }

        String password = args[0];
        String passwordRepeat = args[1];

        // Проверка совпадения паролей
        if (!password.equals(passwordRepeat)) {
            player.sendMessage("§c§l✗ Пароли не совпадают!");
            player.sendMessage("§7Пожалуйста, введите одинаковые пароли");
            return true;
        }

        // Валидация пароля
        if (!AuthManager.isPasswordValid(password)) {
            player.sendMessage("§c§l✗ Недопустимый пароль!");
            player.sendMessage("§7Пароль должен:");
            player.sendMessage("§7• Быть от 6 до 32 символов");
            player.sendMessage("§7• Содержать только буквы, цифры и спецсимволы");
            return true;
        }

        // Показываем силу пароля
        String strength = AuthManager.getPasswordStrength(password);
        String strengthColor = strength.equals("Сильный") ? "§a" : strength.equals("Средний") ? "§e" : "§c";
        
        player.sendMessage("§7Сила пароля: " + strengthColor + strength);
        player.sendMessage("§7Регистрация...");

        // Регистрируем игрока
        authManager.registerPlayer(player, password)
            .thenAccept(success -> {
                if (success) {
                    player.sendMessage("§a§l✓ Регистрация успешна!");
                    player.sendMessage("§7Вы успешно зарегистрированы на сервере");
                    player.sendMessage("§7Используйте §a/login <пароль> §7для входа в следующий раз");
                    player.sendMessage("");
                    player.sendMessage("§e§l⚡ РЕКОМЕНДАЦИЯ:");
                    player.sendMessage("§7Для лучшей защиты аккаунта используйте");
                    player.sendMessage("§7авторизацию через сайт: §a/changelogin");
                    
                    // Логируем БЕЗ пароля
                    logger.info("Игрок " + player.getName() + " зарегистрирован");
                } else {
                    player.sendMessage("§c§l✗ Ошибка регистрации!");
                    player.sendMessage("§7Возможно, вы уже зарегистрированы");
                    player.sendMessage("§7Используйте §a/login <пароль>");
                    logger.warning("Попытка повторной регистрации: " + player.getName());
                }
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка соединения с базой данных!");
                player.sendMessage("§7Попробуйте позже или обратитесь к администрации");
                logger.severe("Ошибка регистрации " + player.getName() + ": " + throwable.getMessage());
                return null;
            });

        return true;
    }
}
