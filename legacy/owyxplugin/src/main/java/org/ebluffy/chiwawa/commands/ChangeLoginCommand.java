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
 * Команда для смены режима авторизации (/changelogin)
 */
public class ChangeLoginCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final AuthManager authManager;
    private final Logger logger;

    public ChangeLoginCommand(ConfigManager configManager, AuthManager authManager, Logger logger) {
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

        // Если уже используется авторизация через сайт
        if (authPlayer.isWebsiteAuth()) {
            player.sendMessage("§e§lВы уже используете авторизацию через сайт!");
            player.sendMessage("§7Чтобы вернуться к простой авторизации,");
            player.sendMessage("§7обратитесь к администрации на сайте:");
            player.sendMessage("§b§nhttps://chiwawa.site");
            return true;
        }

        // Обработка подкоманд
        if (args.length == 0) {
            // Показываем информацию о команде
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            player.sendMessage("§6§l         СМЕНА РЕЖИМА АВТОРИЗАЦИИ");
            player.sendMessage("");
            player.sendMessage("§7Текущий режим: §c§lПростая авторизация");
            player.sendMessage("§7(логин и пароль на сервере)");
            player.sendMessage("");
            player.sendMessage("§a§lПреимущества авторизации через сайт:");
            player.sendMessage("§7• Единый вход для сайта и сервера");
            player.sendMessage("§7• Восстановление доступа через email");
            player.sendMessage("§7• Дополнительная защита аккаунта");
            player.sendMessage("§7• Управление профилем на сайте");
            player.sendMessage("");
            player.sendMessage("§eДля смены режима используйте:");
            player.sendMessage("§a/changelogin start §7- Начать процесс смены");
            player.sendMessage("§a/changelogin confirm <код> §7- Подтвердить код с email");
            player.sendMessage("§e§l━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            return true;
        }

        String subCommand = args[0].toLowerCase();

        switch (subCommand) {
            case "start":
                handleStartChange(player);
                break;

            case "confirm":
                if (args.length < 2) {
                    player.sendMessage("§cИспользование: /changelogin confirm <код>");
                    player.sendMessage("§7Введите 6-значный код из email");
                    return true;
                }
                handleConfirmChange(player, args[1]);
                break;

            default:
                player.sendMessage("§cНеизвестная подкоманда!");
                player.sendMessage("§7Используйте: §a/changelogin §7для справки");
                break;
        }

        return true;
    }

    /**
     * Начать процесс смены режима
     */
    private void handleStartChange(Player player) {
        player.sendMessage("§7Проверка возможности смены режима...");

        authManager.requestAuthModeChange(player)
            .thenAccept(result -> {
                if (result.startsWith("SUCCESS:")) {
                    String email = result.substring(8);
                    String maskedEmail = maskEmail(email);
                    
                    player.sendMessage("§a§l✓ Запрос отправлен!");
                    player.sendMessage("§7На ваш email §b" + maskedEmail + " §7отправлен");
                    player.sendMessage("§76-значный код подтверждения");
                    player.sendMessage("");
                    player.sendMessage("§eПосле получения кода используйте:");
                    player.sendMessage("§a/changelogin confirm <код>");
                    player.sendMessage("");
                    player.sendMessage("§7§oКод действителен 15 минут");
                    
                    logger.info("Игрок " + player.getName() + " запросил смену режима авторизации");
                    
                } else {
                    handleError(player, result);
                }
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка соединения!");
                player.sendMessage("§7Попробуйте позже");
                logger.severe("Ошибка запроса смены режима для " + player.getName() + ": " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Подтвердить смену режима кодом
     */
    private void handleConfirmChange(Player player, String code) {
        // Валидация кода
        if (!code.matches("\\d{6}")) {
            player.sendMessage("§cНеверный формат кода!");
            player.sendMessage("§7Код должен состоять из 6 цифр");
            return;
        }

        player.sendMessage("§7Проверка кода подтверждения...");

        authManager.confirmAuthModeChange(player, code)
            .thenAccept(success -> {
                if (success) {
                    player.sendMessage("§a§l✓ Режим авторизации изменен!");
                    player.sendMessage("§7Теперь вы используете авторизацию через сайт");
                    player.sendMessage("");
                    player.sendMessage("§e§lВажно:");
                    player.sendMessage("§7При следующем входе используйте команду:");
                    player.sendMessage("§a/login <токен>");
                    player.sendMessage("§7Токен можно получить на сайте:");
                    player.sendMessage("§b§nhttps://chiwawa.site");
                    player.sendMessage("");
                    player.sendMessage("§7Ваша текущая сессия останется активной");
                    
                    logger.info("Игрок " + player.getName() + " успешно переключился на авторизацию через сайт");
                    
                } else {
                    player.sendMessage("§c§l✗ Неверный или истекший код!");
                    player.sendMessage("§7Код действителен только 15 минут");
                    player.sendMessage("§7Запросите новый код: §a/changelogin start");
                }
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка подтверждения!");
                player.sendMessage("§7Попробуйте позже");
                logger.severe("Ошибка подтверждения смены режима для " + player.getName() + ": " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Обработать ошибку запроса
     */
    private void handleError(Player player, String errorCode) {
        switch (errorCode) {
            case "EMAIL_NOT_FOUND":
                player.sendMessage("§c§l✗ Email не найден!");
                player.sendMessage("§7Для использования авторизации через сайт");
                player.sendMessage("§7необходимо зарегистрироваться на сайте:");
                player.sendMessage("§b§nhttps://chiwawa.site");
                player.sendMessage("");
                player.sendMessage("§7После регистрации подайте заявку на доступ");
                player.sendMessage("§7к серверу через личный кабинет");
                break;

            case "EMAIL_SEND_ERROR":
                player.sendMessage("§c§l✗ Ошибка отправки email!");
                player.sendMessage("§7Не удалось отправить код подтверждения");
                player.sendMessage("§7Попробуйте позже или обратитесь к администрации");
                break;

            case "DB_ERROR":
                player.sendMessage("§c§l✗ Ошибка базы данных!");
                player.sendMessage("§7Попробуйте позже");
                break;

            default:
                player.sendMessage("§c§l✗ Неизвестная ошибка!");
                player.sendMessage("§7Код ошибки: " + errorCode);
                break;
        }
    }

    /**
     * Замаскировать email для безопасности
     */
    private String maskEmail(String email) {
        int atIndex = email.indexOf('@');
        if (atIndex <= 2) {
            return email.charAt(0) + "***@" + email.substring(atIndex + 1);
        }
        
        String localPart = email.substring(0, atIndex);
        String domain = email.substring(atIndex);
        
        if (localPart.length() <= 3) {
            return localPart.charAt(0) + "***" + domain;
        }
        
        return localPart.substring(0, 2) + "***" + localPart.charAt(localPart.length() - 1) + domain;
    }
}
