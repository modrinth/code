using System;
using System.Diagnostics;
using System.IO;
using System.Threading.Tasks;
using System.Windows.Forms;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    public partial class MainForm : Form
    {
        private GameUpdater updater;
        private MinecraftStarter starter;
        private ConfigManager configManager;
        private ServerStatusChecker serverChecker;
        private DiscordRPCManager discordRPC;
        private Logger logger;
        private ModDownloader modDownloader;
        private bool isUpdating = false;
        private CustomTitleBar titleBar;
        private SettingsPanel settingsPanel;
        private System.Windows.Forms.Timer serverRefreshTimer;

        private const string SERVER_ADDRESS = "play.chiwawa.site";
        private const int SERVER_PORT = 32062;
        
        // SFTP настройки для модов
        private const string SFTP_HOST = "192.168.1.52";
        private const int SFTP_PORT = 22;
        private const string SFTP_USERNAME = "root";
        private const string SFTP_PASSWORD = ""; // stripped — use env/secrets, never commit
        private const string SFTP_MODS_PATH = "/opt/MCMods/mods";

        public MainForm()
        {
            InitializeComponent();
            InitializeServices();
            SetupForm();
        }

        /// <summary>
        /// Инициализация сервисов
        /// </summary>
        private void InitializeServices()
        {
            updater = new GameUpdater();
            starter = new MinecraftStarter();
            configManager = new ConfigManager();
            serverChecker = new ServerStatusChecker(SERVER_ADDRESS, SERVER_PORT);
            discordRPC = new DiscordRPCManager();
            
            // Инициализировать Logger
            var gameDirectory = configManager.GetGameDirectory();
            logger = Logger.GetInstance(gameDirectory);
            logger.Info("=== Owyx Launcher Started ===");
            logger.CleanOldLogs(7); // Удалить логи старше 7 дней
            
            // Инициализировать ModDownloader для SFTP
            modDownloader = new ModDownloader(SFTP_HOST, SFTP_PORT, SFTP_USERNAME, SFTP_PASSWORD, SFTP_MODS_PATH, logger);
            logger.Info($"ModDownloader initialized for SFTP: {SFTP_HOST}:{SFTP_PORT}");
            
            // Таймер для авто-обновления сервера каждую минуту
            serverRefreshTimer = new System.Windows.Forms.Timer();
            serverRefreshTimer.Interval = 60000; // 60 секунд
            serverRefreshTimer.Tick += async (sender, e) => await RefreshServerStatus();
        }

        /// <summary>
        /// Настройка формы
        /// </summary>
        private async void SetupForm()
        {
            // Добавить CustomTitleBar
            titleBar = new CustomTitleBar(this);
            titleBar.Location = new System.Drawing.Point(0, 0);
            this.Controls.Add(titleBar);
            titleBar.BringToFront();

            // Загрузить профиль и создать SettingsPanel
            var profile = await configManager.LoadGameProfile();
            settingsPanel = new SettingsPanel(configManager, profile);
            settingsPanel.Visible = false;
            settingsPanel.Location = new System.Drawing.Point(0, 40);  // Под title bar
            settingsPanel.Size = new System.Drawing.Size(this.ClientSize.Width, this.ClientSize.Height - 40);
            settingsPanel.Anchor = AnchorStyles.Top | AnchorStyles.Bottom | AnchorStyles.Left | AnchorStyles.Right;
            this.Controls.Add(settingsPanel);
            settingsPanel.BringToFront();

            // Form properties are now set in Designer
            // Just set initial status
            statusLabel.Text = "Готово к игре";
            statusDetailLabel.Text = "";
            progressBar.Value = 0;
            
            // Автоматическая проверка статуса сервера при запуске
            await RefreshServerStatus();
            
            // Запустить таймер авто-обновления
            serverRefreshTimer.Start();
        }

        /// <summary>
        /// Обновить статус сервера
        /// </summary>
        private async Task RefreshServerStatus()
        {
            try
            {
                refreshServerButton.Enabled = false;
                serverOnlineLabel.Text = "👥 Проверка...";
                serverVersionLabel.Text = "📦 Версия: Загрузка...";
                serverPingLabel.Text = "📡 Пинг: -- ms";
                logger.Debug("Refreshing server status...");

                var status = await serverChecker.GetServerStatus();

                if (status.IsOnline)
                {
                    serverOnlineLabel.Text = $"👥 Онлайн: {status.OnlinePlayers}/{status.MaxPlayers}";
                    serverOnlineLabel.ForeColor = System.Drawing.Color.FromArgb(100, 200, 100);
                    serverVersionLabel.Text = $"📦 Версия: {status.Version}";
                    serverPingLabel.Text = $"📡 Пинг: {status.Ping} ms";
                    logger.Info($"Server online: {status.OnlinePlayers}/{status.MaxPlayers}, Version: {status.Version}, Ping: {status.Ping}ms");
                    
                    // Обновить Discord RPC с информацией о сервере
                    discordRPC?.SetPlayingState("ChiwawaMine", status.OnlinePlayers, status.MaxPlayers);
                }
                else
                {
                    serverOnlineLabel.Text = "❌ Сервер оффлайн";
                    serverOnlineLabel.ForeColor = System.Drawing.Color.FromArgb(200, 100, 100);
                    serverVersionLabel.Text = "📦 Версия: Недоступно";
                    serverPingLabel.Text = "📡 Пинг: -- ms";
                    logger.Warning("Server is offline");
                }
            }
            catch (Exception ex)
            {
                serverOnlineLabel.Text = "⚠️ Ошибка проверки";
                serverOnlineLabel.ForeColor = System.Drawing.Color.FromArgb(200, 150, 100);
                logger.Error("Failed to check server status", ex);
                Console.WriteLine($"Ошибка при проверке статуса сервера: {ex.Message}");
            }
            finally
            {
                refreshServerButton.Enabled = true;
            }
        }

        /// <summary>
        /// Обработчик кнопки обновления статуса
        /// </summary>
        private async void RefreshServerButton_Click(object sender, EventArgs e)
        {
            await RefreshServerStatus();
        }

        /// <summary>
        /// Обработчик нажатия кнопки "Играть"
        /// </summary>
        private async void PlayButton_Click(object sender, EventArgs e)
        {
            if (isUpdating) return;

            try
            {
                isUpdating = true;
                playButton.Enabled = false;
                playButton.Text = "Обновление...";
                logger.Info("Play button clicked - starting game launch sequence");

                await CheckForUpdates();

                // Загрузить профиль и запустить игру
                UpdateDetailedStatus("Загрузка профиля игры...");
                var profile = await configManager.LoadGameProfile();
                logger.Info($"Game profile loaded: {profile.Name}, Version: {profile.MinecraftVersion}");
                
                UpdateDetailedStatus("Запуск Minecraft...");
                var success = await starter.StartMinecraft(profile);

                if (success)
                {
                    UpdateProgress(100, "Игра запущена!");
                    UpdateDetailedStatus("Minecraft успешно запущен");
                    logger.Info("Minecraft started successfully");
                    
                    // Обновить Discord RPC - игрок в игре
                    var serverStatus = await serverChecker.GetServerStatus();
                    if (serverStatus.IsOnline)
                    {
                        discordRPC?.SetPlayingState("ChiwawaMine", serverStatus.OnlinePlayers, serverStatus.MaxPlayers);
                    }
                    
                    this.WindowState = FormWindowState.Minimized;
                }
                else
                {
                    UpdateProgress(0, "Ошибка при запуске игры");
                    UpdateDetailedStatus("Не удалось запустить Minecraft");
                    logger.Error("Failed to start Minecraft");
                }
            }
            catch (Exception ex)
            {
                UpdateProgress(0, $"Ошибка: {ex.Message}");
                UpdateDetailedStatus("Критическая ошибка при запуске");
                logger.Error("Exception during game launch", ex);
                MessageBox.Show($"Ошибка при запуске игры:\n{ex.Message}", "Ошибка", 
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
            finally
            {
                isUpdating = false;
                playButton.Enabled = true;
                playButton.Text = "⚡ ИГРАТЬ";
            }
        }

        /// <summary>
        /// Обработчик кнопки настроек
        /// </summary>
        private async void SettingsButton_Click(object sender, EventArgs e)
        {
            try
            {
                // Показать/скрыть панель настроек
                settingsPanel.Visible = !settingsPanel.Visible;
                if (settingsPanel.Visible)
                {
                    settingsPanel.BringToFront();
                    settingsButton.Text = "❌ Закрыть настройки";
                    discordRPC?.SetSettingsState();
                }
                else
                {
                    settingsButton.Text = "⚙️ Настройки";
                    discordRPC?.SetIdleState();
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Ошибка при открытии настроек:\n{ex.Message}", "Ошибка", 
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        /// <summary>
        /// Обработчик кнопки открытия папки
        /// </summary>
        private void OpenFolderButton_Click(object sender, EventArgs e)
        {
            try
            {
                var gameDirectory = configManager.GetGameDirectory();
                
                if (!Directory.Exists(gameDirectory))
                {
                    Directory.CreateDirectory(gameDirectory);
                }

                Process.Start(new ProcessStartInfo
                {
                    FileName = gameDirectory,
                    UseShellExecute = true,
                    Verb = "open"
                });
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Ошибка при открытии папки:\n{ex.Message}", "Ошибка", 
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        /// <summary>
        /// Проверка обновлений
        /// </summary>
        private async Task CheckForUpdates()
        {
            UpdateProgress(10, "Проверка версии...");
            UpdateDetailedStatus("Подключение к серверу обновлений...");
            logger.Info("Starting update check");

            var profile = await configManager.LoadGameProfile();
            var minecraftVersion = profile.MinecraftVersion;

            UpdateProgress(30, "Проверка локальных файлов...");
            UpdateDetailedStatus("Сканирование установленных файлов...");
            logger.Info($"Minecraft version: {minecraftVersion}");

            // Скачать моды через SFTP
            UpdateProgress(50, "Загрузка модов...");
            UpdateDetailedStatus("Подключение к серверу модов через SFTP...");
            logger.Info("Starting mod download");

            var gameDirectory = configManager.GetGameDirectory();
            var modsPath = Path.Combine(gameDirectory, "mods");

            var modDownloadSuccess = await modDownloader.DownloadModsForVersion(
                minecraftVersion, 
                modsPath,
                (message, progress) =>
                {
                    var totalProgress = 50 + (progress * 40 / 100);
                    UpdateProgress(totalProgress, "Загрузка модов...");
                    UpdateDetailedStatus(message);
                });

            if (!modDownloadSuccess)
            {
                logger.Warning("Mod download failed, continuing anyway");
                UpdateDetailedStatus("Не удалось скачать моды (продолжаем без них)");
            }

            UpdateProgress(100, "Готово к игре");
            UpdateDetailedStatus("Все файлы актуальны");
            logger.Info("Update check completed");
        }

        /// <summary>
        /// Обновление прогресса и статуса
        /// </summary>
        /// <param name="percentage">Процент выполнения</param>
        /// <param name="status">Статус операции</param>
        private void UpdateProgress(int percentage, string status)
        {
            if (InvokeRequired)
            {
                Invoke(new Action<int, string>(UpdateProgress), percentage, status);
                return;
            }

            progressBar.Value = Math.Min(Math.Max(percentage, 0), 100);
            statusLabel.Text = status;
            logger?.Info($"Progress: {percentage}% - {status}");
        }

        /// <summary>
        /// Обновление детального статуса (под прогресс-баром)
        /// </summary>
        /// <param name="detail">Детальное описание</param>
        private void UpdateDetailedStatus(string detail)
        {
            if (InvokeRequired)
            {
                Invoke(new Action<string>(UpdateDetailedStatus), detail);
                return;
            }

            statusDetailLabel.Text = detail;
            logger?.Debug($"Detail: {detail}");
        }

        /// <summary>
        /// Освобождение ресурсов
        /// </summary>
        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                components?.Dispose();
                updater?.Dispose();
                discordRPC?.Dispose();
                serverRefreshTimer?.Dispose();
                logger?.Info("=== Owyx Launcher Closed ===");
            }
            base.Dispose(disposing);
        }
    }
}
