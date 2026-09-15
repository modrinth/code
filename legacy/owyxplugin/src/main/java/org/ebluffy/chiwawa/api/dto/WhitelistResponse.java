package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * DTO для ответа проверки whitelist
 */
public class WhitelistResponse {
    @SerializedName("hasAccess")
    private boolean hasAccess;

    @SerializedName("reason")
    private String reason;

    @SerializedName("user")
    private ChiwawaUser user;

    @SerializedName("application")
    private Application application;

    // Конструкторы
    public WhitelistResponse() {}

    public WhitelistResponse(boolean hasAccess, String reason) {
        this.hasAccess = hasAccess;
        this.reason = reason;
    }

    // Getters и Setters
    public boolean hasAccess() { return hasAccess; }
    public void setHasAccess(boolean hasAccess) { this.hasAccess = hasAccess; }

    public String getReason() { return reason; }
    public void setReason(String reason) { this.reason = reason; }

    public ChiwawaUser getUser() { return user; }
    public void setUser(ChiwawaUser user) { this.user = user; }

    public Application getApplication() { return application; }
    public void setApplication(Application application) { this.application = application; }

    @Override
    public String toString() {
        return "WhitelistResponse{" +
                "hasAccess=" + hasAccess +
                ", reason='" + reason + '\'' +
                '}';
    }
}
