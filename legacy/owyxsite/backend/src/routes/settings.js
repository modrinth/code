const express = require('express');
const router = express.Router();
const fs = require('fs').promises;
const path = require('path');
const {
    authenticateToken,
    authenticateLongTermApiTokenOnly,
    requireRole
} = require('./auth');
const db = require('../database/connection');
const { status } = require('minecraft-server-util');

// Публичный endpoint для получения основных настроек сервера
router.get('/settings/public', async (req, res) => {
    try {
        // Получаем настройки из базы данных server_settings
        const result = await db.query('SELECT setting_key, setting_value FROM server_settings');
        const settings = {};
        
        // Преобразуем результат в объект
        result.rows.forEach(row => {
            settings[row.setting_key] = row.setting_value;
        });
        
        // Возвращаем только публичную информацию
        res.json({
            serverName: settings['server-name'] || 'Test',
            serverDescription: settings['server-description'] || 'Приватный Minecraft сервер с дружелюбным сообществом',
            serverIp: settings['server-ip'] || '45.131.186.146',
            serverPort: settings['server-port'] || '1488',
            discordInvite: settings['discord-invite'] || null,
            telegramInvite: settings['telegram-invite'] || null
        });
    } catch (error) {
        console.error('Ошибка получения публичных настроек:', error);
        // Fallback к значениям по умолчанию
        res.json({
            serverName: 'Test',
            serverDescription: 'Приватный Minecraft сервер с дружелюбным сообществом',
            serverIp: '45.131.186.146',
            serverPort: '1488',
            discordInvite: null,
            telegramInvite: null
        });
    }
});

// Публичный endpoint для информации о статусе сервера
router.get('/settings/server-info', async (req, res) => {
    try {
        // Получаем настройки сервера из базы данных
        const settingsResult = await db.query(`
            SELECT setting_key, setting_value 
            FROM server_settings 
            WHERE setting_key IN ($1, $2, $3, $4, $5)
        `, ['server-ip', 'server-port', 'server-name', 'server-max-players', 'server-description']);
        
        const settings = {};
        settingsResult.rows.forEach(row => {
            settings[row.setting_key] = row.setting_value;
        });
        
        const serverIp = settings['server-ip'] || '45.131.186.146';
        const serverPort = parseInt(settings['server-port'] || '1488');
        const serverName = settings['server-name'] || 'Owyx';
        const maxPlayers = parseInt(settings['server-max-players'] || '50');
        
        // Проверяем статус Minecraft сервера используя minecraft-server-util
        let minecraftServerData = {
            online: false,
            version: 'Unknown',
            motd: 'Server Offline',
            ping: 0,
            tps: 0,
            players: {
                online: 0,
                max: maxPlayers,
                list: []
            }
        };
        
        try {
            const response = await status(serverIp, serverPort);
            minecraftServerData = {
                online: true,
                version: response.version?.name || 'Unknown',
                motd: response.motd?.clean || 'Minecraft Server',
                ping: response.roundTripLatency || 0,
                players: {
                    online: response.players?.online || 0,
                    max: response.players?.max || maxPlayers,
                    list: response.players?.sample || []
                }
            };
        } catch (minecraftError) {
            console.log('Minecraft сервер недоступен:', minecraftError.message);
        }

        // Получаем TPS и время работы из таблицы server_status (данные от плагина)
        let serverPerformanceData = {
            tps: null,
            uptime_seconds: null,
            memory: {
                max: null,
                used: null,
                free: null
            }
        };

        try {
            const statusResult = await db.query(`
                SELECT tps, uptime_seconds, max_memory, used_memory, free_memory, updated_at
                FROM server_status 
                WHERE server_ip = $1 AND server_port = $2
                    AND updated_at > NOW() - INTERVAL '5 minutes'
                ORDER BY updated_at DESC 
                LIMIT 1
            `, [serverIp, serverPort]);

            if (statusResult.rows.length > 0) {
                const statusData = statusResult.rows[0];
                serverPerformanceData = {
                    tps: statusData.tps ? parseFloat(statusData.tps) : null,
                    uptime_seconds: statusData.uptime_seconds ? parseInt(statusData.uptime_seconds) : null,
                    memory: {
                        max: statusData.max_memory ? parseInt(statusData.max_memory) : null,
                        used: statusData.used_memory ? parseInt(statusData.used_memory) : null,
                        free: statusData.free_memory ? parseInt(statusData.free_memory) : null
                    }
                };
            }
        } catch (statusError) {
            console.log('Ошибка получения статуса сервера из БД:', statusError.message);
        }
        
        // Если сервер онлайн - получаем детальную информацию об игроках из БД
        let enrichedPlayersList = [];
        if (minecraftServerData.online && minecraftServerData.players.online > 0) {
            const onlinePlayersResult = await db.query(`
                SELECT 
                    gs.nickname,
                    gs.player_uuid,
                    gs.last_login,
                    u.trust_level,
                    u.role,
                    ps.time_played_minutes,
                    ps.last_seen,
                    ps.minecraft_stats,
                    ps.current_level,
                    ps.achievements_count
                FROM game_sessions gs
                LEFT JOIN users u ON gs.user_id = u.id
                LEFT JOIN player_stats ps ON u.id = ps.user_id
                WHERE gs.is_active = true 
                    AND gs.expires_at > NOW()
                ORDER BY gs.last_login DESC
            `);
            
            // Формируем список игроков с дополнительной информацией
            enrichedPlayersList = onlinePlayersResult.rows.map(player => ({
                name: player.nickname,
                uuid: player.player_uuid,
                trust_level: player.trust_level || 0,
                role: player.role || 'user',
                playtime: player.time_played_minutes || 0,
                last_seen: player.last_seen,
                level: player.current_level || 1,
                achievements: player.achievements_count || 0,
                minecraft_stats: player.minecraft_stats || {},
                session_start: player.last_login
            }));
        }

        
        res.json({
            // Статус сервера
            status: minecraftServerData.online ? 'online' : 'offline',
            online: minecraftServerData.online,
            
            // Информация о сервере
            server: {
                name: serverName,
                ip: serverIp,
                port: serverPort,
                version: minecraftServerData.version,
                motd: minecraftServerData.motd
            },
            
            // Производительность - используем данные от плагина
            performance: {
                ping: minecraftServerData.ping,
                tps: serverPerformanceData.tps,
                uptime_seconds: serverPerformanceData.uptime_seconds,
                memory: serverPerformanceData.memory
            },
            
            // Игроки - используем реальные данные с сервера
            players: {
                online: minecraftServerData.players.online,
                max: minecraftServerData.players.max,
                list: enrichedPlayersList  // Детальная информация из БД для онлайн игроков
            },
            
            // Дополнительная информация
            timestamp: new Date().toISOString(),
            last_update: new Date().toISOString()
        });
        
    } catch (error) {
        console.error('Ошибка получения информации о сервере:', error);
        res.status(500).json({
            status: 'error',
            online: false,
            server: {
                name: 'Owyx',
                ip: '45.131.186.146',
                port: 1488
            },
            players: { 
                online: 0, 
                max: 50,
                list: []
            },
            error: 'Ошибка сервера'
        });
    }
});

// Получение текущих настроек (только для админов)
router.get('/settings', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        // Читаем текущий файл конфигурации
        const configPath = path.join(__dirname, '../config/settings.js');
        const configContent = await fs.readFile(configPath, 'utf-8');
        
        // Извлекаем текущие значения из config
        const config = require('../config/settings');
        
        res.json({
            success: true,
            settings: {
                server: config.server,
                applications: config.applications,
                trustLevel: config.trustLevel,
                reputation: config.reputation,
                security: config.security,
                email: config.email
            }
        });
    } catch (error) {
        console.error('Ошибка получения настроек:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// Обновление настроек
router.post('/settings', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const { settings } = req.body;
        
        if (!settings) {
            return res.status(400).json({ error: 'Настройки не предоставлены' });
        }
        
        // Читаем текущий файл
        const configPath = path.join(__dirname, '../config/settings.js');
        let configContent = await fs.readFile(configPath, 'utf-8');
        
        // Обновляем значения в файле конфигурации
        if (settings.server) {
            Object.keys(settings.server).forEach(key => {
                const value = settings.server[key];
                const regex = new RegExp(`(${key}:\\s*process\\.env\\.[A-Z_]+\\s*\\|\\|\\s*['"])[^'"]*(['"])`);
                if (configContent.match(regex)) {
                    configContent = configContent.replace(regex, `$1${value}$2`);
                }
            });
        }
        
        if (settings.applications) {
            Object.keys(settings.applications).forEach(key => {
                const value = settings.applications[key];
                if (typeof value === 'number') {
                    const regex = new RegExp(`(${key}:\\s*parseInt\\(process\\.env\\.[A-Z_]+\\)\\s*\\|\\|\\s*)\\d+`);
                    if (configContent.match(regex)) {
                        configContent = configContent.replace(regex, `$1${value}`);
                    }
                } else if (typeof value === 'boolean') {
                    const regex = new RegExp(`(${key}:\\s*process\\.env\\.[A-Z_]+\\s*===\\s*'true'\\s*\\|\\|\\s*)(true|false)`);
                    if (configContent.match(regex)) {
                        configContent = configContent.replace(regex, `$1${value}`);
                    }
                }
            });
        }
        
        if (settings.trustLevel && settings.trustLevel.requirements) {
            Object.keys(settings.trustLevel.requirements).forEach(level => {
                const requirements = settings.trustLevel.requirements[level];
                if (requirements.hoursRequired !== undefined) {
                    const regex = new RegExp(`(${level}:\\s*{[^}]*hoursRequired:\\s*parseInt\\(process\\.env\\.[A-Z_]+\\)\\s*\\|\\|\\s*)\\d+`);
                    if (configContent.match(regex)) {
                        configContent = configContent.replace(regex, `$1${requirements.hoursRequired}`);
                    }
                }
                if (requirements.reputationRequired !== undefined) {
                    const regex = new RegExp(`(${level}:\\s*{[^}]*reputationRequired:\\s*parseInt\\(process\\.env\\.[A-Z_]+\\)\\s*\\|\\|\\s*)\\d+`);
                    if (configContent.match(regex)) {
                        configContent = configContent.replace(regex, `$1${requirements.reputationRequired}`);
                    }
                }
            });
        }
        
        // Сохраняем обновленный файл
        await fs.writeFile(configPath, configContent, 'utf-8');
        
        // Очищаем кеш модуля, чтобы изменения вступили в силу
        delete require.cache[require.resolve('../config/settings')];
        
        // Логируем действие
        await req.db.query(
            'INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)',
            [req.user.id, 'settings_update', `Обновлены настройки сервера`]
        );
        
        res.json({
            success: true,
            message: 'Настройки успешно обновлены'
        });
        
    } catch (error) {
        console.error('Ошибка обновления настроек:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// Сброс настроек к значениям по умолчанию
router.post('/settings/reset', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const { section } = req.body;
        
        // Здесь можно реализовать сброс определенной секции настроек
        // к значениям по умолчанию
        
        await req.db.query(
            'INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)',
            [req.user.id, 'settings_reset', `Сброшена секция настроек: ${section}`]
        );
        
        res.json({
            success: true,
            message: 'Настройки сброшены к значениям по умолчанию'
        });
        
    } catch (error) {
        console.error('Ошибка сброса настроек:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// Защищенный endpoint для плагинов - получение информации о сервере
router.get('/plugin/server-info', authenticateLongTermApiTokenOnly, async (req, res) => {
    try {
        // Получаем настройки сервера из базы данных
        const settingsResult = await db.query(`
            SELECT setting_key, setting_value 
            FROM server_settings 
            WHERE setting_key IN ($1, $2, $3, $4, $5)
        `, ['server-ip', 'server-port', 'server-name', 'server-max-players', 'server-description']);
        
        const settings = {};
        settingsResult.rows.forEach(row => {
            settings[row.setting_key] = row.setting_value;
        });
        
        const serverIp = settings['server-ip'] || '45.131.186.146';
        const serverPort = parseInt(settings['server-port'] || '1488');
        const serverName = settings['server-name'] || 'Owyx';
        const maxPlayers = parseInt(settings['server-max-players'] || '50');
        
        // Получаем список онлайн игроков из базы данных
        const onlinePlayersResult = await db.query(`
            SELECT 
                gs.nickname,
                gs.player_uuid,
                gs.ip_address,
                gs.last_login,
                u.trust_level,
                u.role,
                ps.time_played_minutes,
                ps.last_seen,
                ps.minecraft_stats,
                ps.current_level,
                ps.achievements_count
            FROM game_sessions gs
            LEFT JOIN users u ON gs.user_id = u.id
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            WHERE gs.is_active = true 
                AND gs.expires_at > NOW()
            ORDER BY gs.last_login DESC
        `);
        
        // Формируем список игроков с дополнительной информацией
        const playersList = onlinePlayersResult.rows.map(player => ({
            name: player.nickname,
            uuid: player.player_uuid,
            trust_level: player.trust_level || 0,
            role: player.role || 'user',
            playtime: player.time_played_minutes || 0,
            last_seen: player.last_seen,
            level: player.current_level || 1,
            achievements: player.achievements_count || 0,
            minecraft_stats: player.minecraft_stats || {},
            ip_address: player.ip_address,
            session_start: player.last_login
        }));
        
        // Проверяем статус Minecraft сервера
        let pluginServerData = {
            online: false,
            version: 'Unknown',
            motd: 'Server Offline',
            ping: 0,
            tps: 0
        };
        
        try {
            const response = await status(serverIp, serverPort);
            pluginServerData = {
                online: true,
                version: response.version?.name || 'Unknown',
                motd: response.motd?.clean || 'Minecraft Server',
                ping: response.roundTripLatency || 0,
                tps: null // TPS должен передаваться от плагина через API
            };
        } catch (minecraftError) {
            console.log('Minecraft сервер недоступен:', minecraftError.message);
        }
        
        res.json({
            success: true,
            // Статус сервера
            status: pluginServerData.online ? 'online' : 'offline',
            online: pluginServerData.online,
            
            // Информация о сервере
            server: {
                name: serverName,
                ip: serverIp,
                port: serverPort,
                version: pluginServerData.version,
                motd: pluginServerData.motd
            },
            
            // Производительность
            performance: {
                ping: pluginServerData.ping,
                tps: pluginServerData.tps
            },
            
            // Игроки
            players: {
                online: playersList.length,
                max: maxPlayers,
                list: playersList
            },
            
            // Метаданные
            timestamp: new Date().toISOString(),
            api_version: '2.1'
        });
    } catch (error) {
        console.error('Ошибка получения информации о сервере для плагина:', error);
        res.status(500).json({ 
            success: false, 
            error: 'Ошибка сервера',
            message: error.message 
        });
    }
});

// Защищенный endpoint для плагинов - проверка доступа игрока к серверу
router.get('/plugin/server-access', authenticateLongTermApiTokenOnly, async (req, res) => {
    try {
        const { nickname } = req.query;
        
        if (!nickname) {
            return res.status(400).json({ 
                success: false, 
                error: 'Nickname is required' 
            });
        }
        
        // Open access model: a registered, active, non-banned account is allowed.
        // Applications ("заявки") are no longer a whitelist gate. This keeps the
        // plugin's /login flow working: non-banned = allowed.
        const userResult = await db.query(`
            SELECT id, nickname, role, is_banned, is_active
            FROM users
            WHERE nickname = $1
            LIMIT 1
        `, [nickname]);

        let hasAccess = false;
        let reason = 'No account found';

        if (userResult.rows.length > 0) {
            const account = userResult.rows[0];
            if (account.is_banned) {
                hasAccess = false;
                reason = 'Account banned';
            } else if (account.is_active === false) {
                hasAccess = false;
                reason = 'Account inactive';
            } else {
                hasAccess = true;
                reason = 'Open access (active account)';
            }
        }

        res.json({
            success: true,
            hasAccess: hasAccess,
            reason: reason,
            nickname: nickname
        });
        
    } catch (error) {
        console.error('Ошибка проверки доступа к серверу:', error);
        res.status(500).json({ 
            success: false, 
            error: 'Server error',
            message: error.message 
        });
    }
});

// POST /api/settings/server-data - Получение данных о сервере от плагина (TPS, время работы)
router.post('/settings/server-data', authenticateLongTermApiTokenOnly, async (req, res) => {
    try {
        const { 
            server_ip,
            server_port, 
            tps,
            uptime_seconds,
            max_memory,
            used_memory,
            free_memory,
            online_players,
            max_players,
            server_version,
            plugins_count,
            loaded_worlds,
            timestamp
        } = req.body;

        // Сохраняем данные сервера в таблицу server_status
        await db.query(`
            INSERT INTO server_status (
                server_ip, server_port, tps, uptime_seconds, 
                max_memory, used_memory, free_memory,
                online_players, max_players, server_version,
                plugins_count, loaded_worlds, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW())
            ON CONFLICT (server_ip, server_port) 
            DO UPDATE SET 
                tps = $3,
                uptime_seconds = $4,
                max_memory = $5,
                used_memory = $6, 
                free_memory = $7,
                online_players = $8,
                max_players = $9,
                server_version = $10,
                plugins_count = $11,
                loaded_worlds = $12,
                updated_at = NOW()
        `, [
            server_ip || 'localhost',
            server_port || 25565,
            tps || null,
            uptime_seconds || null,
            max_memory || null,
            used_memory || null,
            free_memory || null,
            online_players || 0,
            max_players || 20,
            server_version || 'Unknown',
            plugins_count || 0,
            loaded_worlds || 1
        ]);

        res.json({
            success: true,
            message: 'Server data updated',
            timestamp: new Date().toISOString()
        });

    } catch (error) {
        console.error('Ошибка обновления данных сервера:', error);
        res.status(500).json({ 
            success: false, 
            error: 'Failed to update server data',
            message: error.message 
        });
    }
});

module.exports = router;
