using System;
using System.IO;
using System.Net.Http;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    public class GameUpdater : IDisposable
    {
        private readonly HttpClient httpClient;
        private const string SERVER_VERSION_URL = "http://yourserver.com/api/version";

        public GameUpdater()
        {
            httpClient = new HttpClient();
            httpClient.Timeout = TimeSpan.FromMinutes(5);
        }

        /// <summary>
        /// Получить информацию о версии с сервера
        /// </summary>
        /// <returns>Информация о сервере</returns>
        public async Task<ServerInfo?> GetServerVersion()
        {
            try
            {
                var response = await httpClient.GetAsync(SERVER_VERSION_URL);
                response.EnsureSuccessStatusCode();
                
                var jsonContent = await response.Content.ReadAsStringAsync();
                return JsonSerializer.Deserialize<ServerInfo>(jsonContent);
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при получении версии сервера: {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Загрузить файл с сервера
        /// </summary>
        /// <param name="url">URL файла</param>
        /// <param name="localPath">Путь для сохранения</param>
        /// <param name="progress">Прогресс загрузки</param>
        public async Task DownloadFile(string url, string localPath, IProgress<int>? progress = null)
        {
            try
            {
                // Создать директорию если не существует
                var directory = Path.GetDirectoryName(localPath);
                if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
                {
                    Directory.CreateDirectory(directory);
                }

                using var response = await httpClient.GetAsync(url, HttpCompletionOption.ResponseHeadersRead);
                response.EnsureSuccessStatusCode();

                var totalBytes = response.Content.Headers.ContentLength ?? -1L;
                var downloadedBytes = 0L;

                using var contentStream = await response.Content.ReadAsStreamAsync();
                using var fileStream = new FileStream(localPath, FileMode.Create, FileAccess.Write, FileShare.None);

                var buffer = new byte[8192];
                int bytesRead;

                while ((bytesRead = await contentStream.ReadAsync(buffer, 0, buffer.Length)) > 0)
                {
                    await fileStream.WriteAsync(buffer, 0, bytesRead);
                    downloadedBytes += bytesRead;

                    if (totalBytes > 0 && progress != null)
                    {
                        var percentage = (int)((downloadedBytes * 100) / totalBytes);
                        progress.Report(percentage);
                    }
                }
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при загрузке файла {url}: {ex.Message}");
                throw;
            }
        }

        /// <summary>
        /// Вычислить SHA256 хеш файла
        /// </summary>
        /// <param name="filePath">Путь к файлу</param>
        /// <returns>SHA256 хеш в виде строки</returns>
        public string CalculateFileHash(string filePath)
        {
            try
            {
                if (!File.Exists(filePath))
                    return string.Empty;

                using var sha256 = SHA256.Create();
                using var fileStream = File.OpenRead(filePath);
                var hashBytes = sha256.ComputeHash(fileStream);
                return Convert.ToHexString(hashBytes).ToLowerInvariant();
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при вычислении хеша файла {filePath}: {ex.Message}");
                return string.Empty;
            }
        }

        /// <summary>
        /// Проверить локальные файлы на соответствие серверным
        /// </summary>
        /// <param name="serverInfo">Информация о версии сервера</param>
        /// <returns>True если все файлы валидны</returns>
        public async Task<bool> ValidateLocalFiles(ServerInfo serverInfo)
        {
            try
            {
                var gameDirectory = GetGameDirectory();

                // Проверить файлы клиента
                foreach (var file in serverInfo.ClientFiles)
                {
                    var localPath = Path.Combine(gameDirectory, "versions", serverInfo.MinecraftVersion, file.Name);
                    if (!File.Exists(localPath))
                        return false;

                    var localHash = CalculateFileHash(localPath);
                    if (localHash != file.Hash.ToLowerInvariant())
                        return false;
                }

                // Проверить моды
                foreach (var mod in serverInfo.Mods)
                {
                    var localPath = Path.Combine(gameDirectory, "mods", mod.Name);
                    if (!File.Exists(localPath))
                        return false;

                    var localHash = CalculateFileHash(localPath);
                    if (localHash != mod.Hash.ToLowerInvariant())
                        return false;
                }

                return true;
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при проверке локальных файлов: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Получить директорию игры
        /// </summary>
        /// <returns>Путь к директории игры</returns>
        private string GetGameDirectory()
        {
            var appDataPath = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            return Path.Combine(appDataPath, ".owyxlauncher");
        }

        public void Dispose()
        {
            httpClient?.Dispose();
        }
    }
}