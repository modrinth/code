import type { NextConfig } from "next";

const backendUrl = process.env.BACKEND_URL || "http://127.0.0.1:3001";

const nextConfig: NextConfig = {
  reactCompiler: true,
  output: "standalone",

  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${backendUrl}/api/:path*`,
      },
      {
        source: "/uploads/:path*",
        destination: `${backendUrl}/uploads/:path*`,
      },
      {
        source: "/fixtures/:path*",
        destination: `${backendUrl}/fixtures/:path*`,
      },
      {
        source: "/health",
        destination: `${backendUrl}/health`,
      },
    ];
  },

  async redirects() {
    return [
      {
        source: "/servers",
        destination: "/download",
        permanent: false,
      },
      // Resend-verification used to email /verify-email; the real page is /verify.
      { source: "/verify-email", destination: "/verify", permanent: false },
    ];
  },

  // Baseline security headers (nuclei: missing-security-headers). Safe defaults
  // that don't break the app; TLS/HSTS is added by the reverse proxy in prod.
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "X-Frame-Options", value: "SAMEORIGIN" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Permissions-Policy", value: "geolocation=(), microphone=(), camera=()" },
          { key: "Cross-Origin-Opener-Policy", value: "same-origin" },
        ],
      },
    ];
  },

  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "api.owyx.site",
        pathname: "/uploads/**",
      },
      {
        protocol: "https",
        hostname: "owyx.site",
        pathname: "/uploads/**",
      },
      {
        protocol: "https",
        hostname: "owyx.site",
        pathname: "/assets/**",
      },
      {
        protocol: "http",
        hostname: "127.0.0.1",
        port: "8055",
        pathname: "/assets/**",
      },
      {
        protocol: "https",
        hostname: "crafatar.com",
        pathname: "/avatars/**",
      },
    ],
  },
};

export default nextConfig;
