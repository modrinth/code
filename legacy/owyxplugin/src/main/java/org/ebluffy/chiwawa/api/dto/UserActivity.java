package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * DTO для активности пользователя из таблицы user_activity
 */
public class UserActivity {
    @SerializedName("user_id")
    private int userId;

    @SerializedName("activity_type")
    private String activityType; // 'login', 'logout', 'join_server', 'leave_server', 'playtime_update'

    @SerializedName("description")
    private String description;

    @SerializedName("metadata")
    private String metadata; // JSON строка

    @SerializedName("created_at")
    private String createdAt;

    // Конструкторы
    public UserActivity() {}

    public UserActivity(int userId, String activityType, String description) {
        this.userId = userId;
        this.activityType = activityType;
        this.description = description;
    }

    public UserActivity(int userId, String activityType, String description, String metadata) {
        this.userId = userId;
        this.activityType = activityType;
        this.description = description;
        this.metadata = metadata;
    }

    // Getters и Setters
    public int getUserId() { return userId; }
    public void setUserId(int userId) { this.userId = userId; }

    public String getActivityType() { return activityType; }
    public void setActivityType(String activityType) { this.activityType = activityType; }

    public String getDescription() { return description; }
    public void setDescription(String description) { this.description = description; }

    public String getMetadata() { return metadata; }
    public void setMetadata(String metadata) { this.metadata = metadata; }

    public String getCreatedAt() { return createdAt; }
    public void setCreatedAt(String createdAt) { this.createdAt = createdAt; }

    /**
     * Проверить, является ли активность входом на сервер
     */
    public boolean isServerJoin() {
        return "join_server".equals(activityType);
    }

    /**
     * Проверить, является ли активность выходом с сервера
     */
    public boolean isServerLeave() {
        return "leave_server".equals(activityType);
    }

    /**
     * Проверить, является ли активность обновлением времени игры
     */
    public boolean isPlaytimeUpdate() {
        return "playtime_update".equals(activityType);
    }

    @Override
    public String toString() {
        return "UserActivity{" +
                "userId=" + userId +
                ", activityType='" + activityType + '\'' +
                ", description='" + description + '\'' +
                ", createdAt='" + createdAt + '\'' +
                '}';
    }
}
