using System;
using System.IO;
using System.Text;

namespace MinecraftLauncher
{
    /// <summary>
    /// Компонент для записи логов лаунчера
    /// </summary>
    public class Logger
    {
        private readonly string logDirectory;
        private readonly string currentLogFile;
        private static Logger instance;
        private readonly object lockObject = new object();

        public enum LogLevel
        {
            Info,
            Warning,
            Error,
            Debug
        }

        private Logger(string gameDirectory)
        {
            logDirectory = Path.Combine(gameDirectory, "logs");
            Directory.CreateDirectory(logDirectory);

            // Создать файл лога с датой и временем
            var timestamp = DateTime.Now.ToString("yyyy-MM-dd_HH-mm-ss");
            currentLogFile = Path.Combine(logDirectory, $"launcher_{timestamp}.log");

            // Записать заголовок
            WriteToFile($"==================================================");
            WriteToFile($"Owyx Launcher - Log Started at {DateTime.Now:yyyy-MM-dd HH:mm:ss}");
            WriteToFile($"==================================================");
        }

        /// <summary>
        /// Получить экземпляр логгера (Singleton)
        /// </summary>
        public static Logger GetInstance(string gameDirectory)
        {
            if (instance == null)
            {
                instance = new Logger(gameDirectory);
            }
            return instance;
        }

        /// <summary>
        /// Записать информационное сообщение
        /// </summary>
        public void Info(string message)
        {
            Log(LogLevel.Info, message);
        }

        /// <summary>
        /// Записать предупреждение
        /// </summary>
        public void Warning(string message)
        {
            Log(LogLevel.Warning, message);
        }

        /// <summary>
        /// Записать ошибку
        /// </summary>
        public void Error(string message, Exception ex = null)
        {
            var fullMessage = message;
            if (ex != null)
            {
                fullMessage += $"\nException: {ex.GetType().Name}\nMessage: {ex.Message}\nStackTrace:\n{ex.StackTrace}";
            }
            Log(LogLevel.Error, fullMessage);
        }

        /// <summary>
        /// Записать отладочное сообщение
        /// </summary>
        public void Debug(string message)
        {
            Log(LogLevel.Debug, message);
        }

        /// <summary>
        /// Основной метод логирования
        /// </summary>
        private void Log(LogLevel level, string message)
        {
            try
            {
                var timestamp = DateTime.Now.ToString("HH:mm:ss.fff");
                var levelString = level.ToString().ToUpper().PadRight(7);
                var logMessage = $"[{timestamp}] [{levelString}] {message}";

                // Записать в файл (thread-safe)
                WriteToFile(logMessage);

                // Также вывести в консоль (если запущено в режиме отладки)
                Console.WriteLine(logMessage);
            }
            catch (Exception ex)
            {
                // Если не удалось записать лог, вывести в консоль
                Console.WriteLine($"Failed to write log: {ex.Message}");
            }
        }

        /// <summary>
        /// Записать строку в файл (thread-safe)
        /// </summary>
        private void WriteToFile(string message)
        {
            lock (lockObject)
            {
                try
                {
                    File.AppendAllText(currentLogFile, message + Environment.NewLine, Encoding.UTF8);
                }
                catch
                {
                    // Игнорировать ошибки записи лога
                }
            }
        }

        /// <summary>
        /// Очистить старые логи (оставить только последние N дней)
        /// </summary>
        public void CleanOldLogs(int daysToKeep = 7)
        {
            try
            {
                var files = Directory.GetFiles(logDirectory, "launcher_*.log");
                var cutoffDate = DateTime.Now.AddDays(-daysToKeep);

                foreach (var file in files)
                {
                    var fileInfo = new FileInfo(file);
                    if (fileInfo.LastWriteTime < cutoffDate)
                    {
                        File.Delete(file);
                        Info($"Deleted old log file: {Path.GetFileName(file)}");
                    }
                }
            }
            catch (Exception ex)
            {
                Error("Failed to clean old logs", ex);
            }
        }

        /// <summary>
        /// Получить путь к текущему файлу лога
        /// </summary>
        public string GetCurrentLogFile()
        {
            return currentLogFile;
        }

        /// <summary>
        /// Получить все файлы логов
        /// </summary>
        public string[] GetAllLogFiles()
        {
            try
            {
                return Directory.GetFiles(logDirectory, "launcher_*.log");
            }
            catch
            {
                return Array.Empty<string>();
            }
        }
    }
}
