/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import React, { useState } from "react";
import { CatalogViews, type CatalogDoc } from "shared/src/data/catalog";
import { createFileRoute, Link } from "@tanstack/react-router";
import { ArrowUpRight, Eye, EyeOff } from "lucide-react";
import {
  getGetConnectorInstanceByDistributionQueryOptions,
  useGetConnectorInstanceByDistribution,
} from "shared/src/data/orval/connector/connector";
import {
  getGetDistributionByIdQueryOptions,
  useGetDistributionById,
} from "shared/src/data/orval/distributions/distributions";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Identifier } from "shared/src/components/identity/Identifier";
import { DatasetLabel } from "shared/src/components/identity/DatasetLabel";
import { JsonTable } from "shared/src/components/json/JsonTable";

export const Route = createFileRoute("/catalog/$catalogId/distribution-connector/$distributionId")({
  component: DistributionConnectorPage,
  loader: async ({ context: { queryClient }, params: { distributionId } }) => {
    await queryClient.ensureQueryData(getGetDistributionByIdQueryOptions(distributionId));
    return queryClient.ensureQueryData(
      getGetConnectorInstanceByDistributionQueryOptions(distributionId),
    );
  },
});

type Spec = {
  protocol?: string;
  urlTemplate?: string;
  method?: string | string[];
  headers?: Record<string, string> | null;
  bodyTemplate?: unknown;
};

const AUTH_LABELS: Record<string, string> = {
  NO_AUTH: "None",
  BASIC_AUTH: "HTTP Basic",
  BASIC: "HTTP Basic",
  BEARER_TOKEN: "Bearer token",
  BEARER: "Bearer token",
  API_KEY: "API key",
  OAUTH2: "OAuth 2.0",
};
const SECRET = /pass|secret|token|key/i;

function DistributionConnectorPage() {
  const { catalogId, distributionId } = Route.useParams();
  const [reveal, setReveal] = useState(false);
  const { data: distributionData } = useGetDistributionById(distributionId);
  const { data: instanceData } = useGetConnectorInstanceByDistribution(distributionId);
  const distribution =
    distributionData?.status === 200
      ? CatalogViews.distribution(distributionData.data as CatalogDoc)
      : undefined;
  const instance = instanceData?.status === 200 ? instanceData.data : undefined;
  if (!distribution) return null;

  const interaction = instance?.interaction as
    | { mode: string; dataAccess?: Spec; subscribe?: Spec; unsubscribe?: Spec | null }
    | undefined;
  const auth = (instance?.authenticationConfig ?? {}) as Record<string, unknown>;
  const authType = String(auth.type ?? "NO_AUTH");
  const authFields = Object.fromEntries(
    Object.entries(auth)
      .filter(([k]) => k !== "type")
      .map(([k, v]) => [k, !reveal && SECRET.test(k) && v ? "••••••••" : v]),
  );
  const hasSecrets = Object.keys(auth).some((k) => SECRET.test(k));
  const steps: [string, Spec | null | undefined][] = interaction?.dataAccess
    ? [["Data access", interaction.dataAccess]]
    : [
        ["Subscribe", interaction?.subscribe],
        ["Unsubscribe", interaction?.unsubscribe],
      ];

  return (
    <PageLayout>
      <PageHeader
        eyebrow="Distribution"
        title={distribution.title || <Identifier value={distribution.id} />}
        badge={
          <>
            {distribution.format && <Badge variant="outline">{distribution.format}</Badge>}
            {interaction && <Badge variant="info">{interaction.mode}</Badge>}
          </>
        }
        description={distribution.description || undefined}
        actions={
          <Button asChild variant="outline" size="sm">
            <Link
              to="/catalog/$catalogId/dataset/$datasetId"
              params={{ catalogId, datasetId: distribution.datasetId }}
            >
              Open dataset
            </Link>
          </Button>
        }
      />

      <FieldGrid
        columns={3}
        className="mb-6"
        fields={[
          { label: "Dataset", value: <DatasetLabel id={distribution.datasetId} /> },
          {
            label: "Data service",
            value: (
              <Link
                to="/catalog/$catalogId/data-service/$dataServiceId"
                params={{ catalogId, dataServiceId: distribution.accessService }}
                className="inline-flex items-center gap-1 text-brand-sky hover:underline"
              >
                <Identifier value={distribution.accessService} copyable={false} />
                <ArrowUpRight className="h-3.5 w-3.5" />
              </Link>
            ),
          },
          {
            label: "Issued",
            value: <FormatDate date={distribution.issued} format="YYYY-MM-DD HH:mm" />,
          },
          { label: "Distribution", value: <Identifier value={distribution.id} /> },
          { label: "Connector instance", value: <Identifier value={instance?.id} /> },
          {
            label: "Built from template",
            value: instance && (
              <span className="inline-flex items-center gap-1.5">
                <Identifier value={instance.name} />
                <span className="text-xs text-ink/50">v{instance.version}</span>
              </span>
            ),
          },
        ]}
      />

      {!instance ? (
        <div className="rounded-lg border border-dashed border-ink/15 py-10 text-center text-sm text-ink/50">
          No connector is bound to this distribution, so the dataplane cannot serve it.
        </div>
      ) : (
        <>
          {steps
            .filter((s): s is [string, Spec] => !!s[1])
            .map(([title, spec]) => (
              <PageSection key={title} title={title}>
                <FieldGrid
                  columns={3}
                  fields={[
                    { label: "Protocol", value: spec.protocol },
                    {
                      label: "Methods",
                      value: (
                        <span className="flex flex-wrap gap-1">
                          {[spec.method ?? []].flat().map((m) => (
                            <span
                              key={m}
                              className="rounded bg-ink/[0.07] px-1.5 py-0.5 font-mono text-xs"
                            >
                              {m}
                            </span>
                          ))}
                        </span>
                      ),
                    },
                    {
                      label: "Transport",
                      value: spec.urlTemplate?.startsWith("https") ? "TLS (HTTPS)" : "Plain HTTP",
                    },
                    {
                      label: "Endpoint",
                      span: 3,
                      value: (
                        <span className="break-all font-mono text-xs">{spec.urlTemplate}</span>
                      ),
                    },
                  ]}
                />
                {(!!spec.headers || !!spec.bodyTemplate) && (
                  <div className="mt-3 grid grid-cols-1 gap-4 xl:grid-cols-2">
                    {spec.headers && (
                      <div>
                        <p className="mb-1.5 text-xs font-medium text-ink/50">Request headers</p>
                        <JsonTable value={spec.headers} rawToggle={false} />
                      </div>
                    )}
                    {!!spec.bodyTemplate && (
                      <div>
                        <p className="mb-1.5 text-xs font-medium text-ink/50">Body template</p>
                        <JsonTable value={spec.bodyTemplate} />
                      </div>
                    )}
                  </div>
                )}
              </PageSection>
            ))}

          <PageSection
            title="Authentication"
            action={
              hasSecrets && (
                <Button variant="ghost" size="xs" onClick={() => setReveal(!reveal)}>
                  {reveal ? <EyeOff /> : <Eye />}
                  {reveal ? "Hide secrets" : "Show secrets"}
                </Button>
              )
            }
          >
            <FieldGrid
              columns={3}
              fields={[{ label: "Method", value: AUTH_LABELS[authType] ?? authType }]}
            />
            {Object.keys(authFields).length > 0 && (
              <JsonTable value={authFields} rawToggle={false} className="mt-3" />
            )}
          </PageSection>
        </>
      )}
    </PageLayout>
  );
}
