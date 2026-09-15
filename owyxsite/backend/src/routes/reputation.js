// Маршруты для системы репутации
// Создатель: ebluffy

const express = require('express');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, requireRole } = require('./auth');

const router = express.Router();

// GET /api/reputation/:userId - Получить репутацию пользователя
router.get('/:userId', async (req, res) => {
    try {
        const { userId } = req.params;
        
        // Получаем данные репутации
        const reputationResult = await db.query(`
            SELECT ur.*, u.nickname, u.trust_level
            FROM user_reputation ur
            JOIN users u ON ur.user_id = u.id
            WHERE ur.user_id = $1
        `, [userId]);
        
        if (reputationResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        
        const reputation = reputationResult.rows[0];
        
        // Получаем историю изменений репутации
        const historyResult = await db.query(`
            SELECT rh.*, u.nickname as voter_nickname
            FROM reputation_history rh
            LEFT JOIN users u ON rh.voter_id = u.id
            WHERE rh.user_id = $1
            ORDER BY rh.created_at DESC
            LIMIT 20
        `, [userId]);
        
        res.json({
            reputation: {
                score: reputation.reputation_score,
                positive_votes: reputation.positive_votes,
                negative_votes: reputation.negative_votes,
                trust_level: reputation.trust_level,
                nickname: reputation.nickname
            },
            history: historyResult.rows
        });
        
    } catch (error) {
        console.error('Ошибка получения репутации:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// POST /api/reputation/:userId/vote - Проголосовать за пользователя
router.post('/:userId/vote', [
    authenticateToken,
    body('type').isIn(['positive', 'negative']),
    body('reason').isLength({ min: 10, max: 200 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }
        
        const { userId } = req.params;
        const { type, reason } = req.body;
        const voterId = req.user.id;
        
        // Нельзя голосовать за себя
        if (parseInt(userId) === voterId) {
            return res.status(400).json({ error: 'Нельзя голосовать за себя' });
        }
        
        const client = await db.getClient();
        
        try {
            await client.query('BEGIN');
            
            // Проверяем, не голосовал ли уже этот пользователь
            const existingVote = await client.query(`
                SELECT id FROM reputation_history 
                WHERE user_id = $1 AND voter_id = $2
                AND created_at > NOW() - INTERVAL '24 hours'
            `, [userId, voterId]);
            
            if (existingVote.rows.length > 0) {
                await client.query('ROLLBACK');
                return res.status(400).json({ error: 'Можно голосовать только раз в 24 часа' });
            }
            
            // Определяем изменение репутации
            const change = type === 'positive' ? 1 : -1;
            
            // Записываем голос в историю
            await client.query(`
                INSERT INTO reputation_history (user_id, voter_id, change_amount, reason, vote_type)
                VALUES ($1, $2, $3, $4, $5)
            `, [userId, voterId, change, reason, type]);
            
            // Обновляем репутацию пользователя
            const newReputationResult = await client.query(`
                UPDATE user_reputation 
                SET 
                    reputation_score = reputation_score + $1,
                    positive_votes = positive_votes + $2,
                    negative_votes = negative_votes + $3,
                    updated_at = NOW()
                WHERE user_id = $4
                RETURNING reputation_score as new_reputation
            `, [
                change,
                type === 'positive' ? 1 : 0,
                type === 'negative' ? 1 : 0,
                userId
            ]);
            
            await client.query('COMMIT');
            
            res.json({
                success: true,
                message: 'Голос учтен',
                new_reputation: newReputationResult.rows[0].new_reputation
            });
            
        } catch (error) {
            await client.query('ROLLBACK');
            throw error;
        } finally {
            client.release();
        }
        
    } catch (error) {
        console.error('Ошибка голосования:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// PUT /api/reputation/:userId/admin - Изменить репутацию (только для админов)
router.put('/:userId/admin', [
    authenticateToken,
    requireRole(['admin']),
    body('change').isInt({ min: -100, max: 100 }),
    body('reason').isLength({ min: 5, max: 200 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }
        
        const { userId } = req.params;
        const { change, reason } = req.body;
        const adminId = req.user.id;
        
        const client = await db.getClient();
        
        try {
            await client.query('BEGIN');
            
            // Записываем изменение в историю
            await client.query(`
                INSERT INTO reputation_history (user_id, voter_id, change_amount, reason, vote_type, is_admin_action)
                VALUES ($1, $2, $3, $4, $5, true)
            `, [userId, adminId, change, reason, change > 0 ? 'positive' : 'negative']);
            
            // Обновляем репутацию
            const newReputationResult = await client.query(`
                UPDATE user_reputation 
                SET 
                    reputation_score = reputation_score + $1,
                    updated_at = NOW()
                WHERE user_id = $2
                RETURNING reputation_score as new_reputation
            `, [change, userId]);
            
            // Логируем действие админа
            await client.query(`
                INSERT INTO admin_logs (admin_id, action, details, target_user_id)
                VALUES ($1, $2, $3, $4)
            `, [adminId, 'reputation_changed', `Изменение репутации на ${change}: ${reason}`, userId]);
            
            await client.query('COMMIT');
            
            res.json({
                success: true,
                message: 'Репутация изменена',
                new_reputation: newReputationResult.rows[0].new_reputation
            });
            
        } catch (error) {
            await client.query('ROLLBACK');
            throw error;
        } finally {
            client.release();
        }
        
    } catch (error) {
        console.error('Ошибка изменения репутации:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

module.exports = router;
