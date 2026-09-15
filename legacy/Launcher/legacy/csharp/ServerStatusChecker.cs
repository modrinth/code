using System;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;
using System.IO;
using System.Text.Json;

namespace MinecraftLauncher
{
    /// <summary>
    /// Класс для проверки статуса Minecraft сервера
    /// </summary>
    public class ServerStatusChecker
    {
        public class ServerStatus
        {
            public bool IsOnline { get; set; }
            public int OnlinePlayers { get; set; }
            public int MaxPlayers { get; set; }
            public string Version { get; set; } = string.Empty;
            public string Description { get; set; } = string.Empty;
            public string Protocol { get; set; } = string.Empty;
            public int Ping { get; set; }
        }

        private readonly string serverAddress;
        private readonly int serverPort;

        public ServerStatusChecker(string address, int port = 25565)
        {
            serverAddress = address;
            serverPort = port;
        }

        /// <summary>
        /// Получить статус сервера
        /// </summary>
        public async Task<ServerStatus> GetServerStatus()
        {
            var status = new ServerStatus { IsOnline = false };

            try
            {
                var startTime = DateTime.Now;
                
                using var client = new TcpClient();
                var connectTask = client.ConnectAsync(serverAddress, serverPort);
                
                // Таймаут 5 секунд
                if (await Task.WhenAny(connectTask, Task.Delay(5000)) != connectTask)
                {
                    return status;
                }

                if (!client.Connected)
                {
                    return status;
                }

                using var stream = client.GetStream();
                
                // Отправить handshake packet
                await SendHandshake(stream);
                
                // Отправить status request
                await SendStatusRequest(stream);
                
                // Прочитать ответ
                var response = await ReadResponse(stream);
                
                var ping = (int)(DateTime.Now - startTime).TotalMilliseconds;
                
                if (!string.IsNullOrEmpty(response))
                {
                    status = ParseServerResponse(response);
                    status.Ping = ping;
                }

                return status;
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при проверке статуса сервера: {ex.Message}");
                return status;
            }
        }

        private async Task SendHandshake(NetworkStream stream)
        {
            using var ms = new MemoryStream();
            using var writer = new BinaryWriter(ms);

            // Packet ID (0x00 для handshake)
            WriteVarInt(writer, 0x00);
            
            // Protocol version (-1 для status ping)
            WriteVarInt(writer, -1);
            
            // Server address
            WriteVarInt(writer, serverAddress.Length);
            writer.Write(Encoding.UTF8.GetBytes(serverAddress));
            
            // Server port
            writer.Write((byte)(serverPort >> 8));
            writer.Write((byte)(serverPort & 0xFF));
            
            // Next state (1 для status)
            WriteVarInt(writer, 1);

            var packet = ms.ToArray();
            
            // Отправить длину пакета
            using var packetStream = new MemoryStream();
            using var packetWriter = new BinaryWriter(packetStream);
            WriteVarInt(packetWriter, packet.Length);
            packetWriter.Write(packet);
            
            var finalPacket = packetStream.ToArray();
            await stream.WriteAsync(finalPacket, 0, finalPacket.Length);
        }

        private async Task SendStatusRequest(NetworkStream stream)
        {
            // Status request - просто packet ID 0x00 с длиной 1
            var packet = new byte[] { 0x01, 0x00 };
            await stream.WriteAsync(packet, 0, packet.Length);
        }

        private async Task<string> ReadResponse(NetworkStream stream)
        {
            // Читаем длину пакета
            var length = await ReadVarInt(stream);
            if (length <= 0) return string.Empty;

            // Читаем packet ID
            var packetId = await ReadVarInt(stream);
            if (packetId != 0x00) return string.Empty;

            // Читаем длину JSON строки
            var jsonLength = await ReadVarInt(stream);
            if (jsonLength <= 0) return string.Empty;

            // Читаем JSON данные
            var buffer = new byte[jsonLength];
            var totalRead = 0;
            while (totalRead < jsonLength)
            {
                var read = await stream.ReadAsync(buffer, totalRead, jsonLength - totalRead);
                if (read == 0) break;
                totalRead += read;
            }

            return Encoding.UTF8.GetString(buffer, 0, totalRead);
        }

        private void WriteVarInt(BinaryWriter writer, int value)
        {
            uint unsigned = (uint)value;
            do
            {
                byte temp = (byte)(unsigned & 0x7F);
                unsigned >>= 7;
                if (unsigned != 0)
                {
                    temp |= 0x80;
                }
                writer.Write(temp);
            } while (unsigned != 0);
        }

        private async Task<int> ReadVarInt(NetworkStream stream)
        {
            int numRead = 0;
            int result = 0;
            byte read;
            
            do
            {
                var buffer = new byte[1];
                var bytesRead = await stream.ReadAsync(buffer, 0, 1);
                if (bytesRead == 0) return -1;
                
                read = buffer[0];
                int value = (read & 0x7F);
                result |= (value << (7 * numRead));

                numRead++;
                if (numRead > 5)
                {
                    throw new Exception("VarInt is too big");
                }
            } while ((read & 0x80) != 0);

            return result;
        }

        private ServerStatus ParseServerResponse(string json)
        {
            var status = new ServerStatus { IsOnline = true };

            try
            {
                using var document = JsonDocument.Parse(json);
                var root = document.RootElement;

                // Версия
                if (root.TryGetProperty("version", out var versionElement))
                {
                    if (versionElement.TryGetProperty("name", out var nameElement))
                    {
                        status.Version = nameElement.GetString() ?? string.Empty;
                    }
                    if (versionElement.TryGetProperty("protocol", out var protocolElement))
                    {
                        status.Protocol = protocolElement.GetInt32().ToString();
                    }
                }

                // Игроки
                if (root.TryGetProperty("players", out var playersElement))
                {
                    if (playersElement.TryGetProperty("online", out var onlineElement))
                    {
                        status.OnlinePlayers = onlineElement.GetInt32();
                    }
                    if (playersElement.TryGetProperty("max", out var maxElement))
                    {
                        status.MaxPlayers = maxElement.GetInt32();
                    }
                }

                // Описание
                if (root.TryGetProperty("description", out var descElement))
                {
                    if (descElement.ValueKind == JsonValueKind.String)
                    {
                        status.Description = descElement.GetString() ?? string.Empty;
                    }
                    else if (descElement.ValueKind == JsonValueKind.Object)
                    {
                        if (descElement.TryGetProperty("text", out var textElement))
                        {
                            status.Description = textElement.GetString() ?? string.Empty;
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Ошибка при парсинге ответа сервера: {ex.Message}");
            }

            return status;
        }
    }
}
