import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import tsconfigPaths from "vite-tsconfig-paths";
import tailwind from "tailwindcss";
import autoprefixer from "autoprefixer";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";

export default defineConfig(() => {
  return {
    base: "/admin/",
    plugins: [react(), tsconfigPaths(), TanStackRouterVite({ autoCodeSplitting: true })],
    publicDir: path.resolve(__dirname, "../shared/public"),
    // One copy of React and TanStack whichever package imports them.
    resolve: {
      dedupe: [
        "react",
        "react-dom",
        "react/jsx-runtime",
        "@tanstack/react-query",
        "@tanstack/react-router",
      ],
    },
    css: {
      postcss: {
        plugins: [tailwind, autoprefixer],
      },
    },
    // @eunomia/ui stays out of the pre-bundle (its monaco workers use `?worker`), so its
    // dependencies, which import CommonJS react-dom, are pre-bundled by name.
    optimizeDeps: {
      exclude: ["lucide-react", "@eunomia/ui"],
      include: [
        "react-dom",
        "@radix-ui/react-accordion",
        "@radix-ui/react-checkbox",
        "@radix-ui/react-dialog",
        "@radix-ui/react-dropdown-menu",
        "@radix-ui/react-icons",
        "@radix-ui/react-label",
        "@radix-ui/react-popover",
        "@radix-ui/react-select",
        "@radix-ui/react-separator",
        "@radix-ui/react-slot",
        "@radix-ui/react-tabs",
        "@radix-ui/react-tooltip",
        "class-variance-authority",
        "clsx",
        "cmdk",
        "dayjs",
        "dayjs/plugin/relativeTime",
        "motion/react",
        "react-hook-form",
        "react-syntax-highlighter",
        "react-syntax-highlighter/dist/esm/styles/hljs",
        "recharts",
        "sonner",
        "tailwind-merge",
        "vaul",
      ],
    },
    server: {
      port: 5174,
      proxy: {
        "/admin/api": {
          target: process.env.VITE_GATEWAY_TARGET || `http://127.0.0.1:${process.env.VITE_GATEWAY_PORT || "1100"}`,
          changeOrigin: true,
          // The live updates socket (`/admin/api/live`) goes through the same proxy.
          ws: true,
        },
        "/api": {
          target: process.env.VITE_GATEWAY_TARGET || `http://127.0.0.1:${process.env.VITE_GATEWAY_PORT || "1100"}`,
          changeOrigin: true,
        },
        "/oauth": {
          target: process.env.VITE_GATEWAY_TARGET || `http://127.0.0.1:${process.env.VITE_GATEWAY_PORT || "1100"}`,
          changeOrigin: true,
        },
        "/.well-known": {
          target: process.env.VITE_GATEWAY_TARGET || `http://127.0.0.1:${process.env.VITE_GATEWAY_PORT || "1100"}`,
          changeOrigin: true,
        },
      },
    },
    build: {
      rollupOptions: {
        output: {
          manualChunks: {
            vendor: ["react", "react-dom"],
            ui: [
              "@radix-ui/react-dialog",
              "@radix-ui/react-slot",
              "@radix-ui/react-tooltip",
              "class-variance-authority",
              "lucide-react",
              "clsx",
              "tailwind-merge",
            ],
            tanstack: ["@tanstack/react-router", "@tanstack/react-query"],
            syntax: ["react-syntax-highlighter"],
          },
        },
      },
    },
  };
});
