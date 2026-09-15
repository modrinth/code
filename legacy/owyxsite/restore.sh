#!/bin/bash

# ==============================================
# OWYX Database Restore Script
# ==============================================
# Восстановление PostgreSQL базы данных из бэкапа
#
# Использование:
# chmod +x restore.sh
# ./restore.sh backups/owyx_backup_20260302_030000.sql.gz

set -e

# Конфигурация
CONTAINER_NAME="owyx-postgres"
DB_NAME="owyx_db"
DB_USER="owyx_user"

# Цвета
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🔄 OWYX Database Restore"
echo "===================================="
echo ""

# Проверка аргумента
if [ -z "$1" ]; then
    echo -e "${RED}Ошибка: Укажите файл бэкапа${NC}"
    echo "Использование: ./restore.sh backups/owyx_backup_20260302_030000.sql.gz"
    echo ""
    echo "Доступные бэкапы:"
    ls -lh backups/owyx_backup_*.sql.gz 2>/dev/null || echo "Нет бэкапов"
    exit 1
fi

BACKUP_FILE="$1"

# Проверка существования файла
if [ ! -f "$BACKUP_FILE" ]; then
    echo -e "${RED}Ошибка: Файл $BACKUP_FILE не найден!${NC}"
    exit 1
fi

# Проверка что контейнер запущен
if ! docker ps | grep -q "$CONTAINER_NAME"; then
    echo -e "${RED}Ошибка: Контейнер $CONTAINER_NAME не запущен!${NC}"
    exit 1
fi

# Предупреждение
echo -e "${YELLOW}⚠️  ВНИМАНИЕ: Это действие перезапишет текущую базу данных!${NC}"
read -p "Продолжить? (yes/no): " -r
if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
    echo "Отменено"
    exit 0
fi

# Создание временного бэкапа текущей БД
echo "Создание бэкапа текущей БД перед восстановлением..."
TEMP_BACKUP="backups/before_restore_$(date +%Y%m%d_%H%M%S).sql"
docker exec "$CONTAINER_NAME" pg_dump -U "$DB_USER" "$DB_NAME" > "$TEMP_BACKUP"
gzip "$TEMP_BACKUP"
echo -e "${GREEN}✓ Текущая БД сохранена: ${TEMP_BACKUP}.gz${NC}"

# Остановка Directus для безопасности
echo "Остановка Directus..."
docker-compose stop directus

# Распаковка если сжато
if [[ "$BACKUP_FILE" == *.gz ]]; then
    echo "Распаковка бэкапа..."
    UNZIPPED_FILE="${BACKUP_FILE%.gz}"
    gunzip -c "$BACKUP_FILE" > "$UNZIPPED_FILE"
    RESTORE_FILE="$UNZIPPED_FILE"
    CLEANUP_FILE="$UNZIPPED_FILE"
else
    RESTORE_FILE="$BACKUP_FILE"
    CLEANUP_FILE=""
fi

# Восстановление
echo "Восстановление базы данных..."

# Удаление старой БД и создание новой
docker exec "$CONTAINER_NAME" psql -U "$DB_USER" -d postgres -c "DROP DATABASE IF EXISTS ${DB_NAME};"
docker exec "$CONTAINER_NAME" psql -U "$DB_USER" -d postgres -c "CREATE DATABASE ${DB_NAME};"

# Импорт данных
docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" < "$RESTORE_FILE"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ База данных восстановлена!${NC}"
    
    # Удаление временного файла
    if [ -n "$CLEANUP_FILE" ]; then
        rm -f "$CLEANUP_FILE"
    fi
    
    # Запуск Directus
    echo "Запуск Directus..."
    docker-compose start directus
    
    echo ""
    echo "===================================="
    echo -e "${GREEN}✅ Восстановление завершено!${NC}"
    echo "===================================="
    echo ""
    echo "Проверьте работу: http://localhost:8055"
else
    echo -e "${RED}Ошибка при восстановлении!${NC}"
    echo "Восстановление из временного бэкапа..."
    
    # Попытка восстановить из временного бэкапа
    gunzip -c "${TEMP_BACKUP}.gz" | docker exec -i "$CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME"
    
    docker-compose start directus
    exit 1
fi
