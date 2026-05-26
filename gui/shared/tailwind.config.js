import { theme as theme_, plugins as plugins_ } from "./../shared/tailwind.config";

/** @type {import('tailwindcss').Config} */

module.exports = {
  darkMode: ["class"],
  content: [
    "index.css",
    "./src/**/*.{ts,tsx,js,jsx,css,sass,scss}",
    "./../shared/src/**/*.{ts,tsx,js,jsx,css,sass,scss}",
    "./../shared/index.css",
  ],
  theme: {
    extend: {
      fontFamily: {
        ubuntu: ["Ubuntu", "sans-serif"],
        source: ["Source Sans Pro", "Source Sans 3", "sans-serif"],
        mono: ['"Source Code Pro"', "monospace"],
        space: ["Space Grotesk", "sans-serif"],
        inter: ["Inter", "sans-serif"],
        spacemono: ["Space Mono", "monospace"],
      },
      fontSize: {
        "3xs": ["0.5rem", { lineHeight: "1.4" }] /* 8px */,
        "2xs": ["0.625rem", { lineHeight: "1.4" }] /* 10px */,
        xs: ["0.75rem", { lineHeight: "1.4" }] /* 12px */,
        sm: ["0.875rem", { lineHeight: "1.4" }] /* 14px */,
        base: ["1rem", { lineHeight: "1.5" }] /* 16px body */,
        18: ["1.125rem", { lineHeight: "1.5" }],
        20: ["1.25rem", { lineHeight: "1.4" }],
        24: ["1.5rem", { lineHeight: "1.4" }],
        28: ["1.75rem", { lineHeight: "1.4" }],
        32: ["2rem", { lineHeight: "1.4" }],
        36: ["2.25rem", { lineHeight: "1.4" }],
        40: ["2.5rem", { lineHeight: "1.4" }],
        48: ["3rem", { lineHeight: "1.4" }],
        56: ["3.5rem", { lineHeight: "1.4" }],
        64: ["4rem", { lineHeight: "1.4" }],
      },
      colors: {
        /** Colores del nuevo Design System */
        newds: {
          white: "#FEFEFE",
          black: "#131313",
          base: {
            50: "#F6F6FA",
            100: "#EEEEF4",
            200: "#DDDDEA",
            300: "#CECEE1",
            400: "#B0B0D0",
            500: "#7878A8",
            600: "#5C5C80",
            700: "#474761",
            800: "#37374A",
            900: "#2C2C38",
            950: "#1C1C20",
          },
          primary: {
            DEFAULT: "#5A42FA",
            50: "#F1F3FE",
            100: "#E4E7FD",
            200: "#CDD4FF",
            300: "#A1A1FC",
            400: "#7878FB",
            500: "#5A42FA",
            600: "#433BCD",
            700: "#392EB0",
            800: "#2D248A",
            900: "#29226B",
            950: "#1B183C",
          },
          error: {
            DEFAULT: "#E94A51",
            50: "#FEF2F3",
            100: "#FDE3E4",
            200: "#FCCCCE",
            300: "#F9A8AC",
            400: "#F3767C",
            500: "#E94A51",
            600: "#D52A32",
            700: "#B32229",
            800: "#942025",
            900: "#6B1014",
            950: "#430C0F",
          },
          warn: {
            DEFAULT: "#FE9A00",
            50: "#FFFBEB",
            100: "#FEF3C6",
            200: "#FEE685",
            300: "#FFD230",
            400: "#FFB900",
            500: "#FE9A00",
            600: "#E17100",
            700: "#CC4B02",
            800: "#973C00",
            900: "#7B3306",
            950: "#461901",
          },
          success: {
            DEFAULT: "#18B77B",
            50: "#DDFEEC",
            100: "#BDFCDB",
            200: "#86F4C4",
            300: "#4EE4A6",
            400: "#2DCE8E",
            500: "#18B77B",
            600: "#009966",
            700: "#0B7851",
            800: "#07583C",
            900: "#033D2D",
            950: "#01231B",
          },
        },
        /** Colores actuales */
        foreground: "#f8f8fa",
        text: "#f1f1f6",
        stroke: "#525880",
        roles: {
          provider: "#52BFE0",
          consumer: "#FF852F",
          bussiness: "#52DBE0",
          customer: "#FF9E2F",
        },
        brand: {
          snow: "#EFF7FB", // white
          sky: "#9DD5F2", // light blue
          purple: "#62388E",
          blue: "#24234C", // dark blue
          black: "#0D0D1C",
        },
        base: {
          main: "#09091B",
          sidebar: "#1D1C3D",
          header: "#24234C",

          50: "#F6F6FA",
          100: "#EEEEF4",
          200: "#EEEEF4",
          300: "#EEEEF4",
          400: "#EEEEF4",
          500: "#EEEEF4",
          600: "#EEEEF4",
          700: "#EEEEF4",
          800: "#EEEEF4",
          900: "#EEEEF4",
          950: "#EEEEF4",
        },
        background: {
          DEFAULT: "#09091B",
          800: "#07070d",
          600: "#09091B",
          400: "#2E3356",
          300: "#2E3356",
          200: "#2E3356",
        },

        foreground: {
          // LIGHT BASE color palette
          DEFAULT: "#d3d2e0",
          950: "#353243",
          900: "#524d65",
          800: "#645d7a",
          700: "#786f92",
          600: "#867ea3", // default
          500: "#9e9ab8",
          400: "#b9b7ce",
          300: "#d3d2e0",
          200: "#d3d2e0",
          100: "#f1f1f6",
          50: "#f8f8fa",
        },
        primary: {
          // blue-ish
          DEFAULT: "#2D3B98",
          950: "#1e244d",
          900: "#2b387d",
          800: "#2d3b98", // default
          700: "#3349c2",
          600: "#3c5cd4",
          500: "#5178e0",
          400: "#729be8",
          300: "#9fbef1",
          200: "#c6d7f7",
          100: "#dfe8fa",
          50: "#f1f5fd",
          foreground: "#FFFFFF",
        },
        secondary: {
          // purple-ish
          DEFAULT: "#542D98",
          950: "#2f195c",
          900: "#4d2a88",
          800: "#542d98", // default
          700: "#6e3bc6",
          600: "#7e4dda",
          500: "#8e6de5",
          400: "#ab97ee",
          300: "#c8bdf5",
          200: "#dfdafa",
          100: "#eeebfc",
          50: "#f6f4fe",
        },
        accent: {
          DEFAULT: "#62388E",
        },

        danger: {
          // RED
          DEFAULT: "#d42643",
          950: "#490818",
          900: "#821933",
          800: "#981935",
          700: "#b61a38",
          600: "#d42643", // default
          500: "#eb485b",
          400: "#f47883",
          300: "#f9a8ae",
          200: "#fccfd3",
          100: "#fee5e6",
          50: "#fef2f2",
        },
        warn: {
          // ORANGE/YELLOW
          DEFAULT: "#f05b06",
          950: "#451405",
          900: "#7f2e0f",
          800: "#9e350e",
          700: "#c74307",
          600: "#f05b06", // default
          500: "#ff7710",
          400: "#ff9537",
          300: "#ffbd70",
          200: "#ffd9a8",
          100: "#ffeed4",
          50: "#fff7ed",
        },
        success: {
          // GREEN
          DEFAULT: "#219c69",
          950: "#021A14",
          900: "#032A20",
          800: "#0c4f33",
          700: "#16744d",
          600: "#219c69", // default
          500: "#27b077",
          400: "#2cc586",
          300: "#33de97",
          200: "#39f3a6",
          100: "#a3fecd",
          50: "#d0fee4",
        },
        process: {
          // BLUE/TURQUOISE
          DEFAULT: "#51FFF0",
          950: "#001a25",
          900: "#002a35",
          800: "#004e5a",
          700: "#007983",
          600: "#00a4a9",
          500: "#00d1cd",
          400: "#51FFF0", // default
          300: "#98fffc",
          200: "#cefcff",
          100: "#edfcff",
          50: "#f8fdff",
        },
        pause: {
          // GREY (NEUTRAL)
          DEFAULT: "#7c7789",
          950: "#141218",
          900: "#232029",
          800: "#3f3c49",
          700: "#5c5769",
          600: "#7c7789", // default
          500: "#9d99a7",
          400: "#bfbdc6",
          300: "#e2e1e5",
          200: "#ebebed",
          100: "#f6f6f7",
          50: "#f9f9fa",
        },
        // Revisiones ----------------------------------
        border: {
          DEFAULT: "#121212",
        },
        ring: {
          DEFAULT: "#9DD5F2",
        },
        // shadow: {
        //   DEFAULT: "#cbd5e1",
        // },
        // title: {
        //   DEFAULT: "#323232",
        // },
      },
    },
    screens: {
      xs: "420px",
      sm: "640px",
      md: "768px",
      lg: "1024px",
      xl: "1280px",
      "2xl": "1536px",
      "3xl": "1700px",
      "4xl": "1920px",
      "5xl": "2120px",
    },
  },
};
// export const content = [
//   "index.css",
//   "./src/**/*.{ts,tsx,js,jsx,css,sass,scss}",
//   "./../shared/src/**/*.{ts,tsx,js,jsx,css,sass,scss}",
//   "./../shared/index.css"
// ];

export const theme = theme_;
export const plugins = plugins_;

// TAILWIND CONFIG POR DEFECTO - Quiza se pueda rescatar algo de esto.

// import { fontFamily } from "tailwindcss/defaultTheme"

// /** @type {import('tailwindcss').Config} */
// export const content = [
//   "index.css",
//   "./src/**/*.{ts,tsx,js,jsx,css,sass,scss}"
// ]
// export const theme = {
//   container: {
//     center: true,
//     padding: "2rem",
//     screens: {
//       "2xl": "1400px",
//     },
//   },
//   extend: {
//     colors: {
//       border: "hsl(var(--border))",
//       input: "hsl(var(--input))",
//       ring: "hsl(var(--ring))",
//       background: "hsl(var(--background))",
//       foreground: "hsl(var(--foreground))",
//       primary: {
//         DEFAULT: "hsl(var(--primary))",
//         foreground: "hsl(var(--primary-foreground))",
//       },
//       secondary: {
//         DEFAULT: "hsl(var(--secondary))",
//         foreground: "hsl(var(--secondary-foreground))",
//       },
//       destructive: {
//         DEFAULT: "hsl(var(--destructive))",
//         foreground: "hsl(var(--destructive-foreground))",
//       },
//       muted: {
//         DEFAULT: "hsl(var(--muted))",
//         foreground: "hsl(var(--muted-foreground))",
//       },
//       accent: {
//         DEFAULT: "hsl(var(--accent))",
//         foreground: "hsl(var(--accent-foreground))",
//       },
//       popover: {
//         DEFAULT: "hsl(var(--popover))",
//         foreground: "hsl(var(--popover-foreground))",
//       },
//       card: {
//         DEFAULT: "hsl(var(--card))",
//         foreground: "hsl(var(--card-foreground))",
//       },
//     },
//     borderRadius: {
//       lg: `var(--radius)`,
//       md: `calc(var(--radius) - 2px)`,
//       sm: "calc(var(--radius) - 4px)",
//     },
//     fontFamily: {
//       sans: ["var(--font-sans)", ...fontFamily.sans],
//     },
//     keyframes: {
//       "accordion-down": {
//         from: { height: "0" },
//         to: { height: "var(--radix-accordion-content-height)" },
//       },
//       "accordion-up": {
//         from: { height: "var(--radix-accordion-content-height)" },
//         to: { height: "0" },
//       },
//     },
//     animation: {
//       "accordion-down": "accordion-down 0.2s ease-out",
//       "accordion-up": "accordion-up 0.2s ease-out",
//     },
//   },
// }
// // eslint-disable-next-line no-undef
// export const plugins = [require("tailwindcss-animate")]
