# Исправление ошибки "does not contain plugin.yml"

## ❌ Проблема

```
java.lang.IllegalArgumentException: plugins\ChiwawaPlugin-2.5-SNAPSHOT.jar does not contain a paper-plugin.yml or plugin.yml!
Could not determine plugin type, cannot load a plugin from it!
```

## 🔍 Причина

Maven Shade Plugin неправильно упаковывал JAR, не включая файлы из `src/main/resources/`.

## ✅ Решение

### 1. Обновлен `pom.xml`

**Добавлено в `<build>`:**

```xml
<resources>
    <resource>
        <directory>src/main/resources</directory>
        <filtering>true</filtering>
        <includes>
            <include>**/*.yml</include>
            <include>**/*.yaml</include>
            <include>**/*.properties</include>
        </includes>
    </resource>
</resources>
```

**Добавлен maven-resources-plugin:**

```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-resources-plugin</artifactId>
    <version>3.3.1</version>
    <configuration>
        <encoding>UTF-8</encoding>
    </configuration>
</plugin>
```

**Обновлен maven-shade-plugin:**

```xml
<configuration>
    <createDependencyReducedPom>false</createDependencyReducedPom>
    <filters>
        <!-- Исключаем подписи -->
        <filter>
            <artifact>*:*</artifact>
            <excludes>
                <exclude>META-INF/*.SF</exclude>
                <exclude>META-INF/*.DSA</exclude>
                <exclude>META-INF/*.RSA</exclude>
            </excludes>
        </filter>
        <!-- Явно включаем всё из нашего проекта -->
        <filter>
            <artifact>${project.groupId}:${project.artifactId}</artifact>
            <includes>
                <include>**</include>
            </includes>
        </filter>
    </filters>
</configuration>
```

### 2. Удалено дублирование

Было дублирование секции `<resources>` в конце `<build>` - удалено.

## 🔧 Как собрать

```bash
mvn clean package
```

**Результат:** `target/ChiwawaPlugin-2.5-SNAPSHOT.jar`

## ✔️ Проверка JAR

Используйте скрипт для проверки:

```powershell
.\check-jar.ps1
```

Или вручную:

```bash
jar tf target/ChiwawaPlugin-2.5-SNAPSHOT.jar | findstr plugin.yml
```

**Ожидаемый вывод:**
```
plugin.yml
config.yml
org/ebluffy/chiwawa/...
```

## 📋 Структура ресурсов

```
src/main/resources/
  ├── plugin.yml     ✓ Главный файл плагина
  ├── config.yml     ✓ Конфигурация
  └── logs/          ✓ Директория для логов
```

## 🎯 Что изменилось

| Компонент | До | После |
|-----------|-----|-------|
| **Resources section** | В конце build | В начале build |
| **maven-resources-plugin** | Отсутствовал | Добавлен 3.3.1 |
| **shade filters** | Базовые | Явное включение проекта |
| **createDependencyReducedPom** | true (default) | false |

## 🚀 Следующие шаги

1. **Пересобрать плагин:**
   ```bash
   mvn clean package
   ```

2. **Проверить содержимое:**
   ```powershell
   .\check-jar.ps1
   ```

3. **Скопировать на сервер:**
   ```bash
   copy target\ChiwawaPlugin-2.5-SNAPSHOT.jar plugins\
   ```

4. **Перезапустить сервер**

## 💡 Объяснение

### Почему это происходит?

Maven собирает проект в несколько фаз:
1. `compile` - компиляция .java файлов
2. `resources` - копирование файлов из src/main/resources
3. `package` - создание JAR
4. `shade` - создание fat-JAR с зависимостями

Если `maven-resources-plugin` не настроен правильно, ресурсы могут не попасть в JAR на этапе shade.

### Решение

1. Явно указываем `<resources>` в начале `<build>`
2. Добавляем `maven-resources-plugin` для гарантии
3. В shade-plugin добавляем фильтр для включения всего из нашего проекта
4. Отключаем `createDependencyReducedPom` для упрощения

## ✅ Итог

После пересборки `plugin.yml` и `config.yml` будут включены в JAR, и плагин успешно загрузится на Paper 1.21.4!
