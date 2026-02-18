import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import tsconfigPaths from "vite-tsconfig-paths";
import tailwind from "tailwindcss";
import autoprefixer from "autoprefixer";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import { parse } from "dotenv";
import { readFileSync } from "fs";

export default defineConfig(({ mode }) => {
  const processEnv = {};
  const envFromProcess = process.env.ENV_FILE;

  if (envFromProcess) {
    try {
      const envsPath = path.resolve(__dirname, envFromProcess);
      const fileContent = readFileSync(envsPath, "utf8");
      const envFromFile = parse(fileContent);

      for (const key in envFromFile) {
        if (
          (key.startsWith("GATEWAY_") ||
            key.startsWith("CATALOG_AS") ||
            key.startsWith("CONFIG_")) &&
          Object.prototype.hasOwnProperty.call(envFromFile, key)
        ) {
          processEnv[`import.meta.env.${key}`] = JSON.stringify(envFromFile[key]);
        }
      }

      processEnv[`import.meta.env.VITE_ENV`] = JSON.stringify(envFromProcess);
    } catch (error) {
      console.error("Error loading environment file:", error);
      process.exit(1);
    }
  }

  return {
    define: processEnv,
    plugins: [react(), tsconfigPaths(), TanStackRouterVite({ autoCodeSplitting: true })],
    publicDir: path.resolve(__dirname, "../shared/public"),
    css: {
      postcss: {
        plugins: [tailwind, autoprefixer],
      },
    },
  };
});
