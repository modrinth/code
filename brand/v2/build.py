"""
Generate Owyx v2 lockup variants.
Output: 3 colors × 2 styles (solid/gradient) × 3 sizes = 18 lockup SVGs + 6 wordmarks.
"""
from pathlib import Path

OUT = Path(__file__).parent

# ===== ICON COMPONENT =====
# Pointy-top hex crystal, viewBox 64x64, center (32,32), radius 28
# 6 facets meeting at center, top-to-bottom gradient

# Just the gradient definitions, no <defs> wrapper (caller wraps in <defs>)
ICON_GRADIENTS = """<linearGradient id="owyx-ic-fill" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#9ff7ff"/>
      <stop offset="55%" stop-color="#00e5ff"/>
      <stop offset="100%" stop-color="#0b7a96"/>
    </linearGradient>
    <radialGradient id="owyx-ic-core" cx="50%" cy="32%" r="38%">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.85"/>
      <stop offset="60%" stop-color="#7df9ff" stop-opacity="0.25"/>
      <stop offset="100%" stop-color="#00e5ff" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="owyx-ic-edge" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#7df9ff"/>
      <stop offset="100%" stop-color="#00b8d4"/>
    </linearGradient>"""

ICON_BODY = """<g stroke="url(#owyx-ic-edge)" stroke-width="0.75" stroke-linejoin="round">
    <path d="M32 4 L56 18 L32 32 Z"  fill="url(#owyx-ic-fill)" fill-opacity="0.95"/>
    <path d="M8 18 L32 4 L32 32 Z"   fill="url(#owyx-ic-fill)" fill-opacity="0.75"/>
    <path d="M56 18 L56 46 L32 32 Z" fill="url(#owyx-ic-fill)" fill-opacity="0.55"/>
    <path d="M8 46 L8 18 L32 32 Z"   fill="url(#owyx-ic-fill)" fill-opacity="0.42"/>
    <path d="M56 46 L32 60 L32 32 Z" fill="url(#owyx-ic-fill)" fill-opacity="0.30"/>
    <path d="M32 60 L8 46 L32 32 Z"  fill="url(#owyx-ic-fill)" fill-opacity="0.18"/>
  </g>
  <polygon points="32,4 56,18 56,46 32,60 8,46 8,18"
           fill="none" stroke="#00e5ff" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>
  <circle cx="32" cy="26" r="9" fill="url(#owyx-ic-core)"/>
  <circle cx="32" cy="32" r="1.4" fill="#bff8ff"/>"""

# Mono icon (for ink/slate variants where the colorful icon might be too much)
ICON_BODY_MONO = """<g transform="translate(8,8)">
    <polygon points="32,4 56,18 56,46 32,60 8,46 8,18"
             fill="none" stroke="{ICON_STROKE}" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>
    <path d="M32 4 L32 32 M56 18 L32 32 M56 46 L32 32 M32 60 L32 32 M8 46 L32 32 M8 18 L32 32"
          stroke="{ICON_STROKE}" stroke-width="1.5" stroke-linecap="round" opacity="0.55"/>
    <circle cx="32" cy="32" r="2" fill="{ICON_STROKE}"/>
  </g>"""


def font_stack():
    return "'Sora','Space Grotesk','Outfit','Geist',system-ui,-apple-system,Segoe UI,Roboto,sans-serif"


# ===== COLOR PALETTES =====
# (variant_name, solid_fill, gradient_stops, mono_icon_stroke)
PALETTES = {
    "cyan": {
        "solid": "#00e5ff",
        "gradient": [("0%", "#bff8ff"), ("45%", "#00e5ff"), ("100%", "#0b7a96")],
        "gradient_id": "owyx-grad-cyan",
        "icon_kind": "color",   # use the colorful icon
        "desc": "Brand cyan — on dark backgrounds",
    },
    "ink": {
        "solid": "#0e0e14",
        "gradient": [("0%", "#475569"), ("50%", "#0e0e14"), ("100%", "#020617")],
        "gradient_id": "owyx-grad-ink",
        "icon_kind": "mono",
        "icon_stroke": "#0e0e14",
        "desc": "Ink — on light backgrounds (печать, документы)",
    },
    "slate": {
        "solid": "#64748b",
        "gradient": [("0%", "#cbd5e1"), ("50%", "#64748b"), ("100%", "#334155")],
        "gradient_id": "owyx-grad-slate",
        "icon_kind": "mono",
        "icon_stroke": "#64748b",
        "desc": "Neutral slate — работает на светлом и тёмном",
    },
}


def gradient_def(palette, grad_style):
    if grad_style == "solid":
        return ""
    stops = "".join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in palette["gradient"])
    return f'<linearGradient id="{palette["gradient_id"]}" x1="0" y1="0" x2="0" y2="1">{stops}</linearGradient>'


def text_fill(palette, grad_style):
    if grad_style == "solid":
        return palette["solid"]
    return f'url(#{palette["gradient_id"]})'


# ===== SIZES =====
# (suffix, canvas_w, canvas_h, icon_translate_x, icon_translate_y, icon_scale, font_size, baseline_y, gap)
SIZES = {
    "lg": dict(canvas=(480, 120), icon_t=(16, 16), icon_scale=1.0, font=68, baseline=84, gap=24, label="hero / marketing"),
    "md": dict(canvas=(320, 80),  icon_t=(8, 8),   icon_scale=1.0, font=56, baseline=56, gap=17, label="primary header / footer"),
    "sm": dict(canvas=(200, 50),  icon_t=(4, 4),   icon_scale=0.625, font=34, baseline=36, gap=10, label="compact nav / inline"),
}


def render_icon_at(palette, x, y, scale=1.0):
    """Render the icon at (x,y) with given scale. Just the body; defs are handled at SVG root."""
    if palette["icon_kind"] == "color":
        body = ICON_BODY
        return f'<g transform="translate({x},{y}) scale({scale})">{body}</g>'
    else:
        body = ICON_BODY_MONO.format(ICON_STROKE=palette["icon_stroke"])
        return f'<g transform="translate({x},{y}) scale({scale})">{body}</g>'


def wordmark_only_svg(palette, grad_style):
    """Standalone wordmark (no icon)."""
    fill = text_fill(palette, grad_style)
    grad = gradient_def(palette, grad_style)
    defs = f"<defs>{grad}</defs>" if grad else ""
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 80" fill="none" role="img" aria-label="owyx — {palette_name(palette, grad_style)}">
  <title>owyx — {palette["desc"]}</title>
  {defs}
  <text x="120" y="58"
        text-anchor="middle"
        fill="{fill}"
        font-family="{font_stack()}"
        font-weight="600"
        font-size="58"
        letter-spacing="-3"
        style="font-feature-settings: 'ss01' on;">owyx</text>
</svg>
'''


def palette_name(palette, grad_style):
    name = "color"
    for k, v in PALETTES.items():
        if v is palette:
            name = k
            break
    return f"{name} {'gradient' if grad_style == 'gradient' else 'solid'}"


def lockup_svg(palette, grad_style, size_key):
    """Horizontal lockup (icon + wordmark)."""
    s = SIZES[size_key]
    w, h = s["canvas"]
    icon_x = s["icon_t"][0]
    icon_y = s["icon_t"][1]
    icon_scale = s["icon_scale"]
    icon_render_w = 64 * icon_scale
    text_x = icon_x + icon_render_w + s["gap"]
    text_y = s["baseline"]
    font_size = s["font"]

    fill = text_fill(palette, grad_style)
    grad = gradient_def(palette, grad_style)
    icon_block = render_icon_at(palette, icon_x, icon_y, icon_scale)

    title = f'Owyx — {palette["desc"]} · {size_key.upper()}'

    defs = ""
    defs_parts = []
    if palette["icon_kind"] == "color":
        defs_parts.append(ICON_GRADIENTS)
    if grad_style == "gradient":
        defs_parts.append(f'<linearGradient id="{palette["gradient_id"]}" x1="0" y1="0" x2="0" y2="1">' +
                          "".join(f'<stop offset="{o}" stop-color="{c}"/>' for o, c in palette["gradient"]) +
                          '</linearGradient>')
    if defs_parts:
        defs = "<defs>" + "".join(defs_parts) + "</defs>"

    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}" fill="none" role="img" aria-label="Owyx">
  <title>{title}</title>
  {defs}
  {icon_block}
  <text x="{text_x}" y="{text_y}"
        fill="{fill}"
        font-family="{font_stack()}"
        font-weight="600"
        font-size="{font_size}"
        letter-spacing="-3"
        style="font-feature-settings: 'ss01' on;">owyx</text>
</svg>
'''


def main():
    # Clear out any old generated files in this folder (keep icon/favicon/preview)
    keep_names = {"owyx-icon.svg", "owyx-icon-favicon.svg", "build.py"}
    for old in OUT.glob("owyx-*.svg"):
        if old.name not in keep_names:
            old.unlink()
            print(f"removed old: {old.name}")

    # 1. Wordmarks (3 colors × 2 styles = 6 files)
    for color, palette in PALETTES.items():
        for grad_style in ["solid", "gradient"]:
            fname = f"owyx-wordmark-{color}.svg" if grad_style == "solid" else f"owyx-wordmark-{color}-gradient.svg"
            (OUT / fname).write_text(wordmark_only_svg(palette, grad_style))
            print(f"wrote: {fname}")

    # 2. Lockups (3 colors × 2 styles × 3 sizes = 18 files)
    for color, palette in PALETTES.items():
        for grad_style in ["solid", "gradient"]:
            for size_key in SIZES:
                suffix = "" if grad_style == "solid" else "-gradient"
                fname = f"owyx-logo-{color}-{size_key}{suffix}.svg"
                (OUT / fname).write_text(lockup_svg(palette, grad_style, size_key))
                print(f"wrote: {fname}")

    print("\nDone.")


if __name__ == "__main__":
    main()
