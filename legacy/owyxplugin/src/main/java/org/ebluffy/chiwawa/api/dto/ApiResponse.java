package org.ebluffy.chiwawa.api.dto;

import com.google.gson.annotations.SerializedName;

/**
 * Общий DTO для ответов API
 */
public class ApiResponse<T> {
    @SerializedName("success")
    private boolean success;

    @SerializedName("message")
    private String message;

    @SerializedName("data")
    private T data;

    @SerializedName("error")
    private String error;

    // Конструкторы
    public ApiResponse() {}

    public ApiResponse(boolean success, String message, T data) {
        this.success = success;
        this.message = message;
        this.data = data;
    }

    public static <T> ApiResponse<T> success(T data) {
        return new ApiResponse<>(true, "Success", data);
    }

    public static <T> ApiResponse<T> success(String message, T data) {
        return new ApiResponse<>(true, message, data);
    }

    public static <T> ApiResponse<T> error(String error) {
        ApiResponse<T> response = new ApiResponse<>();
        response.success = false;
        response.error = error;
        return response;
    }

    // Getters и Setters
    public boolean isSuccess() { return success; }
    public void setSuccess(boolean success) { this.success = success; }

    public String getMessage() { return message; }
    public void setMessage(String message) { this.message = message; }

    public T getData() { return data; }
    public void setData(T data) { this.data = data; }

    public String getError() { return error; }
    public void setError(String error) { this.error = error; }

    /**
     * Проверить успешность ответа
     */
    public boolean hasError() {
        return !success || error != null;
    }

    /**
     * Получить сообщение об ошибке или обычное сообщение
     */
    public String getMessageOrError() {
        return hasError() ? error : message;
    }

    @Override
    public String toString() {
        return "ApiResponse{" +
                "success=" + success +
                ", message='" + message + '\'' +
                ", error='" + error + '\'' +
                ", hasData=" + (data != null) +
                '}';
    }
}

