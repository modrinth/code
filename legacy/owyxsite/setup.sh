#!/bin/bash

# ==============================================
# OWYX Quick Start Script
# ==============================================
# Быстрая настройка проекта
#
# Использование:
# chmod +x setup.sh
# ./setup.sh

set -e

echo "🎮 OWYX - Quick Setup Script"
echo "===================================="
echo ""

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Проверка прав root для некоторых операций
check_root() {
    if [ "$EUID" -ne 0 ]; then 
        echo -e "${YELLOW}Примечание: Для установки системных пакетов может потребоваться sudo${NC}"
    fi
}

# Проверка установки Docker
check_docker() {
    echo "Проверка Docker..."
    if ! command -v docker &> /dev/null; then
        echo -e "${RED}Docker не установлен!${NC}"
        echo "Установка Docker..."
        sudo apt update
        sudo apt install -y docker.io docker-compose
        sudo systemctl enable docker
        sudo systemctl start docker
        sudo usermod -aG docker $USER
        echo -e "${GREEN}Docker установлен!${NC}"
        echo -e "${YELLOW}ВАЖНО: Перелогиньтесь или выполните: newgrp docker${NC}"
    else
        echo -e "${GREEN}✓ Docker установлен${NC}"
    fi
}

# Проверка установки Docker Compose
check_docker_compose() {
    echo "Проверка Docker Compose..."
    if ! command -v docker-compose &> /dev/null; then
        echo -e "${RED}Docker Compose не установлен!${NC}"
        echo "Установка Docker Compose..."
        sudo apt install -y docker-compose
        echo -e "${GREEN}Docker Compose установлен!${NC}"
    else
        echo -e "${GREEN}✓ Docker Compose установлен${NC}"
    fi
}

# Создание .env файла
setup_env() {
    echo ""
    echo "Настройка .env файла..."
    
    if [ -f .env ]; then
        echo -e "${YELLOW}.env файл уже существует${NC}"
        read -p "Перезаписать? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Пропускаем создание .env"
            return
        fi
    fi
    
    cp .env.example .env
    
    # Генерация случайных ключей
    echo "Генерация случайных ключей..."
    
    if command -v openssl &> /dev/null; then
        DB_PASSWORD=$(openssl rand -hex 16)
        DIRECTUS_KEY=$(openssl rand -hex 32)
        DIRECTUS_SECRET=$(openssl rand -hex 32)
        JWT_SECRET=$(openssl rand -hex 32)
        SESSION_SECRET=$(openssl rand -hex 32)
        
        # Замена в .env
        sed -i "s/DB_PASSWORD=.*/DB_PASSWORD=$DB_PASSWORD/" .env
        sed -i "s/DIRECTUS_KEY=.*/DIRECTUS_KEY=$DIRECTUS_KEY/" .env
        sed -i "s/DIRECTUS_SECRET=.*/DIRECTUS_SECRET=$DIRECTUS_SECRET/" .env
        
        echo -e "${GREEN}✓ Ключи сгенерированы и добавлены в .env${NC}"
    else
        echo -e "${YELLOW}openssl не найден, используйте .env.example как шаблон${NC}"
    fi
    
    # Запрос дополнительных данных
    echo ""
    read -p "Email администратора (admin@owyx.site): " admin_email
    admin_email=${admin_email:-admin@owyx.site}
    sed -i "s/ADMIN_EMAIL=.*/ADMIN_EMAIL=$admin_email/" .env
    
    read -p "Пароль администратора (ChangeMe123!): " admin_password
    admin_password=${admin_password:-ChangeMe123!}
    sed -i "s/ADMIN_PASSWORD=.*/ADMIN_PASSWORD=$admin_password/" .env
    
    echo ""
    echo -e "${GREEN}✓ .env файл создан${NC}"
    echo -e "${YELLOW}ВАЖНО: Отредактируйте .env и добавьте данные VPS (FRP_SERVER_ADDR, FRP_TOKEN)${NC}"
}

# Создание необходимых папок
create_directories() {
    echo ""
    echo "Создание директорий для volumes..."
    
    mkdir -p postgres/data
    mkdir -p directus/uploads
    mkdir -p directus/extensions
    mkdir -p directus/database
    
    # Права доступа
    chmod -R 755 postgres/
    chmod -R 755 directus/
    
    echo -e "${GREEN}✓ Директории созданы${NC}"
}

# Запуск контейнеров
start_containers() {
    echo ""
    read -p "Запустить Docker контейнеры сейчас? (Y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        echo "Запуск контейнеров..."
        docker-compose up -d
        echo ""
        echo -e "${GREEN}✓ Контейнеры запущены!${NC}"
        echo ""
        echo "Проверка статуса:"
        docker-compose ps
        echo ""
        echo -e "${GREEN}Доступ к Directus: http://localhost:8055${NC}"
        echo -e "${GREEN}Логин: $admin_email${NC}"
        echo ""
    fi
}

# Показ следующих шагов
show_next_steps() {
    echo ""
    echo "===================================="
    echo "✅ Базовая настройка завершена!"
    echo "===================================="
    echo ""
    echo "📋 Следующие шаги:"
    echo ""
    echo "1️⃣  Настройка Nginx на хосте:"
    echo "   sudo cp nginx-conf/owyx.conf /etc/nginx/sites-available/"
    echo "   sudo ln -s /etc/nginx/sites-available/owyx.conf /etc/nginx/sites-enabled/"
    echo "   sudo nginx -t"
    echo "   sudo systemctl reload nginx"
    echo ""
    echo "2️⃣  Получение SSL сертификата:"
    echo "   sudo certbot --nginx -d owyx.site -d www.owyx.site -d api.owyx.site"
    echo ""
    echo "3️⃣  Настройка FRP Client:"
    echo "   Отредактируйте frpc-conf/frpc.ini (добавьте IP VPS и токен)"
    echo "   sudo cp frpc-conf/frpc.ini /etc/frp/"
    echo "   sudo cp frpc-conf/frpc.service /etc/systemd/system/"
    echo "   sudo systemctl enable frpc"
    echo "   sudo systemctl start frpc"
    echo ""
    echo "4️⃣  Проверка работы:"
    echo "   docker-compose logs -f      # Логи контейнеров"
    echo "   sudo systemctl status frpc  # Статус FRP"
    echo "   sudo systemctl status nginx # Статус Nginx"
    echo ""
    echo "📚 Подробная документация: README.md"
    echo ""
}

# Основная функция
main() {
    check_root
    check_docker
    check_docker_compose
    setup_env
    create_directories
    start_containers
    show_next_steps
}

# Запуск
main
