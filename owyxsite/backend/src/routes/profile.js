// Маршруты профиля пользователя
// Создатель: ebluffy

const express = require('express');
const crypto = require('crypto');
const { body, validationResult } = require('express-validator');
const db = require('../database/connection');
const { authenticateToken, authenticateLongTermApiTokenOnly } = require('./auth');
const multer = require('multer');
const path = require('path');
const fs = require('fs').promises;
const sharp = require('sharp');

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



// POST /api/profile/update-stats - Обновление статистики с сервера Minecraft (только для плагина)
router.post('/update-stats', authenticateLongTermApiTokenOnly, async (req, res) => {
    try {
        const { minecraft_nick, stats = {}, increment_login = false } = req.body;

        if (!minecraft_nick || typeof stats !== 'object' || Array.isArray(stats)) {
            return res.status(400).json({ error: 'Отсутствуют обязательные поля' });
        }

        // Находим пользователя по никнейму Minecraft
        const userResult = await db.query(`
            SELECT u.id, u.nickname 
            FROM users u 
            WHERE LOWER(u.nickname) = LOWER($1) AND u.is_active = true AND u.is_banned = false
        `, [minecraft_nick]);

        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];
        const nonNegativeInt = (value, field) => {
            if (value === undefined || value === null) return null;
            const parsed = Number(value);
            if (!Number.isSafeInteger(parsed) || parsed < 0) {
                const err = new Error(`${field}: нужно целое число >= 0`);
                err.status = 400;
                throw err;
            }
            return parsed;
        };
        const playtime = nonNegativeInt(stats.time_played_minutes, 'time_played_minutes');
        const deaths = nonNegativeInt(stats.deaths_count ?? stats.deaths, 'deaths_count');
        const kills = nonNegativeInt(stats.mobs_killed ?? stats.mob_kills, 'mobs_killed');
        const lastSeen = stats.last_seen || null;
        if (lastSeen && Number.isNaN(Date.parse(lastSeen))) {
            return res.status(400).json({ error: 'last_seen: некорректная дата' });
        }
        const loginIncrement = increment_login || stats.increment_login ? 1 : 0;

        await db.query(`
            INSERT INTO player_stats (
                user_id, time_played_minutes, last_seen, deaths_count, mobs_killed,
                total_logins, stats_last_updated, created_at, updated_at
            ) VALUES (
                $1, COALESCE($2, 0), COALESCE($3::timestamp, NOW()),
                COALESCE($4, 0), COALESCE($5, 0), $6, NOW(), NOW(), NOW()
            )
            ON CONFLICT (user_id) DO UPDATE SET
                time_played_minutes = COALESCE($2, player_stats.time_played_minutes),
                last_seen = COALESCE($3::timestamp, player_stats.last_seen),
                deaths_count = COALESCE($4, player_stats.deaths_count),
                mobs_killed = COALESCE($5, player_stats.mobs_killed),
                total_logins = COALESCE(player_stats.total_logins, 0) + $6,
                stats_last_updated = NOW(),
                updated_at = NOW()
        `, [user.id, playtime, lastSeen, deaths, kills, loginIncrement]);

        res.json({
            success: true,
            message: `Статистика игрока ${minecraft_nick} обновлена`,
            user_id: user.id
        });

    } catch (error) {
        console.error('Ошибка обновления статистики:', error);
        res.status(error.status || 500).json({
            error: error.status ? error.message : 'Ошибка сервера'
        });
    }
});

// Функция для форматирования минут в читаемый формат
function formatMinutesToTime(minutes) {
    if (!minutes || minutes === 0) return '0м';
    
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    
    if (hours === 0) return `${mins}м`;
    if (mins === 0) return `${hours}ч`;
    return `${hours}ч ${mins}м`;
}

// GET /api/profile - Получение данных профиля
router.get('/', authenticateToken, async (req, res) => {
    try {
        // Получаем основные данные пользователя - упрощенно
        const userResult = await db.query(`
            SELECT u.id, u.nickname, u.first_name, u.email, u.role, 
                   u.discord_username, u.trust_level, u.bio, u.avatar_url,
                   u.age, u.is_email_verified, u.is_banned, u.status,
                   u.ban_reason, u.ban_until, u.last_login, u.registered_at,
                   u.skin_url, u.skin_model, u.cape_url, u.cosmetics_updated_at, u.nickname_changed_at,
                   ps.time_played_minutes, ps.last_seen, ps.deaths_count, ps.mobs_killed, ps.total_logins,
                   ur.reputation_score
            FROM users u
            LEFT JOIN player_stats ps ON u.id = ps.user_id
            LEFT JOIN user_reputation ur ON u.id = ur.user_id
            WHERE u.id = $1 AND u.is_active = true
        `, [req.user.id]);

        if (userResult.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        const user = userResult.rows[0];

        // Получаем последнюю активность
        const activityResult = await db.query(`
            SELECT activity_type, description, created_at 
            FROM user_activity 
            WHERE user_id = $1 
            ORDER BY created_at DESC 
            LIMIT 10
        `, [req.user.id]);

        // Если активности нет, создадим базовую запись
        if (activityResult.rows.length === 0) {
            await db.query(`
                INSERT INTO user_activity (user_id, activity_type, description, created_at)
                VALUES ($1, 'registration', 'Регистрация на сайте', $2)
            `, [req.user.id, user.registered_at]);
            
            // Получаем активность снова
            const newActivityResult = await db.query(`
                SELECT activity_type, description, created_at 
                FROM user_activity 
                WHERE user_id = $1 
                ORDER BY created_at DESC 
                LIMIT 10
            `, [req.user.id]);
            
            activityResult.rows = newActivityResult.rows;
        }

        // Достижения убраны - слишком сложно реализовать корректно
        // const achievementsResult = { rows: [] };

        // Получаем репутацию убрана - упрощаем
        // const reputationResult = { rows: [] };
        // const reputation = { reputation_score: 0, positive_votes: 0, negative_votes: 0 };

        // Формируем упрощенную статистику - только основное
        const stats = {
            playtime: user.time_played_minutes || 0,
            days_registered: Math.floor((new Date() - new Date(user.registered_at)) / (1000 * 60 * 60 * 24)),
            last_seen: user.last_seen,
            deaths_count: user.deaths_count || 0,
            mobs_killed: user.mobs_killed || 0,
            total_logins: user.total_logins || 0
        };

        // Формируем прогресс Trust Level
        const trustProgress = {
            current: user.trust_level || 0,
            required: getTrustLevelRequirements(user.trust_level || 0),
            type: 'минут игры',
            reputation: user.reputation_score || 0
        };

        // Applications ("заявки") are no longer part of the product. Kept as
        // null in the response for one transition release so older clients that
        // read `application` don't crash.
        const application = null;

        res.json({
            id: user.id,
            minecraft_nick: user.nickname,
            display_name: user.nickname,
            first_name: user.first_name,
            age: user.age,
            email: user.email,
            discord: user.discord_username,
            bio: user.bio,
            role: user.role || 'user',
            trust_level: user.trust_level || 0,
            trust_level_name: getTrustLevelName(user.trust_level || 0),
            is_email_verified: user.is_email_verified,
            is_banned: user.is_banned || false,
            ban_info: user.is_banned ? {
                reason: user.ban_reason,
                until: user.ban_until,
                is_permanent: !user.ban_until,
                time_remaining: user.ban_until ? calculateTimeRemaining(user.ban_until) : null
            } : null,
            status: user.status || 'active',
            avatar_url: user.avatar_url,
            cosmetics: {
                skin_url: user.skin_url || null,
                skin_model: user.skin_model || 'classic',
                cape_url: user.cape_url || null,
                updated_at: user.cosmetics_updated_at || null
            },
            nickname_changed_at: user.nickname_changed_at || null,
            registered_at: user.registered_at,
            last_login: user.last_login,
            stats,
            trust_progress: trustProgress,
            activity: activityResult.rows,
            // achievements убраны - слишком сложно реализовать корректно
            application,
            player_stats: {
                time_played_minutes: user.time_played_minutes || 0,
                last_seen: user.last_seen,
                deaths_count: user.deaths_count || 0,
                mobs_killed: user.mobs_killed || 0,
                total_logins: user.total_logins || 0
            }
        });

    } catch (error) {
        console.error('Ошибка получения профиля:', error);
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// PUT /api/profile - Обновление профиля
router.put('/', [
    authenticateToken,
    body('first_name').optional().isLength({ max: 50 }),
    body('age').optional().isInt({ min: 10, max: 120 }),
    body('discord_username').optional().isLength({ max: 100 }),
    body('bio').optional().isLength({ max: 1000 }),
    body('current_password').optional().isLength({ min: 1 }),
    body('new_password').optional().isLength({ min: 8 })
], async (req, res) => {
    try {
        const errors = validationResult(req);
        if (!errors.isEmpty()) {
            return res.status(400).json({
                error: 'Ошибка валидации',
                details: errors.array()
            });
        }

        const { first_name, age, discord_username, bio, current_password, new_password } = req.body;
        if (req.body.email !== undefined) {
            return res.status(400).json({
                error: 'Используйте /api/profile/email/request и /email/confirm для смены почты'
            });
        }
        const updateFields = [];
        const updateValues = [];
        let paramIndex = 1;

        // Проверка смены пароля
        if (new_password) {
            if (!current_password) {
                return res.status(400).json({ 
                    error: 'Для смены пароля необходимо указать текущий пароль' 
                });
            }

            // Получаем текущий пароль пользователя
            const userResult = await db.query(
                'SELECT password_hash FROM users WHERE id = $1',
                [req.user.id]
            );

            if (userResult.rows.length === 0) {
                return res.status(404).json({ error: 'Пользователь не найден' });
            }

            // Проверяем текущий пароль
            const bcrypt = require('bcryptjs');
            const isCurrentPasswordValid = await bcrypt.compare(current_password, userResult.rows[0].password_hash);
            
            if (!isCurrentPasswordValid) {
                return res.status(400).json({ 
                    error: 'Неверный текущий пароль' 
                });
            }

            // Хешируем новый пароль
            const saltRounds = 12;
            const hashedNewPassword = await bcrypt.hash(new_password, saltRounds);
            
            updateFields.push(`password_hash = $${paramIndex++}`);
            updateValues.push(hashedNewPassword);
        }

        // Строим динамический запрос обновления остальных полей
        if (first_name !== undefined) {
            updateFields.push(`first_name = $${paramIndex++}`);
            updateValues.push(first_name);
        }
        if (age !== undefined) {
            updateFields.push(`age = $${paramIndex++}`);
            updateValues.push(age);
        }
        if (discord_username !== undefined) {
            updateFields.push(`discord_username = $${paramIndex++}`);
            updateValues.push(discord_username);
        }
        if (bio !== undefined) {
            updateFields.push(`bio = $${paramIndex++}`);
            updateValues.push(bio);
        }

        if (updateFields.length === 0) {
            return res.status(400).json({ error: 'Нет данных для обновления' });
        }

        updateValues.push(req.user.id);

        const updateQuery = `
            UPDATE users 
            SET ${updateFields.join(', ')}
            WHERE id = $${paramIndex} AND is_active = true
            RETURNING id, nickname, email, first_name, age, discord_username, bio
        `;

        const result = await db.query(updateQuery, updateValues);

        if (result.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }

        // Логируем действие
        const actionDetails = [];
        if (new_password) actionDetails.push('пароль');
        if (first_name !== undefined) actionDetails.push('имя');
        if (age !== undefined) actionDetails.push('возраст');
        if (discord_username !== undefined) actionDetails.push('Discord');
        if (bio !== undefined) actionDetails.push('биография');
        
        await db.query(`
            INSERT INTO admin_logs (admin_id, action, details, target_user_id)
            VALUES ($1, $2, $3, $4)
        `, [
            req.user.id,
            'profile_update',
            `Обновлен профиль: ${actionDetails.join(', ')}`,
            req.user.id
        ]);

        res.json({
            success: true,
            message: 'Профиль успешно обновлен',
            user: result.rows[0]
        });

    } catch (error) {
        console.error('Ошибка обновления профиля:', error);
        
        if (error.code === '23505') { // Unique constraint violation
            return res.status(400).json({ 
                error: 'Email уже используется другим пользователем' 
            });
        }
        
        res.status(500).json({ error: 'Внутренняя ошибка сервера' });
    }
});

// Legacy demo endpoint intentionally cannot verify an address. Clients must
// use the signed token flow under /api/auth/verify-email-token.
router.post('/verify-email', authenticateToken, (_req, res) => {
    res.status(410).json({
        error: 'Используйте ссылку подтверждения из письма'
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
        await db.query(`
            INSERT INTO user_activity (user_id, activity_type, description)
            VALUES ($1, 'avatar_update', 'Обновлен аватар профиля')
        `, [req.user.id]);

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
        await db.query(`
            INSERT INTO user_activity (user_id, activity_type, description)
            VALUES ($1, 'avatar_delete', 'Удален аватар профиля')
        `, [req.user.id]);

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

        await db.query(`
            INSERT INTO user_activity (user_id, activity_type, description)
            VALUES ($1, 'skin_update', 'Обновлён скин профиля')
        `, [req.user.id]).catch(() => {});

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

// PUT /api/profile/nickname - change nickname, at most once per 30 days.
const NICK_COOLDOWN_DAYS = 30;
router.put('/nickname', authenticateToken, async (req, res) => {
    try {
        const raw = (req.body && req.body.nickname != null) ? String(req.body.nickname).trim() : '';
        if (raw.length < 3 || raw.length > 16) {
            return res.status(400).json({ error: 'Ник должен быть от 3 до 16 символов' });
        }
        if (!/^[a-zA-Z0-9_]+$/.test(raw)) {
            return res.status(400).json({ error: 'Ник может содержать только буквы, цифры и _' });
        }

        const current = await db.query(
            'SELECT nickname, nickname_changed_at FROM users WHERE id = $1',
            [req.user.id]
        );
        if (current.rows.length === 0) {
            return res.status(404).json({ error: 'Пользователь не найден' });
        }
        const row = current.rows[0];

        if (row.nickname === raw) {
            return res.status(400).json({ error: 'Это уже ваш текущий ник' });
        }

        // 30-day cooldown.
        if (row.nickname_changed_at) {
            const changedAt = new Date(row.nickname_changed_at).getTime();
            const nextAllowed = changedAt + NICK_COOLDOWN_DAYS * 24 * 60 * 60 * 1000;
            if (Date.now() < nextAllowed) {
                const daysLeft = Math.ceil((nextAllowed - Date.now()) / (24 * 60 * 60 * 1000));
                return res.status(429).json({
                    error: `Ник можно менять раз в ${NICK_COOLDOWN_DAYS} дней. Попробуйте через ${daysLeft} дн.`,
                    next_allowed_at: new Date(nextAllowed).toISOString()
                });
            }
        }

        // Uniqueness (case-insensitive).
        const taken = await db.query(
            'SELECT id FROM users WHERE LOWER(nickname) = LOWER($1) AND id <> $2',
            [raw, req.user.id]
        );
        if (taken.rows.length > 0) {
            return res.status(409).json({ error: 'Этот ник уже занят' });
        }

        await db.query(
            'UPDATE users SET nickname = $1, nickname_changed_at = NOW() WHERE id = $2',
            [raw, req.user.id]
        );

        await db.query(`
            INSERT INTO user_activity (user_id, activity_type, description)
            VALUES ($1, 'nickname_change', $2)
        `, [req.user.id, `Ник изменён на ${raw}`]).catch(() => {});

        // Notify by email (best-effort; simulated in logs when SMTP is off).
        try {
            const { sendNicknameChangedEmail } = require('../utils/emailService');
            if (req.user.email) sendNicknameChangedEmail(req.user.email, row.nickname, raw).catch(() => {});
        } catch { /* ignore */ }

        res.json({
            success: true,
            message: 'Ник изменён',
            nickname: raw,
            nickname_changed_at: new Date().toISOString()
        });
    } catch (error) {
        console.error('Ошибка смены ника:', error);
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
            'SELECT email, email_change_expires FROM users WHERE id = $1',
            [req.user.id]
        );
        if (current.rows.length === 0) return res.status(404).json({ error: 'Пользователь не найден' });
        if ((current.rows[0].email || '').toLowerCase() === email) {
            return res.status(400).json({ error: 'Это уже ваша текущая почта' });
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
             email_change_attempts = 0
             WHERE id = $2`,
            [row.pending_email, req.user.id]
        );

        res.json({ success: true, message: 'Почта изменена', email: row.pending_email });
    } catch (error) {
        console.error('Ошибка подтверждения смены почты:', error);
        res.status(500).json({ error: 'Не удалось подтвердить смену почты' });
    }
});
