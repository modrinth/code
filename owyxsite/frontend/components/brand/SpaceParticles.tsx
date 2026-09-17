"use client";

import { useEffect, useRef } from "react";

type Particle = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
  a: number;
};

/**
 * Soft cyan particle field for Owyx space backdrop.
 * SSR-safe (canvas only after mount). Respects prefers-reduced-motion + Page Visibility.
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
      if (area < 500_000) return 28;
      if (area < 1_200_000) return 42;
      return 58;
    }

    function spawn(n: number) {
      particles = Array.from({ length: n }, () => ({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.22,
        vy: (Math.random() - 0.5) * 0.18 - 0.05,
        r: Math.random() * 1.6 + 0.5,
        a: Math.random() * 0.45 + 0.2,
      }));
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

      const linkDist = Math.min(140, w * 0.12);
      const linkDist2 = linkDist * linkDist;

      for (let i = 0; i < particles.length; i++) {
        const p = particles[i];
        p.x += p.vx;
        p.y += p.vy;
        if (p.x < -20) p.x = w + 20;
        if (p.x > w + 20) p.x = -20;
        if (p.y < -20) p.y = h + 20;
        if (p.y > h + 20) p.y = -20;

        for (let j = i + 1; j < particles.length; j++) {
          const q = particles[j];
          const dx = p.x - q.x;
          const dy = p.y - q.y;
          const d2 = dx * dx + dy * dy;
          if (d2 < linkDist2) {
            const t = 1 - Math.sqrt(d2) / linkDist;
            ctx.strokeStyle = `rgba(0, 229, 255, ${0.08 * t})`;
            ctx.lineWidth = 1;
            ctx.beginPath();
            ctx.moveTo(p.x, p.y);
            ctx.lineTo(q.x, q.y);
            ctx.stroke();
          }
        }

        ctx.beginPath();
        ctx.fillStyle = `rgba(0, 229, 255, ${p.a})`;
        ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
        ctx.fill();

        if (p.r > 1.4) {
          ctx.beginPath();
          ctx.fillStyle = `rgba(191, 248, 255, ${p.a * 0.35})`;
          ctx.arc(p.x, p.y, p.r * 0.35, 0, Math.PI * 2);
          ctx.fill();
        }
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
