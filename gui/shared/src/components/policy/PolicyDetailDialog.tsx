import React from "react";
import { Eye } from "lucide-react";
import { Button } from "shared/src/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "shared/src/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { FieldGrid } from "shared/src/components/layout/FieldGrid";
import { JsonViewer } from "shared/src/components/json/JsonViewer";
import { Identifier } from "shared/src/components/identity/Identifier";
import { FormatDate } from "shared/src/components/ui/format-date";
import { OdrlPolicyDto } from "shared/src/data/orval/model";
import { OdrlRulesView } from "./OdrlRulesView";
import { ControlPointChips } from "./ControlPointChips";

/** A button opening the offer in full: its rules, the ODRL a consumer gets, how it was made and
 * the stored record, each JSON with a copy button. */
export function PolicyDetailDialog({ policy }: { policy: OdrlPolicyDto }) {
  const source = policy.composition
    ? `Composed from ${policy.composition.length} ${policy.composition.length === 1 ? "part" : "parts"}`
    : policy.sourceTemplateId
      ? `Template ${policy.sourceTemplateId}@${policy.sourceTemplateVersion}`
      : "Written by hand";

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button size="sm" variant="ghost">
          <Eye /> Details
        </Button>
      </DialogTrigger>
      <DialogContent className="flex max-h-[90vh] max-w-4xl flex-col gap-0 p-0">
        <DialogHeader className="border-b border-ink/10 px-6 py-4">
          <DialogTitle>{policy.description || "Untitled offer"}</DialogTitle>
          <DialogDescription asChild>
            <span className="flex flex-wrap items-center gap-x-2 text-xs">
              <Identifier value={policy.id} />
            </span>
          </DialogDescription>
        </DialogHeader>

        <div className="min-h-0 flex-1 space-y-5 overflow-y-auto px-6 py-5">
          <FieldGrid
            columns={3}
            fields={[
              {
                label: "Created",
                value: <FormatDate date={policy.createdAt} />,
              },
              { label: "Visibility", value: policy.visibility },
              { label: "Made", value: source },
              {
                label: "Meant to apply at (informative)",
                span: 3,
                value: policy.controlPoints?.length ? (
                  <ControlPointChips points={policy.controlPoints} />
                ) : (
                  "—"
                ),
              },
            ]}
          />

          <Tabs defaultValue="rules">
            <TabsList>
              <TabsTrigger value="rules">Rules</TabsTrigger>
              <TabsTrigger value="odrl">ODRL</TabsTrigger>
              {policy.composition && <TabsTrigger value="composition">Composition</TabsTrigger>}
              {policy.instantiationParameters != null && (
                <TabsTrigger value="parameters">Parameters</TabsTrigger>
              )}
              <TabsTrigger value="record">Record</TabsTrigger>
            </TabsList>
            <TabsContent value="rules" className="pt-4">
              <OdrlRulesView policy={policy.odrlOffer} />
            </TabsContent>
            <TabsContent value="odrl" className="space-y-2 pt-4">
              <p className="text-xs text-ink/55">The offer as consumers read it in your catalog.</p>
              <JsonViewer value={policy.odrlOffer} />
            </TabsContent>
            {policy.composition && (
              <TabsContent value="composition" className="space-y-2 pt-4">
                <p className="text-xs text-ink/55">
                  The parts it was composed from, as they were given. Composing them again gives the
                  same ODRL while the templates they name stay as they are.
                </p>
                <JsonViewer value={policy.composition} />
              </TabsContent>
            )}
            {policy.instantiationParameters != null && (
              <TabsContent value="parameters" className="pt-4">
                <JsonViewer value={policy.instantiationParameters} />
              </TabsContent>
            )}
            <TabsContent value="record" className="pt-4">
              <JsonViewer value={policy} />
            </TabsContent>
          </Tabs>
        </div>
      </DialogContent>
    </Dialog>
  );
}
