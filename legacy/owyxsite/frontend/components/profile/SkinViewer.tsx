"use client";

import { useEffect, useRef, useState } from "react";
import SkinPreview from "./SkinPreview";

// Skin preview with a 2D | 3D toggle.
//  - 2D: dependency-free CSS-sprite front render (SkinPreview).
//  - 3D: skinview3d (WebGL), lazy-loaded on demand.
// Shows a local placeholder when the user has no uploaded skin, and gracefully
// drops to 2D if WebGL/skinview3d is unavailable.

export default function SkinViewer({
  skinUrl,
  model = "classic",
  initialMode = "2d",
}: {
  skinUrl: string | null;
  model?: "classic" | "slim";
  initialMode?: "2d" | "3d";
}) {
  const [mode, setMode] = useState<"2d" | "3d">(initialMode);
  const effectiveMode = skinUrl ? mode : "2d";

  return (
    <div className="flex flex-col items-center gap-3">
      <div className="rounded-xl border border-line bg-[#0e0e14] p-2 min-h-[180px] flex items-center justify-center">
        {effectiveMode === "2d" ? (
          <SkinPreview skinUrl={skinUrl} model={model} scale={9} />
        ) : (
          <Skin3D src={skinUrl!} model={model} />
        )}
      </div>
      <div
        className="inline-flex rounded-lg border border-line overflow-hidden text-xs"
        role="tablist"
        aria-label="Режим превью скина"
      >
        {(["2d", "3d"] as const).map((m) => (
          <button
            key={m}
            type="button"
            role="tab"
            aria-selected={effectiveMode === m}
            disabled={!skinUrl && m === "3d"}
            onClick={() => setMode(m)}
            className={`px-3 py-1.5 transition-colors ${
              effectiveMode === m ? "bg-accent text-[#041018] font-semibold" : "text-muted hover:text-accent"
            }`}
          >
            {m.toUpperCase()}
          </button>
        ))}
      </div>
    </div>
  );
}

function Skin3D({ src, model }: { src: string; model: "classic" | "slim" }) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const [error, setError] = useState(false);
  const W = 160;
  const H = 288;

  useEffect(() => {
    let disposed = false;
    let viewer: { dispose: () => void } | null = null;

    (async () => {
      try {
        const { SkinViewer } = await import("skinview3d");
        if (disposed || !canvasRef.current) return;
        const v = new SkinViewer({ canvas: canvasRef.current, width: W, height: H });
        await v.loadSkin(src, { model: model === "slim" ? "slim" : "default" });
        v.autoRotate = !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
        v.zoom = 0.9;
        viewer = v;
      } catch (e) {
        console.warn("skinview3d failed:", e);
        if (!disposed) setError(true);
      }
    })();

    return () => {
      disposed = true;
      try { viewer?.dispose(); } catch { /* ignore */ }
    };
  }, [src, model]);

  if (error) {
    return <SkinPreview skinUrl={src} model={model} scale={9} />;
  }

  return <canvas ref={canvasRef} width={W} height={H} style={{ width: W, height: H }} aria-label="3D-превью скина" />;
}
