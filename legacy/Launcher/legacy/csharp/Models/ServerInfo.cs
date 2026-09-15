using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace MinecraftLauncher.Models
{
    public class ServerInfo
    {
        [JsonPropertyName("minecraft_version")]
        public string MinecraftVersion { get; set; } = string.Empty;
        
        [JsonPropertyName("mods_hash")]
        public string ModsHash { get; set; } = string.Empty;
        
        [JsonPropertyName("client_files")]
        public List<ClientFile> ClientFiles { get; set; } = new List<ClientFile>();
        
        [JsonPropertyName("mods")]
        public List<ModFile> Mods { get; set; } = new List<ModFile>();
    }

    public class ClientFile
    {
        [JsonPropertyName("name")]
        public string Name { get; set; } = string.Empty;
        
        [JsonPropertyName("url")]
        public string Url { get; set; } = string.Empty;
        
        [JsonPropertyName("hash")]
        public string Hash { get; set; } = string.Empty;
    }

    public class ModFile
    {
        [JsonPropertyName("name")]
        public string Name { get; set; } = string.Empty;
        
        [JsonPropertyName("url")]
        public string Url { get; set; } = string.Empty;
        
        [JsonPropertyName("hash")]
        public string Hash { get; set; } = string.Empty;
    }
}