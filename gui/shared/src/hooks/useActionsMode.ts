import { useCallback, useState } from "react";
import { SegmentedOption } from "@eunomia/ui";

export type ActionsMode = "business" | "standard";

const STORAGE_KEY = "eunomia.actions-mode";

export const ACTIONS_MODE_OPTIONS: SegmentedOption<ActionsMode>[] = [
  {
    value: "business",
    label: "Business",
    hint: "Simplified flow: agree and the rest completes itself",
  },
  { value: "standard", label: "Standard", hint: "Every Dataspace Protocol step, one by one" },
];

/** Business or standard process actions, remembered per browser. */
export const useActionsMode = (): [ActionsMode, (mode: ActionsMode) => void] => {
  const [mode, setMode] = useState<ActionsMode>(() => {
    try {
      return localStorage.getItem(STORAGE_KEY) === "standard" ? "standard" : "business";
    } catch {
      return "business";
    }
  });
  const update = useCallback((next: ActionsMode) => {
    setMode(next);
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch {
      // Storage may be unavailable; the choice then lasts for this page only.
    }
  }, []);
  return [mode, update];
};
