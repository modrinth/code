// Маршруты профиля пользователя
// Создатель: ebluffy

const express = require('express');
const crypto = require('crypto');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken } = require('./auth');
const multer = require('multer');
const path = require('path');
const fs = require('fs').promises;
const sharp = require('sharp');
const { logUserActivity } = require('../utils/activityLog');

const router = express.Router();

// Настройка multer для загрузки аватаров
const avatarStorage = multer.diskStorage({
    destination: async (req, file, cb) => {
        const uploadPath = path.join(__dirname, '../../uploads/avatars');
        try {
            await fs.mkdir(uploadPath, { recursive: true });
            cb(null, uploadPath);
        } catch (error) {
            cb(error);
        }
    },
    filename: (req, file, cb) => {
        const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
        const extension = path.extname(file.originalname);
        cb(null, `avatar-${req.user.id}-${uniqueSuffix}${extension}`);
    }
});

const avatarUpload = multer({
    storage: avatarStorage,
    limits: {
        fileSize: 20 * 1024 * 1024, // 20MB
        files: 1
    },
    fileFilter: (req, file, cb) => {
        const allowedTypes = /jpeg|jpg|png|gif|webp/;
        const extname = allowedTypes.test(path.extname(file.originalname).toLowerCase());
        const mimetype = allowedTypes.test(file.mimetype);

        if (mimetype && extname) {
            return cb(null, true);
        } else {
            cb(new Error('Разрешены только изображения (JPEG, JPG, PNG, GIF, WebP). Максимальный размер: 20MB'));
        }
    }
});

// Multer for Minecraft skin uploads. Skins are small PNG files (64x64 or the
// legacy 64x32). We keep the raw upload to a temp name, then validate with sharp.
const skinStorage = multer.diskStorage({
    destination: async (req, file, cb) => {
        const uploadPath = path.join(__dirname, '../../uploads/skins');
        try {
            await fs.mkdir(uploadPath, { recursive: true });
            cb(null, uploadPath);
        } catch (error) {
            cb(error);
        }
    },
    filename: (req, file, cb) => {
        const uniqueSuffix = Date.now() + '-' + Math.round(Math.random() * 1E9);
        cb(null, `skin-${req.user.id}-${uniqueSuffix}.upload.png`);
    }
});

const skinUpload = multer({
    storage: skinStorage,
    limits: {
        fileSize: 512 * 1024, // 512KB — a skin PNG is only a few KB
        files: 1
    },
    fileFilter: (req, file, cb) => {
        // Only PNG. We re-check the real format with sharp after upload.
        const isPng = /\.png$/i.test(file.originalname) && /png/i.test(file.mimetype);
        if (isPng) return cb(null, true);
        cb(new Error('Скин должен быть файлом PNG'));
    }
});

// Функция для расчета оставшегося времени бана
function calculateTimeRemaining(banUntil) {
    const now = new Date();
    const until = new Date(banUntil);
    const diffMs = until.getTime() - now.getTime();
    
    if (diffMs <= 0) {
        return { expired: true };
    }
    
    const days = Math.floor(diffMs / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diffMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));
    
    return {
        expired: false,
        total_minutes: Math.floor(diffMs / (1000 * 60)),
        days,
        hours,
        minutes,
        formatted: `${days > 0 ? days + 'д ' : ''}${hours > 0 ? hours + 'ч ' : ''}${minutes}м`
    };
}

// GET /api/profile/detailed-stats - Получение упрощенной статистики игрока
router.get('/detailed-stats', authenticateToken, async (req, res) => {
    try {
        // Получаем базовую статистику
        const statsResult = await db.query(`
            SELECT 
                ps.time_played_minutes,
                ps.last_seen,
                ps.deaths_count,
                ps.mobs_killed,
                u.nickname,
                u.registered_at,
                u.last_login
            FROM player_stats ps
            JOIN users u ON ps.user_id = u.id
            WHERE ps.user_id = $1
        `, [req.user.id]);

        if (statsResult.rows.length === 0) {
            return res.status(404).json({ error: 'Статистика не найдена' });
        }

        const stats = statsResult.rows[0];

        // Вычисляем статистику
        const totalPlaytime = stats.time_played_minutes || 0;
        const playtimeHours = Math.floor(totalPlaytime / 60);
        const playtimeMinutes = totalPlaytime % 60;

        res.json({
            success: true,
            stats: {
                // Основная статистика - только то что нужно
                total_playtime_minutes: totalPlaytime,
                total_playtime_hours: playtimeHours,
                total_playtime_formatted: `${playtimeHours}ч ${playtimeMinutes}м`,
                
                // Последний вход
                last_seen: stats.last_seen,
                
                // Игровая статистика - только убийства и смерти
                deaths_count: stats.deaths_count || 0,
                mobs_killed: stats.mobs_killed || 0
            }
        });

    } catch (error) {
        console.error('Ошибка получения статистики:', error);
        res.status(500).json({ error: 'Ошибка сервера' });
    }
});



// POST /api/profile/update-stats — retired with the Minecraft plugin.
router.post('/update-stats', (_req, res) => {
    res.status(410).json({
        error: 'gone',
        message: 'Plugin stats API is retired. Use the Owyx launcher and site account.'
    });
});


// GET /api/profile/activity - Получение расширенной активности
router.get('/activity', authenticateToken, async (req, res) => {
    try {
        const page = parseInt(req.query.page) || 1;
        const limit = parseInt(req.query.limit) || 20;
        const offset = (page - 1) * limit;

        const result = await db.query(`
            SELECT activity_type, description, metadata, created_at
            FROM user_activity
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        `, [req.user.id, limit, offset]);

        // Получаем общее количество записей
        const countResult = await db.query(`
            SELECT COUNT(*) as total
            FROM user_activity
            WHERE user_id = $1
        `, [req.user.id]);

        const total = parseInt(countResult.rows[0].total);
        const totalPages = Math.ceil(total / limit);

        res.json({
            activities: result.rows,
            pagination: {
                page,
                limit,
                total,
                totalPages,
                hasNext: page < totalPages,
                hasPrev: page > 1
            }
        });

    } catch (error) {
        console.error('Ошибка получения активности:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// Функция для определения требований Trust Level
function getTrustLevelRequirements(currentLevel) {
    const requirements = {
        0: { time: 0, name: 'Проходимец' },
        1: { time: 0, name: 'Новичок' },        // Только подтверждение email
        2: { time: 25*60, name: 'Проверенный' }, // 25 часов + репутация
        3: { time: 50*60, name: 'Ветеран' },    // 50 часов + репутация
    };

    return requirements[currentLevel + 1] || requirements[3];
}

// Функция для получения названия Trust Level на русском
function getTrustLevelName(level) {
    const names = {
        0: 'Проходимец',
        1: 'Новичок', 
        2: 'Проверенный',
        3: 'Ветеран',
    };
    return names[level] || 'Неизвестно';
}

module.exports = router;

// POST /api/profile/avatar - Загрузка аватара
router.post('/avatar', authenticateToken, avatarUpload.single('avatar'), async (req, res) => {
    try {
        if (!req.file) {
            return res.status(400).json({ error: 'Файл не загружен' });
        }

        const originalPath = req.file.path;
        
        // Получаем данные о кропе если они есть
        let cropData = null;
        if (req.body.cropData) {
            try {
                cropData = JSON.parse(req.body.cropData);
            } catch (e) {
                console.log('Ошибка парсинга данных кропа:', e.message);
            }
        }
        
        // Проверяем размеры изображения
        const metadata = await sharp(originalPath).metadata();
        
        // Создаем финальное изображение
        const finalFileName = `avatar-${req.user.id}-${Date.now()}-final.png`;
        const finalPath = path.join(path.dirname(originalPath), finalFileName);
        
        let sharpInstance = sharp(originalPath);
        
        // Если есть данные кропа, применяем их
        if (cropData) {
            const { scale, rotation, flipX, offsetX, offsetY, cropSize } = cropData;
            
            // Вычисляем размеры для кропа
            const scaledWidth = Math.round(metadata.width * scale);
            const scaledHeight = Math.round(metadata.height * scale);
            
            // Начальная обработка: масштабирование и поворот
            sharpInstance = sharpInstance.resize(scaledWidth, scaledHeight);
            
            if (rotation !== 0) {
                sharpInstance = sharpInstance.rotate(rotation);
            }
            
            if (flipX < 0) {
                sharpInstance = sharpInstance.flop();
            }
            
            // Получаем метаданные после трансформаций
            const processedBuffer = await sharpInstance.toBuffer();
            const processedMetadata = await sharp(processedBuffer).metadata();
            
            // Вычисляем область кропа (256x256 из центра с учетом смещения)
            const centerX = Math.round(processedMetadata.width / 2);
            const centerY = Math.round(processedMetadata.height / 2);
            const cropRadius = 128; // половина от 256
            
            const cropLeft = Math.max(0, centerX - cropRadius - Math.round(offsetX));
            const cropTop = Math.max(0, centerY - cropRadius - Math.round(offsetY));
            const cropWidth = Math.min(256, processedMetadata.width - cropLeft);
            const cropHeight = Math.min(256, processedMetadata.height - cropTop);
            
            // Применяем кроп и финальный ресайз до 512x512
            sharpInstance = sharp(processedBuffer)
                .extract({ 
                    left: cropLeft, 
                    top: cropTop, 
                    width: cropWidth, 
                    height: cropHeight 
                })
                .resize(512, 512, { fit: 'cover' });
        } else {
            // Если нет данных кропа, просто ресайзим
            sharpInstance = sharpInstance.resize(512, 512, { fit: 'cover' });
        }
        
        // Сохраняем финальное изображение
        await sharpInstance
            .png({ quality: 90 })
            .toFile(finalPath);
        
        // Удаляем оригинальный файл
        await fs.unlink(originalPath);

        const avatarUrl = `/uploads/avatars/${finalFileName}`;

        // Получаем старый аватар для удаления
        const oldAvatarResult = await db.query(
            'SELECT avatar_url FROM users WHERE id = $1',
            [req.user.id]
        );

        // Обновляем аватар в БД
        await db.query(
            'UPDATE users SET avatar_url = $1 WHERE id = $2',
            [avatarUrl, req.user.id]
        );

        // Удаляем старый аватар (если он есть и не дефолтный)
        if (oldAvatarResult.rows[0]?.avatar_url && 
            oldAvatarResult.rows[0].avatar_url.includes('/uploads/avatars/')) {
            try {
                const oldPath = path.join(__dirname, '../..', oldAvatarResult.rows[0].avatar_url);
                await fs.unlink(oldPath);
            } catch (error) {
                console.log('Не удалось удалить старый аватар:', error.message);
            }
        }

        // Логируем активность
        await logUserActivity(req.user.id, 'avatar_update', 'Profile avatar updated', { req });

        res.json({
            success: true,
            message: 'Аватар успешно загружен',
            avatar_url: avatarUrl,
            original_size: `${metadata.width}x${metadata.height}`,
            final_size: '512x512',
            crop_applied: !!cropData
        });

    } catch (error) {
        console.error('Ошибка загрузки аватара:', error);
        
        // Удаляем файл при ошибке
        if (req.file && req.file.path) {
            try {
                await fs.unlink(req.file.path);
            } catch (unlinkError) {
                console.error('Ошибка удаления файла при ошибке:', unlinkError);
            }
        }
        
        res.status(500).json({ error: 'Ошибка загрузки аватара' });
    }
});

// DELETE /api/profile/avatar - Удаление аватара
router.delete('/avatar', authenticateToken, async (req, res) => {
    try {
        // Получаем текущий аватар
        const avatarResult = await db.query(
            'SELECT avatar_url FROM users WHERE id = $1',
            [req.user.id]
        );

        const currentAvatar = avatarResult.rows[0]?.avatar_url;

        // Удаляем аватар из БД
        await db.query(
            'UPDATE users SET avatar_url = NULL WHERE id = $1',
            [req.user.id]
        );

        // Удаляем файл аватара (если он кастомный)
        if (currentAvatar && currentAvatar.includes('/uploads/avatars/')) {
            try {
                const avatarPath = path.join(__dirname, '../..', currentAvatar);
                await fs.unlink(avatarPath);
            } catch (error) {
                console.log('Не удалось удалить файл аватара:', error.message);
            }
        }

        // Логируем активность
        await logUserActivity(req.user.id, 'avatar_delete', 'Profile avatar removed', { req });

        res.json({
            success: true,
            message: 'Аватар успешно удален'
        });

    } catch (error) {
        console.error('Ошибка удаления аватара:', error);
        res.status(500).json({ error: 'Ошибка удаления аватара' });
    }
});

// GET /api/profile/ban-status - Проверка статуса бана
router.get('/ban-status', authenticateToken, async (req, res) => {
    try {
        const userResult = await db.query(`
            SELECT is_banned, ban_reason, ban_until 
            FROM users 
            WHERE id = $1
        `, [req.user.id]);

        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];

        if (!user.is_banned) {
            return res.json({
                is_banned: false,
                message: 'Пользователь не забанен'
            });
        }

        const banInfo = {
            is_banned: true,
            reason: user.ban_reason,
            until: user.ban_until,
            is_permanent: !user.ban_until
        };

        if (user.ban_until) {
            const timeRemaining = calculateTimeRemaining(user.ban_until);
            banInfo.time_remaining = timeRemaining;
            
            // Если бан истек, автоматически снимаем его
            if (timeRemaining.expired) {
                await db.query(`
                    UPDATE users 
                    SET is_banned = FALSE, ban_reason = NULL, ban_until = NULL 
                    WHERE id = $1
                `, [req.user.id]);
                
                return res.json({
                    is_banned: false,
                    message: 'Бан автоматически снят (время истекло)'
                });
            }
        }

        res.json(banInfo);

    } catch (error) {
        console.error('Ошибка проверки статуса бана:', error);
        res.status(500).json({ error: 'Ошибка проверки статуса бана' });
    }
});

// ---------------------------------------------------------------------------
// Cosmetics: Minecraft skin for the Owyx account. The launcher reads skin_url /
// skin_model from GET /api/launcher/me and applies it. Capes are reserved.
// ---------------------------------------------------------------------------

// GET /api/profile/skin - current skin/cosmetics of the signed-in user.
router.get('/skin', authenticateToken, async (req, res) => {
    try {
        const result = await db.query(
            'SELECT skin_url, skin_model, cape_url, cosmetics_updated_at FROM users WHERE id = $1',
            [req.user.id]
        );
        if (result.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        const row = result.rows[0];
        res.json({
            skin_url: row.skin_url || null,
            skin_model: row.skin_model || 'classic',
            cape_url: row.cape_url || null,
            updated_at: row.cosmetics_updated_at || null
        });
    } catch (error) {
        console.error('Ошибка получения скина:', error);
        res.status(500).json({ error: 'Не удалось получить скин' });
    }
});

// PUT /api/profile/skin - upload a new skin (PNG 64x64 or 64x32) + optional model.
router.put('/skin', authenticateToken, skinUpload.single('skin'), async (req, res) => {
    let tempPath = req.file ? req.file.path : null;
    try {
        if (!req.file) {
            return res.status(400).json({ error: 'Файл скина не загружен. Нужен PNG.' });
        }

        // Validate the real image: must be a PNG with Minecraft skin dimensions.
        let metadata;
        try {
            metadata = await sharp(tempPath).metadata();
        } catch {
            return res.status(400).json({ error: 'Файл не является корректным PNG' });
        }

        if (metadata.format !== 'png') {
            return res.status(400).json({ error: 'Скин должен быть в формате PNG' });
        }
        const okDims = metadata.width === 64 && (metadata.height === 64 || metadata.height === 32);
        if (!okDims) {
            return res.status(400).json({
                error: 'Неверный размер скина. Нужен PNG 64×64 (или старый 64×32).'
            });
        }

        // Skin model: classic (Steve) or slim (Alex).
        let model = String(req.body.model || 'classic').toLowerCase();
        if (model !== 'classic' && model !== 'slim') model = 'classic';

        // Re-encode to a clean PNG (strips any non-image payload) at the final path.
        const finalFileName = `skin-${req.user.id}-${Date.now()}.png`;
        const finalPath = path.join(path.dirname(tempPath), finalFileName);
        await sharp(tempPath).png().toFile(finalPath);

        // Remove the raw upload now that we have the clean copy.
        await fs.unlink(tempPath).catch(() => {});
        tempPath = null;

        const skinUrl = `/uploads/skins/${finalFileName}`;

        // Swap the skin under a row lock so concurrent uploads can't leak/delete
        // the wrong file (SELECT ... FOR UPDATE, then UPDATE, in one transaction).
        const client = await db.getClient();
        let oldSkinUrl = null;
        try {
            await client.query('BEGIN');
            const oldResult = await client.query('SELECT skin_url FROM users WHERE id = $1 FOR UPDATE', [req.user.id]);
            oldSkinUrl = oldResult.rows[0] ? oldResult.rows[0].skin_url : null;
            await client.query(
                'UPDATE users SET skin_url = $1, skin_model = $2, cosmetics_updated_at = NOW() WHERE id = $3',
                [skinUrl, model, req.user.id]
            );
            await client.query('COMMIT');
        } catch (txErr) {
            await client.query('ROLLBACK').catch(() => {});
            // The new file was written before the tx — clean it up on failure.
            await fs.unlink(finalPath).catch(() => {});
            throw txErr;
        } finally {
            client.release();
        }

        // Delete the previous file only after a successful commit.
        if (oldSkinUrl && oldSkinUrl !== skinUrl && oldSkinUrl.includes('/uploads/skins/')) {
            const oldPath = path.join(__dirname, '../../', oldSkinUrl.replace(/^\//, ''));
            await fs.unlink(oldPath).catch(() => {});
        }

        await logUserActivity(req.user.id, 'skin_update', 'Profile skin updated', {
            req,
            metadata: { model: String(model || '').slice(0, 16) || null },
        });

        res.json({
            success: true,
            message: 'Скин обновлён',
            skin_url: skinUrl,
            skin_model: model
        });
    } catch (error) {
        if (tempPath) await fs.unlink(tempPath).catch(() => {});
        console.error('Ошибка загрузки скина:', error);
        res.status(500).json({ error: 'Не удалось загрузить скин' });
    }
});

// DELETE /api/profile/skin - remove the account skin (back to default).
router.delete('/skin', authenticateToken, async (req, res) => {
    const client = await db.getClient();
    let oldSkinUrl = null;
    try {
        await client.query('BEGIN');
        const oldResult = await client.query('SELECT skin_url FROM users WHERE id = $1 FOR UPDATE', [req.user.id]);
        oldSkinUrl = oldResult.rows[0] ? oldResult.rows[0].skin_url : null;
        await client.query(
            'UPDATE users SET skin_url = NULL, cosmetics_updated_at = NOW() WHERE id = $1',
            [req.user.id]
        );
        await client.query('COMMIT');
    } catch (error) {
        await client.query('ROLLBACK').catch(() => {});
        client.release();
        console.error('Ошибка удаления скина:', error);
        return res.status(500).json({ error: 'Не удалось удалить скин' });
    }
    client.release();

    if (oldSkinUrl && oldSkinUrl.includes('/uploads/skins/')) {
        const oldPath = path.join(__dirname, '../../', oldSkinUrl.replace(/^\//, ''));
        await fs.unlink(oldPath).catch(() => {});
    }
    res.json({ success: true, message: 'Скин удалён' });
});

// PUT /api/profile/nickname - change login nickname, at most once per 30 days.
// Does not overwrite display_nickname (even if it still matched the old login).
const NICK_COOLDOWN_DAYS = 30;
const EMAIL_COOLDOWN_DAYS = 30;
const DISPLAY_NICK_RATE_WINDOW_MS = 60 * 1000;
const DISPLAY_NICK_RATE_MAX = 5;
const displayNickRateBuckets = new Map();

function checkDisplayNickRate(userId) {
    const now = Date.now();
    let bucket = displayNickRateBuckets.get(userId);
    if (!bucket || now - bucket.windowStart >= DISPLAY_NICK_RATE_WINDOW_MS) {
        bucket = { windowStart: now, count: 0 };
        displayNickRateBuckets.set(userId, bucket);
    }
    bucket.count += 1;
    return bucket.count <= DISPLAY_NICK_RATE_MAX;
}

function isValidMcNick(raw) {
    return typeof raw === 'string' && raw.length >= 3 && raw.length <= 16 && /^[A-Za-z0-9_]+$/.test(raw);
}

function isPgUniqueViolation(error) {
    return error && (error.code === '23505' || error.code === 'unique_violation');
}

router.put('/nickname', authenticateToken, async (req, res) => {
    try {
        const raw = (req.body && req.body.nickname != null) ? String(req.body.nickname).trim() : '';
        if (!isValidMcNick(raw)) {
            return res.status(400).json({ error: 'Логин должен быть от 3 до 16 символов (буквы, цифры, _)' });
        }

        const current = await db.query(
            'SELECT nickname, nickname_changed_at, display_nickname FROM users WHERE id = $1',
            [req.user.id]
        );
        if (current.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        const row = current.rows[0];

        if (row.nickname === raw) {
            return res.status(400).json({ error: 'Это уже ваш текущий логин' });
        }

        if (row.nickname_changed_at) {
            const changedAt = new Date(row.nickname_changed_at).getTime();
            const nextAllowed = changedAt + NICK_COOLDOWN_DAYS * 24 * 60 * 60 * 1000;
            if (Date.now() < nextAllowed) {
                const daysLeft = Math.ceil((nextAllowed - Date.now()) / (24 * 60 * 60 * 1000));
                return res.status(429).json({
                    error: `Логин можно менять раз в ${NICK_COOLDOWN_DAYS} дней. Попробуйте через ${daysLeft} дн.`,
                    next_allowed_at: new Date(nextAllowed).toISOString()
                });
            }
        }

        const taken = await db.query(
            `SELECT id FROM users WHERE id <> $2 AND LOWER(nickname) = LOWER($1)`,
            [raw, req.user.id]
        );
        if (taken.rows.length > 0) {
            return res.status(409).json({ error: 'Этот логин уже занят' });
        }

        try {
            await db.query(
                'UPDATE users SET nickname = $1, nickname_changed_at = NOW() WHERE id = $2',
                [raw, req.user.id]
            );
        } catch (updateErr) {
            if (isPgUniqueViolation(updateErr)) {
                return res.status(409).json({ error: 'Этот логин уже занят' });
            }
            throw updateErr;
        }

        await logUserActivity(req.user.id, 'nickname_change', 'Login nickname changed', {
            req,
            metadata: { nickname: raw },
        });

        try {
            const { sendNicknameChangedEmail } = require('../utils/emailService');
            if (req.user.email) sendNicknameChangedEmail(req.user.email, row.nickname, raw).catch(() => {});
        } catch { /* ignore */ }

        res.json({
            success: true,
            message: 'Логин изменён',
            nickname: raw,
            display_nickname: row.display_nickname || row.nickname,
            nickname_changed_at: new Date().toISOString()
        });
    } catch (error) {
        console.error('Ошибка смены логина:', error);
        res.status(500).json({ error: 'Не удалось изменить логин' });
    }
});

// PUT /api/profile/display-nickname — visible / in-game nick (MC format, 5/min).
// Duplicates across accounts are allowed; only login + email stay unique.
router.put('/display-nickname', authenticateToken, async (req, res) => {
    try {
        if (!checkDisplayNickRate(req.user.id)) {
            return res.status(429).json({ error: 'Слишком много попыток. Подождите минуту.' });
        }

        const raw = (req.body && (req.body.displayNickname ?? req.body.display_nickname) != null)
            ? String(req.body.displayNickname ?? req.body.display_nickname).trim()
            : '';
        if (!isValidMcNick(raw)) {
            return res.status(400).json({ error: 'Ник должен быть от 3 до 16 символов (буквы, цифры, _)' });
        }

        const current = await db.query(
            'SELECT nickname, display_nickname FROM users WHERE id = $1',
            [req.user.id]
        );
        if (current.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        const row = current.rows[0];
        const currentDisplay = row.display_nickname || row.nickname;

        if (currentDisplay === raw) {
            return res.status(400).json({ error: 'Это уже ваш текущий ник' });
        }

        try {
            await db.query(
                'UPDATE users SET display_nickname = $1, display_nickname_changed_at = NOW() WHERE id = $2',
                [raw, req.user.id]
            );
        } catch (updateErr) {
            if (isPgUniqueViolation(updateErr)) {
                return res.status(409).json({ error: 'Не удалось сохранить ник' });
            }
            throw updateErr;
        }

        await logUserActivity(req.user.id, 'display_nickname_change', 'Display nickname changed', {
            req,
            metadata: { displayNickname: raw },
        });

        res.json({
            success: true,
            message: 'Ник изменён',
            display_nickname: raw,
            display_nickname_changed_at: new Date().toISOString()
        });
    } catch (error) {
        console.error('Ошибка смены отображаемого ника:', error);
        res.status(500).json({ error: 'Не удалось изменить ник' });
    }
});

// ---------------------------------------------------------------------------
// Email change: confirm a NEW address by a code sent to that new address.
//   POST /api/profile/email/request  { email }  -> sends code to new email
//   POST /api/profile/email/confirm  { code }   -> applies the change
// ---------------------------------------------------------------------------
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

router.post('/email/request', authenticateToken, async (req, res) => {
    try {
        const email = String(req.body.email || '').trim().toLowerCase();
        if (!EMAIL_RE.test(email) || email.length > 255) {
            return res.status(400).json({ error: 'Некорректный email' });
        }

        const current = await db.query(
            'SELECT email, email_change_expires, email_changed_at FROM users WHERE id = $1',
            [req.user.id]
        );
        if (current.rows.length === 0) return res.status(404).json({ error: 'Пользователь не найден' });
        if ((current.rows[0].email || '').toLowerCase() === email) {
            return res.status(400).json({ error: 'Это уже ваша текущая почта' });
        }
        if (current.rows[0].email_changed_at) {
            const changedAt = new Date(current.rows[0].email_changed_at).getTime();
            const nextAllowed = changedAt + EMAIL_COOLDOWN_DAYS * 24 * 60 * 60 * 1000;
            if (Date.now() < nextAllowed) {
                const daysLeft = Math.ceil((nextAllowed - Date.now()) / (24 * 60 * 60 * 1000));
                return res.status(429).json({
                    error: `Почту можно менять раз в ${EMAIL_COOLDOWN_DAYS} дней. Попробуйте через ${daysLeft} дн.`,
                    next_allowed_at: new Date(nextAllowed).toISOString()
                });
            }
        }
        if (current.rows[0].email_change_expires) {
            const requestedAt =
                new Date(current.rows[0].email_change_expires).getTime() - 15 * 60 * 1000;
            const retryAfterMs = 60 * 1000 - (Date.now() - requestedAt);
            if (retryAfterMs > 0) {
                res.set('Retry-After', String(Math.ceil(retryAfterMs / 1000)));
                return res.status(429).json({
                    error: 'Подождите минуту перед повторной отправкой кода'
                });
            }
        }

        const taken = await db.query('SELECT id FROM users WHERE LOWER(email) = $1 AND id <> $2', [email, req.user.id]);
        if (taken.rows.length > 0) return res.status(409).json({ error: 'Эта почта уже используется' });

        const code = String(crypto.randomInt(100000, 1000000)); // 6 digits
        const expires = new Date(Date.now() + 15 * 60 * 1000);
        await db.query(
            'UPDATE users SET pending_email = $1, email_change_code = $2, email_change_expires = $3, email_change_attempts = 0 WHERE id = $4',
            [email, code, expires, req.user.id]
        );

        let simulated = false;
        try {
            const { sendEmailChangeCode } = require('../utils/emailService');
            const result = await sendEmailChangeCode(email, code);
            simulated = Boolean(result && result.simulated);
        } catch (e) {
            console.warn('email change send failed:', e.message);
            simulated = true;
        }

        const payload = { success: true, message: 'Код отправлен на новую почту.' };
        // Dev convenience only: echo the code when SMTP is simulated and not in prod.
        if (simulated && process.env.NODE_ENV !== 'production') {
            payload.devCode = code;
            payload.message = 'SMTP не настроен — код показан для разработки.';
        }
        res.json(payload);
    } catch (error) {
        console.error('Ошибка запроса смены почты:', error);
        res.status(500).json({ error: 'Не удалось начать смену почты' });
    }
});

router.post('/email/confirm', authenticateToken, async (req, res) => {
    try {
        const code = String(req.body.code || '').trim();
        if (!/^\d{6}$/.test(code)) return res.status(400).json({ error: 'Введите 6-значный код' });

        const result = await db.query(
            'SELECT pending_email, email_change_code, email_change_expires, email_change_attempts FROM users WHERE id = $1',
            [req.user.id]
        );
        const row = result.rows[0];
        if (!row || !row.pending_email || !row.email_change_code) {
            return res.status(400).json({ error: 'Нет запроса на смену почты' });
        }
        if (new Date(row.email_change_expires).getTime() < Date.now()) {
            return res.status(400).json({ error: 'Код истёк — запросите новый' });
        }

        // Rate limit: block brute-forcing the 6-digit code. After 5 wrong tries
        // the code is invalidated and the user must request a new one.
        const MAX_EMAIL_CODE_ATTEMPTS = 5;
        if ((row.email_change_attempts || 0) >= MAX_EMAIL_CODE_ATTEMPTS) {
            await db.query(
                'UPDATE users SET pending_email = NULL, email_change_code = NULL, email_change_expires = NULL WHERE id = $1',
                [req.user.id]
            ).catch(() => {});
            return res.status(429).json({ error: 'Слишком много попыток. Запросите новый код.' });
        }

        if (row.email_change_code !== code) {
            await db.query(
                'UPDATE users SET email_change_attempts = COALESCE(email_change_attempts, 0) + 1 WHERE id = $1',
                [req.user.id]
            ).catch(() => {});
            const left = MAX_EMAIL_CODE_ATTEMPTS - (row.email_change_attempts || 0) - 1;
            return res.status(400).json({ error: `Неверный код. Осталось попыток: ${Math.max(0, left)}` });
        }

        // Re-check uniqueness at confirm time.
        const taken = await db.query('SELECT id FROM users WHERE LOWER(email) = LOWER($1) AND id <> $2', [row.pending_email, req.user.id]);
        if (taken.rows.length > 0) {
            return res.status(409).json({ error: 'Эта почта уже используется' });
        }

        await db.query(
            `UPDATE users SET email = $1, is_email_verified = true,
             pending_email = NULL, email_change_code = NULL, email_change_expires = NULL,
             email_change_attempts = 0, email_changed_at = NOW()
             WHERE id = $2`,
            [row.pending_email, req.user.id]
        );

        await logUserActivity(req.user.id, 'email_change', 'Email address changed', {
            req,
            metadata: { verified: true },
        });

        res.json({
            success: true,
            message: 'Почта изменена',
            email: row.pending_email,
            email_changed_at: new Date().toISOString()
        });
    } catch (error) {
        console.error('Ошибка подтверждения смены почты:', error);
        res.status(500).json({ error: 'Не удалось подтвердить смену почты' });
    }
});
