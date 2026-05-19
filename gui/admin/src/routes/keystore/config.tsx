import { createFileRoute } from "@tanstack/react-router";
import { useGetApplicationConfig } from "shared/src/data/orval/keystore-config/keystore-config";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { useState } from "react";
import { cn } from "shared/src/lib/utils";
import { ChevronDown, ChevronUp, Settings2 } from "lucide-react";

const ConfigSection = ({ title, data }: { title: string; data: unknown }) => {
  const [expanded, setExpanded] = useState(true);

  return (
    <div className="bg-white/[0.03] border border-white/10 rounded-xl overflow-hidden">
      <button
        onClick={() => setExpanded((v) => !v)}
        className="w-full flex items-center justify-between px-4 py-3 hover:bg-white/5 transition-colors"
      >
        <span className="text-xs font-bold uppercase tracking-widest text-foreground/70 flex items-center gap-2">
          <Settings2 className="h-3.5 w-3.5" />
          {title}
        </span>
        <span className="text-muted-foreground/40">
          {expanded ? <ChevronUp className="h-4 w-4" /> : <ChevronDown className="h-4 w-4" />}
        </span>
      </button>

      <div
        className={cn(
          "grid transition-all duration-200 ease-in-out",
          expanded ? "grid-rows-[1fr]" : "grid-rows-[0fr]",
        )}
      >
        <div className="overflow-hidden">
          <pre className="px-4 pb-4 text-[11px] font-mono text-muted-foreground/80 bg-black/20 whitespace-pre-wrap break-all leading-relaxed border-t border-white/5">
            {JSON.stringify(data, null, 2)}
          </pre>
        </div>
      </div>
    </div>
  );
};

const KeystoreConfig = () => {
  const { data: response, isLoading, error } = useGetApplicationConfig();

  if (isLoading) {
    return (
      <PageSection title="Application Config">
        <div className="space-y-3">
          {[...Array(4)].map((_, i) => (
            <Skeleton key={i} className="h-24 w-full rounded-xl" />
          ))}
        </div>
      </PageSection>
    );
  }

  if (error || response?.status !== 200) {
    return (
      <PageSection title="Application Config">
        <p className="text-destructive font-mono text-xs">Error loading config</p>
      </PageSection>
    );
  }

  const cfg = response.data as Record<string, unknown>;

  return (
    <PageSection title="Application Config">
      <div className="space-y-3">
        {Object.entries(cfg).map(([key, value]) => (
          <ConfigSection key={key} title={key} data={value} />
        ))}
      </div>
    </PageSection>
  );
};

export const Route = createFileRoute("/keystore/config")({
  component: KeystoreConfig,
});
