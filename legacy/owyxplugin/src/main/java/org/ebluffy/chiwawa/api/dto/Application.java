package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * DTO для заявки из таблицы applications
 */
public class Application {
    @SerializedName("id")
    private int id;

    @SerializedName("user_id")
    private int userId;

    @SerializedName("minecraft_nick")
    private String minecraftNick;

    @SerializedName("email")
    private String email;

    @SerializedName("discord")
    private String discord;

    @SerializedName("status")
    private String status; // 'pending'|'approved'|'rejected'

    @SerializedName("motivation")
    private String motivation;

    @SerializedName("plans")
    private String plans;

    @SerializedName("submitted_at")
    private String submittedAt;

    @SerializedName("reviewed_at")
    private String reviewedAt;

    // Конструкторы
    public Application() {}

    public Application(int userId, String minecraftNick, String email) {
        this.userId = userId;
        this.minecraftNick = minecraftNick;
        this.email = email;
        this.status = "pending";
    }

    // Getters и Setters
    public int getId() { return id; }
    public void setId(int id) { this.id = id; }

    public int getUserId() { return userId; }
    public void setUserId(int userId) { this.userId = userId; }

    public String getMinecraftNick() { return minecraftNick; }
    public void setMinecraftNick(String minecraftNick) { this.minecraftNick = minecraftNick; }

    public String getEmail() { return email; }
    public void setEmail(String email) { this.email = email; }

    public String getDiscord() { return discord; }
    public void setDiscord(String discord) { this.discord = discord; }

    public String getStatus() { return status; }
    public void setStatus(String status) { this.status = status; }

    public String getMotivation() { return motivation; }
    public void setMotivation(String motivation) { this.motivation = motivation; }

    public String getPlans() { return plans; }
    public void setPlans(String plans) { this.plans = plans; }

    public String getSubmittedAt() { return submittedAt; }
    public void setSubmittedAt(String submittedAt) { this.submittedAt = submittedAt; }

    public String getReviewedAt() { return reviewedAt; }
    public void setReviewedAt(String reviewedAt) { this.reviewedAt = reviewedAt; }

    /**
     * Проверить, одобрена ли заявка
     */
    public boolean isApproved() {
        return "approved".equals(status);
    }

    /**
     * Проверить, отклонена ли заявка
     */
    public boolean isRejected() {
        return "rejected".equals(status);
    }

    /**
     * Проверить, на рассмотрении ли заявка
     */
    public boolean isPending() {
        return "pending".equals(status);
    }

    @Override
    public String toString() {
        return "Application{" +
                "id=" + id +
                ", userId=" + userId +
                ", minecraftNick='" + minecraftNick + '\'' +
                ", status='" + status + '\'' +
                ", submittedAt='" + submittedAt + '\'' +
                '}';
    }
}

