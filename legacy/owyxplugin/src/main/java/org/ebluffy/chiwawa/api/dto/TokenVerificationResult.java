package org.ebluffy.chiwawa.api.dto;

/**
 * Результат проверки игрового токена
 */
public class TokenVerificationResult {
    private final boolean valid;
    private final int userId;
    private final String nickname;
    private final String role;
    private final int trustLevel;
    private final String message;

    public TokenVerificationResult(boolean valid, int userId, String nickname, String role, int trustLevel, String message) {
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

    @Override
    public String toString() {
        return "TokenVerificationResult{" +
                "valid=" + valid +
                ", userId=" + userId +
                ", nickname='" + nickname + '\'' +
                ", role='" + role + '\'' +
                ", trustLevel=" + trustLevel +
                ", message='" + message + '\'' +
                '}';
    }
}
