package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * DTO для пользователя из таблицы users
 */
public class ChiwawaUser {
    @SerializedName("id")
    private int id;

    @SerializedName("nickname")
    private String nickname;

    @SerializedName("email")
    private String email;

    @SerializedName("first_name")
    private String firstName;

    @SerializedName("last_name")
    private String lastName;

    @SerializedName("role")
    private String role; // 'user'|'admin'|'moderator'

    @SerializedName("trust_level")
    private int trustLevel; // 0-3

    @SerializedName("is_active")
    private boolean isActive;

    @SerializedName("is_email_verified")
    private boolean isEmailVerified;

    @SerializedName("is_banned")
    private boolean isBanned;

    @SerializedName("registered_at")
    private String registeredAt;

    @SerializedName("last_login")
    private String lastLogin;

    @SerializedName("ban_reason")
    private String banReason;

    @SerializedName("ban_until")
    private String banUntil;

    // Конструкторы
    public ChiwawaUser() {}

    public ChiwawaUser(String nickname) {
        this.nickname = nickname;
    }

    // Getters и Setters
    public int getId() { return id; }
    public void setId(int id) { this.id = id; }

    public String getNickname() { return nickname; }
    public void setNickname(String nickname) { this.nickname = nickname; }

    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }

    public String getFirstName() { return firstName; }
    public void setFirstName(String firstName) { this.firstName = firstName; }

    public String getLastName() { return lastName; }
    public void setLastName(String lastName) { this.lastName = lastName; }

    public String getRole() { return role; }
    public void setRole(String role) { this.role = role; }

    public int getTrustLevel() { return trustLevel; }
    public void setTrustLevel(int trustLevel) { this.trustLevel = trustLevel; }

    public boolean isActive() { return isActive; }
    public void setActive(boolean active) { isActive = active; }

    public boolean isEmailVerified() { return isEmailVerified; }
    public void setEmailVerified(boolean emailVerified) { isEmailVerified = emailVerified; }

    public boolean isBanned() { return isBanned; }
    public void setBanned(boolean banned) { isBanned = banned; }

    public String getRegisteredAt() { return registeredAt; }
    public void setRegisteredAt(String registeredAt) { this.registeredAt = registeredAt; }

    public String getLastLogin() { return lastLogin; }
    public void setLastLogin(String lastLogin) { this.lastLogin = lastLogin; }

    public String getBanReason() { return banReason; }
    public void setBanReason(String banReason) { this.banReason = banReason; }

    public String getBanUntil() { return banUntil; }
    public void setBanUntil(String banUntil) { this.banUntil = banUntil; }

    /**
     * Проверяет, имеет ли игрок доступ к серверу
     */
    public boolean hasServerAccess() {
        return isActive && !isBanned;
    }

    /**
     * Проверяет, является ли игрок проходимцем (trust level 0)
     */
    public boolean isNewcomer() {
        return trustLevel == 0;
    }

    /**
     * Получить название trust level
     */
    public String getTrustLevelName() {
        return switch (trustLevel) {
            case 0 -> "Проходимец";
            case 1 -> "Новичок";
            case 2 -> "Проверенный";
            case 3 -> "Ветеран";
            default -> "Неизвестный";
        };
    }

    /**
     * Получить описание Trust Level
     */
    public String getTrustLevelDescription() {
        switch (trustLevel) {
            case 0: return "Проходимец";
            case 1: return "Новичок";
            case 2: return "Проверенный";
            case 3: return "Ветеран";
            default: return "Неизвестный";
        }
    }

    /**
     * Проверяет, имеет ли пользователь лимит времени
     */
    public boolean hasTimeLimit() {
        return trustLevel == 0 && !isEmailVerified;
    }

    /**
     * Проверяет, является ли пользователь администратором
     */
    public boolean isAdmin() {
        return "admin".equals(role);
    }

    /**
     * Проверяет, является ли пользователь модератором или выше
     */
    public boolean isModerator() {
        return "moderator".equals(role) || "admin".equals(role);
    }

    /**
     * Получить отформатированное время до окончания бана
     */
    public String getFormattedBanTimeRemaining() {
        if (banUntil == null || banUntil.isEmpty()) {
            return "Постоянно";
        }

        try {
            // Парсим дату из ISO формата
            java.time.LocalDateTime banDate = java.time.LocalDateTime.parse(banUntil.replace("Z", "").replace("+00", ""));
            java.time.LocalDateTime now = java.time.LocalDateTime.now();
            
            if (banDate.isBefore(now)) {
                return "Бан истек";
            }
            
            java.time.Duration duration = java.time.Duration.between(now, banDate);
            long days = duration.toDays();
            long hours = duration.toHours() % 24;
            long minutes = duration.toMinutes() % 60;
            
            if (days > 0) {
                return days + "д " + hours + "ч " + minutes + "м";
            } else if (hours > 0) {
                return hours + "ч " + minutes + "м";
            } else {
                return minutes + "м";
            }
        } catch (Exception e) {
            return "Ошибка расчета времени";
        }
    }

    @Override
    public String toString() {
        return "ChiwawaUser{" +
                "id=" + id +
                ", nickname='" + nickname + '\'' +
                ", role='" + role + '\'' +
                ", trustLevel=" + trustLevel +
                ", isActive=" + isActive +
                ", isBanned=" + isBanned +
                '}';
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ChiwawaUser that = (ChiwawaUser) o;
        return id == that.id;
    }

    @Override
    public int hashCode() {
        return Integer.hashCode(id);
    }
}
