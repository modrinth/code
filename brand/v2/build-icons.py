"""
Generate Owyx icon variants.
Output: 3 colors × 3 styles (gradient / solid / outline) + 3 favicons = 12 SVGs.
"""
from pathlib import Path

OUT = Path(__file__).parent

# 6 facet paths meeting at center (32,32)
# Vertex coords: T(32,4) UR(56,18) LR(56,46) B(32,60) LL(8,46) UL(8,18) C(32,32)
FACETS = [
    # (path_d, fill_opacity_for_solid_style)
    ("M32 4 L56 18 L32 32 Z",  0.95),
    ("M8 18 L32 4 L32 32 Z",   0.75),
    ("M56 18 L56 46 L32 32 Z", 0.55),
    ("M8 46 L8 18 L32 32 Z",   0.42),
    ("M56 46 L32 60 L32 32 Z", 0.30),
    ("M32 60 L8 46 L32 32 Z",  0.18),
]

HEX_POINTS = "32,4 56,18 56,46 32,60 8,46 8,18"

# Internal facet lines (for outline style)
INTERNAL_LINES = "M32 4 L32 32 M56 18 L32 32 M56 46 L32 32 M32 60 L32 32 M8 46 L32 32 M8 18 L32 32"


# ===== COLOR PALETTES (per icon variant) =====
# (palette_name, gradient_stops, solid_color, outline_color, core_glow_color, center_dot_color)
PALETTES = {
    "cyan": {
        "gradient_stops": [("0%", "#9ff7ff"), ("55%", "#00e5ff"), ("100%", "#0b7a96")],
        "edge_stops":     [("0%", "#7df9ff"), ("100%", "#00b8d4")],
        "solid": "#00e5ff",
        "outline": "#00e5ff",
        "core": [("0%", "#ffffff", 0.85), ("60%", "#7df9ff", 0.25), ("100%", "#00e5ff", 0)],
        "center_dot": "#bff8ff",
        "core_visible": True,
    },
    "ink": {
        "gradient_stops": [("0%", "#475569"), ("50%", "#0e0e14"), ("100%", "#020617")],
        "edge_stops":     [("0%", "#334155"), ("100%", "#020617")],
        "solid": "#0e0e14",
        "outline": "#0e0e14",
        "core": [("0%", "#94a3b8", 0.55), ("60%", "#475569", 0.18), ("100%", "#0e0e14", 0)],
        "center_dot": "#cbd5e1",
        "core_visible": True,
    },
    "slate": {
        "gradient_stops": [("0%", "#cbd5e1"), ("50%", "#64748b"), ("100%", "#334155")],
        "edge_stops":     [("0%", "#94a3b8"), ("100%", "#475569")],
        "solid": "#64748b",
        "outline": "#64748b",
        "core": [("0%", "#ffffff", 0.65), ("60%", "#cbd5e1", 0.20), ("100%", "#64748b", 0)],
        "center_dot": "#f1f5f9",
        "core_visible": True,
    },
}


def gradient_xml(grad_id, stops, x1="0", y1="0", x2="0", y2="1"):
    parts = "".join(f'<stop offset="{o}" stop-color="{c}"/>' if opacity is None
                    else f'<stop offset="{o}" stop-color="{c}" stop-opacity="{opacity}"/>'
                    for o, c, *opacity in stops)
    return f'<linearGradient id="{grad_id}" x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}">{parts}</linearGradient>'


def radial_xml(grad_id, stops):
    parts = "".join(f'<stop offset="{o}" stop-color="{c}" stop-opacity="{opacity}"/>'
                    for o, c, opacity in stops)
    return f'<radialGradient id="{grad_id}" cx="50%" cy="32%" r="38%">{parts}</radialGradient>'


# ===== STYLE RENDERERS =====

def render_gradient(p, color_name):
    """Full color gradient crystal — the default style."""
    defs = (
        '<defs>'
        + gradient_xml(f"owyx-{color_name}-fill", p["gradient_stops"])
        + gradient_xml(f"owyx-{color_name}-edge", p["edge_stops"])
        + (radial_xml(f"owyx-{color_name}-core", p["core"]) if p["core_visible"] else "")
        + '</defs>'
    )
    facets = '<g stroke="url(#owyx-{c}-edge)" stroke-width="0.75" stroke-linejoin="round">'.format(c=color_name)
    for d, op in FACETS:
        facets += f'<path d="{d}" fill="url(#owyx-{color_name}-fill)" fill-opacity="{op}"/>'
    facets += '</g>'
    core = (f'<circle cx="32" cy="26" r="9" fill="url(#owyx-{color_name}-core)"/>'
            if p["core_visible"] else "")
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" role="img" aria-label="Owyx icon — {color_name} gradient">
  <title>Owyx icon — {color_name} gradient</title>
  {defs}
  {facets}
  <polygon points="{HEX_POINTS}" fill="none" stroke="{p["outline"]}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>
  {core}
  <circle cx="32" cy="32" r="1.4" fill="{p["center_dot"]}"/>
</svg>
'''


def render_solid(p, color_name):
    """Single color crystal — facets with varying opacity for depth, no gradient."""
    facets = ""
    for d, op in FACETS:
        facets += f'<path d="{d}" fill="{p["solid"]}" fill-opacity="{op}"/>'
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" role="img" aria-label="Owyx icon — {color_name} solid">
  <title>Owyx icon — {color_name} solid</title>
  {facets}
  <polygon points="{HEX_POINTS}" fill="none" stroke="{p["outline"]}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>
  <circle cx="32" cy="32" r="1.6" fill="{p["solid"]}"/>
  <circle cx="32" cy="32" r="0.8" fill="{p["center_dot"]}"/>
</svg>
'''


def render_outline(p, color_name):
    """Just the hex outline + internal lines + center dot. Wireframe style."""
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64" fill="none" role="img" aria-label="Owyx icon — {color_name} outline">
  <title>Owyx icon — {color_name} outline</title>
  <polygon points="{HEX_POINTS}" fill="none" stroke="{p["outline"]}" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>
  <path d="{INTERNAL_LINES}" stroke="{p["outline"]}" stroke-width="1.5" stroke-linecap="round" opacity="0.55"/>
  <circle cx="32" cy="32" r="2" fill="{p["outline"]}"/>
</svg>
'''


def render_favicon(p, color_name):
    """Simplified for 16-24px: solid fill + minimal accents (no internal facets)."""
    # For favicon at 16px the internal facets become noise, so we just use 2-3 flat planes.
    return f'''<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" fill="none" role="img" aria-label="Owyx icon — {color_name} favicon">
  <title>Owyx icon — {color_name} favicon (16-24px)</title>
  <defs>{gradient_xml(f"owyx-{color_name}-fav", p["gradient_stops"])}</defs>
  <polygon points="16,2 28,9 28,23 16,30 4,23 4,9" fill="url(#owyx-{color_name}-fav)"/>
  <polygon points="16,2 28,9 16,16" fill="{p["center_dot"]}" fill-opacity="0.7"/>
  <polygon points="16,30 4,23 16,16" fill="{p["outline"]}" fill-opacity="0.35"/>
  <circle cx="16" cy="16" r="1.5" fill="#ffffff"/>
</svg>
'''


def main():
    # Remove any existing icon variants (keep build.py, preview, README)
    keep = {"build.py", "build-icons.py", "preview.html", "README.md",
            "preview-full.png", "preview-hero.png", "font-explore.html", "font-explore.png"}
    for old in OUT.glob("owyx-icon-*.svg"):
        if old.name not in keep:
            old.unlink()
            print(f"removed old: {old.name}")

    # Generate 9 full-detail variants
    style_funcs = {"gradient": render_gradient, "solid": render_solid, "outline": render_outline}
    for color, palette in PALETTES.items():
        for style, fn in style_funcs.items():
            fname = f"owyx-icon-{color}-{style}.svg"
            (OUT / fname).write_text(fn(palette, color))
            print(f"wrote: {fname}")

    # Generate 3 favicon variants
    for color, palette in PALETTES.items():
        fname = f"owyx-icon-favicon-{color}.svg"
        (OUT / fname).write_text(render_favicon(palette, color))
        print(f"wrote: {fname}")

    print("\nDone.")


if __name__ == "__main__":
    main()
