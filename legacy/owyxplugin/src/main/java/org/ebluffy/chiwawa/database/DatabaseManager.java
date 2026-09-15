package org.ebluffy.chiwawa.database;

import com.zaxxer.hikari.HikariConfig;
import com.zaxxer.hikari.HikariDataSource;
import org.bukkit.plugin.java.JavaPlugin;
import org.ebluffy.chiwawa.api.dto.AuthPlayer;

import java.sql.*;
import java.time.LocalDateTime;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.logging.Logger;

/**
 * Менеджер для работы с PostgreSQL базой данных
 */
public class DatabaseManager {
    private final JavaPlugin plugin;
    private final Logger logger;
    private HikariDataSource dataSource;

    public DatabaseManager(JavaPlugin plugin, String host, int port, String database, 
                          String username, String password) {
        this.plugin = plugin;
        this.logger = plugin.getLogger();
        
        initializeDataSource(host, port, database, username, password);
    }

    /**
     * Инициализация пула соединений HikariCP
     */
    private void initializeDataSource(String host, int port, String database, 
                                      String username, String password) {
        try {
            // Явно загружаем PostgreSQL драйвер
            Class.forName("org.postgresql.Driver");
            
            HikariConfig config = new HikariConfig();
            config.setJdbcUrl(String.format("jdbc:postgresql://%s:%d/%s", host, port, database));
            config.setUsername(username);
            config.setPassword(password);
            config.setDriverClassName("org.postgresql.Driver");
            
            // Настройки пула
            config.setMaximumPoolSize(10);
            config.setMinimumIdle(2);
            config.setConnectionTimeout(30000);
            config.setIdleTimeout(600000);
            config.setMaxLifetime(1800000);
            
            // Настройки соединения
            config.addDataSourceProperty("cachePrepStmts", "true");
            config.addDataSourceProperty("prepStmtCacheSize", "250");
            config.addDataSourceProperty("prepStmtCacheSqlLimit", "2048");
            config.addDataSourceProperty("useServerPrepStmts", "true");
            
            dataSource = new HikariDataSource(config);
            // Логирование убрано - успех проверяется через testConnection()
            
        } catch (Exception e) {
            logger.severe("❌ Ошибка подключения к БД: " + e.getMessage());
            // e.printStackTrace(); // Убрано для чистоты логов
        }
    }

    /**
     * Получить соединение из пула
     */
    public Connection getConnection() throws SQLException {
        if (dataSource == null) {
            throw new SQLException("DataSource не инициализирован");
        }
        return dataSource.getConnection();
    }

    /**
     * Проверить подключение к базе данных
     */
    public boolean testConnection() {
        try (Connection conn = getConnection()) {
            return conn.isValid(5);
        } catch (SQLException e) {
            logger.severe("Ошибка проверки подключения к БД: " + e.getMessage());
            return false;
        }
    }

    /**
     * Получить игрока по никнейму
     */
    public CompletableFuture<Optional<AuthPlayer>> getPlayerByUsername(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT * FROM authplugin WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    return Optional.of(mapResultSetToAuthPlayer(rs));
                }
                
                return Optional.empty();
                
            } catch (SQLException e) {
                logger.severe("Ошибка получения игрока " + username + ": " + e.getMessage());
                return Optional.empty();
            }
        });
    }

    /**
     * Получить игрока по UUID
     */
    public CompletableFuture<Optional<AuthPlayer>> getPlayerByUUID(String uuid) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT * FROM authplugin WHERE uuid = ?";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, uuid);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    return Optional.of(mapResultSetToAuthPlayer(rs));
                }
                
                return Optional.empty();
                
            } catch (SQLException e) {
                logger.severe("Ошибка получения игрока по UUID " + uuid + ": " + e.getMessage());
                return Optional.empty();
            }
        });
    }

    /**
     * Зарегистрировать нового игрока
     */
    public CompletableFuture<Boolean> registerPlayer(String username, String passwordHash, 
                                                     String uuid, String ipAddress) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "INSERT INTO authplugin (username, password_hash, uuid, ip_address, login_type) " +
                          "VALUES (?, ?, ?, ?, 0) " +
                          "ON CONFLICT (username) DO NOTHING";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                stmt.setString(2, passwordHash);
                stmt.setString(3, uuid);
                stmt.setString(4, ipAddress);
                
                int affected = stmt.executeUpdate();
                
                if (affected > 0) {
                    // Логирование убрано - уже логируется в AuthManager
                    return true;
                }
                
                // Логирование убрано
                return false;
                
            } catch (SQLException e) {
                logger.severe("Ошибка регистрации игрока " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Обновить последний вход игрока
     */
    public CompletableFuture<Boolean> updateLastLogin(String username, String ipAddress) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET last_login = NOW(), ip_address = ? " +
                          "WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, ipAddress);
                stmt.setString(2, username);
                
                stmt.executeUpdate();
                return true;
                
            } catch (SQLException e) {
                logger.severe("Ошибка обновления последнего входа " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Изменить пароль игрока
     */
    public CompletableFuture<Boolean> changePassword(String username, String newPasswordHash) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET password_hash = ? WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, newPasswordHash);
                stmt.setString(2, username);
                
                int affected = stmt.executeUpdate();
                
                if (affected > 0) {
                    logger.info("Пароль игрока " + username + " успешно изменен");
                    return true;
                }
                
                return false;
                
            } catch (SQLException e) {
                logger.severe("Ошибка изменения пароля " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Получить email игрока из таблицы users (сайта)
     */
    public CompletableFuture<Optional<String>> getEmailByNickname(String nickname) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT email FROM users WHERE LOWER(nickname) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, nickname);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    return Optional.of(rs.getString("email"));
                }
                
                return Optional.empty();
                
            } catch (SQLException e) {
                logger.severe("Ошибка получения email для " + nickname + ": " + e.getMessage());
                return Optional.empty();
            }
        });
    }

    /**
     * Создать код подтверждения для смены режима авторизации
     */
    public CompletableFuture<Boolean> createVerificationCode(String username, String code) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET " +
                          "email_verification_code = ?, " +
                          "email_verification_expires = NOW() + INTERVAL '15 minutes' " +
                          "WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, code);
                stmt.setString(2, username);
                
                int affected = stmt.executeUpdate();
                return affected > 0;
                
            } catch (SQLException e) {
                logger.severe("Ошибка создания кода подтверждения для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Проверить код подтверждения и сменить режим авторизации
     */
    public CompletableFuture<Boolean> verifyAndChangeLoginType(String username, String code, int newLoginType) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET " +
                          "login_type = ?, " +
                          "email_verification_code = NULL, " +
                          "email_verification_expires = NULL " +
                          "WHERE LOWER(username) = LOWER(?) " +
                          "AND email_verification_code = ? " +
                          "AND email_verification_expires > NOW()";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setInt(1, newLoginType);
                stmt.setString(2, username);
                stmt.setString(3, code);
                
                int affected = stmt.executeUpdate();
                
                if (affected > 0) {
                    logger.info("Режим авторизации игрока " + username + " изменен на " + 
                               (newLoginType == 0 ? "простой" : "через сайт"));
                    return true;
                }
                
                logger.warning("Неверный или истекший код подтверждения для " + username);
                return false;
                
            } catch (SQLException e) {
                logger.severe("Ошибка смены режима авторизации для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Обновить email игрока в authplugin из таблицы users
     */
    public CompletableFuture<Boolean> syncEmailFromUsers(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin a SET email = u.email " +
                          "FROM users u " +
                          "WHERE LOWER(a.username) = LOWER(u.nickname) " +
                          "AND LOWER(a.username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                
                int affected = stmt.executeUpdate();
                return affected > 0;
                
            } catch (SQLException e) {
                logger.severe("Ошибка синхронизации email для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Удалить игрока из БД authplugin
     */
    public CompletableFuture<Boolean> deletePlayer(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "DELETE FROM authplugin WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                
                int affected = stmt.executeUpdate();
                
                if (affected > 0) {
                    logger.info("Игрок " + username + " успешно удален из БД authplugin");
                    return true;
                }
                
                logger.warning("Игрок " + username + " не найден в БД authplugin");
                return false;
                
            } catch (SQLException e) {
                logger.severe("Ошибка удаления игрока " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Получить количество неудачных попыток входа для игрока
     */
    public CompletableFuture<Integer> getFailedLoginAttempts(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT failed_attempts FROM authplugin WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    return rs.getInt("failed_attempts");
                }
                
                return 0;
                
            } catch (SQLException e) {
                logger.severe("Ошибка получения попыток входа для " + username + ": " + e.getMessage());
                return 0;
            }
        });
    }

    /**
     * Увеличить счетчик неудачных попыток входа
     */
    public CompletableFuture<Boolean> incrementFailedAttempts(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET " +
                          "failed_attempts = COALESCE(failed_attempts, 0) + 1, " +
                          "last_failed_attempt = NOW() " +
                          "WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                
                stmt.executeUpdate();
                return true;
                
            } catch (SQLException e) {
                logger.severe("Ошибка увеличения счетчика попыток для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Сбросить счетчик неудачных попыток входа
     */
    public CompletableFuture<Boolean> resetFailedAttempts(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "UPDATE authplugin SET " +
                          "failed_attempts = 0, " +
                          "last_failed_attempt = NULL " +
                          "WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                
                stmt.executeUpdate();
                return true;
                
            } catch (SQLException e) {
                logger.severe("Ошибка сброса счетчика попыток для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Проверить, заблокирован ли игрок из-за превышения попыток
     */
    public CompletableFuture<Boolean> isPlayerLocked(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT failed_attempts, last_failed_attempt FROM authplugin " +
                          "WHERE LOWER(username) = LOWER(?)";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    int failedAttempts = rs.getInt("failed_attempts");
                    Timestamp lastFailed = rs.getTimestamp("last_failed_attempt");
                    
                    // Если больше 5 попыток
                    if (failedAttempts >= 5) {
                        // Проверяем, прошло ли 5 минут с последней попытки
                        if (lastFailed != null) {
                            long minutesSinceLastFailed = 
                                (System.currentTimeMillis() - lastFailed.getTime()) / (1000 * 60);
                            
                            // Если прошло менее 5 минут - заблокирован
                            return minutesSinceLastFailed < 5;
                        }
                        return true;
                    }
                }
                
                return false;
                
            } catch (SQLException e) {
                logger.severe("Ошибка проверки блокировки для " + username + ": " + e.getMessage());
                return false;
            }
        });
    }

    /**
     * Получить время до разблокировки (в минутах)
     */
    public CompletableFuture<Integer> getMinutesUntilUnlock(String username) {
        return CompletableFuture.supplyAsync(() -> {
            String query = "SELECT last_failed_attempt FROM authplugin " +
                          "WHERE LOWER(username) = LOWER(?) AND failed_attempts >= 5";
            
            try (Connection conn = getConnection();
                 PreparedStatement stmt = conn.prepareStatement(query)) {
                
                stmt.setString(1, username);
                ResultSet rs = stmt.executeQuery();
                
                if (rs.next()) {
                    Timestamp lastFailed = rs.getTimestamp("last_failed_attempt");
                    
                    if (lastFailed != null) {
                        long minutesSinceLastFailed = 
                            (System.currentTimeMillis() - lastFailed.getTime()) / (1000 * 60);
                        
                        int minutesRemaining = (int) (5 - minutesSinceLastFailed);
                        return Math.max(0, minutesRemaining);
                    }
                }
                
                return 0;
                
            } catch (SQLException e) {
                logger.severe("Ошибка получения времени до разблокировки для " + username + ": " + e.getMessage());
                return 0;
            }
        });
    }

    /**
     * Преобразовать ResultSet в объект AuthPlayer
     */
    private AuthPlayer mapResultSetToAuthPlayer(ResultSet rs) throws SQLException {
        AuthPlayer player = new AuthPlayer();
        player.setId(rs.getInt("id"));
        player.setUsername(rs.getString("username"));
        player.setPasswordHash(rs.getString("password_hash"));
        player.setUuid(rs.getString("uuid"));
        player.setIpAddress(rs.getString("ip_address"));
        player.setLoginType(rs.getInt("login_type"));
        player.setEmail(rs.getString("email"));
        player.setEmailVerificationCode(rs.getString("email_verification_code"));
        
        Timestamp lastLogin = rs.getTimestamp("last_login");
        if (lastLogin != null) {
            player.setLastLogin(lastLogin.toLocalDateTime());
        }
        
        Timestamp registeredAt = rs.getTimestamp("registered_at");
        if (registeredAt != null) {
            player.setRegisteredAt(registeredAt.toLocalDateTime());
        }
        
        Timestamp verificationExpires = rs.getTimestamp("email_verification_expires");
        if (verificationExpires != null) {
            player.setEmailVerificationExpires(verificationExpires.toLocalDateTime());
        }
        
        return player;
    }

    /**
     * Закрыть пул соединений
     */
    public void close() {
        if (dataSource != null && !dataSource.isClosed()) {
            dataSource.close();
            logger.info("Пул соединений PostgreSQL закрыт");
        }
    }
}
