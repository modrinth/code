using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    public class MinecraftStarter
    {
        /// <summary>
        /// Запустить Minecraft с указанным профилем
        /// </summary>
        /// <param name="profile">Профиль игры</param>
        /// <returns>True если запуск прошел успешно</returns>
        public async Task<bool> StartMinecraft(GameProfile profile)
        {
            try
            {
                var javaPath = await FindJavaPath();
                if (string.IsNullOrEmpty(javaPath))
                {
                    throw new Exception("Java не найдена на системе");
                }

                var gameDirectory = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), ".owyxlauncher");
                var minecraftJar = Path.Combine(gameDirectory, "versions", profile.MinecraftVersion, $"minecraft-{profile.MinecraftVersion}.jar");

                if (!File.Exists(minecraftJar))
                {
                    throw new Exception($"Файл Minecraft не найден: {minecraftJar}");
                }

                var arguments = BuildMinecraftArguments(profile, gameDirectory, minecraftJar);

                var processStartInfo = new ProcessStartInfo
                {
                    FileName = javaPath,
                    Arguments = arguments,
                    WorkingDirectory = gameDirectory,
                    UseShellExecute = false,
                    CreateNoWindow = false
                };

                using var process = Process.Start(processStartInfo);
                
                // TODO: Мониторинг процесса
                return process != null;
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при запуске Minecraft: {ex.Message}");
                return false;
            }
        }

        /// <summary>
        /// Найти путь к Java на системе
        /// </summary>
        /// <returns>Путь к java.exe или null если не найдена</returns>
        private async Task<string?> FindJavaPath()
        {
            try
            {
                // Проверить JAVA_HOME
                var javaHome = Environment.GetEnvironmentVariable("JAVA_HOME");
                if (!string.IsNullOrEmpty(javaHome))
                {
                    var javaBin = Path.Combine(javaHome, "bin", "java.exe");
                    if (File.Exists(javaBin))
                        return javaBin;
                }

                // Проверить PATH
                var pathDirs = Environment.GetEnvironmentVariable("PATH")?.Split(';') ?? Array.Empty<string>();
                foreach (var dir in pathDirs)
                {
                    if (string.IsNullOrEmpty(dir)) continue;
                    
                    var javaBin = Path.Combine(dir, "java.exe");
                    if (File.Exists(javaBin))
                        return javaBin;
                }

                // Проверить стандартные расположения
                var commonPaths = new[]
                {
                    @"C:\Program Files\Java\",
                    @"C:\Program Files (x86)\Java\",
                    @"C:\Program Files\Eclipse Adoptium\",
                    @"C:\Program Files\Microsoft\jdk-"
                };

                foreach (var basePath in commonPaths)
                {
                    if (!Directory.Exists(basePath)) continue;

                    var javaDirs = Directory.GetDirectories(basePath);
                    foreach (var javaDir in javaDirs)
                    {
                        var javaBin = Path.Combine(javaDir, "bin", "java.exe");
                        if (File.Exists(javaBin))
                            return javaBin;
                    }
                }

                return null;
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при поиске Java: {ex.Message}");
                return null;
            }
        }

        /// <summary>
        /// Построить аргументы командной строки для запуска Minecraft
        /// </summary>
        /// <param name="profile">Профиль игры</param>
        /// <param name="gameDirectory">Директория игры</param>
        /// <param name="minecraftJar">Путь к jar файлу Minecraft</param>
        /// <returns>Строка аргументов</returns>
        private string BuildMinecraftArguments(GameProfile profile, string gameDirectory, string minecraftJar)
        {
            var args = new List<string>();

            // JVM аргументы
            args.Add("-Xmx2G"); // Максимальная память
            args.Add("-Xms1G"); // Начальная память
            args.Add($"-Djava.library.path={Path.Combine(gameDirectory, "versions", profile.MinecraftVersion, "natives")}");
            args.Add($"-cp \"{minecraftJar}\"");

            // Главный класс Minecraft
            args.Add("net.minecraft.client.main.Main");

            // Minecraft аргументы
            args.Add($"--gameDir \"{gameDirectory}\"");
            args.Add($"--assetsDir \"{Path.Combine(gameDirectory, "assets")}\"");
            args.Add($"--version \"{profile.MinecraftVersion}\"");
            args.Add("--username Player");
            args.Add("--uuid 00000000-0000-0000-0000-000000000000");
            args.Add("--accessToken null");
            args.Add("--userType offline");

            return string.Join(" ", args);
        }

        /// <summary>
        /// Проверить доступность Java
        /// </summary>
        /// <returns>True если Java доступна</returns>
        public async Task<bool> IsJavaAvailable()
        {
            var javaPath = await FindJavaPath();
            return !string.IsNullOrEmpty(javaPath);
        }

        /// <summary>
        /// Получить версию Java
        /// </summary>
        /// <returns>Версия Java или null</returns>
        public async Task<string?> GetJavaVersion()
        {
            try
            {
                var javaPath = await FindJavaPath();
                if (string.IsNullOrEmpty(javaPath))
                    return null;

                var processStartInfo = new ProcessStartInfo
                {
                    FileName = javaPath,
                    Arguments = "-version",
                    UseShellExecute = false,
                    RedirectStandardError = true,
                    CreateNoWindow = true
                };

                using var process = Process.Start(processStartInfo);
                if (process == null) return null;

                var output = await process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync();

                // Парсинг версии из вывода
                var lines = output.Split('\n');
                if (lines.Length > 0)
                {
                    return lines[0].Trim();
                }

                return null;
            }
            catch (Exception ex)
            {
                // TODO: Логирование ошибки
                Console.WriteLine($"Ошибка при получении версии Java: {ex.Message}");
                return null;
            }
        }
    }
}