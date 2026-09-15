using System;
using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Threading.Tasks;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    /// <summary>
    /// Компонент для автоматической установки Fabric Mod Loader
    /// </summary>
    public class FabricInstaller : IDisposable
    {
        private readonly HttpClient httpClient;
        private readonly string gameDirectory;
        
        // URLs для загрузки Fabric
        private const string FABRIC_INSTALLER_URL = "https://maven.fabricmc.net/net/fabricmc/fabric-installer/1.0.1/fabric-installer-1.0.1.jar";
        private const string FABRIC_VERSIONS_API = "https://meta.fabricmc.net/v2/versions/loader";
        
        public FabricInstaller(string gameDirectory)
        {
            this.gameDirectory = gameDirectory;
            httpClient = new HttpClient();
            httpClient.Timeout = TimeSpan.FromMinutes(5);
        }
        
        /// <summary>
        /// Проверить, установлен ли Fabric для указанной версии Minecraft
        /// </summary>
        /// <param name="minecraftVersion">Версия Minecraft (например, "1.20.1")</param>
        /// <returns>True если Fabric установлен</returns>
        public bool IsFabricInstalled(string minecraftVersion)
        {
            try
            {
                var versionsDir = Path.Combine(gameDirectory, "versions");
                if (!Directory.Exists(versionsDir))
                    return false;
                
                // Ищем папку с Fabric версией
                var fabricVersionPattern = $"fabric-loader-*-{minecraftVersion}";
                var directories = Directory.GetDirectories(versionsDir, fabricVersionPattern);
                
                return directories.Length > 0;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при проверке Fabric: {ex.Message}");
                return false;
            }
        }
        
        /// <summary>
        /// Установить Fabric для указанной версии Minecraft
        /// </summary>
        /// <param name="minecraftVersion">Версия Minecraft (например, "1.20.1")</param>
        /// <param name="javaPath">Путь к Java исполняемому файлу</param>
        /// <param name="progressCallback">Callback для отслеживания прогресса</param>
        /// <returns>True если установка успешна</returns>
        public async Task<bool> InstallFabric(string minecraftVersion, string javaPath, Action<string>? progressCallback = null)
        {
            try
            {
                progressCallback?.Invoke("Проверка Fabric...");
                
                // Проверить, установлен ли уже
                if (IsFabricInstalled(minecraftVersion))
                {
                    progressCallback?.Invoke("Fabric уже установлен");
                    return true;
                }
                
                progressCallback?.Invoke("Загрузка Fabric Installer...");
                
                // Скачать Fabric Installer
                var installerPath = await DownloadFabricInstaller();
                if (string.IsNullOrEmpty(installerPath))
                {
                    progressCallback?.Invoke("Ошибка загрузки Fabric Installer");
                    return false;
                }
                
                progressCallback?.Invoke("Установка Fabric...");
                
                // Запустить установку Fabric
                var success = await RunFabricInstaller(installerPath, minecraftVersion, javaPath, progressCallback);
                
                // Удалить временный файл установщика
                try
                {
                    if (File.Exists(installerPath))
                        File.Delete(installerPath);
                }
                catch { }
                
                if (success)
                {
                    progressCallback?.Invoke("Fabric успешно установлен!");
                }
                else
                {
                    progressCallback?.Invoke("Ошибка при установке Fabric");
                }
                
                return success;
            }
            catch (Exception ex)
            {
                progressCallback?.Invoke($"Ошибка: {ex.Message}");
                Console.WriteLine($"Ошибка при установке Fabric: {ex.Message}");
                return false;
            }
        }
        
        /// <summary>
        /// Скачать Fabric Installer
        /// </summary>
        /// <returns>Путь к загруженному файлу</returns>
        private async Task<string> DownloadFabricInstaller()
        {
            try
            {
                var tempPath = Path.Combine(Path.GetTempPath(), "fabric-installer.jar");
                
                // Если уже есть, использовать его
                if (File.Exists(tempPath))
                {
                    var fileInfo = new FileInfo(tempPath);
                    // Если файл не старше 7 дней, использовать его
                    if ((DateTime.Now - fileInfo.LastWriteTime).TotalDays < 7)
                        return tempPath;
                }
                
                // Скачать новый
                var response = await httpClient.GetAsync(FABRIC_INSTALLER_URL);
                response.EnsureSuccessStatusCode();
                
                var content = await response.Content.ReadAsByteArrayAsync();
                await File.WriteAllBytesAsync(tempPath, content);
                
                return tempPath;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при загрузке Fabric Installer: {ex.Message}");
                return string.Empty;
            }
        }
        
        /// <summary>
        /// Запустить Fabric Installer
        /// </summary>
        /// <param name="installerPath">Путь к fabric-installer.jar</param>
        /// <param name="minecraftVersion">Версия Minecraft</param>
        /// <param name="javaPath">Путь к Java</param>
        /// <param name="progressCallback">Callback для прогресса</param>
        /// <returns>True если успешно</returns>
        private async Task<bool> RunFabricInstaller(string installerPath, string minecraftVersion, 
            string javaPath, Action<string>? progressCallback)
        {
            try
            {
                var startInfo = new ProcessStartInfo
                {
                    FileName = javaPath,
                    Arguments = $"-jar \"{installerPath}\" client -mcversion {minecraftVersion} -dir \"{gameDirectory}\" -noprofile",
                    UseShellExecute = false,
                    RedirectStandardOutput = true,
                    RedirectStandardError = true,
                    CreateNoWindow = true
                };
                
                using var process = new Process { StartInfo = startInfo };
                
                // Читать вывод в реальном времени
                process.OutputDataReceived += (sender, e) =>
                {
                    if (!string.IsNullOrEmpty(e.Data))
                    {
                        Console.WriteLine($"Fabric: {e.Data}");
                        progressCallback?.Invoke(e.Data);
                    }
                };
                
                process.ErrorDataReceived += (sender, e) =>
                {
                    if (!string.IsNullOrEmpty(e.Data))
                    {
                        Console.WriteLine($"Fabric Error: {e.Data}");
                    }
                };
                
                process.Start();
                process.BeginOutputReadLine();
                process.BeginErrorReadLine();
                
                await process.WaitForExitAsync();
                
                return process.ExitCode == 0;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при запуске Fabric Installer: {ex.Message}");
                return false;
            }
        }
        
        /// <summary>
        /// Получить название Fabric профиля для указанной версии Minecraft
        /// </summary>
        /// <param name="minecraftVersion">Версия Minecraft</param>
        /// <returns>Название профиля или null</returns>
        public string? GetFabricProfileName(string minecraftVersion)
        {
            try
            {
                var versionsDir = Path.Combine(gameDirectory, "versions");
                if (!Directory.Exists(versionsDir))
                    return null;
                
                var directories = Directory.GetDirectories(versionsDir, $"fabric-loader-*-{minecraftVersion}");
                
                if (directories.Length > 0)
                {
                    return Path.GetFileName(directories[0]);
                }
                
                return null;
            }
            catch
            {
                return null;
            }
        }
        
        /// <summary>
        /// Установить Fabric и обновить GameProfile
        /// </summary>
        /// <param name="profile">Игровой профиль</param>
        /// <param name="javaPath">Путь к Java</param>
        /// <param name="progressCallback">Callback для прогресса</param>
        /// <returns>True если успешно</returns>
        public async Task<bool> InstallAndSetupFabric(GameProfile profile, string javaPath, 
            Action<string>? progressCallback = null)
        {
            try
            {
                // Установить Fabric
                var success = await InstallFabric(profile.MinecraftVersion, javaPath, progressCallback);
                
                if (success)
                {
                    // Обновить профиль с Fabric версией
                    var fabricProfileName = GetFabricProfileName(profile.MinecraftVersion);
                    if (!string.IsNullOrEmpty(fabricProfileName))
                    {
                        profile.MinecraftVersion = fabricProfileName;
                        progressCallback?.Invoke($"Профиль обновлен: {fabricProfileName}");
                    }
                }
                
                return success;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при установке и настройке Fabric: {ex.Message}");
                return false;
            }
        }
        
        public void Dispose()
        {
            httpClient?.Dispose();
        }
    }
}
