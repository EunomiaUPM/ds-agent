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

  // Fallback for non-URN strings (preserve old behavior or just standard truncate)
  if (urn.length > 20) {
    return urn.slice(0, 13) + "[...]";
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
 * - "urn:dataset:abcd123456789" → "abcd1234"
 * - "did:web:example.com:user:xyz987654" → "xyz98765"
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
  truncate: boolean = true,
  maxLength: number = 12,
): string => {
  if (!value || typeof value !== "string") return "";

  if (!truncate) return value;

  const parts = value.split(":");

  // Si hay al menos 3 partes, quitamos las dos primeras
  const formatted = parts.length > 2 ? parts.slice(2).join(":") : value;

  return formatted.length > maxLength ? formatted.slice(0, maxLength) + "..." : formatted;
};
