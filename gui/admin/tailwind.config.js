import uiPreset from "@eunomia/ui/tailwind-preset";

/** @type {import('tailwindcss').Config} */
export default {
  presets: [uiPreset],
  content: [
    "index.html",
    "./src/**/*.{ts,tsx,js,jsx}",
    "./../shared/src/**/*.{ts,tsx,js,jsx}",
    "./../shared/index.css",
    "./../../../ymir/ui/src/**/*.{ts,tsx}",
    "./../../../ymir/ui/dist/**/*.js",
  ],
};
