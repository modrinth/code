# ==============================================
# OWYX Project - Fast Commands
# ==============================================
# Алиасы и функции для быстрой работы с проектом
#
# Установка:
# echo "source /path/to/owyx/aliases.sh" >> ~/.bashrc
# source ~/.bashrc

# Alias для docker-compose в папке проекта
alias owyx-up='docker-compose up -d'
alias owyx-down='docker-compose down'
alias owyx-restart='docker-compose restart'
alias owyx-logs='docker-compose logs -f'
alias owyx-ps='docker-compose ps'
alias owyx-pull='docker-compose pull'
alias owyx-build='docker-compose build'

# Отдельные сервисы
alias owyx-db-restart='docker-compose restart postgres'
alias owyx-api-restart='docker-compose restart directus'
alias owyx-db-logs='docker-compose logs -f postgres'
alias owyx-api-logs='docker-compose logs -f directus'

# База данных
alias owyx-db='docker exec -it owyx-postgres psql -U owyx_user -d owyx_db'
alias owyx-backup='./backup.sh'

# FRP
alias owyx-frp-restart='sudo systemctl restart frpc'
alias owyx-frp-status='sudo systemctl status frpc'
alias owyx-frp-logs='sudo journalctl -u frpc -f'

# Nginx
alias owyx-nginx-restart='sudo systemctl restart nginx'
alias owyx-nginx-status='sudo systemctl status nginx'
alias owyx-nginx-test='sudo nginx -t'
alias owyx-nginx-logs='sudo tail -f /var/log/nginx/owyx_access.log'

# Функция для быстрого доступа к проекту
owyx() {
    cd /path/to/owyx  # Измените на реальный путь
}

# Функция для полного рестарта всех сервисов
owyx-full-restart() {
    echo "Перезапуск всех сервисов OWYX..."
    docker-compose restart
    sudo systemctl restart nginx
    sudo systemctl restart frpc
    echo "Готово!"
}

# Функция для проверки статуса всех сервисов
owyx-status() {
    echo "=== Docker Containers ==="
    docker-compose ps
    echo ""
    echo "=== Nginx ==="
    sudo systemctl status nginx --no-pager
    echo ""
    echo "=== FRP Client ==="
    sudo systemctl status frpc --no-pager
}

# Функция для просмотра всех логов
owyx-logs-all() {
    echo "=== Docker Logs ==="
    docker-compose logs --tail=50
    echo ""
    echo "=== Nginx Access Log ==="
    sudo tail -20 /var/log/nginx/owyx_access.log
    echo ""
    echo "=== Nginx Error Log ==="
    sudo tail -20 /var/log/nginx/owyx_error.log
    echo ""
    echo "=== FRP Logs ==="
    sudo journalctl -u frpc --no-pager -n 20
}

echo "✅ OWYX aliases loaded! Используйте 'owyx-' и нажмите Tab для автодополнения"
