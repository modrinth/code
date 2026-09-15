package org.ebluffy.chiwawa.listeners;

import org.bukkit.entity.Player;
import org.bukkit.event.EventHandler;
import org.bukkit.event.EventPriority;
import org.bukkit.event.Listener;
import org.bukkit.event.block.BlockBreakEvent;
import org.bukkit.event.block.BlockPlaceEvent;
import org.bukkit.event.entity.EntityDamageByEntityEvent;
import org.bukkit.event.entity.EntityDamageEvent;
import org.bukkit.event.inventory.InventoryClickEvent;
import org.bukkit.event.inventory.InventoryOpenEvent;
import org.bukkit.event.player.*;
import org.bukkit.event.entity.FoodLevelChangeEvent;
import org.bukkit.event.entity.EntityPickupItemEvent;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.ChiwawaPlugin;
import org.ebluffy.chiwawa.config.ConfigManager;
import org.ebluffy.chiwawa.managers.AuthManager;
import org.ebluffy.chiwawa.managers.UserManager;

/**
 * Листенер для ограничения действий неавторизованных игроков
 * РАБОТАЕТ ТОЛЬКО ЕСЛИ enable_authentication: true
 */
public class AuthenticationListener implements Listener {
    private final JavaPlugin plugin;
    private final ConfigManager configManager;
    private final UserManager userManager;
    private final AuthManager authManager;

    public AuthenticationListener(JavaPlugin plugin, UserManager userManager) {
        this.plugin = plugin;
        this.configManager = ((ChiwawaPlugin) plugin).getConfigManager();
        this.userManager = userManager;
        this.authManager = ((ChiwawaPlugin) plugin).getAuthManager();
    }

    /**
     * Проверяет, нужно ли требовать авторизацию
     */
    private boolean isAuthenticationRequired() {
        return configManager.isAuthenticationEnabled();
    }

    /**
     * Проверяет, авторизован ли игрок (оба режима)
     */
    private boolean isPlayerAuthenticated(Player player) {
        // Если система авторизации отключена - все авторизованы
        if (!isAuthenticationRequired()) {
            return true;
        }
        
        // Проверяем оба режима авторизации
        return userManager.isPlayerCached(player.getUniqueId()) || 
               authManager.isPlayerAuthorized(player.getUniqueId());
    }

    /**
     * Отправляет сообщение о необходимости авторизации
     */
    private void sendAuthenticationReminder(Player player) {
        player.sendMessage("§c§lВы не авторизованы!");
        player.sendMessage("§7Новый игрок? §a/register <пароль> <повтор>");
        player.sendMessage("§7Уже зарегистрированы? §a/login <пароль>");
        player.sendMessage("§7Или получите токен на: §b§nhttps://chiwawa.site");
    }

    /**
     * Запрещает движение неавторизованным игрокам
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerMove(PlayerMoveEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            // Разрешаем только поворот головы (yaw/pitch), но не движение по координатам
            if (event.getFrom().getX() != event.getTo().getX() ||
                event.getFrom().getY() != event.getTo().getY() ||
                event.getFrom().getZ() != event.getTo().getZ()) {
                
                event.setTo(event.getFrom());
                
                // Отправляем напоминание не чаще раза в 5 секунд
                long currentTime = System.currentTimeMillis();
                Long lastReminder = (Long) player.getMetadata("last_auth_reminder").stream()
                    .findFirst()
                    .map(metadataValue -> metadataValue.value())
                    .orElse(0L);
                
                if (currentTime - lastReminder > 5000) {
                    sendAuthenticationReminder(player);
                    player.setMetadata("last_auth_reminder", 
                        new org.bukkit.metadata.FixedMetadataValue(plugin, currentTime));
                }
            }
        }
    }

    /**
     * Запрещает ломать блоки
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onBlockBreak(BlockBreakEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            sendAuthenticationReminder(player);
        }
    }

    /**
     * Запрещает ставить блоки
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onBlockPlace(BlockPlaceEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            sendAuthenticationReminder(player);
        }
    }

    /**
     * Запрещает взаимодействие с предметами
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerInteract(PlayerInteractEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            sendAuthenticationReminder(player);
        }
    }

    /**
     * Запрещает взаимодействие с сущностями
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerInteractEntity(PlayerInteractEntityEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            sendAuthenticationReminder(player);
        }
    }

    /**
     * Запрещает выбрасывать предметы
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerDropItem(PlayerDropItemEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            sendAuthenticationReminder(player);
        }
    }

    /**
     * Запрещает поднимать предметы (современная версия события)
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onEntityPickupItem(EntityPickupItemEvent event) {
        if (event.getEntity() instanceof Player) {
            Player player = (Player) event.getEntity();
            
            if (!isPlayerAuthenticated(player)) {
                event.setCancelled(true);
            }
        }
    }

    /**
     * Запрещает открывать инвентари
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onInventoryOpen(InventoryOpenEvent event) {
        if (event.getPlayer() instanceof Player) {
            Player player = (Player) event.getPlayer();
            
            if (!isPlayerAuthenticated(player)) {
                event.setCancelled(true);
                sendAuthenticationReminder(player);
            }
        }
    }

    /**
     * Запрещает взаимодействие с инвентарем
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onInventoryClick(InventoryClickEvent event) {
        if (event.getWhoClicked() instanceof Player) {
            Player player = (Player) event.getWhoClicked();
            
            if (!isPlayerAuthenticated(player)) {
                event.setCancelled(true);
                sendAuthenticationReminder(player);
            }
        }
    }

    /**
     * Запрещает наносить урон другим игрокам
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onEntityDamageByEntity(EntityDamageByEntityEvent event) {
        if (event.getDamager() instanceof Player) {
            Player damager = (Player) event.getDamager();
            
            if (!isPlayerAuthenticated(damager)) {
                event.setCancelled(true);
                sendAuthenticationReminder(damager);
            }
        }
    }

    /**
     * Защищает неавторизованных игроков от урона
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onEntityDamage(EntityDamageEvent event) {
        if (event.getEntity() instanceof Player) {
            Player player = (Player) event.getEntity();
            
            if (!isPlayerAuthenticated(player)) {
                // Защищаем от всех видов урона кроме void (чтобы не застрять в void)
                if (event.getCause() != EntityDamageEvent.DamageCause.VOID) {
                    event.setCancelled(true);
                }
            }
        }
    }

    /**
     * Запрещает терять голод
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onFoodLevelChange(FoodLevelChangeEvent event) {
        if (event.getEntity() instanceof Player) {
            Player player = (Player) event.getEntity();
            
            if (!isPlayerAuthenticated(player)) {
                event.setCancelled(true);
            }
        }
    }

    /**
     * Запрещает писать в чат (кроме команд авторизации)
     */
    @SuppressWarnings("deprecation")
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerChat(AsyncPlayerChatEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            event.setCancelled(true);
            
            // Асинхронно отправляем сообщение
            plugin.getServer().getScheduler().runTask(plugin, () -> {
                sendAuthenticationReminder(player);
            });
        }
    }

    /**
     * Разрешаем только команды авторизации и регистрации
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerCommandPreprocess(PlayerCommandPreprocessEvent event) {
        Player player = event.getPlayer();
        String command = event.getMessage().toLowerCase();
        
        if (!isPlayerAuthenticated(player)) {
            // Разрешаем только команды авторизации и регистрации
            if (!command.startsWith("/login") && 
                !command.startsWith("/l ") &&
                !command.startsWith("/register") &&
                !command.startsWith("/reg ")) {
                event.setCancelled(true);
                sendAuthenticationReminder(player);
            }
        }
    }

    /**
     * Запрещает телепортацию
     */
    @EventHandler(priority = EventPriority.HIGHEST)
    public void onPlayerTeleport(PlayerTeleportEvent event) {
        Player player = event.getPlayer();
        
        if (!isPlayerAuthenticated(player)) {
            // Разрешаем телепортацию только при входе на сервер (PLUGIN причина)
            if (event.getCause() != PlayerTeleportEvent.TeleportCause.PLUGIN) {
                event.setCancelled(true);
                sendAuthenticationReminder(player);
            }
        }
    }
}
