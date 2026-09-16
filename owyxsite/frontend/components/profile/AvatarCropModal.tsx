"use client";

import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

export type AvatarCropData = {
  scale: number;
  rotation: number;
  flipX: number;
  offsetX: number;
  offsetY: number;
  cropSize: number;
};

type Props = {
  file: File;
  onCancel: () => void;
  onConfirm: (crop: AvatarCropData) => void;
};

/** Simple zoom / pan / rotate cropper matching backend cropData shape. */
export default function AvatarCropModal({ file, onCancel, onConfirm }: Props) {
  const [url, setUrl] = useState<string | null>(null);
  const [scale, setScale] = useState(1);
  const [rotation, setRotation] = useState(0);
  const [offsetX, setOffsetX] = useState(0);
  const [offsetY, setOffsetY] = useState(0);
  const drag = useRef<{ x: number; y: number; ox: number; oy: number } | null>(null);

  useEffect(() => {
    const objectUrl = URL.createObjectURL(file);
    setUrl(objectUrl);
    return () => URL.revokeObjectURL(objectUrl);
  }, [file]);

  function onPointerDown(e: React.PointerEvent) {
    (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
    drag.current = { x: e.clientX, y: e.clientY, ox: offsetX, oy: offsetY };
  }
  function onPointerMove(e: React.PointerEvent) {
    if (!drag.current) return;
    setOffsetX(drag.current.ox + (e.clientX - drag.current.x));
    setOffsetY(drag.current.oy + (e.clientY - drag.current.y));
  }
  function onPointerUp() {
    drag.current = null;
  }

  if (typeof document === "undefined") return null;

  return createPortal(
    <div
      className="fixed inset-0 z-[70] flex items-center justify-center p-4 bg-black/70"
      onClick={onCancel}
      role="presentation"
    >
      <div
        className="w-full max-w-md rounded-2xl border border-line bg-panel p-5 shadow-xl"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-modal="true"
        aria-labelledby="avatar-crop-title"
      >
        <h3 id="avatar-crop-title" className="font-display text-lg font-bold tracking-tight">
          Обрезка аватара
        </h3>
        <p className="mt-1 text-sm text-muted">Масштаб, сдвиг и поворот. Квадрат 256×256 уйдёт на сервер.</p>

        <div
          className="relative mx-auto mt-4 h-64 w-64 overflow-hidden rounded-2xl border border-line bg-panel-2 touch-none"
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        >
          {url && (
            // eslint-disable-next-line @next/next/no-img-element
            <img
              src={url}
              alt=""
              draggable={false}
              className="absolute left-1/2 top-1/2 max-w-none select-none pointer-events-none"
              style={{
                transform: `translate(calc(-50% + ${offsetX}px), calc(-50% + ${offsetY}px)) rotate(${rotation}deg) scale(${scale})`,
              }}
            />
          )}
          <div className="pointer-events-none absolute inset-0 rounded-2xl ring-2 ring-accent/70" />
        </div>

        <div className="mt-4 space-y-3">
          <label className="block text-xs text-muted">
            Масштаб {scale.toFixed(2)}
            <input
              type="range"
              min={0.5}
              max={3}
              step={0.01}
              value={scale}
              onChange={(e) => setScale(Number(e.target.value))}
              className="mt-1 w-full"
            />
          </label>
          <label className="block text-xs text-muted">
            Поворот {rotation}°
            <input
              type="range"
              min={-180}
              max={180}
              step={1}
              value={rotation}
              onChange={(e) => setRotation(Number(e.target.value))}
              className="mt-1 w-full"
            />
          </label>
        </div>

        <div className="mt-5 flex flex-wrap gap-2">
          <button
            type="button"
            className="btn btn-primary"
            onClick={() =>
              onConfirm({
                scale,
                rotation,
                flipX: 1,
                offsetX,
                offsetY,
                cropSize: 256,
              })
            }
          >
            Сохранить
          </button>
          <button type="button" className="btn btn-ghost" onClick={onCancel}>
            Отмена
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
