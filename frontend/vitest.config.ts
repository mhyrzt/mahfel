import path from "node:path";

import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      "@": path.resolve("."),
    },
  },
  test: {
    globals: true,
    exclude: ["node_modules", ".next", ".storybook", "storybook-static"],
    projects: [
      {
        extends: true,
        test: {
          name: "components",
          environment: "jsdom",
          setupFiles: ["./vitest.setup.ts"],
          include: ["**/*.{test,spec}.{ts,tsx}"],
          exclude: [
            "node_modules",
            ".next",
            ".storybook",
            "storybook-static",
            "features/recordings/server-render/tests/**",
            "app/api/health/*.test.ts",
          ],
        },
      },
      {
        extends: true,
        test: {
          name: "server",
          environment: "node",
          setupFiles: [],
          include: [
            "features/recordings/server-render/tests/**/*.test.ts",
            "app/api/health/*.test.ts",
          ],
        },
      },
    ],
  },
});
