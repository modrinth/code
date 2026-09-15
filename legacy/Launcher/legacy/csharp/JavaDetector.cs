using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading.Tasks;

namespace MinecraftLauncher
{
    /// <summary>
    /// Класс для обнаружения установленных версий Java
    /// </summary>
    public class JavaDetector
    {
        public class JavaInstallation
        {
            public string Path { get; set; } = string.Empty;
            public string Version { get; set; } = string.Empty;
            public int MajorVersion { get; set; }
            public bool Is64Bit { get; set; }
            public string DisplayName => $"Java {Version} ({(Is64Bit ? "64-bit" : "32-bit")})";
        }

        /// <summary>
        /// Найти все установленные версии Java
        /// </summary>
        public async Task<List<JavaInstallation>> FindAllJavaInstallations()
        {
            var installations = new List<JavaInstallation>();
            var foundPaths = new HashSet<string>();

            // Проверить JAVA_HOME
            await CheckJavaHome(installations, foundPaths);

            // Проверить PATH
            await CheckPathEnvironment(installations, foundPaths);

            // Проверить стандартные расположения
            await CheckCommonLocations(installations, foundPaths);

            // Проверить реестр Windows
            await CheckWindowsRegistry(installations, foundPaths);

            return installations.OrderByDescending(j => j.MajorVersion).ToList();
        }

        /// <summary>
        /// Найти последнюю (самую новую) версию Java
        /// </summary>
        public async Task<JavaInstallation?> FindLatestJava()
        {
            var installations = await FindAllJavaInstallations();
            return installations.FirstOrDefault();
        }

        /// <summary>
        /// Найти рекомендуемую версию Java для Minecraft
        /// </summary>
        public async Task<JavaInstallation?> FindRecommendedJava(string minecraftVersion = "")
        {
            var installations = await FindAllJavaInstallations();
            
            // Для Minecraft 1.17+ рекомендуется Java 17
            // Для Minecraft 1.16 и ниже - Java 8
            var recommendedMajor = 17;
            
            // Попытаться найти рекомендуемую версию
            var recommended = installations.FirstOrDefault(j => j.MajorVersion == recommendedMajor);
            if (recommended != null) return recommended;

            // Если не найдена, взять последнюю совместимую
            return installations.FirstOrDefault(j => j.MajorVersion >= 8);
        }

        private async Task CheckJavaHome(List<JavaInstallation> installations, HashSet<string> foundPaths)
        {
            var javaHome = Environment.GetEnvironmentVariable("JAVA_HOME");
            if (!string.IsNullOrEmpty(javaHome))
            {
                var javaBin = Path.Combine(javaHome, "bin", "java.exe");
                if (File.Exists(javaBin) && !foundPaths.Contains(javaBin))
                {
                    var info = await GetJavaInfo(javaBin);
                    if (info != null)
                    {
                        installations.Add(info);
                        foundPaths.Add(javaBin);
                    }
                }
            }
        }

        private async Task CheckPathEnvironment(List<JavaInstallation> installations, HashSet<string> foundPaths)
        {
            var pathDirs = Environment.GetEnvironmentVariable("PATH")?.Split(';') ?? Array.Empty<string>();
            foreach (var dir in pathDirs)
            {
                if (string.IsNullOrEmpty(dir)) continue;

                var javaBin = Path.Combine(dir, "java.exe");
                if (File.Exists(javaBin) && !foundPaths.Contains(javaBin))
                {
                    var info = await GetJavaInfo(javaBin);
                    if (info != null)
                    {
                        installations.Add(info);
                        foundPaths.Add(javaBin);
                    }
                }
            }
        }

        private async Task CheckCommonLocations(List<JavaInstallation> installations, HashSet<string> foundPaths)
        {
            var commonPaths = new[]
            {
                @"C:\Program Files\Java",
                @"C:\Program Files (x86)\Java",
                @"C:\Program Files\Eclipse Adoptium",
                @"C:\Program Files\Eclipse Foundation",
                @"C:\Program Files\Microsoft",
                @"C:\Program Files\BellSoft",
                @"C:\Program Files\Amazon Corretto",
                @"C:\Program Files\Zulu",
                Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Programs", "Eclipse Adoptium")
            };

            foreach (var basePath in commonPaths)
            {
                if (!Directory.Exists(basePath)) continue;

                try
                {
                    var javaDirs = Directory.GetDirectories(basePath, "*", SearchOption.TopDirectoryOnly);
                    foreach (var javaDir in javaDirs)
                    {
                        var javaBin = Path.Combine(javaDir, "bin", "java.exe");
                        if (File.Exists(javaBin) && !foundPaths.Contains(javaBin))
                        {
                            var info = await GetJavaInfo(javaBin);
                            if (info != null)
                            {
                                installations.Add(info);
                                foundPaths.Add(javaBin);
                            }
                        }
                    }
                }
                catch
                {
                    // Игнорировать ошибки доступа к директориям
                }
            }
        }

        private async Task CheckWindowsRegistry(List<JavaInstallation> installations, HashSet<string> foundPaths)
        {
            try
            {
                // Проверить HKEY_LOCAL_MACHINE\SOFTWARE\JavaSoft\Java Runtime Environment
                // Проверить HKEY_LOCAL_MACHINE\SOFTWARE\JavaSoft\Java Development Kit
                // Это требует работы с реестром, что может быть ограничено правами
                // Пока пропустим, так как основные методы уже покрывают большинство случаев
                await Task.CompletedTask;
            }
            catch
            {
                // Игнорировать ошибки доступа к реестру
            }
        }

        /// <summary>
        /// Получить информацию о версии Java
        /// </summary>
        private async Task<JavaInstallation?> GetJavaInfo(string javaPath)
        {
            try
            {
                var startInfo = new ProcessStartInfo
                {
                    FileName = javaPath,
                    Arguments = "-version",
                    UseShellExecute = false,
                    RedirectStandardError = true,
                    RedirectStandardOutput = true,
                    CreateNoWindow = true
                };

                using var process = Process.Start(startInfo);
                if (process == null) return null;

                var output = await process.StandardError.ReadToEndAsync();
                output += await process.StandardOutput.ReadToEndAsync();
                
                await process.WaitForExitAsync();

                if (string.IsNullOrEmpty(output)) return null;

                // Парсинг версии из вывода
                // Формат: java version "1.8.0_XXX" или openjdk version "17.0.X"
                var version = ParseJavaVersion(output);
                if (string.IsNullOrEmpty(version)) return null;

                var majorVersion = GetMajorVersion(version);
                var is64Bit = output.Contains("64-Bit") || output.Contains("64-bit");

                return new JavaInstallation
                {
                    Path = javaPath,
                    Version = version,
                    MajorVersion = majorVersion,
                    Is64Bit = is64Bit
                };
            }
            catch
            {
                return null;
            }
        }

        private string ParseJavaVersion(string output)
        {
            // Ищем строку с версией
            var lines = output.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
            foreach (var line in lines)
            {
                if (line.Contains("version"))
                {
                    var start = line.IndexOf('"');
                    var end = line.LastIndexOf('"');
                    if (start >= 0 && end > start)
                    {
                        return line.Substring(start + 1, end - start - 1);
                    }
                }
            }
            return string.Empty;
        }

        private int GetMajorVersion(string version)
        {
            try
            {
                // Для версий типа "1.8.0_XXX" major version = 8
                // Для версий типа "17.0.X" major version = 17
                var parts = version.Split('.');
                if (parts.Length > 0)
                {
                    if (parts[0] == "1" && parts.Length > 1)
                    {
                        return int.Parse(parts[1].Split('_')[0]);
                    }
                    return int.Parse(parts[0]);
                }
            }
            catch
            {
                // Игнорировать ошибки парсинга
            }
            return 0;
        }
    }
}
