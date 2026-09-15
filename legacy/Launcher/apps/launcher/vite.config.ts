import { defineConfig } from "vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Force IPv4: binding only ::1 makes http://localhost:1420 fail when
    // the browser resolves localhost → 127.0.0.1 (ERR_CONNECTION_REFUSED).
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
    // Browser UI preview without Tauri: proxy official metas (avoid CORS).
    proxy: {
      // Owyx site API for the browser dev preview of Owyx account sign-in.
      // In the packaged app this goes through Rust (reqwest), not this proxy.
      "/proxy/owyx": {
        target: process.env.OWYX_API_BASE_URL || "http://127.0.0.1:3001",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/owyx/, ""),
      },
      "/proxy/mojang": {
        target: "https://piston-meta.mojang.com",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/mojang/, ""),
      },
      "/proxy/fabric": {
        target: "https://meta.fabricmc.net",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/fabric/, ""),
      },
      "/proxy/quilt": {
        target: "https://meta.quiltmc.org",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/quilt/, ""),
      },
      "/proxy/forge": {
        target: "https://maven.minecraftforge.net",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/forge/, ""),
      },
      "/proxy/neoforge": {
        target: "https://maven.neoforged.net",
        changeOrigin: true,
        rewrite: (p) => p.replace(/^\/proxy\/neoforge/, ""),
      },
    },
  },
}));
