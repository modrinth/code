# Owyx Extras · дополнительные палитры

3 add-on color schemes для основного cyan-бренда Owyx. Используй как "дополнительные цвета" в своих дизайнах, если хочешь добавить вариативность.

## Структура

```
owyx-extras/
├── README.md                ← этот файл
├── extras-picker.html       ← визуальное сравнение трёх палитр (открыть в браузере)
│
├── synthwave/               ← cyberpunk duo (pink + cyan gradient)
├── onyx/                    ← luxury white-on-black (Apple vibe)
└── magma/                   ← aggressive red (nether)
```

В каждой подпапке — 6 SVG + README:
- `icon-{name}-gradient.svg` — полноцветный кристалл с градиентом и glow
- `icon-{name}-solid.svg` — один цвет с opacity-вариациями фасет
- `icon-{name}-outline.svg` — только контур гекса
- `favicon-{name}.svg` — упрощённый для 16-24px
- `wordmark-{name}.svg` — только текст "owyx"
- `logo-{name}-md.svg` — горизонтальный лок-ап (320×80)

## Hex коды

| Палитра | Accent | Gradient hi → lo |
|---------|--------|------------------|
| **Synthwave** | `#ec4899` | `#f9a8d4 → #ec4899 → #831843` |
| **Onyx** | `#f5f5f5` | `#ffffff → #f5f5f5 → #a3a3a3` |
| **Magma** | `#f87171` | `#fecaca → #f87171 → #7f1d1d` |

## Когда использовать

- **Synthwave** — для спец-страниц, "outrun"/cyberpunk вайба, нишевых промо
- **Onyx** — для печати, документов, монохромных вариантов, "премиум" контекста
- **Magma** — для агрессивных промо, in-game серверных событий, anti-hero моментов

## Использование в своих дизайнах

Drop-in: те же размеры и viewBox, что у основного бренда. Можно использовать icon/lockup напрямую или подменить CSS-переменные.

```html
<img src="extras/synthwave/icon-synthwave-gradient.svg" alt="" />
<img src="extras/onyx/logo-onyx-md.svg" alt="Owyx" />
```

Или в Figma/дизайне — просто переключи fill/stroke на соответствующий hex.
