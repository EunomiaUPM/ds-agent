import { useIsFetching, useIsMutating } from "@tanstack/react-query";
import { Loader2 } from "lucide-react";
import { useEffect, useState } from "react";

/**
 * Global loading indicator component that shows when queries are fetching
 * or any mutation is in flight.
 */
export function GlobalLoadingIndicator() {
  const isFetching = useIsFetching();
  const isMutating = useIsMutating();
  const inFlight = isFetching + isMutating;
  const [visible, setVisible] = useState(false);

  useEffect(() => {
    if (inFlight > 0) {
      setVisible(true);
    } else {
      const timer = setTimeout(() => setVisible(false), 500);
      return () => clearTimeout(timer);
    }
  }, [inFlight]);

  if (!visible) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 flex items-center gap-2 rounded-full border border-border bg-background/80 px-4 py-2 text-sm shadow-lg backdrop-blur supports-[backdrop-filter]:bg-background/60">
      <Loader2 className="h-4 w-4 animate-spin text-muted-foreground" />
      <span className="text-muted-foreground">Loading...</span>
    </div>
  );
}
