package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * DTO для статистики игрока из таблицы player_stats
 */
public class PlayerStats {
    @SerializedName("user_id")
    private int userId;

    @SerializedName("total_minutes")
    private int totalMinutes;

    @SerializedName("time_played_minutes")
    private int timePlayedMinutes;

    @SerializedName("is_time_limited")
    private boolean isTimeLimited;

    @SerializedName("current_level")
    private int currentLevel;

    @SerializedName("email_verified")
    private boolean emailVerified;

    @SerializedName("discord_verified")
    private boolean discordVerified;

    @SerializedName("minecraft_verified")
    private boolean minecraftVerified;

    @SerializedName("reputation")
    private int reputation;

    @SerializedName("achievements_count")
    private int achievementsCount;

    @SerializedName("total_logins")
    private int totalLogins;

    @SerializedName("warnings_count")
    private int warningsCount;

    // Конструкторы
    public PlayerStats() {}

    public PlayerStats(int userId) {
        this.userId = userId;
    }

    // Getters и Setters
    public int getUserId() { return userId; }
    public void setUserId(int userId) { this.userId = userId; }

    public int getTotalMinutes() { return totalMinutes; }
    public void setTotalMinutes(int totalMinutes) { this.totalMinutes = totalMinutes; }

    public int getTimePlayedMinutes() { return timePlayedMinutes; }
    public void setTimePlayedMinutes(int timePlayedMinutes) { this.timePlayedMinutes = timePlayedMinutes; }

    public boolean isTimeLimited() { return isTimeLimited; }
    public void setTimeLimited(boolean timeLimited) { isTimeLimited = timeLimited; }

    public int getCurrentLevel() { return currentLevel; }
    public void setCurrentLevel(int currentLevel) { this.currentLevel = currentLevel; }

    public boolean isEmailVerified() { return emailVerified; }
    public void setEmailVerified(boolean emailVerified) { this.emailVerified = emailVerified; }

    public boolean isDiscordVerified() { return discordVerified; }
    public void setDiscordVerified(boolean discordVerified) { this.discordVerified = discordVerified; }

    public boolean isMinecraftVerified() { return minecraftVerified; }
    public void setMinecraftVerified(boolean minecraftVerified) { this.minecraftVerified = minecraftVerified; }

    public int getReputation() { return reputation; }
    public void setReputation(int reputation) { this.reputation = reputation; }

    public int getAchievementsCount() { return achievementsCount; }
    public void setAchievementsCount(int achievementsCount) { this.achievementsCount = achievementsCount; }

    public int getTotalLogins() { return totalLogins; }
    public void setTotalLogins(int totalLogins) { this.totalLogins = totalLogins; }

    public int getWarningsCount() { return warningsCount; }
    public void setWarningsCount(int warningsCount) { this.warningsCount = warningsCount; }

    /**
     * Получить время игры в часах
     */
    public double getPlaytimeHours() {
        return timePlayedMinutes / 60.0;
    }

    /**
     * Проверить, превышен ли лимит времени для проходимцев (10 часов)
     */
    public boolean isTimeLimitExceeded() {
        return isTimeLimited && getPlaytimeHours() >= 10.0;
    }

    /**
     * Получить оставшееся время для проходимцев в минутах
     */
    public int getRemainingMinutes() {
        if (!isTimeLimited) return -1;
        return Math.max(0, 600 - timePlayedMinutes); // 600 минут = 10 часов
    }

    /**
     * Форматированное время игры
     */
    public String getFormattedPlaytime() {
        int hours = timePlayedMinutes / 60;
        int minutes = timePlayedMinutes % 60;
        return String.format("%dч %dм", hours, minutes);
    }

    @Override
    public String toString() {
        return "PlayerStats{" +
                "userId=" + userId +
                ", timePlayedMinutes=" + timePlayedMinutes +
                ", reputation=" + reputation +
                ", isTimeLimited=" + isTimeLimited +
                '}';
    }
}
