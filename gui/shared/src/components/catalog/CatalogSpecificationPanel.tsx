import React, { useMemo, useState } from "react";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { Download, FileCode, Loader2 } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { SegmentedControl } from "shared/src/components/ui/segmented";
import { JsonViewer } from "shared/src/components/json/JsonViewer";
import { useGetCatalogInProfile } from "shared/src/data/orval/catalog-exchange/catalog-exchange";
import { useDoctorCatalog } from "shared/src/data/orval/doctor/doctor";
import {
  CATALOG_PROFILES,
  CatalogExchange,
  RDF_FORMATS,
  type CatalogProfile,
  type RdfFormat,
} from "shared/src/data/catalogExchange";

/** Datasets a preview asks for; the whole catalog only comes as a download. */
const PREVIEW_SIZES = [5, 10, 25, 50];
/** Past this many characters the preview is plain text, cut, so the page stays responsive. */
const HIGHLIGHT_LIMIT = 200_000;

/** The catalog as each profile publishes it: a cut preview, the profile's verdict and downloads. */
export function CatalogSpecificationPanel({ catalogId }: { catalogId: string }) {
  const [profile, setProfile] = useState<CatalogProfile>("dcat-ap");
  const [limit, setLimit] = useState(10);
  const [downloading, setDownloading] = useState<RdfFormat | null>(null);

  const { data, isLoading, isError } = useGetCatalogInProfile(catalogId, profile, { limit });
  const { data: doctor } = useDoctorCatalog(catalogId, { profile });

  const document = data?.status === 200 ? data.data : undefined;
  const total = Number(data?.headers?.get?.("x-total-count") ?? 0);
  const shown = Number(data?.headers?.get?.("x-shown-count") ?? 0);
  const diagnosis = doctor?.status === 200 ? doctor.data.profiles[0] : undefined;
  const text = useMemo(() => (document ? JSON.stringify(document, null, 2) : ""), [document]);

  const download = async (format: RdfFormat) => {
    setDownloading(format);
    try {
      await CatalogExchange.download(catalogId, profile, format);
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      setDownloading(null);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-3 border-b border-ink/10 pb-3">
        <SegmentedControl
          ariaLabel="Profile"
          value={profile}
          onChange={setProfile}
          options={CATALOG_PROFILES.map((p) => ({ value: p.key, label: p.label }))}
        />
        {diagnosis && (
          <div className="flex items-center gap-2">
            <Badge variant={diagnosis.compliant ? "success" : "danger"}>
              {diagnosis.compliant ? "Compliant" : `${diagnosis.violations} violations`}
            </Badge>
            {diagnosis.warnings > 0 && <Badge variant="warn">{diagnosis.warnings} warnings</Badge>}
          </div>
        )}
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3 text-xs text-ink/60">
        <div className="flex items-center gap-2">
          <span>Preview of</span>
          <Select value={String(limit)} onValueChange={(v) => setLimit(Number(v))}>
            <SelectTrigger aria-label="Datasets in the preview" className="h-7 w-auto gap-2">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {PREVIEW_SIZES.map((n) => (
                <SelectItem key={n} value={String(n)}>
                  {n}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <span>datasets</span>
          {document && (
            <span className="text-ink/45">
              · showing {shown} of {total}
              {shown < total && ", download it for the rest"}
            </span>
          )}
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          <Download className="h-3.5 w-3.5 opacity-60" />
          {RDF_FORMATS.map((format) => (
            <Button
              key={format.key}
              size="sm"
              variant="outline"
              className="h-7 gap-1 px-2 text-xs"
              disabled={downloading !== null}
              onClick={() => download(format.key)}
            >
              {downloading === format.key && <Loader2 className="h-3 w-3 animate-spin" />}
              {format.label}
            </Button>
          ))}
        </div>
      </div>

      {isLoading && <p className="text-sm text-ink/55">Projecting the catalog…</p>}
      {isError && <p className="text-sm text-danger-600">The catalog could not be projected.</p>}
      {document &&
        (text.length > HIGHLIGHT_LIMIT ? (
          <div className="rounded-lg border border-ink/10 bg-sunken">
            <p className="flex items-center gap-1.5 border-b border-ink/10 px-4 py-2 text-xs text-ink/55">
              <FileCode className="h-3.5 w-3.5" />
              Large document: the first {HIGHLIGHT_LIMIT.toLocaleString()} characters, unformatted.
            </p>
            <pre className="max-h-[60vh] overflow-auto px-4 py-3 font-mono text-xs leading-relaxed">
              {text.slice(0, HIGHLIGHT_LIMIT)}
            </pre>
          </div>
        ) : (
          <JsonViewer value={document} maxHeightClass="max-h-[65vh]" />
        ))}
    </div>
  );
}
