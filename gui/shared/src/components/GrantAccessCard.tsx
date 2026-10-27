import { Layers } from "lucide-react";
import { Badge } from "shared/src/components/ui/badge";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "shared/src/components/ui/card";
import type { AccessItem } from "shared/src/data/orval/model/accessItem";

/** What a grant asked for and what it was granted. */
export interface GrantAccess {
  id: string;
  access: AccessItem[];
  granted?: AccessItem[] | null;
  label?: string | null;
}

/** `https://w3id.org/gnap4ds/dataset` → `dataset`. */
const typeLabel = (type: string) => type.split("/").pop() || type;

const AccessList = ({ items }: { items: AccessItem[] }) => (
  <ul className="space-y-2">
    {items.map((item, idx) => (
      <li key={idx} className="flex flex-wrap items-start gap-2">
        <Badge variant="info" className="font-mono">
          {typeLabel(item.type)}
        </Badge>
        <div className="flex flex-col gap-0.5">
          {item.identifier && (
            <span className="font-mono text-xs break-all">{item.identifier}</span>
          )}
          {(item.locations ?? []).map((location) => (
            <span key={location} className="font-mono text-xs break-all">
              {location}
            </span>
          ))}
        </div>
      </li>
    ))}
  </ul>
);

export const GrantAccessCard = ({ resourceReq }: { resourceReq: GrantAccess | null }) => {
  if (!resourceReq) {
    return (
      <Card className="mt-6 opacity-50 border-dashed">
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-muted-foreground">
            <Layers className="h-5 w-5" />
            Access
          </CardTitle>
          <CardDescription>No access bound to this grant.</CardDescription>
        </CardHeader>
      </Card>
    );
  }
  return (
    <Card className="mt-6">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Layers className="h-5 w-5 text-primary" />
          Access
        </CardTitle>
        <CardDescription>
          What this grant asked for and what it was granted
          {resourceReq.label ? ` (${resourceReq.label})` : ""}.
        </CardDescription>
      </CardHeader>
      <CardContent className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        <div className="space-y-2">
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Requested
          </span>
          <AccessList items={resourceReq.access ?? []} />
        </div>
        <div className="space-y-2">
          <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
            Granted
          </span>
          {resourceReq.granted ? (
            <AccessList items={resourceReq.granted} />
          ) : (
            <p className="text-xs text-muted-foreground">Not granted yet.</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default GrantAccessCard;
