package org.ebluffy.chiwawa.api.dto;

import java.time.LocalDateTime;

/**
 * DTO для игрока с простой авторизацией
 */
public class AuthPlayer {
    private int id;
    private String username;
    private String passwordHash;
    private String uuid;
    private String ipAddress;
    private LocalDateTime lastLogin;
    private LocalDateTime registeredAt;
    private int loginType; // 0 = обычная, 1 = через сайт
    private String email;
    private String emailVerificationCode;
    private LocalDateTime emailVerificationExpires;

    public AuthPlayer() {
    }

    public AuthPlayer(String username, String passwordHash, String uuid, String ipAddress, int loginType) {
        this.username = username;
        this.passwordHash = passwordHash;
        this.uuid = uuid;
        this.ipAddress = ipAddress;
        this.loginType = loginType;
    }

    // Геттеры и сеттеры
    public int getId() {
        return id;
    }

    public void setId(int id) {
        this.id = id;
    }

    public String getUsername() {
        return username;
    }

    public void setUsername(String username) {
        this.username = username;
    }

    public String getPasswordHash() {
        return passwordHash;
    }

    public void setPasswordHash(String passwordHash) {
        this.passwordHash = passwordHash;
    }

    public String getUuid() {
        return uuid;
    }

    public void setUuid(String uuid) {
        this.uuid = uuid;
    }

    public String getIpAddress() {
        return ipAddress;
    }

    public void setIpAddress(String ipAddress) {
        this.ipAddress = ipAddress;
    }

    public LocalDateTime getLastLogin() {
        return lastLogin;
    }

    public void setLastLogin(LocalDateTime lastLogin) {
        this.lastLogin = lastLogin;
    }

    public LocalDateTime getRegisteredAt() {
        return registeredAt;
    }

    public void setRegisteredAt(LocalDateTime registeredAt) {
        this.registeredAt = registeredAt;
    }

    public int getLoginType() {
        return loginType;
    }

    public void setLoginType(int loginType) {
        this.loginType = loginType;
    }

    public String getEmail() {
        return email;
    }

    public void setEmail(String email) {
        this.email = email;
    }

    public String getEmailVerificationCode() {
        return emailVerificationCode;
    }

    public void setEmailVerificationCode(String emailVerificationCode) {
        this.emailVerificationCode = emailVerificationCode;
    }

    public LocalDateTime getEmailVerificationExpires() {
        return emailVerificationExpires;
    }

    public void setEmailVerificationExpires(LocalDateTime emailVerificationExpires) {
        this.emailVerificationExpires = emailVerificationExpires;
    }

    /**
     * Проверить, истек ли срок действия кода подтверждения
     */
    public boolean isVerificationExpired() {
        if (emailVerificationExpires == null) {
            return true;
        }
        return LocalDateTime.now().isAfter(emailVerificationExpires);
    }

    /**
     * Использует ли игрок авторизацию через сайт
     */
    public boolean isWebsiteAuth() {
        return loginType == 1;
    }

    /**
     * Использует ли игрок простую авторизацию
     */
    public boolean isSimpleAuth() {
        return loginType == 0;
    }

    @Override
    public String toString() {
        return "AuthPlayer{" +
                "id=" + id +
                ", username='" + username + '\'' +
                ", uuid='" + uuid + '\'' +
                ", loginType=" + (loginType == 0 ? "Простая" : "Через сайт") +
                ", email='" + email + '\'' +
                ", registeredAt=" + registeredAt +
                '}';
    }
}
