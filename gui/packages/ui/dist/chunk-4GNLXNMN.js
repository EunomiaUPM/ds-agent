// src/lib/utils.ts
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";
function cn(...inputs) {
  return twMerge(clsx(inputs));
}
var generateRandomString = (length) => {
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  const charactersLength = characters.length;
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
};
var mergeStateAndAttribute = (state, attribute) => {
  let textOut = "";
  switch (state) {
    case "SUSPENDED":
      switch (attribute) {
        case "ByProvider":
          textOut = `${state} by Provider`;
          break;
        case "ByConsumer":
          textOut = `${state} by Consumer`;
          break;
        default:
          textOut = state;
      }
      break;
    default:
      textOut = state;
  }
  return textOut;
};
var formatUrn = (urn, truncate = true) => {
  if (!urn || typeof urn !== "string") return "";
  if (!truncate) return urn;
  const isDid = urn.startsWith("did:");
  if (isDid && urn.length < 40) return urn;
  if (urn.startsWith("urn:")) {
    const parts = urn.split(":");
    if (parts.length >= 3) {
      const nid = parts[1];
      const nss = parts.slice(2).join(":");
      const shortNid = nid.length > 7 ? nid.slice(0, 7) : nid;
      const shortNss = nss.length > 8 ? nss.slice(0, 8) : nss;
      return `urn:${shortNid}:${shortNss}`;
    }
  }
  if (urn.length > 20) {
    return urn.slice(0, 13) + "...";
  }
  return urn;
};
var formatIdentifier = (value, nParts = 2, truncate = true, maxLength = 12) => {
  if (!value) return "";
  if (!truncate) return value;
  const parts = value.split(":");
  const formatted = parts.slice(nParts).join(":") || value;
  return formatted.length > maxLength ? formatted.slice(0, maxLength) + "..." : formatted;
};
var getFriendlyVCType = (type) => {
  if (!type) return "";
  let friendly = type;
  const isJwt = type.toLowerCase().includes("jwt");
  friendly = friendly.replace(/(_jwt|_vc|_json)/g, "");
  let isGaiaX = false;
  if (friendly.startsWith("gx_")) {
    isGaiaX = true;
    friendly = friendly.replace(/^gx_/, "");
  }
  friendly = friendly.replace(/_/g, " ");
  friendly = friendly.replace(/([a-z0-9])([A-Z])/g, "$1 $2");
  if (!/credential/i.test(friendly) && !/participant/i.test(friendly)) {
    friendly = `${friendly} Credential`;
  }
  friendly = friendly.trim();
  if (isGaiaX) {
    friendly = `Gaia-X ${friendly}`;
  }
  if (isJwt) {
    friendly = `${friendly} (JWT)`;
  }
  return friendly;
};
var encodePathId = (id) => {
  const bytes = new TextEncoder().encode(id);
  let binary = "";
  bytes.forEach((b) => binary += String.fromCharCode(b));
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
};
var decodePathId = (encoded) => {
  const base64 = encoded.replace(/-/g, "+").replace(/_/g, "/");
  const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
  return new TextDecoder().decode(bytes);
};
var pathIdParams = (key) => ({
  parse: (raw) => {
    try {
      return { ...raw, [key]: decodePathId(raw[key]) };
    } catch {
      return raw;
    }
  },
  stringify: (params) => ({ ...params, [key]: encodePathId(params[key]) })
});
var grantBadgeState = (status) => status === "Finalized" ? "expired" : status;

// src/identity/Identifier.tsx
import { useState } from "react";
import { Check, Copy } from "lucide-react";
import { jsx, jsxs } from "react/jsx-runtime";
function Identifier({ value, className, tail = 8, copyable = true }) {
  const [copied, setCopied] = useState(false);
  if (!value) return /* @__PURE__ */ jsx("span", { className: "text-ink/35", children: "\u2014" });
  const copy = (e) => {
    e.preventDefault();
    e.stopPropagation();
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };
  return /* @__PURE__ */ jsxs(
    "span",
    {
      className: cn("group/id inline-flex min-w-0 items-center gap-1", className),
      title: value,
      children: [
        /* @__PURE__ */ jsx("span", { className: "truncate font-mono text-xs text-ink/75", children: Identifier.short(value, tail) }),
        copyable && /* @__PURE__ */ jsx(
          "button",
          {
            type: "button",
            onClick: copy,
            "aria-label": "Copy identifier",
            className: "shrink-0 rounded p-0.5 text-ink/35 opacity-0 transition-opacity hover:text-ink group-hover/id:opacity-100 focus-visible:opacity-100",
            children: copied ? /* @__PURE__ */ jsx(Check, { className: "h-3 w-3" }) : /* @__PURE__ */ jsx(Copy, { className: "h-3 w-3" })
          }
        )
      ]
    }
  );
}
Identifier.short = (value, tail = 8) => {
  if (value.startsWith("urn:")) {
    const [, nid, ...rest] = value.split(":");
    const nss = rest.join(":");
    return `${nid} \xB7 ${nss.slice(0, tail)}`;
  }
  if (value.startsWith("did:") && value.length > 32) {
    const method = value.split(":").slice(0, 2).join(":");
    return `${method}:\u2026${value.slice(-tail)}`;
  }
  return value.length > 40 ? `${value.slice(0, 36)}\u2026` : value;
};

export {
  cn,
  generateRandomString,
  mergeStateAndAttribute,
  formatUrn,
  formatIdentifier,
  getFriendlyVCType,
  encodePathId,
  decodePathId,
  pathIdParams,
  grantBadgeState,
  Identifier
};
//# sourceMappingURL=chunk-4GNLXNMN.js.map