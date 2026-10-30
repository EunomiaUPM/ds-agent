import React, { useRef, useState } from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import { useQueryClient } from "@tanstack/react-query";
import { toast } from "sonner";
import {
  ArrowLeft,
  CheckCircle2,
  FileUp,
  Loader2,
  ShieldCheck,
  Upload,
  XCircle,
} from "lucide-react";

import { PageLayout } from "@eunomia/ui";
import { PageHeader } from "@eunomia/ui";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@eunomia/ui";
import { Badge } from "@eunomia/ui";
import { Button } from "@eunomia/ui";
import { SegmentedControl } from "@eunomia/ui";
import { Identifier } from "@eunomia/ui";
import { FindingRow } from "shared/src/components/catalog/DoctorPanel";
import { useGetMainCatalogs } from "shared/src/data/orval/catalogs/catalogs";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import {
  CatalogExchange,
  SHAPE_GROUPS,
  type ImportOptions,
  type ImportOutcome,
  type ImportProfile,
} from "shared/src/data/catalogExchange";

export const Route = createFileRoute("/my-catalog/import")({
  component: RouteComponent,
});

/** Where the document's resources go: its own catalogs, or into the main one. */
type Placement = "own" | "main";

function RouteComponent() {
  const queryClient = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [file, setFile] = useState<File | null>(null);
  const [dragging, setDragging] = useState(false);
  const [profile, setProfile] = useState<ImportProfile>("dcat-ap");
  const [mode, setMode] = useState<ImportOptions["mode"]>("upsert");
  const [placement, setPlacement] = useState<Placement>("own");
  const [checks, setChecks] = useState<string[]>(
    SHAPE_GROUPS["dcat-ap"].filter((g) => g.byDefault).map((g) => g.key),
  );
  const [busy, setBusy] = useState<"validate" | "import" | null>(null);
  const [outcome, setOutcome] = useState<ImportOutcome | null>(null);

  const { data: mainData } = useGetMainCatalogs();
  const main =
    mainData?.status === 200 ? CatalogViews.catalog(mainData.data as CatalogDoc) : undefined;

  const choose = (chosen: File | undefined) => {
    if (!chosen) return;
    setFile(chosen);
    setOutcome(null);
  };

  const changeProfile = (next: ImportProfile) => {
    setProfile(next);
    setChecks(SHAPE_GROUPS[next].filter((g) => g.byDefault).map((g) => g.key));
    setOutcome(null);
  };

  const toggleCheck = (key: string) =>
    setChecks((current) =>
      current.includes(key) ? current.filter((k) => k !== key) : [...current, key],
    );

  const run = async (dryRun: boolean) => {
    if (!file) return;
    setBusy(dryRun ? "validate" : "import");
    try {
      const result = await CatalogExchange.importDocument(file, {
        profile,
        mode,
        dryRun,
        checks,
        target: placement === "main" ? main?.id : undefined,
      });
      setOutcome(result);
      if (result.report?.imported) {
        toast.success(
          `Imported ${result.report.created} new and ${result.report.updated} updated resources`,
        );
        await queryClient.invalidateQueries();
      } else if (result.error) {
        toast.error(result.error);
      }
    } catch (e) {
      toast.error((e as Error).message);
    } finally {
      setBusy(null);
    }
  };

  const format = file ? CatalogExchange.formatOf(file.name) : undefined;
  const report = outcome?.report;

  return (
    <PageLayout>
      <PageHeader
        title="Import a DCAT catalog"
        badge={
          <Link
            to="/my-catalog"
            className="inline-flex items-center gap-1 text-xs text-ink/60 hover:text-ink"
          >
            <ArrowLeft className="h-3.5 w-3.5" /> My catalog
          </Link>
        }
      />

      <div className="grid grid-cols-1 gap-4 xl:grid-cols-[minmax(0,420px)_1fr]">
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Document</CardTitle>
            <CardDescription>
              DCAT-AP or DCAT 3 in JSON-LD, Turtle, RDF/XML or N-Triples. It is validated with SHACL
              (SEMIC's official shapes for DCAT-AP) and written whole, or not at all.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-5">
            <button
              type="button"
              onClick={() => input.current?.click()}
              onDragOver={(e) => {
                e.preventDefault();
                setDragging(true);
              }}
              onDragLeave={() => setDragging(false)}
              onDrop={(e) => {
                e.preventDefault();
                setDragging(false);
                choose(e.dataTransfer.files[0]);
              }}
              className={`flex flex-col items-center gap-2 rounded-xl border border-dashed px-4 py-8 text-center transition-colors ${
                dragging ? "border-brand-sky bg-brand-sky/5" : "border-ink/20 hover:border-ink/35"
              }`}
            >
              <FileUp className="h-8 w-8 text-ink/40" />
              {file ? (
                <>
                  <span className="text-sm font-medium break-all">{file.name}</span>
                  <span className="text-xs text-ink/55">
                    {(file.size / 1024).toFixed(1)} KB · {format?.label ?? "read as JSON-LD"}
                  </span>
                </>
              ) : (
                <span className="text-sm text-ink/60">Drop a file here or click to choose one</span>
              )}
            </button>
            <input
              ref={input}
              type="file"
              accept=".jsonld,.json,.ttl,.rdf,.xml,.owl,.nt"
              className="hidden"
              onChange={(e) => choose(e.target.files?.[0])}
            />

            <Option label="Profile">
              <SegmentedControl
                ariaLabel="Profile"
                value={profile}
                onChange={changeProfile}
                options={[
                  { value: "dcat-ap", label: "DCAT-AP 3.0.1" },
                  { value: "dcat3", label: "DCAT 3" },
                ]}
              />
            </Option>

            <Option label="SHACL checks">
              <div className="flex flex-col gap-1.5">
                {SHAPE_GROUPS[profile].map((group) => (
                  <label key={group.key} className="flex items-center gap-2 text-sm">
                    <input
                      type="checkbox"
                      checked={checks.includes(group.key)}
                      onChange={() => toggleCheck(group.key)}
                    />
                    {group.label}
                  </label>
                ))}
              </div>
            </Option>

            <Option label="Catalogs of the document">
              <SegmentedControl
                ariaLabel="Placement"
                value={placement}
                onChange={setPlacement}
                options={[
                  {
                    value: "own",
                    label: "Keep them",
                    hint: "Each catalog becomes a catalog of its own",
                  },
                  {
                    value: "main",
                    label: "Into my catalog",
                    hint: "Their datasets go into the main catalog",
                  },
                ]}
              />
            </Option>

            <Option label="Resources already stored">
              <SegmentedControl
                ariaLabel="Mode"
                value={mode}
                onChange={setMode}
                options={[
                  { value: "upsert", label: "Update them" },
                  { value: "create", label: "Refuse the import" },
                ]}
              />
            </Option>

            <div className="flex gap-2 pt-1">
              <Button
                variant="outline"
                disabled={!file || busy !== null}
                onClick={() => run(true)}
                className="gap-1.5"
              >
                {busy === "validate" ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <ShieldCheck className="h-4 w-4" />
                )}
                Validate
              </Button>
              <Button
                disabled={!file || busy !== null}
                onClick={() => run(false)}
                className="gap-1.5"
              >
                {busy === "import" ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Upload className="h-4 w-4" />
                )}
                Import
              </Button>
            </div>
          </CardContent>
        </Card>

        <div className="flex flex-col gap-4">
          {!outcome && (
            <Card className="flex items-center justify-center border-dashed p-10 text-sm text-ink/55">
              Validate a document to see its SHACL report and what it would create.
            </Card>
          )}
          {outcome?.error && <Card className="p-4 text-sm text-danger-600">{outcome.error}</Card>}
          {report && <ImportReportView outcome={outcome!} />}
        </div>
      </div>
    </PageLayout>
  );
}

function Option({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs font-medium uppercase tracking-wide text-ink/50">{label}</span>
      {children}
    </div>
  );
}

/** The answer of an import: verdict, SHACL results, plan issues and the resources. */
function ImportReportView({ outcome }: { outcome: ImportOutcome }) {
  const report = outcome.report!;
  const { validation } = report;
  return (
    <>
      <Card>
        <CardHeader className="flex flex-row items-center gap-3 space-y-0">
          {report.conforms ? (
            <CheckCircle2 className="h-5 w-5 text-success-500" />
          ) : (
            <XCircle className="h-5 w-5 text-danger-500" />
          )}
          <div className="flex-1">
            <CardTitle className="text-base">
              {report.imported
                ? "Imported"
                : report.conforms
                  ? "Valid: ready to import"
                  : "Not imported"}
            </CardTitle>
            <CardDescription>
              Checks: {validation.checks.join(", ")} · {report.created} to create · {report.updated}{" "}
              to update
            </CardDescription>
          </div>
          <Badge variant={validation.conforms ? "success" : "danger"}>
            {validation.conforms ? "SHACL conforms" : `${validation.violations} violations`}
          </Badge>
          {validation.warnings > 0 && <Badge variant="warn">{validation.warnings} warnings</Badge>}
        </CardHeader>
      </Card>

      {(validation.results.length > 0 || report.issues.length > 0) && (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Findings</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="max-h-[45vh] divide-y divide-ink/[0.07] overflow-auto">
              {report.issues.map((issue, i) => (
                <FindingRow key={`issue-${i}`} finding={issue} root="" />
              ))}
              {validation.results.map((result, i) => (
                <FindingRow key={`shacl-${i}`} finding={CatalogExchange.finding(result)} root="" />
              ))}
            </ul>
          </CardContent>
        </Card>
      )}

      {report.resources.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-sm">Resources</CardTitle>
          </CardHeader>
          <CardContent>
            <table className="w-full text-left text-xs">
              <thead className="text-ink/50">
                <tr>
                  <th className="py-1.5 font-medium">Type</th>
                  <th className="py-1.5 font-medium">Stored as</th>
                  <th className="py-1.5 font-medium">In the document</th>
                  <th className="py-1.5 font-medium">Action</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-ink/[0.07]">
                {report.resources.map((resource) => (
                  <tr key={resource.id}>
                    <td className="py-1.5">{resource.type}</td>
                    <td className="py-1.5">
                      <Identifier value={resource.id} />
                    </td>
                    <td
                      className="max-w-[260px] truncate py-1.5 font-mono text-ink/55"
                      title={resource.source}
                    >
                      {resource.source}
                    </td>
                    <td className="py-1.5">
                      <Badge variant={resource.action === "create" ? "success" : "info"} size="sm">
                        {resource.action}
                      </Badge>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </CardContent>
        </Card>
      )}
    </>
  );
}
