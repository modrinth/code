"use client";

// Front-facing full-body preview built from a Minecraft 64x64 skin PNG using
// CSS sprite slicing — no WebGL, no external service, works with our own
// uploaded skins. Left arm/leg mirror the right slices so it renders for both
// 64x64 and legacy 64x32 skins.

type Part = {
  key: string;
  sx: number; sy: number; sw: number; sh: number; // source rect in skin px
  dx: number; dy: number;                          // dest top-left in figure px
  flip?: boolean;
  z?: number;
};

function bodyParts(model: "classic" | "slim"): Part[] {
  const armW = model === "slim" ? 3 : 4;
  return [
    // base layer
    { key: "head", sx: 8, sy: 8, sw: 8, sh: 8, dx: 4, dy: 0 },
    { key: "body", sx: 20, sy: 20, sw: 8, sh: 12, dx: 4, dy: 8 },
    { key: "armR", sx: 44, sy: 20, sw: armW, sh: 12, dx: 4 - armW, dy: 8 },
    { key: "armL", sx: 44, sy: 20, sw: armW, sh: 12, dx: 12, dy: 8, flip: true },
    { key: "legR", sx: 4, sy: 20, sw: 4, sh: 12, dx: 4, dy: 20 },
    { key: "legL", sx: 4, sy: 20, sw: 4, sh: 12, dx: 8, dy: 20, flip: true },
    // hat overlay
    { key: "hat", sx: 40, sy: 8, sw: 8, sh: 8, dx: 4, dy: 0, z: 2 },
  ];
}

export default function SkinPreview({
  skinUrl,
  model = "classic",
  scale = 9,
  flip = false,
}: {
  skinUrl: string | null;
  model?: "classic" | "slim";
  scale?: number;
  flip?: boolean;
}) {
  const W = 16 * scale;
  const H = 32 * scale;

  if (!skinUrl) {
    return (
      <div
        className="flex items-center justify-center rounded-xl border border-[#2a2b30] bg-[#0e0e14] text-[#9aa0a8] text-xs text-center"
        style={{ width: W, height: H }}
      >
        Стандартный
        <br />
        скин
      </div>
    );
  }

  const parts = bodyParts(model);
  return (
    <div
      className="rounded-xl border border-[#2a2b30] bg-[#0e0e14]"
      style={{ padding: scale }}
    >
      <div
        style={{
          position: "relative",
          width: W,
          height: H,
          transform: flip ? "scaleX(-1)" : undefined,
        }}
      >
        {parts.map((p) => (
          <div
            key={p.key}
            style={{
              position: "absolute",
              left: p.dx * scale,
              top: p.dy * scale,
              width: p.sw * scale,
              height: p.sh * scale,
              backgroundImage: `url(${skinUrl})`,
              backgroundRepeat: "no-repeat",
              backgroundSize: `${64 * scale}px ${64 * scale}px`,
              backgroundPosition: `${-p.sx * scale}px ${-p.sy * scale}px`,
              imageRendering: "pixelated",
              transform: p.flip ? "scaleX(-1)" : undefined,
              zIndex: p.z ?? 1,
            }}
          />
        ))}
      </div>
    </div>
  );
}
