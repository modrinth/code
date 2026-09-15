# Owyx

Кастомный лаунчер Minecraft под **свои** сервера и сборки: ваниль и моды.

Один `.exe` для игроков. Не установщик Windows — сам лаунчер создаёт нужные папки, качает сборку и запускает игру.

> Полный план, этапы и бэклог: **[PLAN.md](../PLAN.md)** (корень монорепо)

---

## Идея

1. Заливаешь сборку(и) на свой update-сервер.
2. Друзья скачивают только лаунчер.
3. Вводят ник → выбирают сборку → «Скачать» → «Играть».
4. Позже: связка с плагином на сервере (автологин, плюшки) для публичных режимов.

---

## Стек

| Слой | Технология |
|------|------------|
| Backend | Rust |
| Desktop shell | Tauri 2 |
| UI | TypeScript + Vite |
| Секреты / URL | `.env` (не в git) |

Старый C# WinForms-прототип **не используется**. Фрагменты идей лежат в [`legacy/csharp/`](./legacy/csharp/) только как справочник.

---

## Требования для разработки

- Windows 10/11
- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) 20+
- Microsoft Edge WebView2 (обычно уже есть на Windows)

```powershell
# проверка
rustc --version
cargo --version
node --version
```

---

## Быстрый старт (когда каркас на месте)

```powershell
copy .env.example .env
# OWYX_API_BASE_URL=http://127.0.0.1:3001  (сайт backend; пусто = prod)

cd apps/launcher
npm install
npm run tauri dev
```

Вкладка **Серверы** читает `GET /api/launcher/v1/servers` с этого base.
Без native окна (Linux/cloud): `OWYX_API_BASE_URL=http://127.0.0.1:3001 npm run dev`
→ Vite `:1420`, каталог через proxy `/proxy/owyx`. Play / unzip — только Tauri.

Проверка / smoke (по умолчанию — быстрее релиза):

```powershell
npm run check
npm run check:rust
npm run build:smoke
# -> src-tauri/target/debug/owyx.exe
```

Ship (только для финального `.exe`):

```powershell
npm run build:release
# -> src-tauri/target/release/Owyx.exe → copy Launcher/Owyx.exe
```

Подробности: [`BUILD.md`](./BUILD.md).

---

## Конфигурация

| Файл | Назначение |
|------|------------|
| `.env` | Локальные URL и секреты (**не коммитить**) |
| `.env.example` | Шаблон для разработчиков |
| `%USERPROFILE%/owyx/` | Данные игрока: конфиг, инстансы, логи |

---

## Структура репозитория

```text
PLAN.md                 # указатель → корневой ../PLAN.md
README.md               # этот файл
.env.example
apps/launcher/          # Tauri + Rust + UI
legacy/csharp/          # справочный код старого прототипа
```

---

## Лицензия

См. [LICENSE.txt](./LICENSE.txt).
