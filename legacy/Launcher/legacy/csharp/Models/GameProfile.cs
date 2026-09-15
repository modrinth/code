using System;

namespace MinecraftLauncher.Models
{
    public class GameProfile
    {
        public string Name { get; set; } = string.Empty;
        public string GameDirectory { get; set; } = string.Empty;
        public string MinecraftVersion { get; set; } = string.Empty;
        public string JavaPath { get; set; } = string.Empty;
        public string LauncherVersion { get; set; } = "1.0.1";
        public DateTime LastUsed { get; set; } = DateTime.Now;
        public bool IsDefault { get; set; } = true;
        
        // Расширенные настройки
        public int AllocatedMemoryMB { get; set; } = 2048; // 2GB по умолчанию
        public string JvmArguments { get; set; } = string.Empty;
        public string GameArguments { get; set; } = string.Empty;
        public bool AutoDetectJava { get; set; } = true;
        public int WindowWidth { get; set; } = 854;
        public int WindowHeight { get; set; } = 480;
        public bool Fullscreen { get; set; } = false;
    }
}