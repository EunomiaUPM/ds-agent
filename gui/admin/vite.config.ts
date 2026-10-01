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
    css: {
      postcss: {
        plugins: [tailwind, autoprefixer],
      },
    },
    optimizeDeps: {
      exclude: ["lucide-react"],
    },
    server: {
      port: 5174,
      proxy: {
        "/admin/api": {
          target: process.env.VITE_GATEWAY_TARGET || `http://127.0.0.1:${process.env.VITE_GATEWAY_PORT || "1100"}`,
          changeOrigin: true,
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
