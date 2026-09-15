# Owyx — Brand Mark v2

Логотип проекта Owyx. v2: перешли с Space Grotesk на **Sora** (лучше гармонирует с геометрией кристалла), добавили **gradient-версии** и **3 цветовые схемы** для разных тем.

Соответствует токенам из [`DESIGN.md` §3](../../attachments/72845d3c79131929/DESIGN.md).

---

## Что нового в v2

| v1 | v2 |
|----|----|
| 1 шрифт (Space Grotesk) | **Sora 600** как primary (Space Grotesk в fallback) |
| 1 цвет (cyan + ink mono) | **3 цветовые схемы**: cyan / ink / slate |
| Только solid | **Solid + gradient** версии каждого варианта |
| 2 размера | **3 размера**: lg (480×120) / md (320×80) / sm (200×50) |
| 1 иконка | **12 иконок**: 3 цвета × {gradient, solid, outline} + 3 favicon |
| 8 SVG | **38 SVG** — все варианты готовы под задачу |

Иконка-кристалл сохранила геометрию, но теперь доступна во всех 3 цветовых схемах × 3 стилях.

---

## Файлы

```
v2/
├── owyx-icon-{cyan,ink,slate}-{gradient,solid,outline}.svg  # 9 иконок (3 цвета × 3 стиля)
├── owyx-icon-favicon-{cyan,ink,slate}.svg                   # 3 favicon (16-24px)
│
├── owyx-wordmark-{cyan,ink,slate}.svg           # 3 wordmark solid
├── owyx-wordmark-{cyan,ink,slate}-gradient.svg  # 3 wordmark gradient
│
├── owyx-logo-{cyan,ink,slate}-{lg,md,sm}.svg          # 9 lockup solid
├── owyx-logo-{cyan,ink,slate}-{lg,md,sm}-gradient.svg # 9 lockup gradient
│
├── build.py             # генератор wordmark + lockup
├── build-icons.py       # генератор иконок
├── preview.html         # превью всех вариантов
├── preview-full.png     # рендер превью
├── icons-preview.html   # отдельное превью иконок
├── icons-preview.png    # рендер
├── font-explore.html    # A/B сравнение шрифтов
└── README.md
```

**Итого:** 9 иконок + 3 favicon + 6 wordmark + 18 lockup = **38 SVG**.

---

## Шрифт

**Sora 600** lowercase, letter-spacing `-3`.

Font stack:
```
'Sora','Space Grotesk','Outfit','Geist',system-ui,-apple-system,
'Segoe UI',Roboto,sans-serif
```

Подключение — Google Fonts CDN:
```html
<link href="https://fonts.googleapis.com/css2?family=Sora:wght@500;600;700&display=swap" rel="stylesheet" />
```

Или локально через `@font-face` (для лаунчера без интернета):
```css
@font-face {
  font-family: 'Sora';
  src: url('/fonts/Sora-SemiBold.woff2') format('woff2');
  font-weight: 600;
  font-style: normal;
  font-display: swap;
}
```

---

## Иконки (12 вариантов)

3 цвета × 3 стиля + 3 favicon = 12 SVG.

| Стиль | Cyan | Ink | Slate | Описание |
|-------|------|-----|-------|----------|
| **Gradient** | `owyx-icon-cyan-gradient.svg` | `owyx-icon-ink-gradient.svg` | `owyx-icon-slate-gradient.svg` | Полноцветный кристалл, 6 фасет с градиентом, внутреннее свечение, accent-точка. Hero / splash / OG |
| **Solid** | `owyx-icon-cyan-solid.svg` | `owyx-icon-ink-solid.svg` | `owyx-icon-slate-solid.svg` | Один цвет с opacity-вариациями фасет для глубины. Header / footer / inline |
| **Outline** | `owyx-icon-cyan-outline.svg` | `owyx-icon-ink-outline.svg` | `owyx-icon-slate-outline.svg` | Только контур гекса + внутренние линии. Для line-icon контекстов |
| **Favicon** | `owyx-icon-favicon-cyan.svg` | `owyx-icon-favicon-ink.svg` | `owyx-icon-favicon-slate.svg` | Упрощённый для 16-24px (без внутренних фасет). Favicon, browser tab, dock |

Геометрия — та же, что v1: pointy-top гекс, viewBox 64×64 (32×32 для favicon), 6 фасет сходятся в центре. Цвета — из палитры DESIGN.md §3, без новых токенов.

**Когда какой стиль:**
- **Gradient** — на промо/hере, где хочется богатого 3D-ощущения
- **Solid** — основной рабочий вариант для UI (кнопки, badges, списки)
- **Outline** — для line-icon контекстов, где иконка должна быть тонкой
- **Favicon** — только для ≤24px (на большем выглядит слишком просто)

### 🟦 Cyan — на тёмном (primary)
| | Solid | Gradient |
|---|---|---|
| Текст | `#00e5ff` | `#bff8ff` → `#00e5ff` → `#0b7a96` |
| Иконка | Полноцветная (cyan facets) | Полноцветная |
| Фон | `--bg` `#050508`, `--panel` `#131319` | то же |

### ⬛ Ink — на светлом
| | Solid | Gradient |
|---|---|---|
| Текст | `#0e0e14` | `#475569` → `#0e0e14` → `#020617` |
| Иконка | Моно-stroke `#0e0e14` | то же |
| Фон | `#f3f4f6` (светлый) | то же |

### ◽ Slate — нейтральный (для обоих тем)
| | Solid | Gradient |
|---|---|---|
| Текст | `#64748b` (slate-500) | `#cbd5e1` → `#64748b` → `#334155` |
| Иконка | Моно-stroke `#64748b` | то же |
| Фон | Любой средне-серый / нейтральный | то же |

**Slate — для случаев, когда логотип должен "не кричать о себе":** документы, footer, watermark, "Powered by Owyx" в подвале партнёрского сайта.

---

## Размеры

| Suffix | Canvas | Wordmark | Icon scale | Где использовать |
|--------|--------|----------|------------|------------------|
| `-lg`  | 480×120 | 68px | 1.0× | Hero / marketing / OG-image |
| `-md`  | 320×80  | 56px | 1.0× | Header / footer / основной хедер сайта |
| `-sm`  | 200×50  | 34px | 0.625× | Compact nav / inline link / mobile |

---

## Маппинг файл → контекст

| Контекст | Файл |
|----------|------|
| Hero сайта (тёмный фон) | `owyx-logo-cyan-lg-gradient.svg` |
| Hero маркетинга (OG, splash) | `owyx-logo-cyan-lg-gradient.svg` |
| Хедер сайта (тёмная тема) | `owyx-logo-cyan-md.svg` |
| Футер сайта (тёмная тема) | `owyx-logo-cyan-sm.svg` |
| Печать, документы, светлые секции | `owyx-logo-ink-md.svg` или `-md-gradient` |
| Партнёрский watermark / footer | `owyx-logo-slate-sm.svg` или `-sm-gradient` |
| Inline-ссылка "owyx.com" | `owyx-wordmark-cyan.svg` (без иконки) |
| Splash лаунчера | `owyx-logo-cyan-lg-gradient.svg` |
| About-панель лаунчера | `owyx-logo-cyan-md.svg` |
| App icon лаунчера (большой) | `owyx-icon-cyan-gradient.svg` |
| App icon лаунчера (UI) | `owyx-icon-cyan-solid.svg` |
| Favicon 16 | `owyx-icon-favicon-cyan.svg` |
| Favicon 32+ | `owyx-icon-cyan-solid.svg` или `-gradient` |
| Favicon для печати (dark) | `owyx-icon-favicon-ink.svg` |
| Favicon для партнёров (нейтрал) | `owyx-icon-favicon-slate.svg` |

---

## Почему Sora, а не Space Grotesk

- Sora — более геометричный, "o" почти идеальный круг, "y" — ровный штрих
- Это лучше гармонирует с кристаллом (тоже геометрия)
- Space Grotesk чуть слябовый, "y" имеет выраженный curved descender
- Geist слишком нейтральный, теряется рядом с цветной иконкой
- Manrope, Outfit, DM Sans, Hanken — мимо по тем или иным параметрам

Sora при этом остаётся достаточно "тёплым" — не такой безжалостно-футуристичный, как, например, Sora Display в 800 weight. На 600 — баланс между технологичностью и читаемостью.

Space Grotesk оставлен в fallback-стеке — если Sora не подгрузится (offline, блокировка CDN), переход будет плавным.

---

## Регенерация

Все 38 SVG генерируются двумя скриптами:
- `build.py` — wordmark (6) + lockup (18) = 24 SVG
- `build-icons.py` — icon (9) + favicon (3) = 12 SVG

Чтобы добавить новый цвет / размер / шрифт — правится скрипт, не файлы руками:

```bash
cd v2
python3 build.py        # wordmark + lockup
python3 build-icons.py  # icons + favicons
```

Затем откройте `preview.html` для визуальной проверки.

---

## Лицензия

Использование внутри проекта Owyx (сайт / лаунчер / плагин). Не для ребрендинга третьими лицами без согласования.
