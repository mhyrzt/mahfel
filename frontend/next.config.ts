import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "standalone",
  allowedDevOrigins: ["localhost", "127.0.0.1"],
  poweredByHeader: false,
  serverExternalPackages: ["sharp", "roughjs"],
  outputFileTracingIncludes: {
    "/api/internal/recording-render": ["./features/recordings/server-render/assets/*"],
  },
  typescript: {
    ignoreBuildErrors: true,
  },
};

export default nextConfig;
