// Who else inside the connector sees a catalog resource.

import { useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@eunomia/ui";
import { CatalogResources, type CatalogResourceKind } from "shared/src/data/catalogResources";
import { Visibility } from "shared/src/data/orval/model/visibility";
import { VISIBILITY_HELP } from "shared/src/components/ConnectPeerForm";

interface SelectProps {
  kind: CatalogResourceKind;
  id: string;
  visibility: Visibility;
}

/** Changes the resource's visibility at once. */
export function VisibilitySelect({ kind, id, visibility }: SelectProps) {
  const queryClient = useQueryClient();
  const [busy, setBusy] = useState(false);

  const change = async (value: string) => {
    if (value === visibility) return;
    setBusy(true);
    try {
      await CatalogResources.setVisibility(kind, id, value as Visibility);
      await CatalogResources.invalidate(queryClient);
      toast.success(`The ${CatalogResources.noun[kind]} is ${value.toLowerCase()} now.`);
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Select value={visibility} onValueChange={change} disabled={busy}>
      <SelectTrigger className="w-36" title={VISIBILITY_HELP[visibility]}>
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {Object.values(Visibility).map((v) => (
          <SelectItem key={v} value={v} title={VISIBILITY_HELP[v]}>
            {v}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
