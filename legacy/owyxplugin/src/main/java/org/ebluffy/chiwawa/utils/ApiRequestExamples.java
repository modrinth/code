package org.ebluffy.chiwawa.utils;

import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.net.URI;
import java.time.Duration;
import com.google.gson.Gson;

/**
 * Примеры типовых HTTP запросов к API сайта
 * Этот класс показывает различные способы взаимодействия с API
 */
public class ApiRequestExamples {

    private static final String API_BASE_URL = "https://your-website.com/api";
    private static final String API_KEY = "your-secret-api-key";
    private static final Gson gson = new Gson();

    /**
     * Пример 1: GET запрос для получения данных игрока (с использованием стандартного HttpClient)
     */
    public static void getPlayerDataExample(String playerUuid) {
        try {
            HttpClient client = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .build();

            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(API_BASE_URL + "/players/" + playerUuid))
                .header("Authorization", "Bearer " + API_KEY)
                .header("Content-Type", "application/json")
                .header("User-Agent", "ChiwawaPlugin/1.0")
                .timeout(Duration.ofSeconds(30))
                .GET()
                .build();

            HttpResponse<String> response = client.send(request,
                HttpResponse.BodyHandlers.ofString());

            if (response.statusCode() == 200) {
                System.out.println("Успешный ответ: " + response.body());
                // PlayerData playerData = gson.fromJson(response.body(), PlayerData.class);
            } else {
                System.err.println("Ошибка HTTP: " + response.statusCode());
            }

        } catch (Exception e) {
            System.err.println("Ошибка запроса: " + e.getMessage());
        }
    }

    /**
     * Пример 2: POST запрос для обновления данных игрока
     */
    public static void updatePlayerDataExample(String playerUuid, Object playerData) {
        try {
            String jsonBody = gson.toJson(playerData);

            HttpClient client = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .build();

            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(API_BASE_URL + "/players/" + playerUuid))
                .header("Authorization", "Bearer " + API_KEY)
                .header("Content-Type", "application/json")
                .timeout(Duration.ofSeconds(30))
                .PUT(HttpRequest.BodyPublishers.ofString(jsonBody))
                .build();

            HttpResponse<String> response = client.send(request,
                HttpResponse.BodyHandlers.ofString());

            System.out.println("Статус: " + response.statusCode());
            System.out.println("Ответ: " + response.body());

        } catch (Exception e) {
            System.err.println("Ошибка при обновлении данных: " + e.getMessage());
        }
    }

    /**
     * Пример 3: GET запрос whitelist с обработкой ошибок
     */
    public static void getWhitelistExample() {
        try {
            HttpClient client = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(10))
                .followRedirects(HttpClient.Redirect.NORMAL)
                .build();

            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(API_BASE_URL + "/whitelist"))
                .header("Authorization", "Bearer " + API_KEY)
                .header("Accept", "application/json")
                .timeout(Duration.ofSeconds(30))
                .GET()
                .build();

            HttpResponse<String> response = client.send(request,
                HttpResponse.BodyHandlers.ofString());

            switch (response.statusCode()) {
                case 200:
                    System.out.println("Whitelist получен: " + response.body());
                    break;
                case 401:
                    System.err.println("Ошибка авторизации: неверный API ключ");
                    break;
                case 404:
                    System.err.println("Эндпоинт не найден");
                    break;
                case 429:
                    System.err.println("Превышен лимит запросов");
                    break;
                case 500:
                    System.err.println("Внутренняя ошибка сервера");
                    break;
                default:
                    System.err.println("Неожиданный статус: " + response.statusCode());
            }

        } catch (Exception e) {
            System.err.println("Ошибка сети: " + e.getMessage());
        }
    }

    /**
     * Пример 4: Асинхронный запрос
     */
    public static void asyncRequestExample() {
        HttpClient client = HttpClient.newBuilder()
            .connectTimeout(Duration.ofSeconds(10))
            .build();

        HttpRequest request = HttpRequest.newBuilder()
            .uri(URI.create(API_BASE_URL + "/health"))
            .header("Authorization", "Bearer " + API_KEY)
            .timeout(Duration.ofSeconds(30))
            .GET()
            .build();

        client.sendAsync(request, HttpResponse.BodyHandlers.ofString())
            .thenApply(HttpResponse::body)
            .thenAccept(body -> System.out.println("Асинхронный ответ: " + body))
            .exceptionally(throwable -> {
                System.err.println("Асинхронная ошибка: " + throwable.getMessage());
                return null;
            });
    }

    /**
     * Пример 5: Batch запрос для множественного обновления
     */
    public static void batchUpdateExample() {
        try {
            // Создаем массив данных для batch обновления
            Object[] batchData = {
                // Массив PlayerData объектов
            };

            String jsonBody = gson.toJson(batchData);

            HttpClient client = HttpClient.newBuilder().build();

            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(API_BASE_URL + "/players/batch"))
                .header("Authorization", "Bearer " + API_KEY)
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(jsonBody))
                .build();

            HttpResponse<String> response = client.send(request,
                HttpResponse.BodyHandlers.ofString());

            System.out.println("Batch обновление: " + response.statusCode());

        } catch (Exception e) {
            System.err.println("Ошибка batch запроса: " + e.getMessage());
        }
    }

    /**
     * Пример авторизации через JWT токен
     */
    public static String authenticateAndGetToken(String username, String password) {
        try {
            // Создаем Map вместо анонимного объекта, чтобы избежать self-reference
            java.util.Map<String, String> loginData = new java.util.HashMap<>();
            loginData.put("username", username);
            loginData.put("password", password);
            loginData.put("grant_type", "password");

            String jsonBody = gson.toJson(loginData);

            HttpClient client = HttpClient.newBuilder().build();

            HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(API_BASE_URL + "/auth/login"))
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(jsonBody))
                .build();

            HttpResponse<String> response = client.send(request,
                HttpResponse.BodyHandlers.ofString());

            if (response.statusCode() == 200) {
                // Парсим JWT токен из ответа
                return extractTokenFromResponse(response.body());
            } else {
                System.err.println("Ошибка авторизации: " + response.statusCode());
                return null;
            }

        } catch (Exception e) {
            System.err.println("Ошибка авторизации: " + e.getMessage());
            return null;
        }
    }

    private static String extractTokenFromResponse(String responseBody) {
        // Реализация извлечения токена из ответа
        return "extracted_jwt_token";
    }
}

