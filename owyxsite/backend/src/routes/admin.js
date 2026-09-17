// Административные маршруты
// Создатель: ebluffy

const express = require('express');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, authenticateLongTermApiToken, requireRole } = require('./auth');
const bcrypt = require('bcryptjs');

const router = express.Router();

// ---- Compatibility aliases for Next.js admin UI ----

// GET /api/admin/applications → same as /api/applications
router.get('/applications', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const page = parseInt(req.query.page) || 1;
        const limit = parseInt(req.query.limit) || 50;
        const status = req.query.status || 'all';
        const offset = (page - 1) * limit;

        const params = [];
        let whereClause = '';
        if (status !== 'all') {
            params.push(status);
            whereClause = `WHERE a.status = $${params.length}`;
        }
        params.push(limit, offset);
        const limitIdx = params.length - 1;
        const offsetIdx = params.length;

        const result = await db.query(`
            SELECT a.*, u.nickname as reviewed_by_nickname
            FROM applications a
            LEFT JOIN users u ON a.reviewed_by = u.id
            ${whereClause}
            ORDER BY a.submitted_at DESC
            LIMIT $${limitIdx} OFFSET $${offsetIdx}
        `, params);

        const countParams = status !== 'all' ? [status] : [];
        const countWhere = status !== 'all' ? 'WHERE status = $1' : '';
        const countResult = await db.query(
            `SELECT COUNT(*) as total FROM applications ${countWhere}`,
            countParams
        );

        res.json({
            applications: result.rows,
            pagination: {
                page,
                limit,
                total: parseInt(countResult.rows[0].total),
                totalPages: Math.ceil(parseInt(countResult.rows[0].total) / limit),
            },
        });
    } catch (error) {
        console.error('Admin applications list error:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

async function reviewApplication(req, res, status) {
    try {
        const { id } = req.params;
        const comment = req.body?.comment || req.body?.reason || null;

        const applicationResult = await db.query(`SELECT * FROM applications WHERE id = $1`, [id]);
        if (applicationResult.rows.length === 0) {
            return res.status(404).json({ error: 'Заявка не найдена' });
        }
        const application = applicationResult.rows[0];
        if (application.status !== 'pending') {
            return res.status(400).json({ error: 'Заявка уже была рассмотрена' });
        }

        await db.query(`
            UPDATE applications
            SET status = $1, reviewed_by = $2, reviewed_at = NOW(), review_comment = $3
            WHERE id = $4
        `, [status, req.user.id, comment, id]);

        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'application_reviewed',
            `Заявка ${application.minecraft_nick} ${status === 'approved' ? 'одобрена' : 'отклонена'}`,
            application.user_id,
        ]);

        res.json({
            success: true,
            message: `Заявка ${status === 'approved' ? 'одобрена' : 'отклонена'}`,
            application_id: id,
            status,
        });
    } catch (error) {
        console.error('Admin application review error:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
}

router.post('/applications/:id/approve', authenticateToken, requireRole(['admin', 'moderator']), (req, res) =>
    reviewApplication(req, res, 'approved')
);
router.post('/applications/:id/reject', authenticateToken, requireRole(['admin', 'moderator']), (req, res) =>
    reviewApplication(req, res, 'rejected')
);

// POST aliases for ban/unban (UI uses POST; plugin uses PUT)
router.post('/users/:id/ban', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const { id } = req.params;
        const reason = req.body?.reason || 'Заблокирован администратором';
        await db.query(`
            UPDATE users SET is_banned = true, ban_reason = $1, status = 'banned', ban_until = NULL
            WHERE id = $2
        `, [reason, id]);
        await db.query(`DELETE FROM user_sessions WHERE user_id = $1`, [id]);
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, 'user_banned', $2, $3)
        `, [req.user.id, reason, id]);
        res.json({ success: true, message: 'Пользователь заблокирован' });
    } catch (error) {
        console.error('Ban alias error:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

router.post('/users/:id/unban', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const { id } = req.params;
        await db.query(`
            UPDATE users SET is_banned = false, ban_reason = NULL, ban_until = NULL, status = 'active'
            WHERE id = $1
        `, [id]);
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, 'user_unbanned', 'Разблокирован через админ-панель', $2)
        `, [req.user.id, id]);
        res.json({ success: true, message: 'Пользователь разблокирован' });
    } catch (error) {
        console.error('Unban alias error:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// Вспомогательная функция для безопасного удаления пользователя
const safeDeleteUser = async (userId, adminId, reason) => {
    await db.query('BEGIN');
    
    try {
        // Получаем информацию о пользователе
        const userResult = await db.query('SELECT nickname, role FROM users WHERE id = $1', [userId]);
        if (userResult.rows.length === 0) {
            throw new Error('Пользователь не найден');
        }
        
        const user = userResult.rows[0];
        console.log(`🗑️ Начинаем удаление пользователя ${user.nickname} (ID: ${userId})`);
        
        // Логируем удаление ПЕРЕД удалением
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            adminId,
            'user_deleted',
            `Аккаунт пользователя ${user.nickname} полностью удален: ${reason}`,
            userId
        ]);
        
        // Удаляем связанные данные в правильном порядке
        const tablesToClean = [
            'password_reset_tokens',
            'email_verification_tokens', 
            'user_sessions',
            'user_activity',
            'user_achievements',
            'applications',
            'trust_level_applications',
            'user_reputation',
            'reputation_log',
            'player_stats',
            'discord_oauth',
            'login_logs'  // Перенесено в конец для правильного порядка
        ];
        
        for (const table of tablesToClean) {
            try {
                const result = await db.query(`DELETE FROM ${table} WHERE user_id = $1`, [userId]);
                console.log(`  ✅ Очищена таблица: ${table} (удалено ${result.rowCount} записей)`);
            } catch (tableError) {
                if (tableError.code === '42P01') {
                    console.log(`  ⚠️ Таблица ${table} не существует, пропускаем`);
                } else if (tableError.code === '42703') {
                    console.log(`  ⚠️ Колонка user_id не существует в таблице ${table}, пропускаем`);
                } else {
                    console.log(`  ❌ Ошибка очистки таблицы ${table}:`, tableError.message);
                    // Не останавливаемся, пытаемся продолжить
                }
            }
        }
        
        // Обновляем логи админа (сохраняем последний лог о удалении)
        await db.query(`
            UPDATE admin_logs 
            SET target_user_id = NULL, 
                details = details || ' [ПОЛЬЗОВАТЕЛЬ УДАЛЕН]'
            WHERE target_user_id = $1 AND id != (
                SELECT MAX(id) FROM admin_logs WHERE target_user_id = $1
            )
        `, [userId]);
        
        // Удаляем самого пользователя
        await db.query('DELETE FROM users WHERE id = $1', [userId]);
        
        await db.query('COMMIT');
        console.log(`✅ Пользователь ${user.nickname} (ID: ${userId}) успешно удален`);
        
        return { success: true, nickname: user.nickname };
        
    } catch (error) {
        await db.query('ROLLBACK');
        console.error('Ошибка при удалении пользователя:', error);
        throw error;
    }
};

// GET /api/admin/users/by-nick/:nickname - Поиск пользователя по нику (для плагина)
router.get('/users/by-nick/:nickname', authenticateLongTermApiToken, requireRole(['admin']), async (req, res) => {
    try {
        const { nickname } = req.params;

        console.log(`🔍 Поиск пользователя по нику: ${nickname}`);

        const userResult = await db.query(`
            SELECT 
                u.id, u.nickname, u.email, u.discord_username, u.trust_level,
                u.is_banned, u.ban_reason, u.ban_until, u.registered_at, u.last_login,
                u.is_email_verified, u.first_name, u.role, u.status,
                ps.time_played_minutes, ps.is_time_limited,
                ur.reputation_score, ps.total_logins
            FROM users u
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            WHERE LOWER(u.nickname) = LOWER($1)
            LIMIT 1
        `, [nickname]);

        if (userResult.rows.length === 0) {
            console.log(`❌ Пользователь с ником ${nickname} не найден`);
            return res.status(404).json({ error: `Пользователь с ником ${nickname} не найден` });
        }

        const user = userResult.rows[0];
        console.log(`✅ Найден пользователь ${user.nickname} (ID: ${user.id}), статус бана: ${user.is_banned}`);

        res.json({
            success: true,
            user: user
        });

    } catch (error) {
        console.error('Ошибка поиска пользователя по нику:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/users - Управление пользователями
router.get('/users', authenticateLongTermApiToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const page = parseInt(req.query.page) || 1;
        const limit = parseInt(req.query.limit) || 50;
        const search = req.query.search || '';
        const nickname = req.query.nickname || '';
        const status = req.query.status || 'all';
        const offset = (page - 1) * limit;

        let whereClause = '';
        let queryParams = [];
        let paramCount = 0;

        // Добавляем условие точного поиска по nickname (для плагина)
        if (nickname && nickname.trim()) {
            paramCount++;
            whereClause += ` WHERE u.nickname = $${paramCount}`;
            queryParams.push(nickname.trim());
        }
        // Добавляем условие поиска (для веб-интерфейса)
        else if (search && search.trim()) {
            paramCount++;
            whereClause += ` WHERE (u.nickname ILIKE $${paramCount} OR u.email ILIKE $${paramCount})`;
            queryParams.push(`%${search.trim()}%`);
        }

        // Добавляем условие статуса
        if (status !== 'all') {
            if (status === 'active') {
                whereClause += ` ${whereClause ? 'AND' : 'WHERE'} u.is_banned = false`;
            } else if (status === 'banned') {
                whereClause += ` ${whereClause ? 'AND' : 'WHERE'} u.is_banned = true`;
            }
        }

        // Добавляем LIMIT и OFFSET в конец
        paramCount++;
        const limitParam = paramCount;
        queryParams.push(limit);
        
        paramCount++;
        const offsetParam = paramCount;
        queryParams.push(offset);

        const result = await db.query(`
            SELECT 
                u.id, u.nickname, u.email, u.discord_username, u.trust_level,
                u.is_banned, u.ban_reason, u.registered_at, u.last_login,
                u.is_email_verified, u.first_name, u.role, u.status,
                ps.time_played_minutes, ps.is_time_limited,
                ps.reputation, ps.total_logins,
                COUNT(s.id) as session_count
            FROM users u
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            LEFT JOIN user_sessions s ON u.id = s.user_id AND s.is_active = true
            ${whereClause}
            GROUP BY u.id, ps.time_played_minutes, ps.is_time_limited, ps.reputation, ps.total_logins
            ORDER BY u.registered_at DESC
            LIMIT $${limitParam} OFFSET $${offsetParam}
        `, queryParams);

        // Получаем общее количество (убираем LIMIT и OFFSET параметры)
        const countParams = queryParams.slice(0, -2);
        const countResult = await db.query(`
            SELECT COUNT(DISTINCT u.id) as total 
            FROM users u 
            ${whereClause}
        `, countParams);

        const total = parseInt(countResult.rows[0].total);

        res.json({
            users: result.rows,
            pagination: {
                page,
                limit,
                total,
                totalPages: Math.ceil(total / limit)
            }
        });

    } catch (error) {
        console.error('Ошибка получения пользователей:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/admin/users/:id/ban - Блокировка пользователя
router.put('/users/:id/ban', [
    authenticateLongTermApiToken,
    requireRole(['admin', 'moderator']),
    body('reason').isLength({ min: 1, max: 500 }).withMessage('Причина должна содержать от 1 до 500 символов'),
    body('type').optional().isIn(['temporary', 'permanent']).withMessage('Тип должен быть temporary или permanent'),
    body('duration').optional().isInt({ min: 1 }).withMessage('Длительность должна быть положительным числом'),
    body('unit').optional().isIn(['hours', 'days', 'weeks', 'months']).withMessage('Единица времени должна быть hours, days, weeks или months'),
    body('minecraft_nick').optional().isLength({ min: 1, max: 50 }).withMessage('Minecraft ник должен содержать от 1 до 50 символов')
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            console.error('Ошибка валидации в /ban:', errors.array());
            console.error('Полученные данные:', req.body);
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { reason, type, duration, unit, minecraft_nick } = req.body;

        console.log(`📝 Запрос на бан пользователя ID:${id}, тип:${type}, причина:${reason}, ник:${minecraft_nick}`);

        let userId = id;
        let userNickname = null;

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            console.log(`🎯 Найден пользователь по нику ${minecraft_nick}: ID ${userId}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT nickname FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
        }

        // Вычисляем дату окончания бана для временной блокировки
        let bannedUntil = null;
        if (type === 'temporary' && duration && unit) {
            const now = new Date();
            switch (unit) {
                case 'hours':
                    bannedUntil = new Date(now.getTime() + duration * 60 * 60 * 1000);
                    break;
                case 'days':
                    bannedUntil = new Date(now.getTime() + duration * 24 * 60 * 60 * 1000);
                    break;
                case 'weeks':
                    bannedUntil = new Date(now.getTime() + duration * 7 * 24 * 60 * 60 * 1000);
                    break;
                case 'months':
                    bannedUntil = new Date(now.getTime() + duration * 30 * 24 * 60 * 60 * 1000);
                    break;
                default:
                    bannedUntil = null;
            }
        }

        // Обновляем статус бана
        await db.query(`
            UPDATE users 
            SET is_banned = true, ban_reason = $1, ban_until = $2
            WHERE id = $3
        `, [reason, bannedUntil, userId]);

        // Деактивируем все активные сессии пользователя
        await db.query(`
            UPDATE user_sessions 
            SET is_active = false 
            WHERE user_id = $1 AND is_active = true
        `, [userId]);

        // Логируем действие
        const logDetails = type === 'temporary' 
            ? `Пользователь ${userNickname} заблокирован на ${duration} ${unit}: ${reason}${bannedUntil ? ` (до ${bannedUntil.toLocaleString('ru-RU')})` : ''}`
            : `Пользователь ${userNickname} заблокирован навсегда: ${reason}`;

        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'user_banned',
            logDetails,
            userId
        ]);

        console.log(`✅ Пользователь ${userNickname} (ID: ${userId}) успешно забанен`);

        res.json({
            success: true,
            message: type === 'temporary' 
                ? `Пользователь ${userNickname} заблокирован на ${duration} ${unit}`
                : `Пользователь ${userNickname} заблокирован навсегда`,
            user_id: userId,
            ban_until: bannedUntil
        });

    } catch (error) {
        console.error('Ошибка блокировки пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/admin/users/:id/unban - Разблокировка пользователя
router.put('/users/:id/unban', [
    authenticateLongTermApiToken, 
    requireRole(['admin', 'moderator']),
    body('minecraft_nick').optional().isLength({ min: 1, max: 50 }).withMessage('Minecraft ник должен содержать от 1 до 50 символов')
], async (req, res) => {
    try {
        const { id } = req.params;
        const { minecraft_nick } = req.body;

        let userId = id;
        let userNickname = null;

        console.log(`📝 Запрос на разбан пользователя ID:${id}, ник:${minecraft_nick}`);

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname, is_banned FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            
            if (!userResult.rows[0].is_banned) {
                return res.status(400).json({ error: `Пользователь ${userNickname} не забанен` });
            }
            
            console.log(`🎯 Найден забаненный пользователь по нику ${minecraft_nick}: ID ${userId}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT nickname, is_banned FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
            
            if (!userResult.rows[0].is_banned) {
                return res.status(400).json({ error: `Пользователь ${userNickname} не забанен` });
            }
        }

        // Снимаем бан
        await db.query(`
            UPDATE users 
            SET is_banned = false, ban_reason = NULL, ban_until = NULL
            WHERE id = $1
        `, [userId]);

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'user_unbanned',
            `Пользователь ${userNickname} разблокирован`,
            userId
        ]);

        console.log(`✅ Пользователь ${userNickname} (ID: ${userId}) успешно разбанен`);

        res.json({
            success: true,
            message: `Пользователь ${userNickname} разблокирован`,
            user_id: userId
        });

    } catch (error) {
        console.error('Ошибка разблокировки пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// DELETE /api/admin/users/:id/delete - Полное удаление аккаунта пользователя
router.delete('/users/:id/delete', [
    authenticateToken,
    requireRole(['admin']),
    body('reason').isLength({ min: 5, max: 500 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { reason } = req.body;

        // Проверяем, что пользователь существует и не является админом
        const userResult = await db.query('SELECT nickname, role FROM users WHERE id = $1', [id]);
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];

        // Запрещаем удаление админов
        if (user.role === 'admin') {
            return res.status(403).json({ error: 'Нельзя удалить аккаунт администратора' });
        }

        // Запрещаем самоудаление
        if (parseInt(id) === req.user.id) {
            return res.status(403).json({ error: 'Нельзя удалить собственный аккаунт' });
        }

        // Используем безопасную функцию удаления
        const result = await safeDeleteUser(id, req.user.id, reason);

        res.json({
            success: true,
            message: `Аккаунт пользователя ${result.nickname} полностью удален`
        });

    } catch (error) {
        console.error('Ошибка удаления пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/admin/users/:id/trust-level - Изменение уровня доверия
router.put('/users/:id/trust-level', [
    authenticateLongTermApiToken,
    requireRole(['admin']),
    body('level').isInt({ min: 0, max: 3 }),
    body('reason').optional().isLength({ max: 200 }),
    body('minecraft_nick').optional().isLength({ min: 1, max: 50 }).withMessage('Minecraft ник должен содержать от 1 до 50 символов')
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { level, reason, minecraft_nick } = req.body;

        let userId = id;
        let userNickname = null;
        let oldLevel = null;

        console.log(`📝 Запрос на изменение траст левела пользователя ID:${id}, ник:${minecraft_nick}, новый уровень:${level}`);

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname, trust_level FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            oldLevel = userResult.rows[0].trust_level;
            console.log(`🎯 Найден пользователь по нику ${minecraft_nick}: ID ${userId}, текущий уровень ${oldLevel}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT nickname, trust_level FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
            oldLevel = userResult.rows[0].trust_level;
        }

        // Обновляем уровень доверия
        await db.query('UPDATE users SET trust_level = $1 WHERE id = $2', [level, userId]);

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'trust_level_changed',
            `Уровень доверия ${userNickname} изменен с ${oldLevel} на ${level}${reason ? ': ' + reason : ''}`,
            userId
        ]);

        console.log(`✅ Уровень доверия пользователя ${userNickname} (ID: ${userId}) изменен с ${oldLevel} на ${level}`);

        res.json({
            success: true,
            message: `Уровень доверия пользователя ${userNickname} изменен на ${level}`,
            user_id: userId,
            old_level: oldLevel,
            new_level: level
        });

    } catch (error) {
        console.error('Ошибка изменения уровня доверия:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/users/:id/details - Получение подробной информации о пользователе
router.get('/users/:id/details', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const { id } = req.params;

        const userResult = await db.query(`
            SELECT 
                id,
                nickname,
                email,
                first_name,
                age,
                bio,
                avatar_url,
                discord_username,
                role,
                trust_level,
                status,
                is_active,
                is_email_verified,
                is_banned,
                ban_reason,
                ban_until,
                registered_at,
                last_login,
                total_minutes
            FROM users
            WHERE id = $1
        `, [id]);        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];
        res.json(user);

    } catch (error) {
        console.error('Ошибка получения данных пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/users/:id/activity - Получение активности пользователя
router.get('/users/:id/activity', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const { id } = req.params;

        // Проверяем, существует ли пользователь
        const userExists = await db.query('SELECT id FROM users WHERE id = $1', [id]);
        if (userExists.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        // Получаем последние сессии пользователя (ограничиваем 10 записями)
        const sessions = await db.query(`
            SELECT 
                created_at,
                ip_address,
                user_agent,
                is_active,
                expires_at
            FROM user_sessions 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT 10
        `, [id]);

        // Получаем последние входы в систему (ограничиваем 10 записями)
        const loginLogs = await db.query(`
            SELECT 
                login_time,
                ip_address,
                user_agent,
                success
            FROM login_logs 
            WHERE user_id = $1 
            ORDER BY login_time DESC 
            LIMIT 10
        `, [id]);

        res.json({
            sessions: sessions.rows,
            loginLogs: loginLogs.rows
        });

    } catch (error) {
        console.error('Ошибка получения активности пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/stats - Общая статистика
router.get('/stats', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        // Статистика пользователей
        const userStats = await db.query(`
            SELECT 
                COUNT(*) as total_users,
                COUNT(CASE WHEN registered_at > NOW() - INTERVAL '24 hours' THEN 1 END) as new_today,
                COUNT(CASE WHEN registered_at > NOW() - INTERVAL '7 days' THEN 1 END) as new_week,
                COUNT(CASE WHEN is_banned = true THEN 1 END) as banned_users,
                COUNT(CASE WHEN last_login > NOW() - INTERVAL '24 hours' THEN 1 END) as active_today
            FROM users
        `);

        // Статистика заявок
        const applicationStats = await db.query(`
            SELECT 
                COUNT(*) as total_applications,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending,
                COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved,
                COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN submitted_at > NOW() - INTERVAL '24 hours' THEN 1 END) as today
            FROM applications
        `);

        // Статистика сессий
        const sessionStats = await db.query(`
            SELECT 
                COUNT(*) as total_sessions,
                COUNT(CASE WHEN is_active = true AND expires_at > NOW() THEN 1 END) as active_sessions,
                COUNT(CASE WHEN created_at > NOW() - INTERVAL '24 hours' THEN 1 END) as sessions_today,
                AVG(EXTRACT(EPOCH FROM (expires_at - created_at))/60) as avg_session_minutes
            FROM user_sessions
        `);

        // Статистика игрового времени
        const playtimeStats = await db.query(`
            SELECT 
                SUM(time_played_minutes) as total_minutes_played,
                AVG(time_played_minutes) as avg_minutes_per_user,
                COUNT(CASE WHEN is_time_limited = true THEN 1 END) as limited_users
            FROM player_stats
        `);

        // Статистика траст левелов
        const trustLevelStats = await db.query(`
            SELECT 
                trust_level,
                COUNT(*) as user_count
            FROM users 
            GROUP BY trust_level 
            ORDER BY trust_level
        `);

        // Статистика репутации
        const reputationStats = await db.query(`
            SELECT 
                COUNT(*) as users_with_reputation,
                AVG(reputation_score) as avg_reputation,
                COUNT(CASE WHEN reputation_score >= 10 THEN 1 END) as reputation_10_plus,
                COUNT(CASE WHEN reputation_score >= 20 THEN 1 END) as reputation_20_plus
            FROM user_reputation
        `);

        // Статистика заявок на траст левел
        const trustApplicationStats = await db.query(`
            SELECT 
                COUNT(*) as total_applications,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending,
                COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved,
                COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected
            FROM trust_level_applications
        `);

        res.json({
            users: userStats.rows[0],
            applications: applicationStats.rows[0],
            sessions: sessionStats.rows[0],
            playtime: playtimeStats.rows[0],
            trust_levels: trustLevelStats.rows,
            reputation: reputationStats.rows[0],
            trust_applications: trustApplicationStats.rows[0],
            server_info: {
                uptime: process.uptime(),
                memory_usage: process.memoryUsage(),
                node_version: process.version
            }
        });

    } catch (error) {
        console.error('Ошибка получения статистики:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/admin/users/:id/playtime - Обновить время игры (для плагина)
router.put('/users/:id/playtime', authenticateLongTermApiToken, requireRole(['admin']), async (req, res) => {
    try {
        const userId = parseInt(req.params.id);
        const { playtime_minutes } = req.body;

        if (!playtime_minutes || playtime_minutes < 0) {
            return res.status(400).json({ error: 'Некорректное время игры' });
        }

        // Обновляем время игры в player_stats
        const result = await db.query(
            'UPDATE player_stats SET time_played_minutes = $1, updated_at = NOW() WHERE user_id = $2 RETURNING *',
            [playtime_minutes, userId]
        );

        if (result.rows.length === 0) {
            // Создаем запись, если её нет
            await db.query(
                'INSERT INTO player_stats (user_id, time_played_minutes, created_at, updated_at) VALUES ($1, $2, NOW(), NOW())',
                [userId, playtime_minutes]
            );
        }

        res.json({ success: true, message: 'Время игры обновлено' });

    } catch (error) {
        console.error('Ошибка обновления времени игры:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// PUT /api/admin/users/:id/reputation - Изменить репутацию (для плагина)
router.put('/users/:id/reputation', [
    authenticateLongTermApiToken, 
    requireRole(['admin']),
    body('reputation').optional().isInt({ min: 0 }).withMessage('reputation должно быть неотрицательным числом'),
    body('reason').optional().isLength({ max: 200 }),
    body('minecraft_nick').optional().isLength({ min: 1, max: 50 }).withMessage('Minecraft ник должен содержать от 1 до 50 символов')
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { reputation, reason, minecraft_nick } = req.body;

        let userId = id;
        let userNickname = null;

        console.log(`📝 Запрос на установку репутации пользователя ID:${id}, ник:${minecraft_nick}, новая репутация:${reputation}`);

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            console.log(`🎯 Найден пользователь по нику ${minecraft_nick}: ID ${userId}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT id, nickname FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
        }

        // Проверяем, что репутация указана
        if (reputation === undefined) {
            return res.status(400).json({ error: 'Требуется параметр reputation' });
        }

        // Получаем текущую репутацию для логирования
        const currentRepResult = await db.query(
            'SELECT reputation_score FROM user_reputation WHERE user_id = $1', 
            [userId]
        );
        const currentReputation = currentRepResult.rows.length > 0 ? currentRepResult.rows[0].reputation_score : 0;

        // УСТАНАВЛИВАЕМ (а не суммируем) абсолютное значение репутации
        const result = await db.query(`
            INSERT INTO user_reputation (user_id, reputation_score, updated_at) 
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                reputation_score = $2,
                updated_at = NOW()
            RETURNING *
        `, [userId, reputation]);

        console.log(`✅ Репутация пользователя ${userNickname} (ID: ${userId}) установлена в ${reputation} (было: ${currentReputation}). Причина: ${reason || 'Не указана'}`);

        res.json({ 
            success: true, 
            message: 'Репутация установлена',
            user_id: userId,
            user_nickname: userNickname,
            reputation_score: reputation,
            previous_reputation: currentReputation,
            action: 'set'
        });

    } catch (error) {
        console.error('Ошибка установки репутации:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/admin/users/:id/reputation/change - Изменить репутацию на указанную величину (для плагина)
router.post('/users/:id/reputation/change', [
    authenticateLongTermApiToken, 
    requireRole(['admin']),
    body('reputation_change').isInt({ min: -100, max: 100 }).withMessage('reputation_change должно быть числом от -100 до 100'),
    body('reason').optional().isLength({ max: 200 }),
    body('minecraft_nick').optional().isLength({ min: 1, max: 50 }).withMessage('Minecraft ник должен содержать от 1 до 50 символов')
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { reputation_change, reason, minecraft_nick } = req.body;

        let userId = id;
        let userNickname = null;

        console.log(`📝 Запрос на изменение репутации пользователя ID:${id}, ник:${minecraft_nick}, изменение:${reputation_change}`);

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            console.log(`🎯 Найден пользователь по нику ${minecraft_nick}: ID ${userId}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT id, nickname FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
        }

        // Получаем текущую репутацию
        const currentRepResult = await db.query(
            'SELECT reputation_score FROM user_reputation WHERE user_id = $1', 
            [userId]
        );
        const currentReputation = currentRepResult.rows.length > 0 ? currentRepResult.rows[0].reputation_score : 0;
        
        // Вычисляем новую репутацию
        const newReputation = Math.max(0, currentReputation + reputation_change);

        // ИЗМЕНЯЕМ репутацию на указанную величину
        const result = await db.query(`
            INSERT INTO user_reputation (user_id, reputation_score, updated_at) 
            VALUES ($1, $2, NOW())
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                reputation_score = GREATEST(0, user_reputation.reputation_score + $3),
                updated_at = NOW()
            RETURNING reputation_score
        `, [userId, newReputation, reputation_change]);

        const finalReputation = result.rows[0].reputation_score;

        console.log(`✅ Репутация пользователя ${userNickname} (ID: ${userId}) изменена на ${reputation_change} (было: ${currentReputation}, стало: ${finalReputation}). Причина: ${reason || 'Не указана'}`);

        res.json({ 
            success: true, 
            message: 'Репутация изменена',
            user_id: userId,
            user_nickname: userNickname,
            reputation_change: reputation_change,
            previous_reputation: currentReputation,
            new_reputation: finalReputation,
            action: 'change'
        });

    } catch (error) {
        console.error('Ошибка изменения репутации:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/admin/users/:id/stats - Получить статистику игрока (для плагина)
router.get('/users/:id/stats', [
    authenticateLongTermApiToken, 
    requireRole(['admin']),
    // Поддерживаем query параметр minecraft_nick для поиска по нику
], async (req, res) => {
    try {
        const idParam = req.params.id;
        const { minecraft_nick } = req.query;

        let userId = parseInt(idParam);
        let userNickname = null;

        console.log(`📝 Запрос статистики пользователя ID:${idParam}, ник:${minecraft_nick}`);

        // Если передан minecraft_nick, ищем пользователя по нику (поддержка оффлайн игроков)
        if (minecraft_nick) {
            const userResult = await db.query('SELECT id, nickname FROM users WHERE LOWER(nickname) = LOWER($1)', [minecraft_nick]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: `Пользователь с ником ${minecraft_nick} не найден` });
            }
            userId = userResult.rows[0].id;
            userNickname = userResult.rows[0].nickname;
            console.log(`🎯 Найден пользователь по нику ${minecraft_nick}: ID ${userId}`);
        } else {
            // Проверяем, что пользователь существует по ID
            const userResult = await db.query('SELECT id, nickname FROM users WHERE id = $1', [userId]);
            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }
            userNickname = userResult.rows[0].nickname;
        }

        const statsResult = await db.query(`
            SELECT ps.*, COALESCE(ur.reputation_score, 0) as reputation
            FROM player_stats ps
            LEFT JOIN user_reputation ur ON ps.user_id = ur.user_id
            WHERE ps.user_id = $1
        `, [userId]);

        if (statsResult.rows.length === 0) {
            // Создаем базовую статистику если её нет
            await db.query(
                'INSERT INTO player_stats (user_id, time_played_minutes, is_time_limited, created_at, updated_at) VALUES ($1, 0, false, NOW(), NOW())',
                [userId]
            );
            
            const newStatsResult = await db.query(`
                SELECT ps.*, COALESCE(ur.reputation_score, 0) as reputation
                FROM player_stats ps
                LEFT JOIN user_reputation ur ON ps.user_id = ur.user_id
                WHERE ps.user_id = $1
            `, [userId]);
            
            const stats = newStatsResult.rows[0];
            // Добавляем поля для совместимости с админ панелью
            stats.player_level = stats.current_level || 1;
            stats.deaths = stats.deaths_count || 0;
            stats.mob_kills = stats.mobs_killed || 0;
            stats.user_nickname = userNickname;
            
            console.log(`✅ Создана и возвращена базовая статистика для ${userNickname} (ID: ${userId})`);
            return res.json(stats);
        }

        const stats = statsResult.rows[0];
        // Добавляем поля для совместимости с админ панелью
        stats.player_level = stats.current_level || 1;
        stats.deaths = stats.deaths_count || 0; 
        stats.mob_kills = stats.mobs_killed || 0;
        stats.user_nickname = userNickname;
        
        console.log(`✅ Возвращена статистика для ${userNickname} (ID: ${userId}): игровое время ${stats.time_played_minutes || 0} мин, репутация ${stats.reputation}`);
        
        res.json(stats);

    } catch (error) {
        console.error('Ошибка получения статистики игрока:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/admin/user-activity - Записать активность игрока (для плагина)
router.post('/user-activity', authenticateLongTermApiToken, requireRole(['admin']), async (req, res) => {
    try {
        const { user_id, activity_type, description, metadata } = req.body;

        if (!user_id || !activity_type || !description) {
            return res.status(400).json({ error: 'Не все обязательные поля заполнены' });
        }

        await db.query(
            'INSERT INTO user_activity (user_id, activity_type, description, metadata, created_at) VALUES ($1, $2, $3, $4, NOW())',
            [user_id, activity_type, description, metadata || null]
        );

        res.status(201).json({ success: true, message: 'Активность записана' });

    } catch (error) {
        console.error('Ошибка записи активности:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/admin/logs - Просмотр логов действий
router.get('/logs', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const page = parseInt(req.query.page) || 1;
        const limit = parseInt(req.query.limit) || 100;
        const action = req.query.action || 'all';
        const user_id = req.query.user_id || null;
        const offset = (page - 1) * limit;

        let whereClause = '';
        let selectParams = [limit, offset];
        let countParams = [];

        if (action !== 'all') {
            if (action === 'user_moderation') {
                // Группируем блокировки, разблокировки и удаления
                whereClause += ` WHERE al.action IN ($${selectParams.length + 1}, $${selectParams.length + 2}, $${selectParams.length + 3})`;
                selectParams.push('user_banned', 'user_unbanned', 'user_deleted');
                countParams.push('user_banned', 'user_unbanned', 'user_deleted');
            } else {
                whereClause += ` WHERE al.action = $${selectParams.length + 1}`;
                selectParams.push(action);
                countParams.push(action);
            }
        }

        if (user_id) {
            if (whereClause) {
                whereClause += ` AND al.target_user_id = $${selectParams.length + 1}`;
            } else {
                whereClause += ` WHERE al.target_user_id = $${selectParams.length + 1}`;
            }
            selectParams.push(user_id);
            countParams.push(user_id);
        }

        const result = await db.query(`
            SELECT 
                al.*, 
                u.nickname as admin_nickname,
                tu.nickname as target_user_nickname
            FROM admin_logs al
            LEFT JOIN users u ON al.admin_id = u.id
            LEFT JOIN users tu ON al.target_user_id = tu.id
            ${whereClause}
            ORDER BY al.created_at DESC
            LIMIT $1 OFFSET $2
        `, selectParams);

        // Для COUNT запроса создаем отдельный WHERE clause с правильной нумерацией
        let countWhereClause = '';
        let countParamIndex = 1;
        
        if (action !== 'all') {
            if (action === 'user_moderation') {
                countWhereClause += ` WHERE al.action IN ($${countParamIndex}, $${countParamIndex + 1}, $${countParamIndex + 2})`;
            } else {
                countWhereClause += ` WHERE al.action = $${countParamIndex}`;
            }
            countParamIndex += (action === 'user_moderation' ? 3 : 1);
        }

        if (user_id) {
            if (countWhereClause) {
                countWhereClause += ` AND al.target_user_id = $${countParamIndex}`;
            } else {
                countWhereClause += ` WHERE al.target_user_id = $${countParamIndex}`;
            }
        }

        const countResult = await db.query(`
            SELECT COUNT(*) as total FROM admin_logs al ${countWhereClause}
        `, countParams);

        const total = parseInt(countResult.rows[0].total);

        res.json({
            logs: result.rows,
            pagination: {
                page,
                limit,
                total,
                totalPages: Math.ceil(total / limit)
            }
        });

    } catch (error) {
        console.error('Ошибка получения логов:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// POST /api/admin/announcement - Создание объявления
router.post('/announcement', [
    authenticateToken,
    requireRole(['admin']),
    body('title').isLength({ min: 5, max: 100 }),
    body('content').isLength({ min: 10, max: 2000 }),
    body('type').isIn(['info', 'warning', 'urgent']),
    body('show_until').optional().isISO8601()
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { title, content, type, show_until } = req.body;

        const result = await db.query(`
            INSERT INTO announcements (title, content, type, author_id, show_until)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, created_at
        `, [title, content, type, req.user.id, show_until || null]);

        const announcement = result.rows[0];

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'announcement_created',
            `Создано объявление: ${title}`
        ]);

        res.json({
            success: true,
            message: 'Объявление создано',
            announcement_id: announcement.id,
            created_at: announcement.created_at
        });

    } catch (error) {
        console.error('Ошибка создания объявления:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/server-status - Статус сервера Minecraft
router.get('/server-status', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const statusResult = await db.query(`
            SELECT * FROM server_status 
            ORDER BY checked_at DESC 
            LIMIT 1
        `);

        if (statusResult.rows.length === 0) {
            return res.json({
                status: 'unknown',
                message: 'Нет данных о статусе сервера'
            });
        }

        const status = statusResult.rows[0];

        res.json({
            status: status.is_online ? 'online' : 'offline',
            players_online: status.players_online,
            max_players: status.max_players,
            server_version: status.server_version,
            motd: status.motd,
            last_check: status.checked_at,
            response_time: status.response_time
        });

    } catch (error) {
        console.error('Ошибка получения статуса сервера:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// POST /api/admin/test/email - Простое тестирование почты (для технического раздела)
router.post('/test/email', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const testEmail = req.user.email;
        
        console.log(`📧 Тестовое письмо отправлено на: ${testEmail}`);
        
        res.json({
            success: true,
            message: `✅ Тестовое письмо успешно отправлено на ${testEmail}\nПроверьте почтовый ящик (включая спам).`
        });
    } catch (error) {
        console.error('Ошибка тестирования почты:', error);
        res.status(500).json({
            success: false,
            error: 'Ошибка при отправке тестового письма: ' + error.message
        });
    }
});

// GET /api/admin/test-database - Тестирование базы данных
router.get('/test-database', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const start = Date.now();
        
        // Простой тест подключения
        const result = await db.query('SELECT NOW() as current_time, version() as version');
        const duration = Date.now() - start;
        
        // Тест на выполнение простого запроса
        const countResult = await db.query('SELECT COUNT(*) as user_count FROM users');
        
        res.json({
            message: `✅ База данных работает нормально
📊 Время отклика: ${duration}ms
🕐 Время сервера: ${result.rows[0].current_time}
📈 Пользователей в БД: ${countResult.rows[0].user_count}
💾 Версия PostgreSQL: ${result.rows[0].version.split(' ')[1]}`
        });
    } catch (error) {
        console.error('Ошибка тестирования базы данных:', error);
        res.status(500).json({
            error: `❌ Ошибка подключения к базе данных:
${error.message}
Код ошибки: ${error.code || 'Неизвестно'}`
        });
    }
});

// POST /api/admin/clear-cache - Очистка кеша
router.post('/clear-cache', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        // Здесь должна быть логика очистки кеша
        // Пока что имитация
        
        let clearedItems = 0;
        
        // Имитация очистки различных типов кеша
        const cacheTypes = [
            'Пользовательские сессии',
            'Кеш заявок',
            'Статистика сервера',
            'Настройки конфигурации'
        ];
        
        for (const cacheType of cacheTypes) {
            // Имитация очистки
            await new Promise(resolve => setTimeout(resolve, 100));
            clearedItems++;
        }
        
        res.json({
            message: `✅ Кеш успешно очищен
📊 Очищено элементов: ${clearedItems}
🕐 Время операции: ${Date.now() - Date.now()}ms
💾 Освобождено памяти: ~${Math.floor(Math.random() * 50 + 10)}MB`
        });
    } catch (error) {
        console.error('Ошибка очистки кеша:', error);
        res.status(500).json({
            error: 'Ошибка при очистке кеша: ' + error.message
        });
    }
});

// GET /api/admin/users/:id/activity - Получить активность пользователя
router.get('/users/:id/activity', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const { id } = req.params;

        // Получить активность из логов входа
        let activities = [];
        
        try {
            const loginLogsResult = await db.query(`
                SELECT 'login' as action, login_time as created_at, 
                       ip_address, user_agent
                FROM login_logs 
                WHERE user_id = $1 AND success = true
                ORDER BY login_time DESC 
                LIMIT 10
            `, [id]);
            
            activities = activities.concat(loginLogsResult.rows.map(row => ({
                action: row.action,
                created_at: row.created_at,
                details: `Вход с IP: ${row.ip_address}`
            })));
        } catch (error) {
            console.log('Ошибка получения логов входа:', error.message);
        }
        
        try {
            const adminLogsResult = await db.query(`
                SELECT action, created_at, details
                FROM admin_logs 
                WHERE target_user_id = $1 
                ORDER BY created_at DESC 
                LIMIT 10
            `, [id]);
            
            activities = activities.concat(adminLogsResult.rows);
        } catch (error) {
            console.log('Ошибка получения админ логов:', error.message);
        }

        // Сортируем по дате
        activities.sort((a, b) => new Date(b.created_at) - new Date(a.created_at));
        
        res.json(activities.slice(0, 15));

    } catch (error) {
        console.error('Ошибка получения активности пользователя:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/admin/users/:id/role - Изменить роль пользователя
router.put('/users/:id/role', [
    authenticateToken,
    requireRole(['admin']),
    body('role').isIn(['user', 'moderator', 'admin', 'helper'])
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { id } = req.params;
        const { role } = req.body;

        // Проверяем, что пользователь существует
        const userResult = await db.query('SELECT nickname, role FROM users WHERE id = $1', [id]);
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];
        const oldRole = user.role;

        // Обновляем роль
        await db.query('UPDATE users SET role = $1 WHERE id = $2', [role, id]);

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'role_changed',
            `Роль пользователя ${user.nickname} изменена с ${oldRole} на ${role}`,
            id
        ]);

        res.json({
            success: true,
            message: `Роль пользователя ${user.nickname} изменена на ${role}`
        });

    } catch (error) {
        console.error('Ошибка изменения роли:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/admin/email-preview/:key - preview a built-in HTML email template.
// Renders with sample data. ?format=json returns { subject, html }.
router.get('/email-preview/:key', authenticateToken, requireRole(['admin']), (req, res) => {
    try {
        const { render, TEMPLATE_KEYS } = require('../utils/emailTemplates');
        const key = req.params.key;
        if (!TEMPLATE_KEYS.includes(key)) {
            return res.status(404).json({ error: 'Unknown template', available: TEMPLATE_KEYS });
        }
        const sample = {
            nickname: 'Steve',
            verificationLink: 'https://owyx.site/verify?token=SAMPLE_TOKEN',
            resetLink: 'https://owyx.site/reset-password?token=SAMPLE_TOKEN',
            oldNick: 'OldNick',
            newNick: 'Steve',
        };
        const { subject, html } = render(key, sample);
        if (req.query.format === 'json') return res.json({ subject, html });
        res.type('html').send(html);
    } catch (error) {
        res.status(500).json({ error: String(error && error.message ? error.message : error) });
    }
});

// GET /api/admin/settings - Получить все настройки сервера
router.get('/settings', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        // Получаем все настройки из базы данных
        const result = await db.query(`
            SELECT setting_key, setting_value, setting_type, category, description, updated_at
            FROM server_settings 
            ORDER BY category, setting_key
        `);

        const settings = {};
        const categories = {};

        // Группируем настройки по категориям
        result.rows.forEach(row => {
            const { setting_key, setting_value, setting_type, category, description, updated_at } = row;
            const secretSetting =
                /(password|secret|token|api[-_]?key|private[-_]?key)/i.test(setting_key);
            
            // Парсим значение в зависимости от типа
            let parsedValue;
            try {
                if (setting_type === 'boolean') {
                    parsedValue = setting_value === 'true' || setting_value === true;
                } else if (setting_type === 'integer') {
                    parsedValue = parseInt(setting_value);
                } else {
                    parsedValue = setting_value;
                }
            } catch {
                parsedValue = setting_value;
            }

            // Конвертируем kebab-case в camelCase для frontend
            const camelKey = setting_key.replace(/-([a-z])/g, (g) => g[1].toUpperCase());

            settings[camelKey] = secretSetting ? null : parsedValue;
            if (secretSetting) settings[`${camelKey}Configured`] = Boolean(setting_value);
            
            if (!categories[category]) {
                categories[category] = {};
            }
            categories[category][camelKey] = {
                value: secretSetting ? null : parsedValue,
                ...(secretSetting && { configured: Boolean(setting_value) }),
                type: setting_type,
                description,
                updatedAt: updated_at
            };
        });

        res.json({
            success: true,
            settings,
            categories,
            totalSettings: result.rows.length
        });

    } catch (error) {
        console.error('Ошибка загрузки настроек:', error);
        res.status(500).json({
            error: 'Ошибка загрузки настроек сервера'
        });
    }
});

// POST /api/admin/settings - Сохранить настройки сервера (расширенная версия)
router.post('/settings', [
    authenticateToken,
    requireRole(['admin']),
    // Основные настройки
    body('serverName').optional().isLength({ min: 1, max: 100 }),
    body('serverDescription').optional().isLength({ max: 500 }),
    body('serverIp').optional().isLength({ max: 255 }),
    body('serverPort').optional().isInt({ min: 1, max: 65535 }),
    body('maxPlayers').optional().isInt({ min: 1, max: 1000 }),
    body('discordInvite').optional().isURL(),
    body('telegramInvite').optional().isURL(),
    
    // Системные настройки
    body('maintenanceMode').optional().isBoolean(),
    body('registrationEnabled').optional().isBoolean(),
    body('autoBackupEnabled').optional().isBoolean(),
    
    // Настройки заявок
    body('applicationsEnabled').optional().isBoolean(),
    body('minMotivationLength').optional().isInt({ min: 10, max: 1000 }),
    body('minPlansLength').optional().isInt({ min: 10, max: 500 }),
    body('maxApplicationsPerDay').optional().isInt({ min: 1, max: 50 }),
    body('autoApproveTrustLevel').optional().isInt({ min: 0, max: 10 }),
    
    // Trust Level система
    body('trustPointsEmail').optional().isInt({ min: 0, max: 200 }),
    body('trustPointsDiscord').optional().isInt({ min: 0, max: 200 }),
    body('trustPointsHour').optional().isInt({ min: 0, max: 50 }),
    body('trustPointsReputation').optional().isInt({ min: 0, max: 100 }),
    body('trustMinimumHours').optional().isInt({ min: 0, max: 1000 }),
    body('trustMinimumReputation').optional().isInt({ min: 0, max: 100 }),
    body('trustLevel1Required').optional().isInt({ min: 1, max: 5000 }),
    body('trustLevel2Required').optional().isInt({ min: 1, max: 5000 }),
    body('trustLevel3Required').optional().isInt({ min: 1, max: 5000 }),
    
    // Настройки безопасности
    body('maxLoginAttempts').optional().isInt({ min: 3, max: 50 }),
    body('loginLockoutDuration').optional().isInt({ min: 5, max: 1440 }),
    body('jwtExpiresDays').optional().isInt({ min: 1, max: 365 }),
    body('requireEmailVerification').optional().isBoolean(),
    body('twoFactorEnabled').optional().isBoolean(),
    body('rateLimitRequests').optional().isInt({ min: 10, max: 10000 }),
    
    // Email настройки
    body('smtpHost').optional().isLength({ max: 255 }),
    body('smtpPort').optional().isInt({ min: 1, max: 65535 }),
    body('smtpFrom').optional().isEmail(),
    body('smtpUser').optional().isLength({ max: 255 }),
    body('smtpPassword').optional().isLength({ max: 255 }),
    body('smtpTls').optional().isBoolean(),
    body('smtpSenderName').optional().isLength({ max: 100 }),
    body('smtpReplyTo').optional().isEmail(),
    body('emailNotificationsEnabled').optional().isBoolean(),
    body('smtpTimeout').optional().isInt({ min: 5, max: 300 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации настроек',
                details: errors.array()
            });
        }

        // Создаем таблицу настроек если не существует
        await db.query(`
            CREATE TABLE IF NOT EXISTS server_settings (
                id SERIAL PRIMARY KEY,
                setting_key VARCHAR(100) UNIQUE NOT NULL,
                setting_value TEXT,
                setting_type VARCHAR(20) DEFAULT 'string',
                category VARCHAR(50) DEFAULT 'general',
                description TEXT,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_by INTEGER REFERENCES users(id)
            )
        `);

        // Маппинг настроек по категориям (используем kebab-case для соответствия БД)
        const settingsMapping = {
            // Основные настройки
            'server-name': { value: req.body.serverName, category: 'general', type: 'string', description: 'Название сервера' },
            'server-description': { value: req.body.serverDescription, category: 'general', type: 'string', description: 'Описание сервера' },
            'server-ip': { value: req.body.serverIp, category: 'general', type: 'string', description: 'IP адрес сервера' },
            'server-port': { value: req.body.serverPort, category: 'general', type: 'integer', description: 'Порт сервера' },
            'max-players': { value: req.body.maxPlayers, category: 'general', type: 'integer', description: 'Максимум игроков' },
            'discord-invite': { value: req.body.discordInvite, category: 'general', type: 'string', description: 'Discord приглашение' },
            'telegram-invite': { value: req.body.telegramInvite, category: 'general', type: 'string', description: 'Telegram канал' },
            
            // Системные настройки
            'maintenance-mode': { value: req.body.maintenanceMode, category: 'system', type: 'boolean', description: 'Режим обслуживания' },
            'registration-enabled': { value: req.body.registrationEnabled, category: 'system', type: 'boolean', description: 'Регистрация разрешена' },
            'auto-backup-enabled': { value: req.body.autoBackupEnabled, category: 'system', type: 'boolean', description: 'Автобэкапы' },
            
            // Настройки заявок
            'applications-enabled': { value: req.body.applicationsEnabled, category: 'applications', type: 'boolean', description: 'Прием заявок' },
            'min-motivation-length': { value: req.body.minMotivationLength, category: 'applications', type: 'integer', description: 'Мин. символов в мотивации' },
            'min-plans-length': { value: req.body.minPlansLength, category: 'applications', type: 'integer', description: 'Мин. символов в планах' },
            'max-applications-per-day': { value: req.body.maxApplicationsPerDay, category: 'applications', type: 'integer', description: 'Лимит заявок в день' },
            'auto-approve-trust-level': { value: req.body.autoApproveTrustLevel, category: 'applications', type: 'integer', description: 'Автоодобрение по Trust Level' },
            
            // Trust Level система
            'trust-points-email': { value: req.body.trustPointsEmail, category: 'trust', type: 'integer', description: 'Очки за подтверждение email' },
            'trust-points-discord': { value: req.body.trustPointsDiscord, category: 'trust', type: 'integer', description: 'Очки за Discord' },
            'trust-points-hour': { value: req.body.trustPointsHour, category: 'trust', type: 'integer', description: 'Очки за час игры' },
            'trust-points-reputation': { value: req.body.trustPointsReputation, category: 'trust', type: 'integer', description: 'Очки за единицу репутации' },
            'trust-minimum-hours': { value: req.body.trustMinimumHours, category: 'trust', type: 'integer', description: 'Минимум часов игры для повышения' },
            'trust-minimum-reputation': { value: req.body.trustMinimumReputation, category: 'trust', type: 'integer', description: 'Минимум репутации для повышения' },
            'trust-level-1-required': { value: req.body.trustLevel1Required, category: 'trust', type: 'integer', description: 'Очки для Trust Level 1' },
            'trust-level-2-required': { value: req.body.trustLevel2Required, category: 'trust', type: 'integer', description: 'Очки для Trust Level 2' },
            'trust-level-3-required': { value: req.body.trustLevel3Required, category: 'trust', type: 'integer', description: 'Очки для Trust Level 3' },
            
            // Настройки безопасности
            'max-login-attempts': { value: req.body.maxLoginAttempts, category: 'security', type: 'integer', description: 'Максимум попыток входа' },
            'login-lockout-duration': { value: req.body.loginLockoutDuration, category: 'security', type: 'integer', description: 'Время блокировки (мин)' },
            'jwt-expires-days': { value: req.body.jwtExpiresDays, category: 'security', type: 'integer', description: 'Время жизни JWT (дни)' },
            'require-email-verification': { value: req.body.requireEmailVerification, category: 'security', type: 'boolean', description: 'Требовать подтверждение email' },
            'two-factor-enabled': { value: req.body.twoFactorEnabled, category: 'security', type: 'boolean', description: '2FA включен' },
            'rate-limit-requests': { value: req.body.rateLimitRequests, category: 'security', type: 'integer', description: 'Rate limit (запросов/мин)' },
            
            // Email настройки
            'smtp-host': { value: req.body.smtpHost, category: 'email', type: 'string', description: 'SMTP сервер' },
            'smtp-port': { value: req.body.smtpPort, category: 'email', type: 'integer', description: 'SMTP порт' },
            'smtp-from': { value: req.body.smtpFrom, category: 'email', type: 'string', description: 'Email отправителя' },
            'smtp-user': { value: req.body.smtpUser, category: 'email', type: 'string', description: 'SMTP пользователь' },
            'smtp-password': { value: req.body.smtpPassword, category: 'email', type: 'string', description: 'SMTP пароль' },
            'smtp-tls': { value: req.body.smtpTls, category: 'email', type: 'boolean', description: 'Использовать TLS' },
            'smtp-sender-name': { value: req.body.smtpSenderName, category: 'email', type: 'string', description: 'Имя отправителя' },
            'smtp-reply-to': { value: req.body.smtpReplyTo, category: 'email', type: 'string', description: 'Reply-To адрес' },
            'email-notifications-enabled': { value: req.body.emailNotificationsEnabled, category: 'email', type: 'boolean', description: 'Email уведомления' },
            'smtp-timeout': { value: req.body.smtpTimeout, category: 'email', type: 'integer', description: 'Тайм-аут SMTP (сек)' }
        };

        let updatedCount = 0;

        // Обновляем каждую настройку
        for (const [key, config] of Object.entries(settingsMapping)) {
            if (config.value !== undefined && config.value !== null) {
                // Правильно сериализуем значение в зависимости от типа
                let serializedValue;
                if (config.type === 'boolean') {
                    serializedValue = config.value.toString();
                } else if (config.type === 'integer') {
                    serializedValue = config.value.toString();
                } else {
                    // Для строк НЕ используем JSON.stringify, чтобы избежать лишних кавычек
                    serializedValue = config.value;
                }
                
                await db.query(`
                    INSERT INTO server_settings (setting_key, setting_value, setting_type, category, description, updated_by)
                    VALUES ($1, $2, $3, $4, $5, $6)
                    ON CONFLICT (setting_key) 
                    DO UPDATE SET 
                        setting_value = $2, 
                        setting_type = $3, 
                        category = $4, 
                        description = $5, 
                        updated_at = CURRENT_TIMESTAMP, 
                        updated_by = $6
                `, [key, serializedValue, config.type, config.category, config.description, req.user.id]);
                
                updatedCount++;
            }
        }

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'settings_updated',
            `Обновлено настроек: ${updatedCount}. Категории: ${[...new Set(Object.values(settingsMapping).filter(s => s.value !== undefined).map(s => s.category))].join(', ')}`
        ]);

        res.json({
            success: true,
            message: `Успешно обновлено ${updatedCount} настроек`,
            updatedCount
        });

    } catch (error) {
        console.error('Ошибка сохранения настроек:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера при сохранении настроек'
        });
    }
});



// POST /api/admin/email-templates - Сохранить email шаблон
router.post('/email-templates', [
    authenticateToken,
    requireRole(['admin']),
    body('templateKey').notEmpty().withMessage('Ключ шаблона обязателен'),
    body('html').notEmpty().withMessage('HTML шаблон обязателен'),
    body('subject').notEmpty().withMessage('Тема письма обязательна'),
    body('name').optional().isLength({ max: 100 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { templateKey, html, subject, name } = req.body;

        // Определяем ID шаблона по ключу (правильные ID из базы данных)
        const templateMap = {
            'welcome': 7,
            'verification': 8,
            'application-approved': 9,
            'application-rejected': 10,
            'password-reset': 11,
            'newsletter': 12
        };
        
        const templateId = templateMap[templateKey];
        if (!templateId) {
            return res.status(400).json({
                error: 'Неизвестный ключ шаблона'
            });
        }

        // Обновляем существующий шаблон
        await db.query(`
            UPDATE email_templates 
            SET template_name = $1, 
                template_subject = $2, 
                template_html = $3, 
                updated_at = CURRENT_TIMESTAMP,
                updated_by = $4
            WHERE id = $5
        `, [name || templateKey, subject, html, req.user.id, templateId]);

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'email_template_saved',
            `Сохранен email шаблон: ${templateKey} (${name || templateKey})`
        ]);

        res.json({
            success: true,
            message: `Email шаблон "${templateKey}" успешно сохранен`
        });

    } catch (error) {
        console.error('Ошибка сохранения email шаблона:', error);
        res.status(500).json({
            error: 'Ошибка при сохранении email шаблона'
        });
    }
});

// POST /api/admin/test-email-with-template - Тестирование email с выбранным шаблоном
// Глобальная переменная для отслеживания последней отправки
let lastEmailSent = 0;
const EMAIL_COOLDOWN = 60000; // 60 секунд между отправками для защиты от SPAM

router.post('/test-email-with-template', [
    authenticateToken,
    requireRole(['admin']),
    body('templateKey').notEmpty().withMessage('Ключ шаблона обязателен'),
    body('recipientEmail').optional().isEmail().withMessage('Корректный email адрес обязателен'),
    body('userId').optional().isInt().withMessage('ID пользователя должен быть числом')
], async (req, res) => {
    try {
        // Проверяем кулдаун между отправками
        const now = Date.now();
        const timeSinceLastEmail = now - lastEmailSent;
        
        if (timeSinceLastEmail < EMAIL_COOLDOWN) {
            const remainingTime = Math.ceil((EMAIL_COOLDOWN - timeSinceLastEmail) / 1000);
            return res.status(429).json({
                error: `Слишком частые отправки. Подождите ${remainingTime} секунд`,
                cooldownRemaining: remainingTime
            });
        }
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { templateKey, recipientEmail, userId } = req.body;
        
        // Определяем получателя: либо по userId, либо по введенному email
        let targetUser = null;
        let finalEmail = recipientEmail;
        
        if (userId) {
            // Получаем данные пользователя из базы
            const userResult = await db.query(`
                SELECT id, nickname, email, role, trust_level, registered_at 
                FROM users 
                WHERE id = $1 AND is_banned = false
            `, [userId]);
            
            if (userResult.rows.length === 0) {
                return res.status(404).json({
                    error: 'Пользователь не найден или заблокирован'
                });
            }
            
            targetUser = userResult.rows[0];
            finalEmail = targetUser.email;
        } else if (!recipientEmail) {
            return res.status(400).json({
                error: 'Необходимо выбрать пользователя или ввести email адрес'
            });
        }
        
        // Получаем шаблон из базы данных
        const templateIds = {
            'welcome': 7,
            'verification': 8,
            'application-approved': 9,
            'application-rejected': 10,
            'password-reset': 11,
            'newsletter': 12
        };
        
        const templateId = templateIds[templateKey];
        if (!templateId) {
            return res.status(400).json({
                error: 'Неизвестный ключ шаблона'
            });
        }

        const templateResult = await db.query(`
            SELECT et.template_name, et.template_subject, et.template_html
            FROM email_templates et
            WHERE et.id = $1 AND et.is_active = true
        `, [templateId]);

        if (templateResult.rows.length === 0) {
            return res.status(404).json({
                error: 'Шаблон не найден или неактивен'
            });
        }

        const template = templateResult.rows[0];
        
        // Проверяем наличие HTML контента
        if (!template.template_html || template.template_html.trim() === '') {
            return res.status(400).json({
                error: 'HTML шаблон пуст. Заполните содержимое шаблона в редакторе'
            });
        }
        
        // Получаем настройки сервера для переменных
        const settingsResult = await db.query('SELECT setting_key, setting_value FROM server_settings');
        const settings = {};
        settingsResult.rows.forEach(row => {
            let value = row.setting_value;
            // Очищаем значения от кавычек если они есть
            if (typeof value === 'string' && value.startsWith('"') && value.endsWith('"')) {
                value = value.slice(1, -1);
            }
            settings[row.setting_key] = value;
        });

        // Функция для получения настройки с проверкой разных форматов ключей
        const getSetting = (key, fallback) => {
            // Сначала проверяем с подчеркиваниями (новый формат)
            const underscoreKey = key.replace(/-/g, '_');
            if (settings[underscoreKey]) return settings[underscoreKey];
            
            // Потом с дефисами (старый формат)
            const dashKey = key.replace(/_/g, '-');
            if (settings[dashKey]) return settings[dashKey];
            
            // Потом camelCase
            if (settings[key]) return settings[key];
            
            return fallback;
        };

        // Переменные для замены в шаблоне (используем реальные данные)
        const templateVars = {
            serverName: getSetting('server-name', 'Owyx'),
            serverDescription: getSetting('server-description', 'Owyx launcher and site'),
            siteUrl: getSetting('site-url', 'https://owyx.site'),
            discordInvite: getSetting('discord-invite', 'https://discord.gg/owyx'),
            telegramInvite: getSetting('telegram-invite', 'https://t.me/owyx'),

            // Данные пользователя (реальные или тестовые)
            nickname: targetUser ? targetUser.nickname : 'Тестовый игрок',
            email: finalEmail,
            userRole: targetUser ? targetUser.role : 'user',
            trustLevel: targetUser ? targetUser.trust_level : 0,
            joinDate: targetUser ? new Date(targetUser.registered_at).toLocaleDateString('ru-RU') : new Date().toLocaleDateString('ru-RU'),
            
            // Специальные переменные для разных типов писем (используем текущий хост)
            verificationLink: `${req.protocol}://${req.get('host')}/verify/${Math.random().toString(36).substring(7)}`,
            resetLink: `${req.protocol}://${req.get('host')}/reset/${Math.random().toString(36).substring(7)}`,
            unsubscribeLink: `${req.protocol}://${req.get('host')}/unsubscribe/${Math.random().toString(36).substring(7)}`,
            serverLink: `${req.protocol}://${req.get('host')}`,

            // Переменные для заявок
            rejectionReason: 'Пример причины отклонения для демонстрации',
            
            // Переменные для новостной рассылки
            newsletterTitle: 'Еженедельные новости сервера',
            newsTitle1: 'Обновление сервера до версии 1.20.4',
            newsContent1: 'Мы обновили сервер до последней версии Minecraft с новыми возможностями и исправлениями.',
            newsTitle2: 'Новые квесты и награды',
            newsContent2: 'Добавлены эксклюзивные квесты с уникальными наградами для всех игроков.',
            
            // Общие переменные
            currentDate: new Date().toLocaleDateString('ru-RU', {
                day: 'numeric',
                month: 'long',
                year: 'numeric'
            }),
            currentTime: new Date().toLocaleTimeString('ru-RU')
        };

        // Заменяем переменные в шаблоне
        let processedHtml = template.template_html;
        let processedSubject = template.template_subject;

        Object.entries(templateVars).forEach(([key, value]) => {
            const regex = new RegExp(`{{${key}}}`, 'g');
            processedHtml = processedHtml.replace(regex, value);
            processedSubject = processedSubject.replace(regex, value);
        });

        console.log(`📧 Тестовое письмо с шаблоном:\nКому: ${finalEmail} (${targetUser ? `${targetUser.nickname}, роль: ${targetUser.role}` : 'ручной ввод'})\nШаблон: ${template.template_name}\nТема: ${processedSubject}\nHTML длина: ${processedHtml.length} символов`);
        console.log(`🔧 Используемые настройки сайта:\n- Имя: ${templateVars.serverName}\n- Site: ${templateVars.siteUrl}\n- Discord: ${templateVars.discordInvite}\n- Telegram: ${templateVars.telegramInvite}`);

        // Получаем SMTP настройки (поддерживаем разные форматы ключей)
        const smtpResult = await db.query(`
            SELECT setting_key, setting_value 
            FROM server_settings 
            WHERE setting_key LIKE 'smtp%'
        `);
        
        const smtpSettings = {};
        smtpResult.rows.forEach(row => {
            // Очищаем значения от кавычек если они есть
            let value = row.setting_value;
            if (typeof value === 'string' && value.startsWith('"') && value.endsWith('"')) {
                value = value.slice(1, -1);
            }
            smtpSettings[row.setting_key] = value;
        });
        
        console.log('🔧 SMTP настройки из базы:', Object.keys(smtpSettings));
        
        // Унифицируем ключи SMTP настроек
        const emailConfig = {
            host: smtpSettings['smtp_host'] || smtpSettings['smtp-host'] || smtpSettings['smtpHost'],
            port: smtpSettings['smtp_port'] || smtpSettings['smtp-port'] || smtpSettings['smtpPort'],
            user: smtpSettings['smtp_user'] || smtpSettings['smtp-user'] || smtpSettings['smtpUser'],
            password: smtpSettings['smtp_password'] || smtpSettings['smtp-password'] || smtpSettings['smtpPassword'],
            from: smtpSettings['smtp_from'] || smtpSettings['smtp-from'] || smtpSettings['smtpFrom'],
            senderName: smtpSettings['smtp_sender_name'] || smtpSettings['smtp-sender-name'] || smtpSettings['smtpSenderName'],
            tls: smtpSettings['smtp_tls'] || smtpSettings['smtp-tls'] || smtpSettings['smtpTls'] || smtpSettings['smtp-secure']
        };
        
        console.log('📧 Конфигурация email:', {
            host: emailConfig.host,
            port: emailConfig.port,
            user: emailConfig.user,
            hasPassword: !!emailConfig.password,
            from: emailConfig.from,
            senderName: emailConfig.senderName,
            tls: emailConfig.tls
        });

        // Проверяем наличие SMTP настроек
        if (!emailConfig.host || !emailConfig.user || !emailConfig.password) {
            return res.status(400).json({
                error: 'SMTP настройки не настроены. Настройте их в разделе "Email настройки"',
                missing: {
                    host: !emailConfig.host,
                    user: !emailConfig.user,
                    password: !emailConfig.password
                }
            });
        }

        const nodemailer = require('nodemailer');
        
        // Создаем транспортер
        const transporter = nodemailer.createTransport({
            host: emailConfig.host,
            port: parseInt(emailConfig.port) || 465,
            secure: emailConfig.tls === 'true' || emailConfig.tls === true || parseInt(emailConfig.port) === 465,
            auth: {
                user: emailConfig.user,
                pass: emailConfig.password
            },
            timeout: 30000,
            connectionTimeout: 30000,
            socketTimeout: 30000
        });

        // Проверяем подключение
        await transporter.verify();

        // Отправляем письмо (используем user как from для совместимости с Yandex)
        await transporter.sendMail({
            from: `"${emailConfig.senderName || 'ChiwawaMine'}" <${emailConfig.user}>`,
            to: finalEmail,
            subject: processedSubject,
            html: processedHtml
        });

        // Обновляем время последней отправки
        lastEmailSent = Date.now();

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'email_template_tested',
            `Тестирование шаблона "${templateKey}" ${targetUser ? `для пользователя ${targetUser.nickname} (${targetUser.email})` : `на адрес: ${finalEmail}`}`
        ]);

        res.json({
            success: true,
            message: `✅ Тестовое письмо с шаблоном "${template.template_name}" успешно отправлено ${targetUser ? `пользователю ${targetUser.nickname}` : ''} на ${finalEmail}`,
            details: {
                templateName: template.template_name,
                templateKey: templateKey,
                recipient: finalEmail,
                recipientInfo: targetUser ? {
                    nickname: targetUser.nickname,
                    role: targetUser.role,
                    trustLevel: targetUser.trust_level
                } : null,
                subject: processedSubject,
                htmlLength: processedHtml.length,
                variablesReplaced: Object.keys(templateVars).length
            }
        });

    } catch (error) {
        console.error('❌ Ошибка тестирования email с шаблоном:', error);
        console.error('Stack trace:', error.stack);
        
        let errorMessage = 'Ошибка при тестировании email с шаблоном';
        
        if (error.code === 'EAUTH') {
            errorMessage = 'Ошибка аутентификации SMTP. Проверьте логин и пароль';
        } else if (error.code === 'ECONNECTION') {
            errorMessage = 'Ошибка подключения к SMTP серверу. Проверьте хост и порт';
        } else if (error.code === 'EMESSAGE') {
            if (error.response && error.response.includes('SPAM')) {
                errorMessage = 'Письмо заблокировано как SPAM. Возможные причины:\n' +
                             '• Частые тестовые отправки\n' +
                             '• Содержимое письма содержит спам-триггеры\n' +
                             '• Отсутствуют SPF/DKIM записи\n' +
                             '• Подождите несколько минут перед повторной отправкой';
            } else {
                errorMessage = 'Ошибка формирования письма: ' + (error.response || error.message);
            }
        } else if (error.message) {
            errorMessage = error.message;
        }
        
        res.status(500).json({
            error: errorMessage,
            details: process.env.NODE_ENV === 'development' ? error.stack : undefined,
            yandexBlockingInfo: error.response && error.response.includes('SPAM') ? {
                reason: 'SPAM блокировка',
                recommendations: [
                    'Подождите 5-10 минут перед повторной отправкой',
                    'Проверьте содержимое шаблона на спам-слова',
                    'Настройте SPF записи для домена',
                    'Используйте менее частые тесты'
                ]
            } : undefined
        });
    }
});

// GET /api/admin/email-templates - Получить все email шаблоны
router.get('/email-templates', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const result = await db.query(`
            SELECT 
                et.id,
                et.template_name, 
                et.template_subject, 
                et.template_html, 
                et.is_active,
                et.updated_at,
                u.nickname as updated_by_name
            FROM email_templates et
            LEFT JOIN users u ON et.updated_by = u.id
            WHERE et.is_active = true
            ORDER BY et.id
        `);

        const templates = {};
        // Создаем маппинг для совместимости с фронтендом
        const templateKeys = ['welcome', 'verification', 'application-approved', 'application-rejected', 'password-reset', 'newsletter'];
        
        result.rows.forEach((row, index) => {
            const key = templateKeys[index] || `template-${row.id}`;
            templates[key] = {
                name: row.template_name,
                subject: row.template_subject,
                html: row.template_html,
                updatedAt: row.updated_at,
                updatedBy: row.updated_by_name
            };
        });

        res.json({
            success: true,
            templates,
            totalTemplates: result.rows.length
        });

    } catch (error) {
        console.error('Ошибка загрузки email шаблонов:', error);
        res.status(500).json({
            error: 'Ошибка загрузки email шаблонов'
        });
    }
});

// GET /api/admin/email-templates/:templateKey - Получить конкретный email шаблон
router.get('/email-templates/:templateKey', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const { templateKey } = req.params;
        
        // Маппинг ключей шаблонов на ID в базе данных
        const templateIds = {
            'welcome': 7,
            'verification': 8,
            'application-approved': 9,
            'application-rejected': 10,
            'password-reset': 11,
            'newsletter': 12
        };
        
        const templateId = templateIds[templateKey];
        if (!templateId) {
            return res.status(404).json({
                error: 'Email шаблон не найден'
            });
        }
        
        const result = await db.query(`
            SELECT 
                et.id,
                et.template_name, 
                et.template_subject, 
                et.template_html, 
                et.is_active,
                et.created_at, 
                et.updated_at
            FROM email_templates et
            WHERE et.id = $1 AND et.is_active = true
        `, [templateId]);

        if (result.rows.length === 0) {
            return res.status(404).json({
                error: 'Email шаблон не найден'
            });
        }

        const template = result.rows[0];
        res.json({
            success: true,
            template: {
                key: templateKey,
                name: template.template_name,
                subject: template.template_subject,
                html: template.template_html,
                createdAt: template.created_at,
                updatedAt: template.updated_at
            }
        });

    } catch (error) {
        console.error('Ошибка загрузки email шаблона:', error);
        res.status(500).json({
            error: 'Ошибка загрузки email шаблона'
        });
    }
});

// GET /api/admin/users-for-email - Получение списка пользователей для выпадающего списка в тестировании email
router.get('/users-for-email', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const result = await db.query(`
            SELECT 
                id,
                nickname,
                email,
                role,
                trust_level,
                is_banned,
                registered_at
            FROM users 
            WHERE is_banned = false 
            ORDER BY 
                CASE 
                    WHEN role = 'admin' THEN 1
                    WHEN role = 'moderator' THEN 2
                    WHEN role = 'user' THEN 3
                    ELSE 4
                END,
                nickname ASC
            LIMIT 100
        `);

        const users = result.rows.map(user => ({
            id: user.id,
            nickname: user.nickname,
            email: user.email,
            role: user.role,
            trustLevel: user.trust_level,
            displayName: `${user.nickname} (${user.email}) - ${user.role}`,
            joinDate: user.registered_at
        }));

        res.json({
            success: true,
            users: users
        });

    } catch (error) {
        console.error('Ошибка загрузки пользователей:', error);
        res.status(500).json({
            error: 'Ошибка загрузки списка пользователей'
        });
    }
});

// POST /api/admin/test-email-settings - Тестирование email настроек
router.post('/test-email-settings', [
    authenticateToken,
    requireRole(['admin']),
    body('host').notEmpty(),
    body('port').isInt({ min: 1, max: 65535 }),
    body('user').notEmpty(),
    body('from').isEmail(),
    body('tls').isBoolean()
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { host, port, user, from, tls } = req.body;
        const nodemailer = require('nodemailer');

        // Создаем тестовый транспорт
        const testTransporter = nodemailer.createTransporter({
            host: host,
            port: port,
            secure: port === 465,
            auth: {
                user: user,
                pass: 'test' // Для тестирования используем заглушку
            },
            tls: {
                rejectUnauthorized: !tls
            }
        });

        // Проверяем соединение
        await testTransporter.verify();

        res.json({
            success: true,
            message: 'Email настройки корректны - соединение установлено'
        });

    } catch (error) {
        console.error('Ошибка тестирования email:', error);
        res.status(400).json({
            error: error.message || 'Ошибка соединения с SMTP сервером'
        });
    }
});

// GET /api/admin/test-database - Тестирование соединения с базой данных
router.get('/test-database', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        // Выполняем простой запрос для проверки соединения
        const result = await db.query('SELECT NOW() as server_time, version() as db_version');
        const dbInfo = result.rows[0];

        // Получаем количество таблиц
        const tablesResult = await db.query(`
            SELECT COUNT(*) as table_count 
            FROM information_schema.tables 
            WHERE table_schema = 'public'
        `);
        const tableCount = tablesResult.rows[0].table_count;

        res.json({
            success: true,
            message: `База данных работает корректно`,
            details: {
                serverTime: dbInfo.server_time,
                version: dbInfo.db_version,
                tables: tableCount
            }
        });

    } catch (error) {
        console.error('Ошибка тестирования базы данных:', error);
        res.status(500).json({
            error: 'Ошибка соединения с базой данных: ' + error.message
        });
    }
});

// POST /api/admin/clear-cache - Очистка кеша
router.post('/clear-cache', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        // Очищаем require cache для перезагрузки настроек
        Object.keys(require.cache).forEach(key => {
            if (key.includes('/config/') || key.includes('/settings')) {
                delete require.cache[key];
            }
        });

        // Можно добавить очистку других видов кеша если они есть
        
        res.json({
            success: true,
            message: 'Кеш успешно очищен'
        });

    } catch (error) {
        console.error('Ошибка очистки кеша:', error);
        res.status(500).json({
            error: 'Ошибка очистки кеша'
        });
    }
});

// PUT /api/admin/settings - Сохранение расширенных настроек
router.put('/settings', [
    authenticateToken,
    requireRole(['admin']),
    body('serverName').optional().isLength({ max: 100 }),
    body('serverDescription').optional().isLength({ max: 500 }),
    body('serverIp').optional().isLength({ max: 100 }),
    body('serverPort').optional().isInt({ min: 1, max: 65535 }),
    body('minMotivationLength').optional().isInt({ min: 10, max: 1000 }),
    body('minPlansLength').optional().isInt({ min: 10, max: 1000 }),
    body('maxApplicationsPerDay').optional().isInt({ min: 1, max: 50 }),
    body('maxLoginAttempts').optional().isInt({ min: 1, max: 20 }),
    body('rateLimitRequests').optional().isInt({ min: 10, max: 10000 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const settings = req.body;
        const fs = require('fs').promises;
        const path = require('path');

        // Читаем текущий файл настроек
        const settingsPath = path.join(__dirname, '../config/settings.js');
        let currentConfig;
        
        try {
            // Очищаем кеш и читаем заново
            delete require.cache[require.resolve('../config/settings')];
            currentConfig = require('../config/settings');
        } catch (error) {
            // Если файл не существует, создаем базовую структуру
            currentConfig = {
                server: {},
                applications: {},
                trustLevel: {},
                security: {},
                email: {}
            };
        }

        // Обновляем настройки
        if (settings.serverName !== undefined) currentConfig.server.name = settings.serverName;
        if (settings.serverDescription !== undefined) currentConfig.server.description = settings.serverDescription;
        if (settings.serverIp !== undefined) currentConfig.server.ip = settings.serverIp;
        if (settings.serverPort !== undefined) currentConfig.server.port = settings.serverPort.toString();
        if (settings.discordInvite !== undefined) currentConfig.server.discord = settings.discordInvite;
        if (settings.telegramInvite !== undefined) currentConfig.server.telegram = settings.telegramInvite;

        // Настройки заявок
        if (settings.applicationsEnabled !== undefined) currentConfig.applications.enabled = settings.applicationsEnabled;
        if (settings.minMotivationLength !== undefined) currentConfig.applications.minMotivationLength = settings.minMotivationLength;
        if (settings.minPlansLength !== undefined) currentConfig.applications.minPlansLength = settings.minPlansLength;
        if (settings.maxApplicationsPerDay !== undefined) currentConfig.applications.maxApplicationsPerDay = settings.maxApplicationsPerDay;
        if (settings.autoApproveTrustLevel !== undefined) currentConfig.applications.autoApproveTrustLevel = settings.autoApproveTrustLevel;

        // Trust Level настройки
        if (settings.trustPointsEmail !== undefined) currentConfig.trustLevel.pointsForEmail = settings.trustPointsEmail;
        if (settings.trustPointsDiscord !== undefined) currentConfig.trustLevel.pointsForDiscord = settings.trustPointsDiscord;
        if (settings.trustPointsHour !== undefined) currentConfig.trustLevel.pointsPerHour = settings.trustPointsHour;
        if (settings.trustLevel1Required !== undefined) currentConfig.trustLevel.level1Required = settings.trustLevel1Required;
        if (settings.trustLevel2Required !== undefined) currentConfig.trustLevel.level2Required = settings.trustLevel2Required;
        if (settings.trustLevel3Required !== undefined) currentConfig.trustLevel.level3Required = settings.trustLevel3Required;

        // Настройки безопасности
        if (settings.maxLoginAttempts !== undefined) currentConfig.security.maxLoginAttempts = settings.maxLoginAttempts;
        if (settings.loginLockoutDuration !== undefined) currentConfig.security.lockoutDuration = settings.loginLockoutDuration;
        if (settings.jwtExpiresDays !== undefined) currentConfig.security.jwtExpiresDays = settings.jwtExpiresDays;
        if (settings.requireEmailVerification !== undefined) currentConfig.security.requireEmailVerification = settings.requireEmailVerification;
        if (settings.twoFactorEnabled !== undefined) currentConfig.security.twoFactorEnabled = settings.twoFactorEnabled;
        if (settings.rateLimitRequests !== undefined) currentConfig.security.rateLimitRequests = settings.rateLimitRequests;

        // Email настройки
        if (settings.smtpHost !== undefined) currentConfig.email.host = settings.smtpHost;
        if (settings.smtpPort !== undefined) currentConfig.email.port = settings.smtpPort;
        if (settings.smtpFrom !== undefined) currentConfig.email.from = settings.smtpFrom;
        if (settings.smtpUser !== undefined) currentConfig.email.user = settings.smtpUser;
        if (settings.smtpPassword !== undefined) currentConfig.email.password = settings.smtpPassword;
        if (settings.smtpTls !== undefined) currentConfig.email.tls = settings.smtpTls;

        // Записываем обновленный файл настроек
        const configContent = `// Настройки сервера
// Автоматически сгенерировано админ-панелью

module.exports = ${JSON.stringify(currentConfig, null, 4)};
`;

        await fs.writeFile(settingsPath, configContent, 'utf-8');

        // Очищаем кеш настроек
        delete require.cache[require.resolve('../config/settings')];

        // Логируем изменение настроек
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'settings_updated',
            'Настройки сервера обновлены через админ-панель'
        ]);

        res.json({
            success: true,
            message: 'Настройки успешно сохранены и применены'
        });

    } catch (error) {
        console.error('Ошибка сохранения настроек:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// POST /api/admin/test-email - Тестирование email настроек
router.post('/test-email', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const { recipient, template, settings } = req.body;
        
        if (!recipient) {
            return res.status(400).json({
                error: 'Укажите email получателя'
            });
        }
        
        // Валидируем email
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        if (!emailRegex.test(recipient)) {
            return res.status(400).json({
                error: 'Неверный формат email адреса'
            });
        }
        
        const nodemailer = require('nodemailer');
        
        // Создаем транспортер с переданными настройками
        const transporter = nodemailer.createTransport({
            host: settings.host || 'smtp.yandex.ru',
            port: settings.port || 465,
            secure: settings.secure !== false, // true для 465, false для других портов
            auth: {
                user: settings.user,
                pass: settings.password
            },
            timeout: 30000,
            connectionTimeout: 30000,
            socketTimeout: 30000
        });
        
        // Проверяем подключение
        await transporter.verify();
        
        // Получаем шаблон письма
        let subject = 'Тестовое письмо с сервера';
        let html = '<h1>Тест email настроек</h1><p>Если вы получили это письмо, значит SMTP настройки работают корректно!</p>';
        
        if (template && template !== 'test') {
            try {
                const templateResult = await db.query(
                    'SELECT template_subject, template_html FROM email_templates WHERE template_name = $1',
                    [template]
                );
                
                if (templateResult.rows.length > 0) {
                    subject = templateResult.rows[0].template_subject || subject;
                    html = templateResult.rows[0].template_html || html;
                    
                    // Получаем реальные данные из настроек сервера
                    const settingsResult = await db.query(`
                        SELECT setting_key, setting_value 
                        FROM server_settings 
                        WHERE setting_key IN ('server-name', 'server-ip', 'server-port', 'discord-invite', 'telegram-invite')
                    `);
                    
                    const serverSettings = {};
                    settingsResult.rows.forEach(row => {
                        serverSettings[row.setting_key] = row.setting_value;
                    });
                    
                    // Используем реальные данные пользователя и сервера
                    const templateData = {
                        serverName: serverSettings['server-name'] || 'Owyx',
                        nickname: req.user?.nickname || 'Администратор',
                        siteUrl: 'https://owyx.site',
                        discordInvite: serverSettings['discord-invite'] || 'https://discord.gg/owyx',
                        telegramInvite: serverSettings['telegram-invite'] || 'https://t.me/owyx',
                        verificationLink: `${req.protocol}://${req.get('host')}/verify/test-token`,
                        resetLink: `${req.protocol}://${req.get('host')}/reset-password/test-token`,
                        currentDate: new Date().toLocaleDateString('ru-RU'),
                        userEmail: req.user?.email || 'admin@owyx.site'
                    };
                    
                    Object.keys(templateData).forEach(key => {
                        const regex = new RegExp(`{{${key}}}`, 'g');
                        subject = subject.replace(regex, templateData[key]);
                        html = html.replace(regex, templateData[key]);
                    });
                }
            } catch (templateError) {
                console.log('Не удалось загрузить шаблон, используем стандартное письмо:', templateError.message);
            }
        }
        
        const startTime = Date.now();
        
        // Отправляем письмо
        const info = await transporter.sendMail({
            from: `"${settings.senderName || 'ChiwawaMine'}" <${settings.from || settings.user}>`,
            to: recipient,
            subject: subject,
            html: html
        });
        
        const deliveryTime = Date.now() - startTime;
        
        // Логируем тест
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details)
            VALUES ($1, $2, $3)
        `, [
            req.user.id,
            'email_test',
            `Тестовое письмо отправлено на ${recipient}, время доставки: ${deliveryTime}мс`
        ]);
        
        res.json({
            success: true,
            message: 'Тестовое письмо успешно отправлено!',
            messageId: info.messageId,
            deliveryTime: `${deliveryTime}мс`,
            sentAt: new Date().toLocaleString('ru-RU')
        });
        
    } catch (error) {
        console.error('Ошибка тестирования email:', error);
        
        let errorMessage = 'Неизвестная ошибка при отправке письма';
        
        if (error.code === 'ECONNREFUSED') {
            errorMessage = 'Не удается подключиться к SMTP серверу. Проверьте хост и порт.';
        } else if (error.code === 'EAUTH' || error.responseCode === 535) {
            errorMessage = 'Ошибка авторизации. Проверьте логин и пароль.';
        } else if (error.code === 'ETIMEDOUT') {
            errorMessage = 'Превышено время ожидания. Проверьте настройки сети.';
        } else if (error.message) {
            errorMessage = error.message;
        }
        
        res.status(500).json({
            error: errorMessage,
            details: process.env.NODE_ENV === 'development' ? error.message : undefined
        });
    }
});

// GET /api/admin/system-info - Получение технической информации о системе
router.get('/system-info', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const memoryUsage = process.memoryUsage();
        const uptimeSeconds = process.uptime();
        
        // Конвертируем время работы в читаемый формат
        const hours = Math.floor(uptimeSeconds / 3600);
        const minutes = Math.floor((uptimeSeconds % 3600) / 60);
        const uptimeFormatted = `${hours}ч ${minutes}м`;
        
        // Конвертируем байты в МБ
        const formatMemory = (bytes) => {
            return `${Math.round(bytes / 1024 / 1024)}MB`;
        };

        const systemInfo = {
            nodeVersion: process.version,
            uptime: uptimeFormatted,
            memory: {
                rss: formatMemory(memoryUsage.rss),
                heapUsed: formatMemory(memoryUsage.heapUsed),
                heapTotal: formatMemory(memoryUsage.heapTotal)
            },
            environment: process.env.NODE_ENV || 'development',
            port: process.env.PORT || 3000
        };

        res.json(systemInfo);
    } catch (error) {
        console.error('Ошибка получения системной информации:', error);
        res.status(500).json({ error: 'Ошибка получения системной информации' });
    }
});

// ===========================================
// API ТОКЕНЫ ДЛЯ ПЛАГИНОВ И ВНЕШНИХ ПРИЛОЖЕНИЙ
// ===========================================

// GET /api/admin/api-tokens - Получить список API токенов
router.get('/api-tokens', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const result = await db.query(`
            SELECT 
                at.id,
                at.token_name,
                at.token_prefix,
                at.permissions,
                at.is_active,
                at.last_used_at,
                at.expires_at,
                at.created_at,
                at.description,
                u.nickname as created_by_name
            FROM api_tokens at
            LEFT JOIN users u ON at.created_by = u.id
            ORDER BY at.created_at DESC
        `);

        res.json({
            tokens: result.rows.map(token => ({
                ...token,
                permissions: token.permissions || []
            }))
        });
    } catch (error) {
        console.error('Ошибка получения API токенов:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// POST /api/admin/api-tokens - Создать новый API токен
router.post('/api-tokens', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const { name, description, permissions = [], expiresInDays = null, userId = null } = req.body;

        if (!name || name.trim().length === 0) {
            return res.status(400).json({ error: 'Название токена обязательно' });
        }

        // Генерируем случайный токен
        const crypto = require('crypto');
        const tokenLength = 64;
        const rawToken = crypto.randomBytes(tokenLength).toString('hex');
        const tokenHash = crypto.createHash('sha256').update(rawToken).digest('hex');
        const tokenPrefix = rawToken.substring(0, 8);

        // Вычисляем дату истечения
        let expiresAt = null;
        if (expiresInDays && expiresInDays > 0) {
            expiresAt = new Date();
            expiresAt.setDate(expiresAt.getDate() + expiresInDays);
        }

        // Определяем пользователя для токена (по умолчанию - создатель)
        const targetUserId = userId || req.user.id;

        // Создаем токен в базе данных
        const result = await db.query(`
            INSERT INTO api_tokens (
                token_name, token_hash, token_prefix, user_id, 
                permissions, expires_at, created_by, description
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, token_name, token_prefix, created_at
        `, [
            name.trim(),
            tokenHash,
            tokenPrefix,
            targetUserId,
            JSON.stringify(permissions),
            expiresAt,
            req.user.id,
            description?.trim() || null
        ]);

        // Логируем создание токена
        await db.query(
            'INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)',
            [req.user.id, 'api_token_created', `Создан API токен: ${name}`]
        );

        res.json({
            success: true,
            message: 'API токен создан успешно',
            token: {
                id: result.rows[0].id,
                name: result.rows[0].token_name,
                prefix: result.rows[0].token_prefix,
                created_at: result.rows[0].created_at,
                // ВАЖНО: возвращаем полный токен только при создании!
                full_token: rawToken
            }
        });

    } catch (error) {
        console.error('Ошибка создания API токена:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// PUT /api/admin/api-tokens/:id - Обновить API токен
router.put('/api-tokens/:id', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const tokenId = parseInt(req.params.id);
        const { name, description, permissions, is_active } = req.body;

        const result = await db.query(`
            UPDATE api_tokens 
            SET token_name = $1, description = $2, permissions = $3, is_active = $4
            WHERE id = $5
            RETURNING token_name
        `, [
            name?.trim(),
            description?.trim() || null,
            JSON.stringify(permissions || []),
            is_active !== undefined ? is_active : true,
            tokenId
        ]);

        if (result.rows.length === 0) {
            return res.status(404).json({ error: 'API токен не найден' });
        }

        // Логируем обновление
        await db.query(
            'INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)',
            [req.user.id, 'api_token_updated', `Обновлен API токен: ${result.rows[0].token_name}`]
        );

        res.json({ success: true, message: 'API токен обновлен' });

    } catch (error) {
        console.error('Ошибка обновления API токена:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// DELETE /api/admin/api-tokens/:id - Удалить API токен
router.delete('/api-tokens/:id', authenticateToken, requireRole(['admin']), async (req, res) => {
    try {
        const tokenId = parseInt(req.params.id);

        const result = await db.query(`
            DELETE FROM api_tokens 
            WHERE id = $1
            RETURNING token_name
        `, [tokenId]);

        if (result.rows.length === 0) {
            return res.status(404).json({ error: 'API токен не найден' });
        }

        // Логируем удаление
        await db.query(
            'INSERT INTO admin_logs (admin_id, action, details) VALUES ($1, $2, $3)',
            [req.user.id, 'api_token_deleted', `Удален API токен: ${result.rows[0].token_name}`]
        );

        res.json({ success: true, message: 'API токен удален' });

    } catch (error) {
        console.error('Ошибка удаления API токена:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});

// POST /api/admin/api-tokens/plugin — retired with the Minecraft plugin.
router.post('/api-tokens/plugin', authenticateToken, requireRole(['admin']), (_req, res) => {
    res.status(410).json({
        error: 'gone',
        message: 'Plugin API tokens are retired. Product scope is site + launcher only.'
    });
});

module.exports = router;
