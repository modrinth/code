"use client";

import { useEffect, useRef } from "react";

type Particle = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
  a: number;
  layer: 1 | 2 | 3;
};

/**
 * Deep-space particle field (no spiderweb links — anti AI-slop).
 * Three depth layers, O(N). Respects prefers-reduced-motion + Page Visibility.
 */
export default function SpaceParticles() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    if (reduceMotion) {
      canvas.style.display = "none";
      return;
    }

    const ctx = canvas.getContext("2d", { alpha: true });
    if (!ctx) return;

    let raf = 0;
    let running = true;
    let w = 0;
    let h = 0;
    let dpr = 1;
    let particles: Particle[] = [];

    function countForViewport() {
      const area = w * h;
      if (area < 500_000) return 48;
      if (area < 1_200_000) return 72;
      return 96;
    }

    function spawn(n: number) {
      particles = Array.from({ length: n }, (_, i) => {
        const layer = (i % 3 === 0 ? 3 : i % 3 === 1 ? 2 : 1) as 1 | 2 | 3;
        const speed = layer === 1 ? 0.08 : layer === 2 ? 0.14 : 0.2;
        return {
          x: Math.random() * w,
          y: Math.random() * h,
          vx: (Math.random() - 0.5) * speed,
          vy: (Math.random() - 0.5) * speed * 0.85 - 0.03,
          r: layer === 1 ? Math.random() * 0.7 + 0.25 : layer === 2 ? Math.random() * 1.1 + 0.4 : Math.random() * 1.8 + 0.7,
          a: layer === 1 ? Math.random() * 0.25 + 0.12 : layer === 2 ? Math.random() * 0.35 + 0.18 : Math.random() * 0.5 + 0.25,
          layer,
        };
      });
    }

    function resize() {
      dpr = Math.min(window.devicePixelRatio || 1, 2);
      w = window.innerWidth;
      h = window.innerHeight;
      canvas!.width = Math.floor(w * dpr);
      canvas!.height = Math.floor(h * dpr);
      canvas!.style.width = `${w}px`;
      canvas!.style.height = `${h}px`;
      ctx!.setTransform(dpr, 0, 0, dpr, 0, 0);
      spawn(countForViewport());
    }

    function tick() {
      if (!running || !ctx) return;
      ctx.clearRect(0, 0, w, h);

      for (const p of particles) {
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < -20) p.x = w + 20;
        if (p.x > w + 20) p.x = -20;
        if (p.y < -20) p.y = h + 20;
        if (p.y > h + 20) p.y = -20;

        if (p.layer === 3) {
          const g = ctx.createRadialGradient(p.x, p.y, 0, p.x, p.y, p.r * 4);
          g.addColorStop(0, `rgba(0, 229, 255, ${p.a * 0.35})`);
          g.addColorStop(1, "rgba(0, 229, 255, 0)");
          ctx.fillStyle = g;
          ctx.beginPath();
          ctx.arc(p.x, p.y, p.r * 4, 0, Math.PI * 2);
          ctx.fill();
        }

        ctx.beginPath();
        ctx.fillStyle =
          p.layer === 1
            ? `rgba(220, 230, 240, ${p.a})`
            : `rgba(0, 229, 255, ${p.a})`;
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fill();
      }

      raf = window.requestAnimationFrame(tick);
    }

    function onVisibility() {
      running = document.visibilityState === "visible";
      if (running) raf = window.requestAnimationFrame(tick);
      else window.cancelAnimationFrame(raf);
    }

    resize();
    raf = window.requestAnimationFrame(tick);
    window.addEventListener("resize", resize);
    document.addEventListener("visibilitychange", onVisibility);

    return () => {
      running = false;
      window.cancelAnimationFrame(raf);
      window.removeEventListener("resize", resize);
      document.removeEventListener("visibilitychange", onVisibility);
    };
  }, []);

  return <canvas ref={canvasRef} className="owyx-particles" aria-hidden="true" />;
}
