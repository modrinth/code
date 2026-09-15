#!/bin/bash

# ==============================================
# OWYX Database Backup Script
# ==============================================
# Автоматический бэкап PostgreSQL базы данных
#
# Использование:
# chmod +x backup.sh
# ./backup.sh
#
# Автоматизация (cron):
# crontab -e
# 0 3 * * * /path/to/owyx/backup.sh

set -e

# Конфигурация
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="owyx_backup_${TIMESTAMP}.sql"
CONTAINER_NAME="owyx-postgres"
DB_NAME="owyx_db"
DB_USER="owyx_user"

# Количество дней хранения бэкапов
RETENTION_DAYS=7

# Цвета
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🗄️  OWYX Database Backup"
echo "===================================="
echo ""

# Создание папки для бэкапов
mkdir -p "$BACKUP_DIR"

# Проверка что контейнер запущен
if ! docker ps | grep -q "$CONTAINER_NAME"; then
    echo -e "${RED}Ошибка: Контейнер $CONTAINER_NAME не запущен!${NC}"
    exit 1
fi

# Создание бэкапа
echo "Создание бэкапа базы данных $DB_NAME..."
docker exec "$CONTAINER_NAME" pg_dump -U "$DB_USER" "$DB_NAME" > "$BACKUP_DIR/$BACKUP_FILE"

if [ $? -eq 0 ]; then
    # Сжатие
    echo "Сжатие бэкапа..."
    gzip "$BACKUP_DIR/$BACKUP_FILE"
    BACKUP_FILE="${BACKUP_FILE}.gz"
    
    # Размер файла
    SIZE=$(du -h "$BACKUP_DIR/$BACKUP_FILE" | cut -f1)
    
    echo -e "${GREEN}✓ Бэкап создан: $BACKUP_FILE ($SIZE)${NC}"
    
    # Удаление старых бэкапов
    echo "Удаление бэкапов старше $RETENTION_DAYS дней..."
    find "$BACKUP_DIR" -name "owyx_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete
    
    # Количество бэкапов
    COUNT=$(ls -1 "$BACKUP_DIR"/owyx_backup_*.sql.gz 2>/dev/null | wc -l)
    echo -e "${GREEN}Всего бэкапов: $COUNT${NC}"
    
    # Опционально: загрузка в облако (раскомментируйте если настроен rclone)
    # echo "Загрузка в облако..."
    # rclone copy "$BACKUP_DIR/$BACKUP_FILE" remote:backups/owyx/
    
else
    echo -e "${RED}Ошибка при создании бэкапа!${NC}"
    exit 1
fi

echo ""
echo "===================================="
echo -e "${GREEN}✅ Бэкап завершен!${NC}"
echo "===================================="
