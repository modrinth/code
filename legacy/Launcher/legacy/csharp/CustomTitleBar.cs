using System;
using System.Drawing;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace MinecraftLauncher
{
    /// <summary>
    /// Кастомный Title Bar для borderless формы
    /// </summary>
    public class CustomTitleBar : Panel
    {
        private Label titleLabel;
        private Button minimizeButton;
        private Button maximizeButton;
        private Button closeButton;
        private Form parentForm;
        private bool isDragging = false;
        private Point dragStart;

        // Цвета Owyx
        private readonly Color BackgroundColor = Color.FromArgb(24, 24, 32);
        private readonly Color ButtonHoverColor = Color.FromArgb(40, 40, 50);
        private readonly Color CloseHoverColor = Color.FromArgb(200, 50, 50);
        private readonly Color TextColor = Color.FromArgb(240, 240, 245);

        public CustomTitleBar(Form parent)
        {
            parentForm = parent;
            InitializeComponent();
            SetupDragHandlers();
        }

        private void InitializeComponent()
        {
            this.Height = 35;
            this.Dock = DockStyle.Top;
            this.BackColor = BackgroundColor;

            // Заголовок
            titleLabel = new Label
            {
                Text = parentForm.Text,
                ForeColor = TextColor,
                Font = new Font("Segoe UI", 10F, FontStyle.Bold),
                Location = new Point(15, 8),
                Size = new Size(400, 20),
                AutoSize = false
            };
            this.Controls.Add(titleLabel);

            // Кнопки управления
            int buttonX = parentForm.ClientSize.Width - 135;

            minimizeButton = CreateControlButton("─", buttonX);
            minimizeButton.Click += (s, e) => parentForm.WindowState = FormWindowState.Minimized;

            maximizeButton = CreateControlButton("□", buttonX + 45);
            maximizeButton.Click += MaximizeButton_Click;

            closeButton = CreateControlButton("✕", buttonX + 90);
            closeButton.Click += (s, e) => parentForm.Close();
            closeButton.MouseEnter += (s, e) => closeButton.BackColor = CloseHoverColor;
            closeButton.MouseLeave += (s, e) => closeButton.BackColor = BackgroundColor;

            this.Controls.Add(minimizeButton);
            this.Controls.Add(maximizeButton);
            this.Controls.Add(closeButton);

            // Обновить позиции при изменении размера формы
            parentForm.Resize += (s, e) => UpdateButtonPositions();
        }

        private Button CreateControlButton(string text, int x)
        {
            var button = new Button
            {
                Text = text,
                ForeColor = TextColor,
                BackColor = BackgroundColor,
                FlatStyle = FlatStyle.Flat,
                Size = new Size(45, 35),
                Location = new Point(x, 0),
                Font = new Font("Segoe UI", 10F),
                Cursor = Cursors.Hand
            };

            button.FlatAppearance.BorderSize = 0;
            button.FlatAppearance.MouseOverBackColor = ButtonHoverColor;
            button.FlatAppearance.MouseDownBackColor = ButtonHoverColor;

            return button;
        }

        private void MaximizeButton_Click(object sender, EventArgs e)
        {
            if (parentForm.WindowState == FormWindowState.Maximized)
            {
                parentForm.WindowState = FormWindowState.Normal;
                maximizeButton.Text = "□";
            }
            else
            {
                parentForm.WindowState = FormWindowState.Maximized;
                maximizeButton.Text = "❐";
            }
        }

        private void UpdateButtonPositions()
        {
            int buttonX = parentForm.ClientSize.Width - 135;
            minimizeButton.Location = new Point(buttonX, 0);
            maximizeButton.Location = new Point(buttonX + 45, 0);
            closeButton.Location = new Point(buttonX + 90, 0);
        }

        private void SetupDragHandlers()
        {
            this.MouseDown += TitleBar_MouseDown;
            this.MouseMove += TitleBar_MouseMove;
            this.MouseUp += TitleBar_MouseUp;
            titleLabel.MouseDown += TitleBar_MouseDown;
            titleLabel.MouseMove += TitleBar_MouseMove;
            titleLabel.MouseUp += TitleBar_MouseUp;
        }

        private void TitleBar_MouseDown(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                isDragging = true;
                dragStart = e.Location;
            }
        }

        private void TitleBar_MouseMove(object sender, MouseEventArgs e)
        {
            if (isDragging)
            {
                Point currentScreenPos = PointToScreen(e.Location);
                parentForm.Location = new Point(currentScreenPos.X - dragStart.X, currentScreenPos.Y - dragStart.Y);
            }
        }

        private void TitleBar_MouseUp(object sender, MouseEventArgs e)
        {
            isDragging = false;
        }

        public void UpdateTitle(string title)
        {
            titleLabel.Text = title;
        }
    }
}
