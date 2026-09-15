// Маршруты для системы Trust Level
// Создатель: ebluffy

const express = require('express');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, requireRole } = require('./auth');

const router = express.Router();

// POST /api/trust-level/apply - Подача заявки на повышение траст левела
router.post('/apply', [
    authenticateToken,
    body('requestedLevel').isInt({ min: 1, max: 3 }),
    body('motivation').isLength({ min: 50, max: 1000 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({ errors: errors.array() });
        }
        
        const userId = req.user.id;
        const { requestedLevel, motivation } = req.body;
        
        // Получаем текущие данные пользователя
        const userResult = await db.query(`
            SELECT u.trust_level, ur.reputation_score,
                   ps.time_played_minutes AS total_playtime, u.is_email_verified
            FROM users u
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            WHERE u.id = $1
        `, [userId]);
        
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        
        const user = userResult.rows[0];
        const currentLevel = user.trust_level || 0;
        const reputation = user.reputation_score || 0;
        const playtime = user.total_playtime || 0;
        
        // Проверяем, можно ли подать заявку
        if (requestedLevel <= currentLevel) {
            return res.status(400).json({ error: 'Нельзя подать заявку на текущий или более низкий уровень' });
        }
        
        if (requestedLevel > currentLevel + 1) {
            return res.status(400).json({ error: 'Можно повысить уровень только на один за раз' });
        }
        
        // Проверяем требования для каждого уровня
        const requirements = {
            1: { playtime: 0, reputation: 0, emailRequired: true },
            2: { playtime: 25, reputation: 10, emailRequired: true },
            3: { playtime: 50, reputation: 20, emailRequired: true }
        };
        
        const req_level = requirements[requestedLevel];
        if (!req_level) {
            return res.status(400).json({ error: 'Неверный уровень' });
        }
        
        if (!user.is_email_verified && req_level.emailRequired) {
            return res.status(400).json({ error: 'Необходимо подтвердить email' });
        }
        
        if (playtime < req_level.playtime) {
            return res.status(400).json({ 
                error: `Необходимо ${req_level.playtime} часов игры. У вас: ${playtime}` 
            });
        }
        
        if (reputation < req_level.reputation) {
            return res.status(400).json({ 
                error: `Необходимо ${req_level.reputation} репутации. У вас: ${reputation}` 
            });
        }
        
        // Проверяем, нет ли активной заявки
        const existingApplication = await db.query(`
            SELECT id FROM trust_level_applications 
            WHERE user_id = $1 AND status = 'pending'
        `, [userId]);
        
        if (existingApplication.rows.length > 0) {
            return res.status(400).json({ error: 'У вас уже есть активная заявка на рассмотрении' });
        }
        
        // Создаем заявку
        const result = await db.query(`
            INSERT INTO trust_level_applications 
            (user_id, current_level, requested_level, motivation, current_reputation, current_playtime)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
        `, [userId, currentLevel, requestedLevel, motivation, reputation, playtime]);
        
        res.json({
            success: true,
            message: 'Заявка на повышение траст левела отправлена',
            applicationId: result.rows[0].id
        });
        
    } catch (error) {
        console.error('Ошибка подачи заявки:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/trust-level/applications - Получить заявки на повышение (для админов)
router.get('/applications', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const status = req.query.status || 'all';
        const limit = parseInt(req.query.limit) || 20;
        
        let whereClause = '';
        let queryParams = [limit];
        
        if (status !== 'all') {
            whereClause = 'WHERE tla.status = $2';
            queryParams.push(status);
        }
        
        const result = await db.query(`
            SELECT 
                tla.*,
                u.nickname,
                u.email,
                COALESCE(ur.reputation_score, 0) as current_reputation
            FROM trust_level_applications tla
            JOIN users u ON tla.user_id = u.id
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            ${whereClause}
            ORDER BY tla.submitted_at DESC
            LIMIT $1
        `, queryParams);
        
        res.json({
            success: true,
            applications: result.rows,
            total: result.rows.length
        });
        
    } catch (error) {
        console.error('Ошибка получения заявок:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// PUT /api/trust-level/applications/:id/review - Рассмотреть заявку на повышение
router.put('/applications/:id/review', [
    authenticateToken,
    requireRole(['admin', 'moderator']),
    body('decision').isIn(['approved', 'rejected']),
    body('comment').optional().isLength({ max: 500 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({ errors: errors.array() });
        }
        
        const applicationId = parseInt(req.params.id);
        const { decision, comment } = req.body;
        const reviewerId = req.user.id;
        
        // Получаем данные заявки
        const applicationResult = await db.query(`
            SELECT * FROM trust_level_applications WHERE id = $1 AND status = 'pending'
        `, [applicationId]);
        
        if (applicationResult.rows.length === 0) {
            return res.status(404).json({ error: 'Заявка не найдена или уже рассмотрена' });
        }
        
        const application = applicationResult.rows[0];
        
        // Обновляем статус заявки
        await db.query(`
            UPDATE trust_level_applications 
            SET status = $1, reviewer_id = $2, reviewed_at = NOW(), review_comment = $3
            WHERE id = $4
        `, [decision, reviewerId, comment, applicationId]);
        
        // Если заявка одобрена, повышаем траст левел пользователя
        if (decision === 'approved') {
            await db.query(`
                UPDATE users 
                SET trust_level = $1, trust_level_updated_at = NOW()
                WHERE id = $2
            `, [application.requested_level, application.user_id]);
        }
        
        res.json({
            success: true,
            message: decision === 'approved' ? 'Заявка одобрена' : 'Заявка отклонена'
        });
        
    } catch (error) {
        console.error('Ошибка рассмотрения заявки:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/trust-level/stats - Статистика по траст левелам (для админов)
router.get('/stats', authenticateToken, requireRole(['admin', 'moderator']), async (req, res) => {
    try {
        const result = await db.query(`
            SELECT 
                trust_level,
                COUNT(*) as count
            FROM users 
            GROUP BY trust_level 
            ORDER BY trust_level
        `);
        
        const stats = {
            0: 0, 1: 0, 2: 0, 3: 0
        };
        
        result.rows.forEach(row => {
            stats[row.trust_level || 0] = parseInt(row.count);
        });
        
        res.json({
            success: true,
            stats: stats,
            total: Object.values(stats).reduce((sum, count) => sum + count, 0)
        });
        
    } catch (error) {
        console.error('Ошибка получения статистики:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// GET /api/trust-level/requirements - Проверка требований для повышения уровня доверия
router.get('/requirements', authenticateToken, async (req, res) => {
    try {
        const userId = req.user.id;
        
        // Получаем текущие данные пользователя
        const userResult = await db.query(`
            SELECT u.trust_level, ur.reputation_score, u.is_email_verified, ps.time_played_minutes
            FROM users u
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            WHERE u.id = $1
        `, [userId]);
        
        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        
        const user = userResult.rows[0];
        const currentLevel = user.trust_level || 0;
        const reputation = user.reputation_score || 0;
        const playtime = 0; // Пока нет данных о времени игры, используем 0
        const nextLevel = currentLevel + 1;
        
        // Максимальный уровень доверия (Ветеран = 3)
        if (currentLevel >= 3) {
            return res.json({
                canApply: false,
                nextLevel: currentLevel,
                reason: 'Достигнут максимальный уровень доверия',
                requirements: [
                    { description: 'Максимальный уровень', met: true }
                ]
            });
        }
        
        // Требования для каждого уровня
        const requirements = {
            1: { playtime: 0, reputation: 0, emailRequired: true },
            2: { playtime: 25, reputation: 10, emailRequired: true },
            3: { playtime: 50, reputation: 20, emailRequired: true },
            4: { playtime: 100, reputation: 50, emailRequired: true }
        };
        
        const req_level = requirements[nextLevel];
        if (!req_level) {
            return res.json({
                canApply: false,
                nextLevel: nextLevel,
                reason: 'Неверный уровень доверия',
                requirements: []
            });
        }
        
        // Проверяем, нет ли активной заявки
        const existingApplication = await db.query(`
            SELECT id FROM trust_level_applications 
            WHERE user_id = $1 AND status = 'pending'
        `, [userId]);
        
        if (existingApplication.rows.length > 0) {
            return res.json({
                canApply: false,
                nextLevel: nextLevel,
                reason: 'У вас уже есть активная заявка на рассмотрении',
                requirements: []
            });
        }
        
        // Формируем список требований с проверкой
        const reqList = [
            {
                description: 'Подтвержденный email',
                met: user.is_email_verified || false,
                progress: user.is_email_verified ? 'Выполнено' : 'Не выполнено'
            },
            {
                description: `Время игры: ${req_level.playtime}+ часов`,
                met: playtime >= req_level.playtime,
                progress: `${playtime}/${req_level.playtime} часов`
            },
            {
                description: `Репутация: ${req_level.reputation}+`,
                met: reputation >= req_level.reputation,
                progress: `${reputation}/${req_level.reputation}`
            }
        ];
        
        const canApply = reqList.every(req => req.met);
        
        res.json({
            canApply: canApply,
            nextLevel: nextLevel,
            reason: canApply ? null : 'Не выполнены все требования',
            requirements: reqList
        });
        
    } catch (error) {
        console.error('Ошибка проверки требований:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

module.exports = router;
