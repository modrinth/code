// Маршруты для заявок на whitelist
// Создатель: ebluffy

const express = require('express');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, authenticateApiToken, requireRole } = require('./auth');
const bcrypt = require('bcryptjs');
const jwt = require('jsonwebtoken');

const router = express.Router();

// Optional auth — attaches req.user when a valid Bearer JWT is present
async function optionalAuth(req, _res, next) {
    try {
        const authHeader = req.headers['authorization'];
        const token = authHeader && authHeader.split(' ')[1];
        if (!token || !process.env.JWT_SECRET) return next();
        const decoded = jwt.verify(token, process.env.JWT_SECRET);
        const userResult = await db.query('SELECT * FROM users WHERE id = $1 AND is_active = true', [decoded.userId]);
        if (userResult.rows.length > 0) {
            req.user = userResult.rows[0];
            if (!req.user.role) req.user.role = 'user';
        }
    } catch {
        // ignore invalid token for optional auth
    }
    next();
}

// GET /api/applications/mine — current user's latest application
router.get('/mine', authenticateToken, async (req, res) => {
    try {
        const result = await db.query(`
            SELECT id, minecraft_nick, age, discord, email, experience, motivation, plans,
                   status, submitted_at, reviewed_at, review_comment, user_id
            FROM applications
            WHERE user_id = $1 OR email = $2
            ORDER BY submitted_at DESC
            LIMIT 1
        `, [req.user.id, req.user.email]);

        if (result.rows.length === 0) {
            return res.json({ application: null });
        }
        return res.json({ application: result.rows[0] });
    } catch (error) {
        console.error('Error fetching own application:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/applications - Подача заявки
router.post('/', optionalAuth, [
    body('minecraft_nick').isLength({ min: 3, max: 16 }).matches(/^[a-zA-Z0-9_]+$/),
    body('age').isInt({ min: 10, max: 100 }),
    body('discord').isLength({ min: 3, max: 100 }),
    body('email').isEmail().normalizeEmail(),
    body('experience').isIn(['beginner', 'intermediate', 'advanced', 'expert']),
    body('motivation').isLength({ min: 50, max: 800 }),
    body('plans').isLength({ min: 30, max: 600 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const {
            minecraft_nick,
            age,
            discord,
            email,
            experience,
            motivation,
            plans
        } = req.body;

        const ip = req.clientIp || req.ip || req.connection.remoteAddress;
        const userAgent = req.get('User-Agent') || '';
        const userId = req.user?.id || null;

        // Проверяем, есть ли уже заявка с таким email или ником
        const existingApplication = await db.query(`
            SELECT id, status FROM applications 
            WHERE email = $1 OR minecraft_nick = $2 OR ($3::int IS NOT NULL AND user_id = $3)
            ORDER BY submitted_at DESC LIMIT 1
        `, [email, minecraft_nick, userId]);

        if (existingApplication.rows.length > 0) {
            const existing = existingApplication.rows[0];
            if (existing.status === 'pending') {
                return res.status(400).json({
                    error: 'У вас уже есть заявка на рассмотрении'
                });
            }
            if (existing.status === 'approved') {
                return res.status(400).json({
                    error: 'Вы уже были одобрены для игры на сервере'
                });
            }
        }

        // Проверяем лимит заявок с одного IP (10 в день)
        const ipLimitResult = await db.query(`
            SELECT COUNT(*) as count 
            FROM applications 
            WHERE ip_address = $1 AND submitted_at > NOW() - INTERVAL '24 hours'
        `, [ip]);

        if (parseInt(ipLimitResult.rows[0].count) >= 10) {
            return res.status(429).json({
                error: 'Превышен лимит заявок с вашего IP адреса'
            });
        }

        // Создаем заявку
        const insertResult = await db.query(`
            INSERT INTO applications (
                user_id, minecraft_nick, age, discord, email, experience, 
                motivation, plans, ip_address, user_agent
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, submitted_at
        `, [
            userId, minecraft_nick, age, discord, email, experience,
            motivation, plans, ip, userAgent
        ]);

        const application = insertResult.rows[0];

        // Логируем действие
        await db.query(`
            INSERT INTO user_activity (user_id, activity_type, description, ip_address)
            VALUES ($1, $2, $3, $4)
        `, [
            null,
            'application_submitted',
            `Подана заявка от ${minecraft_nick} (${email})`,
            ip
        ]);

        console.log(`✅ Новая заявка от ${minecraft_nick}:`, {
            id: application.id,
            email,
            experience,
            motivation: motivation.substring(0, 100) + '...'
        });

        res.json({
            success: true,
            message: 'Заявка успешно отправлена! Мы рассмотрим её в ближайшее время.',
            application_id: application.id,
            submitted_at: application.submitted_at
        });

    } catch (error) {
        console.error('Ошибка при обработке заявки:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/applications/status/:email - Проверка статуса заявки
router.get('/status/:email', async (req, res) => {
    try {
        const { email } = req.params;

        const result = await db.query(`
            SELECT id, minecraft_nick, status, submitted_at, reviewed_at, review_comment
            FROM applications
            WHERE email = $1
            ORDER BY submitted_at DESC
            LIMIT 1
        `, [email]);

        if (result.rows.length === 0) {
            return res.status(404).json({
                error: 'Заявка не найдена'
            });
        }

        const application = result.rows[0];

        res.json({
            id: application.id,
            minecraft_nick: application.minecraft_nick,
            status: application.status,
            submitted_at: application.submitted_at,
            reviewed_at: application.reviewed_at,
            review_comment: application.review_comment,
            status_text: getStatusText(application.status)
        });

    } catch (error) {
        console.error('Ошибка проверки статуса заявки:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/applications - Получение всех заявок (только для админов)
router.get('/', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const page = parseInt(req.query.page) || 1;
        const limit = parseInt(req.query.limit) || 20;
        const status = req.query.status || 'all';
        const offset = (page - 1) * limit;

        let whereClause = '';
        let queryParams = [limit, offset];

        if (status !== 'all') {
            whereClause = 'WHERE a.status = $3';
            queryParams.push(status);
        }

        const result = await db.query(`
            SELECT a.*, u.nickname as reviewed_by_nickname
            FROM applications a
            LEFT JOIN users u ON a.reviewed_by = u.id
            ${whereClause}
            ORDER BY a.submitted_at DESC
            LIMIT $1 OFFSET $2
        `, queryParams);

        // Получаем общее количество
        const countParams = status !== 'all' ? [status] : [];
        const countResult = await db.query(`
            SELECT COUNT(*) as total FROM applications ${whereClause}
        `, countParams);

        const total = parseInt(countResult.rows[0].total);

        res.json({
            applications: result.rows,
            pagination: {
                page,
                limit,
                total,
                totalPages: Math.ceil(total / limit)
            }
        });

    } catch (error) {
        console.error('Ошибка получения заявок:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// PUT /api/applications/:id/review - Рассмотрение заявки (только для админов)
router.put('/:id/review', [
    authenticateToken,
    requireRole(['admin', 'moderator']),
    body('status').isIn(['approved', 'rejected']),
    body('comment').optional().isLength({ max: 1000 })
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
        const { status, comment } = req.body;

        // Получаем данные заявки
        const applicationResult = await db.query(`
            SELECT * FROM applications WHERE id = $1
        `, [id]);

        if (applicationResult.rows.length === 0) {
            return res.status(404).json({ error: 'Заявка не найдена' });
        }

        const application = applicationResult.rows[0];

        if (application.status !== 'pending') {
            return res.status(400).json({
                error: 'Заявка уже была рассмотрена'
            });
        }

        // Обновляем статус заявки
        await db.query(`
            UPDATE applications 
            SET status = $1, reviewed_by = $2, reviewed_at = NOW(), review_comment = $3
            WHERE id = $4
        `, [status, req.user.id, comment, id]);

        // Если заявка одобрена и у неё нет связанного пользователя, создаем пользователя
        console.log(`Проверка условий: status=${status}, application.user_id=${application.user_id}`);
        if (status === 'approved' && !application.user_id) {
            console.log(`Создаем пользователя для заявки ${application.id}`);
            await createUserFromApplication(application);
        } else {
            console.log(`Пропускаем создание пользователя: status=${status}, user_id=${application.user_id}`);
        }

        // Логируем действие
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'application_reviewed',
            `Заявка ${application.minecraft_nick} ${status === 'approved' ? 'одобрена' : 'отклонена'}${comment ? ': ' + comment : ''}`,
            application.user_id
        ]);

        res.json({
            success: true,
            message: `Заявка ${status === 'approved' ? 'одобрена' : 'отклонена'}`,
            application_id: id,
            status
        });

    } catch (error) {
        console.error('Ошибка рассмотрения заявки:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// GET /api/applications/stats - Статистика заявок (для админов)
router.get('/stats', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const statsResult = await db.query(`
            SELECT 
                COUNT(*) as total,
                COUNT(CASE WHEN status = 'pending' THEN 1 END) as pending,
                COUNT(CASE WHEN status = 'approved' THEN 1 END) as approved,
                COUNT(CASE WHEN status = 'rejected' THEN 1 END) as rejected,
                COUNT(CASE WHEN submitted_at > NOW() - INTERVAL '24 hours' THEN 1 END) as today,
                COUNT(CASE WHEN submitted_at > NOW() - INTERVAL '7 days' THEN 1 END) as this_week
            FROM applications
        `);

        res.json(statsResult.rows[0]);

    } catch (error) {
        console.error('Ошибка получения статистики заявок:', error);
        res.status(500).json({
            error: 'Внутренняя ошибка сервера'
        });
    }
});

// Функция создания пользователя из одобренной заявки
async function createUserFromApplication(application) {
    try {
        // Проверяем, что заявка еще не связана с пользователем
        if (application.user_id) {
            console.log(`Заявка ${application.id} уже связана с пользователем ${application.user_id}`);
            return { userId: application.user_id };
        }

        // Проверяем, существует ли пользователь с таким никнеймом
        const existingUser = await db.query(`
            SELECT id FROM users WHERE nickname = $1 LIMIT 1
        `, [application.minecraft_nick]);

        let userId;
        let tempPassword = null;

        if (existingUser.rows.length > 0) {
            // Пользователь уже существует, используем его ID
            userId = existingUser.rows[0].id;
            console.log(`Пользователь с никнеймом ${application.minecraft_nick} уже существует (ID: ${userId}), связываем с заявкой`);
        } else {
            // Создаем нового пользователя
            tempPassword = Math.random().toString(36).slice(-8);
            const hashedPassword = await bcrypt.hash(tempPassword, 12);

            const userResult = await db.query(`
                INSERT INTO users (
                    nickname, email, discord_username, password_hash, 
                    is_email_verified, trust_level
                ) VALUES ($1, $2, $3, $4, false, 0)
                RETURNING id
            `, [
                application.minecraft_nick,
                application.email,
                application.discord,
                hashedPassword
            ]);

            userId = userResult.rows[0].id;
            console.log(`✅ Создан новый пользователь ${application.minecraft_nick} (ID: ${userId})`);
        }

        // Связываем заявку с пользователем
        await db.query(`
            UPDATE applications SET user_id = $1 WHERE id = $2
        `, [userId, application.id]);

        // Создаем записи в связанных таблицах только для новых пользователей
        if (tempPassword) {
            // Проверяем, есть ли уже player_stats для этого пользователя
            const existingStats = await db.query(`
                SELECT id FROM player_stats WHERE user_id = $1 LIMIT 1
            `, [userId]);

            if (existingStats.rows.length === 0) {
                await db.query(`
                    INSERT INTO player_stats (
                        user_id, time_played_minutes, is_time_limited,
                        current_level, u.is_email_verified, reputation, total_logins
                    ) VALUES ($1, 0, 600, true, 0, false, false, false, 0, 0, 0)
                `, [userId]);
            }

            // Добавляем достижение "Первый вход" только если его еще нет
            const achievementResult = await db.query(`
                SELECT id FROM achievements WHERE name = 'Первый вход' LIMIT 1
            `);

            if (achievementResult.rows.length > 0) {
                const existingAchievement = await db.query(`
                    SELECT id FROM user_achievements 
                    WHERE user_id = $1 AND achievement_id = $2 LIMIT 1
                `, [userId, achievementResult.rows[0].id]);

                if (existingAchievement.rows.length === 0) {
                    await db.query(`
                        INSERT INTO user_achievements (user_id, achievement_id)
                        VALUES ($1, $2)
                    `, [userId, achievementResult.rows[0].id]);
                }
            }

            console.log(`✅ Создан новый пользователь для ${application.minecraft_nick}, временный пароль: ${tempPassword}`);
        } else {
            console.log(`✅ Заявка ${application.id} связана с существующим пользователем ${application.minecraft_nick} (ID: ${userId})`);
        }

        return { userId, tempPassword };

    } catch (error) {
        console.error('Ошибка создания пользователя из заявки:', error);
        throw error;
    }
}

// Функция получения текста статуса
function getStatusText(status) {
    const statusTexts = {
        'pending': 'На рассмотрении',
        'approved': 'Одобрена',
        'rejected': 'Отклонена',
        'banned': 'Заблокирована'
    };
    return statusTexts[status] || 'Неизвестно';
}

// GET /api/applications/server-access - Получение информации о заявке на доступ к серверу
router.get('/server-access', authenticateApiToken, async (req, res) => {
    try {
        let userId;
        
        // Если передан nickname (для плагина), находим пользователя по никнейму
        if (req.query.nickname) {
            const userResult = await db.query('SELECT id FROM users WHERE nickname = $1', [req.query.nickname]);
            if (userResult.rows.length === 0) {
                return res.json({ hasAccess: false, application: null });
            }
            userId = userResult.rows[0].id;
        } else {
            // Иначе используем аутентифицированного пользователя
            userId = req.user.id;
        }
        
        // Ищем последнюю заявку пользователя
        const applicationResult = await db.query(`
            SELECT * FROM applications 
            WHERE user_id = $1 
            ORDER BY submitted_at DESC 
            LIMIT 1
        `, [userId]);
        
        const application = applicationResult.rows.length > 0 ? applicationResult.rows[0] : null;
        
        // Проверяем, есть ли у пользователя доступ к серверу (одобренная заявка)
        const hasAccess = application !== null && application.status === 'approved';
        
        res.json({
            hasAccess: hasAccess,
            application: application
        });
        
    } catch (error) {
        console.error('Ошибка получения заявки на доступ:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/applications/server-access - Подача заявки на доступ к серверу
router.post('/server-access', [
    authenticateToken,
    body('age').isInt({ min: 10, max: 100 }),
    body('source').isLength({ min: 1, max: 50 }),
    body('experience').isLength({ min: 1, max: 50 }),
    body('about').isLength({ min: 50, max: 1000 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const userId = req.user.id;
        const { age, source, experience, about } = req.body;
        
        // Получаем данные пользователя
        const userResult = await db.query(`
            SELECT email, nickname, first_name, role, discord_username FROM users WHERE id = $1
        `, [userId]);
        
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        
        const user = userResult.rows[0];
        
        // Проверяем, нет ли уже одобренной заявки
        const approvedApplication = await db.query(`
            SELECT id FROM applications 
            WHERE user_id = $1 AND status = 'approved'
            LIMIT 1
        `, [userId]);
        
        if (approvedApplication.rows.length > 0) {
            return res.status(400).json({ error: 'У вас уже есть доступ к серверу' });
        }
        
        // Проверяем, нет ли активной заявки
        const existingApplication = await db.query(`
            SELECT id, status FROM applications 
            WHERE user_id = $1 AND status = 'pending'
            ORDER BY submitted_at DESC LIMIT 1
        `, [userId]);
        
        if (existingApplication.rows.length > 0) {
            return res.status(400).json({ error: 'У вас уже есть активная заявка на рассмотрении' });
        }
        
        const ip = req.ip || req.connection.remoteAddress;
        const userAgent = req.get('User-Agent') || '';
        
        // Создаем заявку (адаптируем под структуру БД)
        const result = await db.query(`
            INSERT INTO applications 
            (user_id, minecraft_nick, email, age, discord, experience, motivation, plans, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id
        `, [userId, user.nickname, user.email, age.toString(), user.discord_username || 'Не указан', experience, about, 'Планы будут указаны позже', ip, userAgent]);
        
        res.json({
            success: true,
            message: 'Заявка на доступ к серверу успешно подана',
            applicationId: result.rows[0].id
        });
        
    } catch (error) {
        console.error('Ошибка подачи заявки на доступ:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/applications/trust-level - Получение информации о заявке на повышение уровня доверия  
router.get('/trust-level', authenticateToken, async (req, res) => {
    try {
        const userId = req.user.id;
        
        // Ищем последнюю заявку пользователя на повышение уровня доверия
        const applicationResult = await db.query(`
            SELECT * FROM trust_level_applications 
            WHERE user_id = $1 
            ORDER BY submitted_at DESC 
            LIMIT 1
        `, [userId]);
        
        const application = applicationResult.rows.length > 0 ? applicationResult.rows[0] : null;
        
        res.json({
            application: application
        });
        
    } catch (error) {
        console.error('Ошибка получения заявки на повышение:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/applications/trust-level - Подача заявки на повышение уровня доверия
router.post('/trust-level', [
    authenticateToken,
    body('motivation').isLength({ min: 50, max: 1000 }),
    body('plans').isLength({ min: 30, max: 500 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }
        
        const userId = req.user.id;
        const { motivation, plans } = req.body;
        
        // Получаем текущие данные пользователя
        const userResult = await db.query(`
            SELECT u.trust_level, ur.reputation_score, u.is_email_verified
            FROM users u
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            WHERE u.id = $1
        `, [userId]);
        
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        
        const user = userResult.rows[0];
        const currentLevel = user.trust_level || 0;
        const reputation = user.reputation_score || 0;
        const requestedLevel = currentLevel + 1;
        
        // Проверяем, нет ли активной заявки
        const existingApplication = await db.query(`
            SELECT id FROM trust_level_applications 
            WHERE user_id = $1 AND status = 'pending'
        `, [userId]);
        
        if (existingApplication.rows.length > 0) {
            return res.status(400).json({ error: 'У вас уже есть активная заявка на рассмотрении' });
        }
        
        // Создаем заявку (объединяем motivation и plans в поле reason)
        const result = await db.query(`
            INSERT INTO trust_level_applications 
            (user_id, current_level, requested_level, reason, reputation_score, hours_played)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
        `, [userId, currentLevel, requestedLevel, `Мотивация: ${motivation}\n\nПланы: ${plans}`, reputation, 0]);
        
        res.json({
            success: true,
            message: 'Заявка на повышение уровня доверия успешно подана',
            applicationId: result.rows[0].id
        });
        
    } catch (error) {
        console.error('Ошибка подачи заявки на повышение:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

module.exports = router;
