import React from "react";
import { createPortal } from "react-dom";
import { ArrowRight } from "lucide-react";
import { useLink } from "../link";
import { Button } from "../ui/button";
import Heading from "../ui/heading";
import { Badge } from "../ui/badge";

export interface WizardEndDialogProps {
  open: boolean;
  onClose?: () => void;
  title?: React.ReactNode;
  content?: React.ReactNode;
  actionHref?: string;
  actionLabel?: string;
  sectionTitle: string;
}

export default function WizardEndDialog({
  open,
  title = "Congratulations",
  content,
  actionHref = "/catalog/",
  actionLabel = "See catalog",
  sectionTitle,
}: WizardEndDialogProps) {
  const Link = useLink();
  if (!open) return null;
  const portalRoot = typeof window !== "undefined" ? document.body : null;

  const portalContent = (
    <div className="fixed left-1/2 top-16 z-[9999] w-full max-w-lg -translate-x-1/2 transform">
      <div className="relative bg-background-300 border border-secondary-800 text-ink p-3 rounded-md shadow-lg pointer-events-auto">
        <Badge variant="wizardSuccess" className="mb-2">
          {sectionTitle}
        </Badge>
        <Heading level="h5" className="mb-2 text-xl font-semibold">
          {title}
        </Heading>
        <p className="mb-4 text-sm text-muted-foreground">{content}</p>

        <div className="flex justify-end gap-2">
          <Link to={actionHref}>
            <Button className="animate-pulse bg-secondary-500 hover:bg-secondary-600 text-white">
              {actionLabel}
              <ArrowRight />
            </Button>
          </Link>
        </div>
      </div>
    </div>
  );

  if (portalRoot) return createPortal(portalContent, portalRoot);
  return portalContent;
}
