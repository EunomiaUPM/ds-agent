import {
  Identifier,
  cn
} from "./chunk-4GNLXNMN.js";

// src/json/JsonViewer.tsx
import React, { Suspense, useMemo as useMemo2, useState as useState2 } from "react";
import { Check, Copy } from "lucide-react";

// src/hooks/useTheme.ts
import { useContext } from "react";

// src/theme/ThemeContext.tsx
import { createContext, useCallback, useEffect, useMemo, useState } from "react";
import { jsx } from "react/jsx-runtime";
var THEME_STORAGE_KEY = "eunomia.theme";
var ThemeContext = createContext(null);
var prefersDark = () => typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches;
function readStoredPreference(fallback) {
  try {
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") return stored;
  } catch {
  }
  return fallback;
}
function applyTheme(resolved) {
  const root = document.documentElement;
  root.classList.toggle("dark", resolved === "dark");
  root.style.colorScheme = resolved;
}
var ThemeContextProvider = ({
  children,
  defaultTheme = "system"
}) => {
  const [theme, setThemeState] = useState(
    () => readStoredPreference(defaultTheme)
  );
  const [systemTheme, setSystemTheme] = useState(
    () => prefersDark() ? "dark" : "light"
  );
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const onChange = (e) => setSystemTheme(e.matches ? "dark" : "light");
    media.addEventListener("change", onChange);
    return () => media.removeEventListener("change", onChange);
  }, []);
  const resolvedTheme = theme === "system" ? systemTheme : theme;
  useEffect(() => {
    applyTheme(resolvedTheme);
  }, [resolvedTheme]);
  const setTheme = useCallback((next) => {
    setThemeState(next);
    try {
      localStorage.setItem(THEME_STORAGE_KEY, next);
    } catch {
    }
  }, []);
  const toggleTheme = useCallback(() => {
    setTheme(resolvedTheme === "dark" ? "light" : "dark");
  }, [resolvedTheme, setTheme]);
  const value = useMemo(
    () => ({ theme, resolvedTheme, setTheme, toggleTheme }),
    [theme, resolvedTheme, setTheme, toggleTheme]
  );
  return /* @__PURE__ */ jsx(ThemeContext.Provider, { value, children });
};

// src/hooks/useTheme.ts
function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used within a ThemeContextProvider");
  }
  return ctx;
}

// src/json/JsonViewer.tsx
import { jsx as jsx2, jsxs } from "react/jsx-runtime";
var MonacoJson = React.lazy(() => import("./MonacoJson-WVP24WDG.js"));
function JsonViewer({ value, className, maxHeightClass = "max-h-[60vh]" }) {
  const { resolvedTheme } = useTheme();
  const [copied, setCopied] = useState2(false);
  const text = useMemo2(
    () => typeof value === "string" ? JsonViewer.prettify(value) : JSON.stringify(value, null, 2),
    [value]
  );
  const maxHeight = JsonViewer.pixels(maxHeightClass);
  const copy = () => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };
  const dark = resolvedTheme === "dark";
  const lines = text.split("\n").length;
  return /* @__PURE__ */ jsxs(
    "div",
    {
      className: cn(
        "overflow-hidden rounded-md border",
        dark ? "border-[#3c3c3c] bg-[#1e1e1e]" : "border-[#e5e5e5] bg-white",
        className
      ),
      children: [
        /* @__PURE__ */ jsxs(
          "div",
          {
            className: cn(
              "flex h-[30px] items-center justify-between border-b pl-3 pr-1.5 font-sans text-[12px]",
              dark ? "border-[#3c3c3c] bg-[#252526] text-[#cccccc]" : "border-[#e5e5e5] bg-[#f3f3f3] text-[#616161]"
            ),
            children: [
              /* @__PURE__ */ jsxs("span", { className: "flex items-center gap-1.5", children: [
                /* @__PURE__ */ jsx2(
                  "span",
                  {
                    className: cn(
                      "font-mono text-[11px] font-semibold",
                      dark ? "text-[#cbcb41]" : "text-[#a16207]"
                    ),
                    children: "{}"
                  }
                ),
                "JSON",
                /* @__PURE__ */ jsxs("span", { className: "opacity-60", children: [
                  "\xB7 ",
                  lines,
                  " ",
                  lines === 1 ? "line" : "lines"
                ] })
              ] }),
              /* @__PURE__ */ jsxs(
                "button",
                {
                  type: "button",
                  onClick: copy,
                  "aria-label": "Copy JSON",
                  title: "Copy",
                  className: cn(
                    "inline-flex h-[22px] items-center gap-1 rounded px-1.5 transition-colors",
                    dark ? "hover:bg-[#3c3c3c]" : "hover:bg-[#e1e1e1]"
                  ),
                  children: [
                    copied ? /* @__PURE__ */ jsx2(Check, { className: "h-3.5 w-3.5" }) : /* @__PURE__ */ jsx2(Copy, { className: "h-3.5 w-3.5" }),
                    copied ? "Copied" : "Copy"
                  ]
                }
              )
            ]
          }
        ),
        /* @__PURE__ */ jsx2(
          Suspense,
          {
            fallback: /* @__PURE__ */ jsx2(
              "pre",
              {
                className: cn(
                  "overflow-auto py-1.5 pl-[52px] pr-4 text-[12px] leading-[18px]",
                  dark ? "text-[#d4d4d4]" : "text-black"
                ),
                style: { maxHeight, fontFamily: 'Menlo, Monaco, Consolas, "Courier New", monospace' },
                children: text
              }
            ),
            children: /* @__PURE__ */ jsx2(MonacoJson, { text, dark, maxHeight })
          }
        )
      ]
    }
  );
}
JsonViewer.prettify = (raw) => {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2);
  } catch {
    return raw;
  }
};
JsonViewer.pixels = (heightClass) => {
  const vh = typeof window === "undefined" ? 800 : window.innerHeight;
  const arbitrary = heightClass.match(/max-h-\[(\d+(?:\.\d+)?)(vh|px|rem)\]/);
  if (arbitrary) {
    const n = Number(arbitrary[1]);
    return arbitrary[2] === "vh" ? n / 100 * vh : arbitrary[2] === "rem" ? n * 16 : n;
  }
  const scale = heightClass.match(/max-h-(\d+)$/);
  if (scale) return Number(scale[1]) * 4;
  return 0.6 * vh;
};

// src/json/JsonTable.tsx
import { useState as useState3 } from "react";
import dayjs from "dayjs";
import { Braces, ChevronRight, Table2 } from "lucide-react";
import { jsx as jsx3, jsxs as jsxs2 } from "react/jsx-runtime";
function JsonTable({
  value,
  className,
  expandDepth = 1,
  rawToggle = true,
  emptyText = "No data"
}) {
  const [raw, setRaw] = useState3(false);
  const data = JsonTable.normalize(value);
  if (JsonTable.isEmpty(data)) {
    return /* @__PURE__ */ jsx3("p", { className: cn("text-sm text-ink/50 italic", className), children: emptyText });
  }
  return /* @__PURE__ */ jsxs2("div", { className: cn("space-y-2", className), children: [
    rawToggle && /* @__PURE__ */ jsx3("div", { className: "flex justify-end", children: /* @__PURE__ */ jsx3("div", { className: "inline-flex rounded-md border border-ink/10 p-0.5 text-xs", children: [
      { on: false, label: "Table", Icon: Table2 },
      { on: true, label: "JSON", Icon: Braces }
    ].map(({ on, label, Icon }) => /* @__PURE__ */ jsxs2(
      "button",
      {
        type: "button",
        onClick: () => setRaw(on),
        className: cn(
          "inline-flex items-center gap-1 rounded px-2 py-1 transition-colors",
          raw === on ? "bg-ink/10 text-ink font-medium" : "text-ink/55 hover:text-ink"
        ),
        children: [
          /* @__PURE__ */ jsx3(Icon, { className: "h-3.5 w-3.5" }),
          label
        ]
      },
      label
    )) }) }),
    raw ? /* @__PURE__ */ jsx3(JsonViewer, { value: data }) : /* @__PURE__ */ jsx3("div", { className: "overflow-hidden rounded-lg border border-ink/10", children: /* @__PURE__ */ jsx3(JsonTable.Node, { value: data, depth: 0, expandDepth }) })
  ] });
}
JsonTable.normalize = (value) => {
  if (typeof value === "string") {
    const t = value.trim();
    if (t.startsWith("{") || t.startsWith("[")) {
      try {
        return JSON.parse(t);
      } catch {
        return value;
      }
    }
  }
  return value ?? null;
};
JsonTable.isEmpty = (v) => v === null || v === "" || Array.isArray(v) && v.length === 0 || typeof v === "object" && !Array.isArray(v) && Object.keys(v).length === 0;
JsonTable.isComposite = (v) => v !== null && typeof v === "object" && !JsonTable.isEmpty(v);
JsonTable.summary = (v) => Array.isArray(v) ? `${v.length} item${v.length === 1 ? "" : "s"}` : `${Object.keys(v).length} field${Object.keys(v).length === 1 ? "" : "s"}`;
JsonTable.Node = function Node({
  value,
  depth,
  expandDepth
}) {
  if (!JsonTable.isComposite(value)) return /* @__PURE__ */ jsx3(JsonTable.Primitive, { value });
  if (Array.isArray(value) && value.every((v) => !JsonTable.isComposite(v))) {
    return /* @__PURE__ */ jsx3("div", { className: "flex flex-wrap gap-1.5 p-2.5", children: value.map((v, i) => /* @__PURE__ */ jsx3("span", { className: "rounded-md bg-ink/[0.06] px-2 py-0.5 text-[0.8125rem]", children: /* @__PURE__ */ jsx3(JsonTable.Primitive, { value: v }) }, i)) });
  }
  const entries = Array.isArray(value) ? value.map((v, i) => [`#${i + 1}`, v]) : Object.entries(value);
  return /* @__PURE__ */ jsx3("table", { className: "w-full table-fixed border-collapse text-[0.8125rem]", children: /* @__PURE__ */ jsx3("tbody", { children: entries.map(([k, v]) => /* @__PURE__ */ jsx3(JsonTable.Row, { name: k, value: v, depth, expandDepth }, k)) }) });
};
JsonTable.Row = function Row({
  name,
  value,
  depth,
  expandDepth
}) {
  const composite = JsonTable.isComposite(value);
  const [open, setOpen] = useState3(depth < expandDepth);
  if (!composite) {
    return /* @__PURE__ */ jsxs2("tr", { className: "border-b border-ink/[0.06] last:border-0 align-top", children: [
      /* @__PURE__ */ jsx3(
        "th",
        {
          scope: "row",
          className: "w-[34%] px-3 py-2 text-left font-mono text-xs font-normal text-ink/55 break-all",
          children: name
        }
      ),
      /* @__PURE__ */ jsx3("td", { className: "px-3 py-2 break-words", children: /* @__PURE__ */ jsx3(JsonTable.Primitive, { value }) })
    ] });
  }
  return /* @__PURE__ */ jsx3("tr", { className: "border-b border-ink/[0.06] last:border-0", children: /* @__PURE__ */ jsxs2("td", { colSpan: 2, className: "p-0", children: [
    /* @__PURE__ */ jsxs2(
      "button",
      {
        type: "button",
        onClick: () => setOpen(!open),
        "aria-expanded": open,
        className: "flex w-full items-center gap-1.5 px-3 py-2 text-left hover:bg-ink/[0.03]",
        children: [
          /* @__PURE__ */ jsx3(
            ChevronRight,
            {
              className: cn("h-3.5 w-3.5 text-ink/40 transition-transform", open && "rotate-90")
            }
          ),
          /* @__PURE__ */ jsx3("span", { className: "font-mono text-xs text-ink/75", children: name }),
          /* @__PURE__ */ jsx3("span", { className: "text-xs text-ink/40", children: JsonTable.summary(value) })
        ]
      }
    ),
    open && /* @__PURE__ */ jsx3("div", { className: "mb-2 ml-4 mr-2 overflow-hidden rounded-md border border-ink/[0.08] bg-ink/[0.015]", children: /* @__PURE__ */ jsx3(JsonTable.Node, { value, depth: depth + 1, expandDepth }) })
  ] }) });
};
var ISO_DATE = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}/;
var IDENTIFIER = /^(urn:|did:|https?:\/\/)/;
JsonTable.Primitive = function Primitive({ value }) {
  if (value === null || value === "") return /* @__PURE__ */ jsx3("span", { className: "text-ink/35 italic", children: "empty" });
  if (typeof value === "boolean")
    return /* @__PURE__ */ jsx3("span", { className: cn("font-medium", value ? "text-emerald-500" : "text-ink/50"), children: String(value) });
  if (typeof value === "number") return /* @__PURE__ */ jsx3("span", { className: "font-mono tabular-nums", children: value });
  if (typeof value === "object") return /* @__PURE__ */ jsx3("span", { className: "text-ink/35 italic", children: "empty" });
  if (ISO_DATE.test(value) && dayjs(value).isValid())
    return /* @__PURE__ */ jsx3("span", { title: value, children: dayjs(value).format("YYYY-MM-DD HH:mm:ss") });
  if (IDENTIFIER.test(value))
    return value.length > 64 ? /* @__PURE__ */ jsx3(Identifier, { value, tail: 16 }) : /* @__PURE__ */ jsx3("span", { className: "font-mono text-xs break-all", children: value });
  return /* @__PURE__ */ jsx3("span", { className: "whitespace-pre-wrap", children: value });
};

export {
  THEME_STORAGE_KEY,
  ThemeContext,
  ThemeContextProvider,
  useTheme,
  JsonViewer,
  JsonTable
};
//# sourceMappingURL=chunk-YTAH6WED.js.map