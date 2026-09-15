type Particle = { x: number; y: number; vx: number; vy: number; r: number; a: number; hue: number };

export function startParticles(canvas: HTMLCanvasElement) {
  const reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduced) {
    canvas.hidden = true;
    return () => undefined;
  }

  const ctx = canvas.getContext("2d");
  if (!ctx) return () => undefined;

  let raf = 0;
  let w = 0;
  let h = 0;
  const particles: Particle[] = [];

  const resize = () => {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    w = window.innerWidth;
    h = window.innerHeight;
    canvas.width = Math.floor(w * dpr);
    canvas.height = Math.floor(h * dpr);
    canvas.style.width = `${w}px`;
    canvas.style.height = `${h}px`;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  };

  const spawn = (n: number) => {
    for (let i = 0; i < n; i++) {
      particles.push({
        x: Math.random() * w,
        y: Math.random() * h,
        vx: (Math.random() - 0.5) * 0.25,
        vy: -0.15 - Math.random() * 0.35,
        r: 0.6 + Math.random() * 1.8,
        a: 0.15 + Math.random() * 0.35,
        hue: Math.random() > 0.55 ? 188 : 262,
      });
    }
  };

  const tick = () => {
    ctx.clearRect(0, 0, w, h);
    for (const p of particles) {
      p.x += p.vx;
      p.y += p.vy;
      if (p.y < -4) {
        p.y = h + 4;
        p.x = Math.random() * w;
      }
      if (p.x < -4) p.x = w + 4;
      if (p.x > w + 4) p.x = -4;
      ctx.beginPath();
      ctx.fillStyle = `hsla(${p.hue} 90% 65% / ${p.a})`;
      ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
      ctx.fill();
    }
    raf = requestAnimationFrame(tick);
  };

  resize();
  spawn(42);
  window.addEventListener("resize", resize);
  raf = requestAnimationFrame(tick);

  return () => {
    cancelAnimationFrame(raf);
    window.removeEventListener("resize", resize);
  };
}
