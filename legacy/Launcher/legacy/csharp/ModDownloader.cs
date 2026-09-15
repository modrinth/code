using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using System.Collections.Generic;
using Renci.SshNet;
using Renci.SshNet.Sftp;

namespace MinecraftLauncher
{
    /// <summary>
    /// Компонент для скачивания модов через SFTP
    /// </summary>
    public class ModDownloader : IDisposable
    {
        private readonly string sftpHost;
        private readonly int sftpPort;
        private readonly string sftpUsername;
        private readonly string sftpPassword;
        private readonly string remoteModsPath;
        private Logger logger;

        public ModDownloader(string host, int port, string username, string password, string remotePath, Logger logger)
        {
            sftpHost = host;
            sftpPort = port;
            sftpUsername = username;
            sftpPassword = password;
            remoteModsPath = remotePath;
            this.logger = logger;
        }

        /// <summary>
        /// Скачать все моды для указанной версии Minecraft
        /// </summary>
        /// <param name="minecraftVersion">Версия Minecraft (например, "1.20.1")</param>
        /// <param name="localModsPath">Локальная папка для модов</param>
        /// <param name="progressCallback">Callback для прогресса</param>
        /// <returns>True если успешно</returns>
        public async Task<bool> DownloadModsForVersion(string minecraftVersion, string localModsPath, 
            Action<string, int>? progressCallback = null)
        {
            try
            {
                logger?.Info($"Starting mod download for Minecraft {minecraftVersion}");
                progressCallback?.Invoke("Подключение к серверу модов...", 0);

                // Создаем ConnectionInfo с несколькими методами аутентификации
                var connectionInfo = new Renci.SshNet.ConnectionInfo(sftpHost, sftpPort, sftpUsername,
                    new Renci.SshNet.PasswordAuthenticationMethod(sftpUsername, sftpPassword),
                    new Renci.SshNet.KeyboardInteractiveAuthenticationMethod(sftpUsername));
                
                // Обработчик для keyboard-interactive
                var keyboardAuth = connectionInfo.AuthenticationMethods.OfType<Renci.SshNet.KeyboardInteractiveAuthenticationMethod>().FirstOrDefault();
                if (keyboardAuth != null)
                {
                    keyboardAuth.AuthenticationPrompt += (sender, e) =>
                    {
                        foreach (var prompt in e.Prompts)
                        {
                            if (prompt.Request.IndexOf("Password", StringComparison.InvariantCultureIgnoreCase) != -1)
                            {
                                prompt.Response = sftpPassword;
                            }
                        }
                    };
                }

                using var client = new SftpClient(connectionInfo);
                
                await Task.Run(() => client.Connect());
                
                if (!client.IsConnected)
                {
                    logger?.Error("Failed to connect to SFTP server");
                    progressCallback?.Invoke("Ошибка подключения к серверу", 0);
                    return false;
                }

                logger?.Info($"Connected to SFTP server {sftpHost}:{sftpPort}");
                progressCallback?.Invoke("Получение списка модов...", 10);

                // Путь к модам для конкретной версии
                var versionPath = $"{remoteModsPath}/{minecraftVersion}";
                
                if (!client.Exists(versionPath))
                {
                    logger?.Warning($"Mods directory not found: {versionPath}");
                    progressCallback?.Invoke($"Моды для версии {minecraftVersion} не найдены", 0);
                    return false;
                }

                // Получить список модов
                var files = client.ListDirectory(versionPath);
                var modFiles = new List<Renci.SshNet.Sftp.ISftpFile>();
                
                foreach (var file in files)
                {
                    if (file.IsRegularFile && file.Name.EndsWith(".jar"))
                    {
                        modFiles.Add(file);
                    }
                }

                if (modFiles.Count == 0)
                {
                    logger?.Info("No mods found for this version");
                    progressCallback?.Invoke("Моды не найдены", 100);
                    return true;
                }

                logger?.Info($"Found {modFiles.Count} mods to download");
                progressCallback?.Invoke($"Найдено {modFiles.Count} модов", 20);

                // Создать локальную папку модов
                Directory.CreateDirectory(localModsPath);

                // Очистить старые моды (опционально)
                CleanOldMods(localModsPath);

                // Скачать каждый мод
                int downloaded = 0;
                foreach (var modFile in modFiles)
                {
                    var localFilePath = Path.Combine(localModsPath, modFile.Name);
                    
                    // Проверить, нужно ли скачивать (сравнить размер)
                    if (File.Exists(localFilePath))
                    {
                        var localFileInfo = new FileInfo(localFilePath);
                        if (localFileInfo.Length == modFile.Length)
                        {
                            logger?.Debug($"Skipping {modFile.Name} (already exists)");
                            downloaded++;
                            continue;
                        }
                    }

                    progressCallback?.Invoke($"Скачивание {modFile.Name}...", 20 + (downloaded * 70 / modFiles.Count));
                    logger?.Info($"Downloading {modFile.Name} ({modFile.Length / 1024 / 1024:F2} MB)");

                    await Task.Run(() =>
                    {
                        using var fileStream = File.Create(localFilePath);
                        client.DownloadFile(modFile.FullName, fileStream);
                    });

                    downloaded++;
                    logger?.Info($"Downloaded {modFile.Name}");
                }

                client.Disconnect();
                
                progressCallback?.Invoke($"Скачано {downloaded} модов", 100);
                logger?.Info($"Successfully downloaded {downloaded} mods");
                return true;
            }
            catch (Exception ex)
            {
                logger?.Error("Failed to download mods via SFTP", ex);
                progressCallback?.Invoke($"Ошибка: {ex.Message}", 0);
                return false;
            }
        }

        /// <summary>
        /// Получить список доступных версий Minecraft с модами
        /// </summary>
        public async Task<List<string>> GetAvailableVersions()
        {
            var versions = new List<string>();
            
            try
            {
                using var client = new SftpClient(sftpHost, sftpPort, sftpUsername, sftpPassword);
                await Task.Run(() => client.Connect());

                if (!client.IsConnected) return versions;

                if (!client.Exists(remoteModsPath)) return versions;

                var directories = client.ListDirectory(remoteModsPath);
                
                foreach (var dir in directories)
                {
                    if (dir.IsDirectory && !dir.Name.StartsWith("."))
                    {
                        versions.Add(dir.Name);
                    }
                }

                client.Disconnect();
                logger?.Info($"Found {versions.Count} available mod versions");
            }
            catch (Exception ex)
            {
                logger?.Error("Failed to get available versions", ex);
            }

            return versions;
        }

        /// <summary>
        /// Проверить подключение к SFTP серверу
        /// </summary>
        public async Task<bool> TestConnection()
        {
            try
            {
                using var client = new SftpClient(sftpHost, sftpPort, sftpUsername, sftpPassword);
                await Task.Run(() => client.Connect());
                
                var connected = client.IsConnected;
                
                if (connected)
                {
                    client.Disconnect();
                    logger?.Info("SFTP connection test successful");
                }
                else
                {
                    logger?.Error("SFTP connection test failed");
                }
                
                return connected;
            }
            catch (Exception ex)
            {
                logger?.Error("SFTP connection test failed", ex);
                return false;
            }
        }

        /// <summary>
        /// Очистить старые моды из папки
        /// </summary>
        private void CleanOldMods(string modsPath)
        {
            try
            {
                if (!Directory.Exists(modsPath)) return;

                var files = Directory.GetFiles(modsPath, "*.jar");
                foreach (var file in files)
                {
                    File.Delete(file);
                    logger?.Debug($"Deleted old mod: {Path.GetFileName(file)}");
                }
            }
            catch (Exception ex)
            {
                logger?.Warning($"Failed to clean old mods: {ex.Message}");
            }
        }

        public void Dispose()
        {
            // Cleanup if needed
        }
    }
}
