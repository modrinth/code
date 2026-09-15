using System;
using DiscordRPC;
using DiscordRPC.Logging;

namespace MinecraftLauncher
{
    /// <summary>
    /// Менеджер Discord Rich Presence для отображения активности в Discord
    /// </summary>
    public class DiscordRPCManager : IDisposable
    {
        private DiscordRpcClient client;
        private bool isInitialized = false;
        
        // Discord Application ID (нужно создать приложение на https://discord.com/developers/applications)
        private const string APPLICATION_ID = "1433150569010167980"; // ЗАМЕНИТЬ на реальный ID
        
        // Иконки (нужно загрузить в Discord Developer Portal -> Rich Presence -> Art Assets)
        private const string LARGE_IMAGE_KEY = "owyx_logo";
        private const string SMALL_IMAGE_KEY = "minecraft_icon";
        
        public DiscordRPCManager()
        {
            try
            {
                Initialize();
            }
            catch (Exception ex)
            {
                // Discord может быть не установлен или не запущен
                Console.WriteLine($"Не удалось инициализировать Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Инициализация Discord RPC клиента
        /// </summary>
        private void Initialize()
        {
            client = new DiscordRpcClient(APPLICATION_ID);
            
            // Настройка логирования (опционально)
            client.Logger = new ConsoleLogger() { Level = LogLevel.Warning };
            
            // События подключения
            client.OnReady += (sender, e) =>
            {
                Console.WriteLine($"Discord RPC подключен к пользователю {e.User.Username}");
            };
            
            client.OnPresenceUpdate += (sender, e) =>
            {
                Console.WriteLine("Discord RPC обновлен");
            };
            
            // Подключение
            if (client.Initialize())
            {
                isInitialized = true;
                SetIdleState();
            }
        }
        
        /// <summary>
        /// Установить статус "В меню лаунчера"
        /// </summary>
        public void SetIdleState()
        {
            if (!isInitialized) return;
            
            try
            {
                client.SetPresence(new RichPresence()
                {
                    Details = "В меню лаунчера",
                    State = "Выбирает профиль",
                    Assets = new Assets()
                    {
                        LargeImageKey = LARGE_IMAGE_KEY,
                        LargeImageText = "Owyx Launcher",
                        SmallImageKey = SMALL_IMAGE_KEY,
                        SmallImageText = "Minecraft"
                    },
                    Timestamps = Timestamps.Now
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при обновлении Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Установить статус "Играет на сервере"
        /// </summary>
        /// <param name="serverName">Название сервера</param>
        /// <param name="playerCount">Количество игроков</param>
        /// <param name="maxPlayers">Максимальное количество игроков</param>
        public void SetPlayingState(string serverName, int playerCount, int maxPlayers)
        {
            if (!isInitialized) return;
            
            try
            {
                client.SetPresence(new RichPresence()
                {
                    Details = $"Играет на {serverName}",
                    State = $"Игроков: {playerCount}/{maxPlayers}",
                    Assets = new Assets()
                    {
                        LargeImageKey = LARGE_IMAGE_KEY,
                        LargeImageText = "Owyx Launcher",
                        SmallImageKey = SMALL_IMAGE_KEY,
                        SmallImageText = "Minecraft"
                    },
                    Party = new Party()
                    {
                        ID = "chiwawa_server",
                        Size = playerCount,
                        Max = maxPlayers
                    },
                    Timestamps = Timestamps.Now
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при обновлении Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Установить статус "Обновление игры"
        /// </summary>
        /// <param name="progress">Прогресс обновления (0-100)</param>
        public void SetUpdatingState(int progress)
        {
            if (!isInitialized) return;
            
            try
            {
                client.SetPresence(new RichPresence()
                {
                    Details = "Обновление игры",
                    State = $"Прогресс: {progress}%",
                    Assets = new Assets()
                    {
                        LargeImageKey = LARGE_IMAGE_KEY,
                        LargeImageText = "Owyx Launcher",
                        SmallImageKey = SMALL_IMAGE_KEY,
                        SmallImageText = "Minecraft"
                    },
                    Timestamps = Timestamps.Now
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при обновлении Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Установить статус "В настройках"
        /// </summary>
        public void SetSettingsState()
        {
            if (!isInitialized) return;
            
            try
            {
                client.SetPresence(new RichPresence()
                {
                    Details = "Настраивает лаунчер",
                    State = "В меню настроек",
                    Assets = new Assets()
                    {
                        LargeImageKey = LARGE_IMAGE_KEY,
                        LargeImageText = "Owyx Launcher",
                        SmallImageKey = SMALL_IMAGE_KEY,
                        SmallImageText = "Minecraft"
                    },
                    Timestamps = Timestamps.Now
                });
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при обновлении Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Очистить Discord Rich Presence
        /// </summary>
        public void ClearPresence()
        {
            if (!isInitialized) return;
            
            try
            {
                client.ClearPresence();
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при очистке Discord RPC: {ex.Message}");
            }
        }
        
        /// <summary>
        /// Освобождение ресурсов
        /// </summary>
        public void Dispose()
        {
            if (client != null)
            {
                ClearPresence();
                client.Dispose();
                isInitialized = false;
            }
        }
    }
}
