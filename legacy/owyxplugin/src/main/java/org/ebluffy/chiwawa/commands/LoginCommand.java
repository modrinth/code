package org.ebluffy.chiwawa.commands;

import org.bukkit.command.Command;
import org.bukkit.command.CommandExecutor;
import org.bukkit.command.CommandSender;
import org.bukkit.entity.Player;
import org.ebluffy.chiwawa.api.ApiClient;
import org.ebluffy.chiwawa.api.dto.AuthPlayer;
import org.ebluffy.chiwawa.api.dto.TokenVerificationResult;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.AuthManager;
import org.ebluffy.chiwawa.managers.UserManager;

import java.util.Optional;
import java.util.logging.Logger;

/**
 * Команда для авторизации игрока (поддерживает оба режима)
 * /login <пароль> - простая авторизация
 * /login <токен> - авторизация через сайт
 */
public class LoginCommand implements CommandExecutor {
    private final ConfigManager configManager;
    private final ApiClient apiClient;
    private final UserManager userManager;
    private final AuthManager authManager;
    private final Logger logger;

    public LoginCommand(ConfigManager configManager, ApiClient apiClient, UserManager userManager, 
                       AuthManager authManager, Logger logger) {
        this.configManager = configManager;
        this.apiClient = apiClient;
        this.userManager = userManager;
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
        if (authManager.isPlayerAuthorized(player.getUniqueId()) || 
            userManager.isPlayerCached(player.getUniqueId())) {
            player.sendMessage("§cВы уже авторизованы!");
            return true;
        }

        if (args.length != 1) {
            player.sendMessage("§cИспользование:");
            player.sendMessage("§7• /login <пароль> §f- простая авторизация");
            player.sendMessage("§7• /login <токен> §f- авторизация через сайт");
            player.sendMessage("§7Токен можно получить на: §b§nhttps://chiwawa.site");
            return true;
        }

        String input = args[0];
        String nickname = player.getName();

        // Определяем тип входа: токен обычно длиннее 32 символов
        if (input.length() >= 32) {
            // Авторизация через токен (сайт)
            handleWebsiteLogin(player, input, nickname);
        } else {
            // Простая авторизация по паролю
            handleSimpleLogin(player, input);
        }

        return true;
    }

    /**
     * Обработка простой авторизации по паролю
     */
    private void handleSimpleLogin(Player player, String password) {
        player.sendMessage("§7Проверка пароля...");

        // Сначала проверяем, что игрок зарегистрирован
        authManager.getAuthorizedPlayer(player.getUniqueId())
            .or(() -> {
                // Если нет в кеше, пытаемся загрузить из БД
                try {
                    return authManager.loginPlayer(player, password).get() 
                        ? authManager.getAuthorizedPlayer(player.getUniqueId())
                        : Optional.empty();
                } catch (Exception e) {
                    return Optional.empty();
                }
            });

        // Сначала проверяем блокировку
        org.ebluffy.chiwawa.ChiwawaPlugin plugin = 
            (org.ebluffy.chiwawa.ChiwawaPlugin) org.bukkit.Bukkit.getPluginManager().getPlugin("ChiwawaPlugin");
        
        plugin.getDatabaseManager().isPlayerLocked(player.getName())
            .thenCompose(isLocked -> {
                if (isLocked) {
                    // Игрок заблокирован - получаем время до разблокировки
                    return plugin.getDatabaseManager().getMinutesUntilUnlock(player.getName())
                        .thenApply(minutesRemaining -> {
                            player.sendMessage("§c§l✗ Аккаунт временно заблокирован!");
                            player.sendMessage("§7Причина: Превышено количество попыток входа (5)");
                            player.sendMessage("§7Время до разблокировки: §c" + minutesRemaining + " минут");
                            player.sendMessage("§7После разблокировки попробуйте снова");
                            return false;
                        });
                }
                
                // Пытаемся авторизоваться
                return authManager.loginPlayer(player, password)
                    .thenApply(success -> {
                        if (success) {
                            Optional<AuthPlayer> authPlayerOpt = authManager.getAuthorizedPlayer(player.getUniqueId());
                            
                            player.sendMessage("§a§l✓ Авторизация успешна!");
                            player.sendMessage("§7Добро пожаловать, §a" + player.getName() + "§7!");
                            
                            if (authPlayerOpt.isPresent()) {
                                player.sendMessage("§7Режим: §aПростая авторизация");
                                
                                // Предлагаем переключиться на сайт
                                player.sendMessage("");
                                player.sendMessage("§e§l💡 СОВЕТ:");
                                player.sendMessage("§7Для лучшей защиты используйте");
                                player.sendMessage("§7авторизацию через сайт: §a/changelogin");
                            }
                            
                            // Логируем БЕЗ пароля
                            logger.info("Игрок " + player.getName() + " авторизован");
                        } else {
                            // Получаем количество попыток
                            plugin.getDatabaseManager().getFailedLoginAttempts(player.getName())
                                .thenAccept(attempts -> {
                                    player.sendMessage("§c§l✗ Неверный пароль!");
                                    
                                    if (attempts >= 5) {
                                        player.sendMessage("§c§lВНИМАНИЕ: Аккаунт заблокирован на 5 минут!");
                                        player.sendMessage("§7Причина: Превышено количество попыток входа");
                                    } else if (attempts >= 3) {
                                        player.sendMessage("§e§lВНИМАНИЕ: Попытка " + attempts + "/5");
                                        player.sendMessage("§7После 5 неудачных попыток аккаунт будет заблокирован на 5 минут");
                                    } else {
                                        player.sendMessage("§7Попытка " + attempts + "/5");
                                    }
                                    
                                    player.sendMessage("§7Если забыли пароль, обратитесь к администрации");
                                });
                        }
                        return success;
                    });
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка авторизации!");
                player.sendMessage("§7Попробуйте позже");
                logger.severe("Ошибка простой авторизации " + player.getName() + ": " + throwable.getMessage());
                return false;
            });
    }

    /**
     * Обработка авторизации через токен сайта
     */
    private void handleWebsiteLogin(Player player, String token, String nickname) {
        player.sendMessage("§7Проверка токена...");

        apiClient.verifyGameToken(token, nickname)
            .thenAccept(result -> {
                if (result.isValid()) {
                    // Токен действителен, авторизуем игрока
                    userManager.authorizePlayer(player, result.getUserId());
                    
                    // Создаем игровую сессию
                    apiClient.createGameSession(
                        nickname, 
                        player.getUniqueId().toString(),
                        player.getAddress().getAddress().getHostAddress(),
                        "Minecraft Client"
                    ).thenAccept(sessionResult -> {
                        if (sessionResult) {
                            logger.info("Игровая сессия создана для игрока " + nickname);
                        } else {
                            logger.warning("Не удалось создать игровую сессию для игрока " + nickname);
                        }
                    }).exceptionally(throwable -> {
                        logger.warning("Ошибка создания игровой сессии для " + nickname + ": " + throwable.getMessage());
                        return null;
                    });
                    
                    player.sendMessage("§a§l✓ Авторизация успешна!");
                    player.sendMessage("§7Добро пожаловать, §a" + result.getNickname() + "§7!");
                    player.sendMessage("§7Роль: §a" + getRoleDisplayName(result.getRole()));
                    player.sendMessage("§7Trust Level: §a" + result.getTrustLevel());
                    player.sendMessage("§7Сессия создана на 7 дней");
                    
                    logger.info("Игрок " + nickname + " успешно авторизовался с токеном (ID: " + result.getUserId() + ")");
                } else {
                    // Токен недействителен
                    player.sendMessage("§c§l✗ Ошибка авторизации!");
                    player.sendMessage("§7" + result.getMessage());
                    player.sendMessage("§7Получите новый токен на сайте: §b§nhttps://chiwawa.site");
                    
                    logger.warning("Неудачная попытка авторизации игрока " + nickname + ": " + result.getMessage());
                }
            })
            .exceptionally(throwable -> {
                player.sendMessage("§c§l✗ Ошибка соединения с сервером!");
                player.sendMessage("§7Попробуйте позже или обратитесь к администрации.");
                
                logger.severe("Ошибка проверки токена для игрока " + nickname + ": " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Получить отображаемое название роли
     */
    private String getRoleDisplayName(String role) {
        switch (role) {
            case "admin": return "Администратор";
            case "moderator": return "Модератор";
            case "user": return "Пользователь";
            default: return "Неизвестная роль";
        }
    }
}
