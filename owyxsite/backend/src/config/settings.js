// Настройки сервера
// Автоматически сгенерировано админ-панелью

module.exports = {
    "server": {
        "name": "ChiwawaMine",
        "description": "Приватный Minecraft сервер для своих.<br><span class=\"text-chiwawa-primary font-semibold\">Вход только по заявку.</span>",
        "ip": "play.chiwawa.site",
        "port": "25164",
        "website": "https://chiwawa.ru",
        "discord": "https://discord.gg/your-invite",
        "telegram": "https://t.me/chiwawa"
    },
    "applications": {
        "minMotivationLength": 50,
        "minPlansLength": 30,
        "maxApplicationsPerDay": 3,
        "autoApprovalEnabled": false
    },
    "trustLevel": {
        "requirements": {
            "0": {
                "name": "🚶 Проходимец",
                "hoursRequired": 0,
                "reputationRequired": 0,
                "emailVerified": false,
                "description": "Лимит 10 часов ВСЕГО до подтверждения email"
            },
            "1": {
                "name": "👤 Новичок",
                "hoursRequired": 0,
                "reputationRequired": 0,
                "emailVerified": true,
                "description": "Стандартная группа после подтверждения email, без лимитов"
            },
            "2": {
                "name": "✅ Проверенный",
                "hoursRequired": 25,
                "reputationRequired": 10,
                "emailVerified": true,
                "description": "Требует заявки на одобрение повышения"
            },
            "3": {
                "name": "🏆 Ветеран",
                "hoursRequired": 50,
                "reputationRequired": 20,
                "emailVerified": true,
                "description": "Высший уровень доверия для игроков"
            }
        },
        "maxPlayTimeForNovice": 10,
        "autoPromotionEnabled": false,
        "applicationCooldownHours": 24
    },
    "reputation": {
        "voteWeight": {
            "novice": 1,
            "verified": 2,
            "veteran": 3,
            "moderator": 5,
            "admin": 10
        },
        "activityBonus": {
            "daily": 1,
            "weekly": 5,
            "monthly": 20
        },
        "penalties": {
            "ban": -50,
            "warning": -10
        }
    },
    "security": {
        "maxLoginAttempts": 5,
        "lockoutTime": 15,
        "tokenExpiration": "7d",
        "bcryptRounds": 12
    },
    "email": {
        "from": "noreply@chiwawa.ru",
        "service": "gmail",
        "verificationExpiration": 24
    }
};
