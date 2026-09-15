using System;
using System.Drawing;
using System.Windows.Forms;
using System.Threading.Tasks;
using System.Linq;
using MinecraftLauncher.Models;

namespace MinecraftLauncher
{
    /// <summary>
    /// Панель настроек внутри главного окна
    /// </summary>
    public class SettingsPanel : Panel
    {
        private ConfigManager configManager;
        private JavaDetector javaDetector;
        private GameProfile currentProfile;
        
        // Цвета Owyx дизайна
        private readonly Color BackgroundColor = Color.FromArgb(18, 18, 24);
        private readonly Color SecondaryBackgroundColor = Color.FromArgb(24, 24, 32);
        private readonly Color AccentColor = Color.FromArgb(138, 43, 226);
        private readonly Color AccentHoverColor = Color.FromArgb(148, 53, 236);
        private readonly Color TextColor = Color.FromArgb(240, 240, 245);
        private readonly Color SecondaryTextColor = Color.FromArgb(150, 150, 160);
        private readonly Color BorderColor = Color.FromArgb(60, 60, 70);

        public event EventHandler SettingsClosed;
        public event EventHandler SettingsSaved;

        public SettingsPanel(ConfigManager config, GameProfile profile)
        {
            configManager = config;
            currentProfile = profile;
            javaDetector = new JavaDetector();
            
            InitializeComponent();
            SetupDesign();
            LoadSettings();
        }

        private void InitializeComponent()
        {
            this.Dock = DockStyle.Fill;
            this.BackColor = BackgroundColor;
            this.Visible = false;
        }

        private void SetupDesign()
        {
            this.Controls.Clear();

            // Создать скроллируемую панель
            var scrollPanel = new Panel
            {
                Dock = DockStyle.Fill,
                AutoScroll = true,
                BackColor = BackgroundColor
            };
            this.Controls.Add(scrollPanel);

            var contentPanel = new Panel
            {
                Location = new Point(0, 0),
                Size = new Size(760, 800),
                BackColor = BackgroundColor,
                AutoSize = true
            };
            scrollPanel.Controls.Add(contentPanel);

            int yPos = 20;

            // Заголовок
            var titleLabel = new Label
            {
                Text = "⚙️ НАСТРОЙКИ",
                Font = new Font("Segoe UI", 18, FontStyle.Bold),
                ForeColor = TextColor,
                Location = new Point(30, yPos),
                Size = new Size(700, 40)
            };
            contentPanel.Controls.Add(titleLabel);
            yPos += 60;

            // Кнопка закрытия
            var closeButton = new Button
            {
                Text = "←",
                Font = new Font("Segoe UI", 16, FontStyle.Bold),
                ForeColor = TextColor,
                BackColor = SecondaryBackgroundColor,
                FlatStyle = FlatStyle.Flat,
                Size = new Size(50, 50),
                Location = new Point(contentPanel.Width - 80, 20),
                Cursor = Cursors.Hand
            };
            closeButton.FlatAppearance.BorderSize = 0;
            closeButton.Click += (s, e) => Hide();
            contentPanel.Controls.Add(closeButton);

            // Панель настроек Java
            yPos = CreateJavaSettingsPanel(contentPanel, yPos);
            yPos += 20;

            // Панель настроек памяти
            yPos = CreateMemorySettingsPanel(contentPanel, yPos);
            yPos += 20;

            // Панель настроек окна игры
            yPos = CreateGameWindowSettingsPanel(contentPanel, yPos);
            yPos += 20;

            // Панель аргументов
            yPos = CreateArgumentsPanel(contentPanel, yPos);
            yPos += 20;

            // Панель Discord RPC
            yPos = CreateDiscordRPCPanel(contentPanel, yPos);
            yPos += 20;

            // Кнопка сохранения
            var saveButton = new Button
            {
                Text = "💾 Сохранить настройки",
                Location = new Point(30, yPos),
                Size = new Size(700, 50),
                BackColor = AccentColor,
                ForeColor = TextColor,
                Font = new Font("Segoe UI", 12, FontStyle.Bold),
                FlatStyle = FlatStyle.Flat,
                Cursor = Cursors.Hand
            };
            saveButton.FlatAppearance.BorderSize = 0;
            saveButton.Click += SaveButton_Click;
            saveButton.MouseEnter += (s, e) => saveButton.BackColor = AccentHoverColor;
            saveButton.MouseLeave += (s, e) => saveButton.BackColor = AccentColor;
            contentPanel.Controls.Add(saveButton);

            contentPanel.Height = yPos + 70;
        }

        private int CreateJavaSettingsPanel(Panel parent, int yPos)
        {
            var panel = CreateSectionPanel(parent, yPos, 120, "☕ Java");

            var autoDetectCheckBox = new CheckBox
            {
                Name = "autoDetectJavaCheckBox",
                Text = "Автоматически определять Java",
                ForeColor = TextColor,
                Location = new Point(15, 40),
                Size = new Size(300, 25),
                Checked = currentProfile.AutoDetectJava
            };
            autoDetectCheckBox.CheckedChanged += AutoDetectJava_CheckedChanged;
            panel.Controls.Add(autoDetectCheckBox);

            var javaPathLabel = new Label
            {
                Text = "Путь к Java:",
                ForeColor = SecondaryTextColor,
                Location = new Point(15, 70),
                Size = new Size(100, 20)
            };
            panel.Controls.Add(javaPathLabel);

            var javaPathTextBox = new TextBox
            {
                Name = "javaPathTextBox",
                Location = new Point(120, 68),
                Size = new Size(470, 23),
                BackColor = BackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle,
                Text = currentProfile.JavaPath,
                Enabled = !currentProfile.AutoDetectJava
            };
            panel.Controls.Add(javaPathTextBox);

            var browseButton = new Button
            {
                Name = "browseJavaButton",
                Text = "📁",
                Location = new Point(595, 67),
                Size = new Size(60, 25),
                BackColor = SecondaryBackgroundColor,
                ForeColor = TextColor,
                FlatStyle = FlatStyle.Flat,
                Enabled = !currentProfile.AutoDetectJava,
                Cursor = Cursors.Hand
            };
            browseButton.FlatAppearance.BorderColor = BorderColor;
            browseButton.Click += BrowseJava_Click;
            panel.Controls.Add(browseButton);

            var detectButton = new Button
            {
                Name = "detectJavaButton",
                Text = "🔍",
                Location = new Point(660, 67),
                Size = new Size(60, 25),
                BackColor = AccentColor,
                ForeColor = TextColor,
                FlatStyle = FlatStyle.Flat,
                Cursor = Cursors.Hand
            };
            detectButton.FlatAppearance.BorderSize = 0;
            detectButton.Click += DetectJava_Click;
            panel.Controls.Add(detectButton);

            // Показать версию Java
            var javaVersionLabel = new Label
            {
                Name = "javaVersionLabel",
                Text = "Определяем версию Java...",
                ForeColor = SecondaryTextColor,
                Font = new Font("Segoe UI", 9, FontStyle.Italic),
                Location = new Point(120, 95),
                Size = new Size(600, 20)
            };
            panel.Controls.Add(javaVersionLabel);
            
            // Асинхронно определить версию Java
            Task.Run(async () =>
            {
                var version = await GetJavaVersion(currentProfile.JavaPath);
                if (javaVersionLabel.IsHandleCreated)
                {
                    javaVersionLabel.Invoke(new Action(() =>
                    {
                        javaVersionLabel.Text = version;
                    }));
                }
            });

            return yPos + 150;
        }

        private int CreateMemorySettingsPanel(Panel parent, int yPos)
        {
            var panel = CreateSectionPanel(parent, yPos, 100, "💾 Память (RAM)");

            var memoryLabel = new Label
            {
                Text = "Выделено памяти:",
                ForeColor = SecondaryTextColor,
                Location = new Point(15, 45),
                Size = new Size(120, 20)
            };
            panel.Controls.Add(memoryLabel);

            var memoryTrackBar = new TrackBar
            {
                Name = "memoryTrackBar",
                Location = new Point(140, 40),
                Size = new Size(320, 45),
                Minimum = 512,
                Maximum = 16384,
                TickFrequency = 512,
                Value = currentProfile.AllocatedMemoryMB
            };
            memoryTrackBar.ValueChanged += MemoryTrackBar_ValueChanged;
            panel.Controls.Add(memoryTrackBar);

            // TextBox для ручного ввода (рядом со слайдером)
            var memoryTextBox = new TextBox
            {
                Name = "memoryTextBox",
                Location = new Point(470, 43),
                Size = new Size(80, 23),
                BackColor = SecondaryBackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle,
                Text = currentProfile.AllocatedMemoryMB.ToString(),
                MaxLength = 5,
                TextAlign = HorizontalAlignment.Center
            };
            memoryTextBox.KeyPress += (s, e) =>
            {
                if (!char.IsControl(e.KeyChar) && !char.IsDigit(e.KeyChar))
                {
                    e.Handled = true;
                }
            };
            memoryTextBox.Leave += (s, e) =>
            {
                if (int.TryParse(memoryTextBox.Text, out int value))
                {
                    if (value < 512) value = 512;
                    if (value > 16384) value = 16384;
                    memoryTextBox.Text = value.ToString();
                    
                    var trackBar = FindControl<TrackBar>("memoryTrackBar");
                    if (trackBar != null && trackBar.Value != value)
                    {
                        trackBar.Value = value;
                    }
                }
                else
                {
                    memoryTextBox.Text = currentProfile.AllocatedMemoryMB.ToString();
                }
            };
            panel.Controls.Add(memoryTextBox);

            var memoryValueLabel = new Label
            {
                Name = "memoryValueLabel",
                Text = $"{currentProfile.AllocatedMemoryMB} MB ({currentProfile.AllocatedMemoryMB / 1024.0:F1} GB)",
                ForeColor = TextColor,
                Font = new Font("Segoe UI", 10, FontStyle.Bold),
                Location = new Point(560, 45),
                Size = new Size(160, 20)
            };
            panel.Controls.Add(memoryValueLabel);

            return yPos + 100;
        }

        private int CreateGameWindowSettingsPanel(Panel parent, int yPos)
        {
            var panel = CreateSectionPanel(parent, yPos, 100, "🖥️ Окно игры");

            var fullscreenCheckBox = new CheckBox
            {
                Name = "fullscreenCheckBox",
                Text = "Полноэкранный режим",
                ForeColor = TextColor,
                Location = new Point(15, 45),
                Size = new Size(200, 25),
                Checked = currentProfile.Fullscreen
            };
            fullscreenCheckBox.CheckedChanged += Fullscreen_CheckedChanged;
            panel.Controls.Add(fullscreenCheckBox);

            var widthLabel = new Label
            {
                Text = "Ширина:",
                ForeColor = SecondaryTextColor,
                Location = new Point(250, 48),
                Size = new Size(60, 20)
            };
            panel.Controls.Add(widthLabel);

            var widthTextBox = new TextBox
            {
                Name = "widthTextBox",
                Text = currentProfile.WindowWidth.ToString(),
                Location = new Point(315, 45),
                Size = new Size(80, 23),
                BackColor = BackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle,
                Enabled = !currentProfile.Fullscreen
            };
            panel.Controls.Add(widthTextBox);

            var heightLabel = new Label
            {
                Text = "Высота:",
                ForeColor = SecondaryTextColor,
                Location = new Point(420, 48),
                Size = new Size(60, 20)
            };
            panel.Controls.Add(heightLabel);

            var heightTextBox = new TextBox
            {
                Name = "heightTextBox",
                Text = currentProfile.WindowHeight.ToString(),
                Location = new Point(485, 45),
                Size = new Size(80, 23),
                BackColor = BackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle,
                Enabled = !currentProfile.Fullscreen
            };
            panel.Controls.Add(heightTextBox);

            return yPos + 100;
        }

        private int CreateArgumentsPanel(Panel parent, int yPos)
        {
            var panel = CreateSectionPanel(parent, yPos, 100, "⚡ Дополнительные аргументы");

            var jvmArgsLabel = new Label
            {
                Text = "JVM аргументы:",
                ForeColor = SecondaryTextColor,
                Location = new Point(15, 42),
                Size = new Size(110, 20)
            };
            panel.Controls.Add(jvmArgsLabel);

            var jvmArgsTextBox = new TextBox
            {
                Name = "jvmArgsTextBox",
                Text = currentProfile.JvmArguments,
                Location = new Point(130, 40),
                Size = new Size(590, 23),
                BackColor = BackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle
            };
            panel.Controls.Add(jvmArgsTextBox);

            var gameArgsLabel = new Label
            {
                Text = "Аргументы игры:",
                ForeColor = SecondaryTextColor,
                Location = new Point(15, 72),
                Size = new Size(110, 20)
            };
            panel.Controls.Add(gameArgsLabel);

            var gameArgsTextBox = new TextBox
            {
                Name = "gameArgsTextBox",
                Text = currentProfile.GameArguments,
                Location = new Point(130, 70),
                Size = new Size(590, 23),
                BackColor = BackgroundColor,
                ForeColor = TextColor,
                BorderStyle = BorderStyle.FixedSingle
            };
            panel.Controls.Add(gameArgsTextBox);

            return yPos + 100;
        }

        private int CreateDiscordRPCPanel(Panel parent, int yPos)
        {
            var panel = CreateSectionPanel(parent, yPos, 80, "💬 Discord Rich Presence");

            var enableDiscordCheckBox = new CheckBox
            {
                Name = "enableDiscordRPCCheckBox",
                Text = "Показывать активность в Discord",
                ForeColor = TextColor,
                Location = new Point(15, 45),
                Size = new Size(300, 25),
                Checked = false // TODO: Загрузить из конфига
            };
            panel.Controls.Add(enableDiscordCheckBox);

            return yPos + 80;
        }

        private Panel CreateSectionPanel(Panel parent, int yPos, int height, string title)
        {
            var panel = new Panel
            {
                Location = new Point(30, yPos),
                Size = new Size(700, height),
                BackColor = SecondaryBackgroundColor
            };
            parent.Controls.Add(panel);

            var titleLabel = new Label
            {
                Text = title,
                Font = new Font("Segoe UI", 12, FontStyle.Bold),
                ForeColor = AccentColor,
                Location = new Point(15, 10),
                Size = new Size(300, 25)
            };
            panel.Controls.Add(titleLabel);

            return panel;
        }

        private async void LoadSettings()
        {
            // Загрузить настройки из конфига
            var config = await configManager.LoadConfig();
            
            // Загрузить Discord RPC
            var discordCheckBox = FindControl<CheckBox>("discordRPCCheckBox");
            if (discordCheckBox != null)
            {
                discordCheckBox.Checked = config.EnableDiscordRPC;
            }

            // Загрузить другие настройки
            if (currentProfile.AutoDetectJava && string.IsNullOrEmpty(currentProfile.JavaPath))
            {
                await AutoDetectJavaPath();
            }
        }

        private async void AutoDetectJava_CheckedChanged(object sender, EventArgs e)
        {
            var checkBox = sender as CheckBox;
            if (checkBox == null) return;

            var javaPathTextBox = FindControl<TextBox>("javaPathTextBox");
            var browseButton = FindControl<Button>("browseJavaButton");

            if (javaPathTextBox != null) javaPathTextBox.Enabled = !checkBox.Checked;
            if (browseButton != null) browseButton.Enabled = !checkBox.Checked;

            if (checkBox.Checked)
            {
                await AutoDetectJavaPath();
            }
        }

        private async Task AutoDetectJavaPath()
        {
            var javaPathTextBox = FindControl<TextBox>("javaPathTextBox");
            if (javaPathTextBox == null) return;

            javaPathTextBox.Text = "Поиск Java...";
            javaPathTextBox.Enabled = false;

            var java = await javaDetector.FindLatestJava();
            if (java != null)
            {
                javaPathTextBox.Text = java.Path;
                currentProfile.JavaPath = java.Path;
            }
            else
            {
                javaPathTextBox.Text = "Java не найдена!";
            }

            var autoDetectCheckBox = FindControl<CheckBox>("autoDetectJavaCheckBox");
            if (autoDetectCheckBox != null)
            {
                javaPathTextBox.Enabled = !autoDetectCheckBox.Checked;
            }
        }

        private void BrowseJava_Click(object sender, EventArgs e)
        {
            using var openFileDialog = new OpenFileDialog
            {
                Filter = "Java Executable|java.exe|All Files|*.*",
                Title = "Выберите java.exe"
            };

            if (openFileDialog.ShowDialog() == DialogResult.OK)
            {
                var javaPathTextBox = FindControl<TextBox>("javaPathTextBox");
                if (javaPathTextBox != null)
                {
                    javaPathTextBox.Text = openFileDialog.FileName;
                }
            }
        }

        private async void DetectJava_Click(object sender, EventArgs e)
        {
            var installations = await javaDetector.FindAllJavaInstallations();
            if (installations.Count == 0)
            {
                MessageBox.Show("Java не найдена на вашем компьютере.", "Java не найдена", 
                    MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            var message = "Найденные установки Java:\n\n";
            for (int i = 0; i < installations.Count; i++)
            {
                message += $"{i + 1}. {installations[i].DisplayName}\n   {installations[i].Path}\n\n";
            }

            MessageBox.Show(message, "Найденные версии Java", MessageBoxButtons.OK, MessageBoxIcon.Information);
        }

        private void MemoryTrackBar_ValueChanged(object sender, EventArgs e)
        {
            var trackBar = sender as TrackBar;
            if (trackBar == null) return;

            var memoryValueLabel = FindControl<Label>("memoryValueLabel");
            var memoryTextBox = FindControl<TextBox>("memoryTextBox");
            
            if (memoryValueLabel != null)
            {
                memoryValueLabel.Text = $"{trackBar.Value} MB ({trackBar.Value / 1024.0:F1} GB)";
            }

            if (memoryTextBox != null && memoryTextBox.Text != trackBar.Value.ToString())
            {
                memoryTextBox.Text = trackBar.Value.ToString();
            }
        }

        private async Task<string> GetJavaVersion(string javaPath)
        {
            if (string.IsNullOrEmpty(javaPath) || !System.IO.File.Exists(javaPath))
            {
                return "❌ Java не найдена";
            }

            try
            {
                var processStartInfo = new System.Diagnostics.ProcessStartInfo
                {
                    FileName = javaPath,
                    Arguments = "-version",
                    UseShellExecute = false,
                    RedirectStandardError = true,
                    CreateNoWindow = true
                };

                using var process = System.Diagnostics.Process.Start(processStartInfo);
                if (process == null) return "❌ Не удалось запустить Java";

                var output = await process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync();

                // Парсинг версии из вывода
                var lines = output.Split('\n');
                if (lines.Length > 0)
                {
                    var versionLine = lines[0];
                    // Пример: java version "17.0.1" или openjdk version "11.0.12"
                    if (versionLine.Contains("version"))
                    {
                        var versionMatch = System.Text.RegularExpressions.Regex.Match(versionLine, "\"(.+?)\"");
                        if (versionMatch.Success)
                        {
                            return $"✅ Java {versionMatch.Groups[1].Value}";
                        }
                    }
                }

                return "✅ Java установлена";
            }
            catch
            {
                return "❌ Ошибка определения версии";
            }
        }

        private void Fullscreen_CheckedChanged(object sender, EventArgs e)
        {
            var checkBox = sender as CheckBox;
            if (checkBox == null) return;

            var widthTextBox = FindControl<TextBox>("widthTextBox");
            var heightTextBox = FindControl<TextBox>("heightTextBox");

            if (widthTextBox != null) widthTextBox.Enabled = !checkBox.Checked;
            if (heightTextBox != null) heightTextBox.Enabled = !checkBox.Checked;
        }

        private async void SaveButton_Click(object sender, EventArgs e)
        {
            try
            {
                // Собрать данные из контролов
                var autoDetectCheckBox = FindControl<CheckBox>("autoDetectJavaCheckBox");
                var javaPathTextBox = FindControl<TextBox>("javaPathTextBox");
                var memoryTrackBar = FindControl<TrackBar>("memoryTrackBar");
                var fullscreenCheckBox = FindControl<CheckBox>("fullscreenCheckBox");
                var widthTextBox = FindControl<TextBox>("widthTextBox");
                var heightTextBox = FindControl<TextBox>("heightTextBox");
                var jvmArgsTextBox = FindControl<TextBox>("jvmArgsTextBox");
                var gameArgsTextBox = FindControl<TextBox>("gameArgsTextBox");
                var discordCheckBox = FindControl<CheckBox>("discordRPCCheckBox");

                if (autoDetectCheckBox != null) currentProfile.AutoDetectJava = autoDetectCheckBox.Checked;
                if (javaPathTextBox != null) currentProfile.JavaPath = javaPathTextBox.Text;
                if (memoryTrackBar != null) currentProfile.AllocatedMemoryMB = memoryTrackBar.Value;
                if (fullscreenCheckBox != null) currentProfile.Fullscreen = fullscreenCheckBox.Checked;
                
                if (widthTextBox != null && int.TryParse(widthTextBox.Text, out int width))
                    currentProfile.WindowWidth = width;
                
                if (heightTextBox != null && int.TryParse(heightTextBox.Text, out int height))
                    currentProfile.WindowHeight = height;
                
                if (jvmArgsTextBox != null) currentProfile.JvmArguments = jvmArgsTextBox.Text;
                if (gameArgsTextBox != null) currentProfile.GameArguments = gameArgsTextBox.Text;

                // Сохранить Discord RPC
                if (discordCheckBox != null)
                {
                    var config = await configManager.LoadConfig();
                    config.EnableDiscordRPC = discordCheckBox.Checked;
                    await configManager.SaveConfig(config);
                }

                await configManager.SaveGameProfile(currentProfile);

                MessageBox.Show("Настройки успешно сохранены!", "Успех", 
                    MessageBoxButtons.OK, MessageBoxIcon.Information);
                
                SettingsSaved?.Invoke(this, EventArgs.Empty);
                Hide();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Ошибка при сохранении настроек: {ex.Message}", "Ошибка", 
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        private T FindControl<T>(string name) where T : Control
        {
            return this.Controls.OfType<Panel>()
                .SelectMany(p => p.Controls.OfType<Panel>())
                .SelectMany(p => p.Controls.OfType<T>())
                .FirstOrDefault(c => c.Name == name);
        }

        public new void Show()
        {
            this.Visible = true;
            this.BringToFront();
        }

        public new void Hide()
        {
            this.Visible = false;
            SettingsClosed?.Invoke(this, EventArgs.Empty);
        }
    }
}
