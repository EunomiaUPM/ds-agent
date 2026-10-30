import React from "react";
import { ArrowRight } from "lucide-react";
import { useLink } from "../link";

export interface FeatureCardProps {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  description: string;
  /** Standards or technologies behind the feature, shown as chips. */
  tags?: string[];
  to?: string;
  action?: string;
}

/** A capability of the node with an optional link to where it is managed. */
export function FeatureCard({
  icon: Icon,
  title,
  description,
  tags,
  to,
  action,
}: FeatureCardProps) {
  const Link = useLink();
  const className =
    "group flex h-full flex-col rounded-xl border border-ink/10 bg-ink/[0.02] p-5 transition-colors hover:border-brand-sky/40 hover:bg-ink/[0.04]";

  const content = (
    <>
      <span className="mb-4 inline-flex h-9 w-9 items-center justify-center rounded-lg bg-brand-sky/10 text-brand-sky">
        <Icon className="h-[18px] w-[18px]" />
      </span>
      <h3 className="text-base font-semibold tracking-tight">{title}</h3>
      <p className="mt-1.5 flex-1 text-sm leading-relaxed text-ink/60">{description}</p>
      {tags && (
        <div className="mt-4 flex flex-wrap gap-1.5">
          {tags.map((t) => (
            <span key={t} className="rounded-md bg-ink/[0.06] px-2 py-0.5 text-xs text-ink/70">
              {t}
            </span>
          ))}
        </div>
      )}
      {action && (
        <span className="mt-4 inline-flex items-center gap-1 text-sm font-medium text-brand-sky">
          {action}
          <ArrowRight className="h-3.5 w-3.5 transition-transform group-hover:translate-x-0.5" />
        </span>
      )}
    </>
  );

  return to ? (
    <Link to={to} className={className}>
      {content}
    </Link>
  ) : (
    <div className={className}>{content}</div>
  );
}
