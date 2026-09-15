using System;
using System.IO;
using System.Text.Json;
using System.Threading.Tasks;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    public class ConfigManager
    {
        private const string CONFIG_FILE_NAME = "launcher_config.json";
        private const string PROFILES_FILE_NAME = "launcher_profiles.json";
        private readonly string configDirectory;
        private readonly string configFilePath;
        private readonly string profilesFilePath;

        public ConfigManager()
        {
            configDirectory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ".owyxlauncher");
            configFilePath = Path.Combine(configDirectory, CONFIG_FILE_NAME);
            profilesFilePath = Path.Combine(configDirectory, PROFILES_FILE_NAME);

            EnsureConfigDirectoryExists();
        }

        /// <summary>
        /// Загрузить конфигурацию лаунчера
        /// </summary>
        /// <returns>Конфигурация лаунчера</returns>
        public async Task<LauncherConfig> LoadConfig()
        {
            try
            {
                if (!File.Exists(configFilePath))
                {
                    var defaultConfig = CreateDefaultConfig();
                    await SaveConfig(defaultConfig);
                    return defaultConfig;
                }

                var jsonContent = await File.ReadAllTextAsync(configFilePath);
                var config = JsonSerializer.Deserialize<LauncherConfig>(jsonContent);
                return config ?? CreateDefaultConfig();
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при загрузке конфигурации: {ex.Message}");
                return CreateDefaultConfig();
            }
        }

        /// <summary>
        /// Сохранить конфигурацию лаунчера
        /// </summary>
        /// <param name="config">Конфигурация для сохранения</param>
        public async Task SaveConfig(LauncherConfig config)
        {
            try
            {
                var options = new JsonSerializerOptions
                {
                    WriteIndented = true
                };

                var jsonContent = JsonSerializer.Serialize(config, options);
                await File.WriteAllTextAsync(configFilePath, jsonContent);
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при сохранении конфигурации: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Загрузить профиль игры
        /// </summary>
        /// <returns>Профиль игры</returns>
        public async Task<GameProfile> LoadGameProfile()
        {
            try
            {
                if (!File.Exists(profilesFilePath))
                {
                    var defaultProfile = CreateDefaultProfile();
                    await SaveGameProfile(defaultProfile);
                    return defaultProfile;
                }

                var jsonContent = await File.ReadAllTextAsync(profilesFilePath);
                var profile = JsonSerializer.Deserialize<GameProfile>(jsonContent);
                return profile ?? CreateDefaultProfile();
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при загрузке профиля: {ex.Message}");
                return CreateDefaultProfile();
            }
        }

        /// <summary>
        /// Сохранить профиль игры
        /// </summary>
        /// <param name="profile">Профиль для сохранения</param>
        public async Task SaveGameProfile(GameProfile profile)
        {
            try
            {
                var options = new JsonSerializerOptions
                {
                    WriteIndented = true
                };

                var jsonContent = JsonSerializer.Serialize(profile, options);
                await File.WriteAllTextAsync(profilesFilePath, jsonContent);
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при сохранении профиля: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Получить путь к директории игры
        /// </summary>
        /// <returns>Путь к директории игры</returns>
        public string GetGameDirectory()
        {
            return configDirectory;
        }

        /// <summary>
        /// Убедиться что директория конфигурации существует
        /// </summary>
        private void EnsureConfigDirectoryExists()
        {
            try
            {
                if (!Directory.Exists(configDirectory))
                {
                    Directory.CreateDirectory(configDirectory);
                }

                // Создать поддиректории
                var subdirectories = new[] { "versions", "mods", "assets", "logs" };
                foreach (var subdir in subdirectories)
                {
                    var path = Path.Combine(configDirectory, subdir);
                    if (!Directory.Exists(path))
                    {
                        Directory.CreateDirectory(path);
                    }
                }
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при создании директорий: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Создать конфигурацию по умолчанию
        /// </summary>
        /// <returns>Конфигурация по умолчанию</returns>
        private LauncherConfig CreateDefaultConfig()
        {
            return new LauncherConfig
            {
                ServerUrl = "http://yourserver.com",
                LauncherVersion = "1.0.0",
                AutoUpdate = true,
                CheckUpdatesOnStartup = true,
                GameDirectory = configDirectory
            };
        }

        /// <summary>
        /// Создать профиль по умолчанию
        /// </summary>
        /// <returns>Профиль по умолчанию</returns>
        private GameProfile CreateDefaultProfile()
        {
            return new GameProfile
            {
                Name = "Default",
                GameDirectory = configDirectory,
                MinecraftVersion = "1.20.1",
                LauncherVersion = "1.0.0",
                IsDefault = true
            };
        }
    }

    /// <summary>
    /// Конфигурация лаунчера
    /// </summary>
    public class LauncherConfig
    {
        public string ServerUrl { get; set; } = string.Empty;
        public string LauncherVersion { get; set; } = "1.0.0";
        public bool AutoUpdate { get; set; } = true;
        public bool CheckUpdatesOnStartup { get; set; } = true;
        public string GameDirectory { get; set; } = string.Empty;
        public DateTime LastUpdateCheck { get; set; } = DateTime.Now;
        public bool EnableDiscordRPC { get; set; } = true;
    }
}