import React from "react";
import { CheckCircle2, CircleAlert, Info, XCircle } from "lucide-react";
import type { DoctorReport, Finding, ProfileDiagnosis } from "shared/src/data/orval/model";
import { Badge } from "shared/src/components/ui/badge";
import { Identifier } from "shared/src/components/identity/Identifier";

const PROFILE_NAMES: Record<string, string> = {
  dsp: "DSP 2025-1",
  dcat3: "DCAT 3",
  "dcat-ap": "DCAT-AP 3.0.1",
};

const PREFIXES: [string, string][] = [
  ["http://purl.org/dc/terms/", "dct:"],
  ["http://www.w3.org/ns/dcat#", "dcat:"],
  ["http://www.w3.org/ns/odrl/2/", "odrl:"],
  ["https://w3id.org/dspace/2025/1/", "dspace:"],
  ["http://xmlns.com/foaf/0.1/", "foaf:"],
  ["http://www.w3.org/ns/adms#", "adms:"],
];

/** A property IRI with the usual DCAT prefixes. */
const compact = (iri?: string) => {
  if (!iri) return "";
  const prefix = PREFIXES.find(([ns]) => iri.startsWith(ns));
  return prefix ? prefix[1] + iri.slice(prefix[0].length) : iri;
};

const SEVERITY_ICONS = {
  violation: <XCircle className="h-4 w-4 shrink-0 text-danger-500" />,
  warning: <CircleAlert className="h-4 w-4 shrink-0 text-warn-500" />,
  info: <Info className="h-4 w-4 shrink-0 text-brand-sky" />,
};

function FindingRow({ finding, root }: { finding: Finding; root: string }) {
  return (
    <li className="flex items-start gap-2.5 py-2">
      {SEVERITY_ICONS[finding.severity]}
      <div className="min-w-0 flex-1">
        <p className="text-sm">{finding.message}</p>
        <p className="mt-0.5 flex flex-wrap items-center gap-x-2 text-xs text-ink/50">
          {finding.property && <span className="font-mono">{compact(finding.property)}</span>}
          {finding.resource !== root && (
            <>
              {finding.property && <span>·</span>}
              <span>on</span>
              <Identifier value={finding.resource} copyable={false} />
            </>
          )}
        </p>
      </div>
    </li>
  );
}

function ProfileCard({ diagnosis, root }: { diagnosis: ProfileDiagnosis; root: string }) {
  return (
    <article className="flex flex-col rounded-xl border border-ink/10 bg-ink/[0.015]">
      <header className="flex items-center gap-3 border-b border-ink/[0.07] px-4 py-3">
        {diagnosis.compliant ? (
          <CheckCircle2 className="h-4 w-4 shrink-0 text-success-500" />
        ) : (
          <XCircle className="h-4 w-4 shrink-0 text-danger-500" />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-sm font-medium">{PROFILE_NAMES[diagnosis.profile] ?? diagnosis.profile}</p>
          <a
            href={diagnosis.profileIri}
            target="_blank"
            rel="noreferrer"
            className="block truncate text-xs text-ink/50 hover:underline"
          >
            {diagnosis.profileIri}
          </a>
        </div>
        <Badge variant={diagnosis.compliant ? "success" : "danger"}>
          {diagnosis.compliant ? "Compliant" : `${diagnosis.violations} violations`}
        </Badge>
        {diagnosis.warnings > 0 && <Badge variant="warn">{diagnosis.warnings} warnings</Badge>}
      </header>
      {diagnosis.findings.length === 0 ? (
        <p className="px-4 py-3 text-sm text-ink/55">Nothing to report.</p>
      ) : (
        <ul className="divide-y divide-ink/[0.07] px-4">
          {diagnosis.findings.map((finding, i) => (
            <FindingRow key={i} finding={finding} root={root} />
          ))}
        </ul>
      )}
    </article>
  );
}

/** What the catalog doctor says of a resource: one card per profile, with its findings. */
export function DoctorPanel({ report }: { report?: DoctorReport }) {
  if (!report) return <p className="text-sm text-ink/55">Checking conformance…</p>;
  return (
    <div className="flex flex-col gap-4">
      <p className="text-xs text-ink/55">
        Checked {report.checked.length} resource{report.checked.length === 1 ? "" : "s"}: the{" "}
        {report.type.toLowerCase()} and what it links to.
      </p>
      <div className="grid grid-cols-1 gap-4 xl:grid-cols-3">
        {report.profiles.map((diagnosis) => (
          <ProfileCard key={diagnosis.profile} diagnosis={diagnosis} root={report.resource} />
        ))}
      </div>
    </div>
  );
}
