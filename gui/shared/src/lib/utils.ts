import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/*
  Utility function to merge class names using clsx and tailwind-merge.
  This helps in conditionally applying class names and resolving conflicts
  in Tailwind CSS classes.
*/
export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

/*
  Utility function to generate a random alphanumeric string of a given length.
  This can be used for creating unique identifiers or tokens.
*/
export const generateRandomString = (length: number) => {
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  let result = "";
  const charactersLength = characters.length;
  for (let i = 0; i < length; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }
  return result;
};

/*
  Utility function to merge state and attribute into a formatted string.
  This is useful for displaying combined status information in a user-friendly way.
*/
export const mergeStateAndAttribute = (state: string, attribute: string): string => {
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

/*
  Utility function to format URNs (Uniform Resource Names).
  It can truncate long URNs for better readability while preserving key parts.
*/
export const formatUrn = (urn: string | undefined, truncate: boolean = true): string => {
  if (!urn || typeof urn !== "string") return "";

  if (!truncate) return urn;

  const isDid = urn.startsWith("did:");

  // If it's a DID and short enough, show it full
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

  // Aggressive truncation for IDs that are not DIDs or are too long
  if (urn.length > 20) {
    return urn.slice(0, 13) + "...";
  }

  return urn;
};

/**
 * Formats structured identifiers (e.g., URNs, DIDs, or similar colon-separated strings)
 * into a shorter, human-readable form.
 *
 * The function extracts the most specific segment of the identifier (typically the last
 * portion after the final ":"), optionally truncating it to a configurable maximum length.
 *
 * This makes long technical identifiers easier to display in UI components while preserving
 * their uniqueness and recognizability.
 *
 * Examples:
 * - "urn:dataset:abcd123456789" - "abcd1234"
 * - "did:web:example.com:user:xyz987654" - "xyz98765"
 *
 * If the input does not contain a structured prefix, a fallback truncation strategy may be applied.
 *
 * @param value - The identifier string to format.
 * @param truncate - Whether to truncate the extracted segment (default: true).
 * @param maxLength - Maximum length of the returned string when truncating (default: 8).
 * @returns A shortened, display-friendly version of the identifier.
 */
export const formatIdentifier = (
  value: string | undefined,
  nParts: number = 2,
  truncate: boolean = true,
  maxLength: number = 12,
): string => {
  if (!value) return "";

  if (!truncate) return value;

  const parts = value.split(":");
  const formatted = parts.slice(nParts).join(":") || value;

  return formatted.length > maxLength ? formatted.slice(0, maxLength) + "..." : formatted;
};

/**
 * Utility function to convert technical VC type names into friendly readable names.
 * Rules:
 * - gx_ prefix -> Gaia-X
 * - CamelCase/PascalCase -> Split with spaces
 * - Add "Credential" if not present (unless it's a Participant type)
 * - jwt -> (JWT) suffix
 *
 * @example
 * gx_VatId_jwt_vc_json -> Gaia-X Vat Id Credential (JWT)
 */
export const getFriendlyVCType = (type: string): string => {
  if (!type) return "";

  let friendly = type;

  // 1. Identify JWT suffix before cleaning
  const isJwt = type.toLowerCase().includes("jwt");

  // 2. Remove common suffixes/technical parts
  friendly = friendly.replace(/(_jwt|_vc|_json)/g, "");

  // 3. Handle gx_ prefix
  let isGaiaX = false;
  if (friendly.startsWith("gx_")) {
    isGaiaX = true;
    friendly = friendly.replace(/^gx_/, "");
  }

  // 4. Split by underscores and CamelCase
  friendly = friendly.replace(/_/g, " ");
  // Match a lowercase letter or digit followed by an uppercase letter
  friendly = friendly.replace(/([a-z0-9])([A-Z])/g, "$1 $2");

  // 5. Add "Credential" if not already present
  // Based on user feedback: DataspaceParticipant doesn't get "Credential"
  if (!/credential/i.test(friendly) && !/participant/i.test(friendly)) {
    friendly = `${friendly} Credential`;
  }

  // 6. Final assembly
  friendly = friendly.trim();

  if (isGaiaX) {
    friendly = `Gaia-X ${friendly}`;
  }

  if (isJwt) {
    friendly = `${friendly} (JWT)`;
  }

  return friendly;
};
