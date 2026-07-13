import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  async rewrites() {
    return [
      {
        source: "/api/issues/:path*",
        destination: "http://127.0.0.1:3000/issues/:path*",
      },
      {
        source: "/api/issues",
        destination: "http://127.0.0.1:3000/issues",
      },
    ];
  },
};

export default nextConfig;
