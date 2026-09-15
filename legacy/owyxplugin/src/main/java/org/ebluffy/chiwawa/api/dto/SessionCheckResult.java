package org.ebluffy.chiwawa.api.dto;

/**
 * Результат проверки игровой сессии
 */
public class SessionCheckResult {
    private final boolean valid;
    private final int userId;
    private final String nickname;
    private final String role;
    private final int trustLevel;
    private final String message;

    public SessionCheckResult(boolean valid, int userId, String nickname, String role, int trustLevel, String message) {
        this.valid = valid;
        this.userId = userId;
        this.nickname = nickname;
        this.role = role;
        this.trustLevel = trustLevel;
        this.message = message;
    }

    public boolean isValid() {
        return valid;
    }

    public int getUserId() {
        return userId;
    }

    public String getNickname() {
        return nickname;
    }

    public String getRole() {
        return role;
    }

    public int getTrustLevel() {
        return trustLevel;
    }

    public String getMessage() {
        return message;
    }
}
