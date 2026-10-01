import React, { useState, useMemo } from "react";
import { createFileRoute, useNavigate, Link } from "@tanstack/react-router";
import { toast } from "sonner";
import {
  ArrowLeft,
  ArrowRight,
  Check,
  CheckCircle2,
  Database,
  Layers,
  ShieldCheck,
  Plus,
  Trash2,
  Code2,
  Loader2,
  Sparkles,
} from "lucide-react";

import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  CardFooter,
} from "shared/src/components/ui/card";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { Textarea } from "shared/src/components/ui/textarea";
import { Badge } from "shared/src/components/ui/badge";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "shared/src/components/ui/select";

import {
  useCreateDatasetOffering,
  useGetMainCatalogs,
} from "shared/src/data/orval/catalogs/catalogs";
import { useGetDataServicesByCatalogId } from "shared/src/data/orval/data-services/data-services";
import type { CreateDatasetOfferingRequest } from "shared/src/data/orval/model";

export const Route = createFileRoute("/my-catalog/new")({
  component: NewDatasetOfferingPage,
});

type WizardStep = 1 | 2 | 3 | 4;

interface ConstraintRow {
  id: string;
  leftOperand: string;
  operator: string;
  rightOperand: string;
}

function NewDatasetOfferingPage() {
  const navigate = useNavigate();

  // 1. Wizard navigation state
  const [currentStep, setCurrentStep] = useState<WizardStep>(1);

  // 2. Fetch main catalog for context and data services
  const { data: mainCatalogData } = useGetMainCatalogs();
  const catalogId =
    mainCatalogData && "id" in ((mainCatalogData.data as any) || {})
      ? ((mainCatalogData.data as any).id as string)
      : "";

  // 3. Fetch data services under main catalog
  const { data: dataServicesData } = useGetDataServicesByCatalogId(catalogId, {
    query: { enabled: !!catalogId },
  });
  const dataServices: any[] =
    dataServicesData && Array.isArray(dataServicesData.data) ? dataServicesData.data : [];

  // 4. Form State - Step 1: Dataset Metadata
  const [datasetTitle, setDatasetTitle] = useState("");
  const [datasetDescription, setDatasetDescription] = useState("");
  const [datasetConformsTo, setDatasetConformsTo] = useState("https://w3id.org/dspace/v0.8/dcat");
  const [datasetCreator, setDatasetCreator] = useState("");

  // 5. Form State - Step 2: Distribution Profile
  const [distributionTitle, setDistributionTitle] = useState("");
  const [distributionDescription, setDistributionDescription] = useState("");
  const [distributionFormat, setDistributionFormat] = useState("application/json");
  const [selectedDataServiceId, setSelectedDataServiceId] = useState<string>("");

  // 6. Form State - Step 3: ODRL Policy
  const [enablePolicy, setEnablePolicy] = useState(true);
  const [policyPreset, setPolicyPreset] = useState<
    "permissive" | "non-commercial" | "read-only" | "custom"
  >("permissive");
  const [policyDescription, setPolicyDescription] = useState("Permissive Data Usage Policy");
  const [policyAction, setPolicyAction] = useState("http://www.w3.org/ns/odrl/2/use");
  const [policyProfile, setPolicyProfile] = useState("http://www.w3.org/ns/odrl/2/");
  const [constraints, setConstraints] = useState<ConstraintRow[]>([]);

  // 7. Result state upon success
  const [createdResult, setCreatedResult] = useState<any | null>(null);

  // Auto-fill the distribution name based on dataset title
  const handleDatasetTitleChange = (val: string) => {
    setDatasetTitle(val);
    if (!distributionTitle || distributionTitle === `${datasetTitle} Distribution`) {
      setDistributionTitle(`${val} Distribution`);
    }
  };

  // Handle Policy Preset changes
  const applyPolicyPreset = (preset: "permissive" | "non-commercial" | "read-only" | "custom") => {
    setPolicyPreset(preset);
    if (preset === "permissive") {
      setPolicyAction("http://www.w3.org/ns/odrl/2/use");
      setPolicyDescription("Full unrestricted use of the dataset");
      setConstraints([]);
    } else if (preset === "non-commercial") {
      setPolicyAction("http://www.w3.org/ns/odrl/2/use");
      setPolicyDescription("Non-commercial research use only");
      setConstraints([
        {
          id: "c1",
          leftOperand: "http://www.w3.org/ns/odrl/2/purpose",
          operator: "http://www.w3.org/ns/odrl/2/eq",
          rightOperand: "ResearchAndDevelopment",
        },
      ]);
    } else if (preset === "read-only") {
      setPolicyAction("http://www.w3.org/ns/odrl/2/read");
      setPolicyDescription("Read and query permissions without redistribution");
      setConstraints([]);
    }
  };

  const addConstraint = () => {
    setConstraints((prev) => [
      ...prev,
      {
        id: Math.random().toString(36).substring(2, 9),
        leftOperand: "http://www.w3.org/ns/odrl/2/spatial",
        operator: "http://www.w3.org/ns/odrl/2/eq",
        rightOperand: "https://publications.europa.eu/resource/authority/country/ESP",
      },
    ]);
  };

  const removeConstraint = (id: string) => {
    setConstraints((prev) => prev.filter((c) => c.id !== id));
  };

  const updateConstraint = (id: string, field: keyof ConstraintRow, val: string) => {
    setConstraints((prev) => prev.map((c) => (c.id === id ? { ...c, [field]: val } : c)));
  };

  // Build the complete offering payload for review and submit
  const payload: CreateDatasetOfferingRequest = useMemo(() => {
    const offering: CreateDatasetOfferingRequest = {
      dataset: {
        title: datasetTitle.trim(),
        description: datasetDescription.trim() || undefined,
        conformsTo: datasetConformsTo.trim() || undefined,
        creator: datasetCreator.trim() || undefined,
        catalogId: catalogId || undefined,
      },
      distribution: {
        title: (distributionTitle.trim() || `${datasetTitle} Distribution`).trim(),
        description: distributionDescription.trim() || undefined,
        formats: distributionFormat.trim() || undefined,
        accessServiceId: selectedDataServiceId || undefined,
      },
    };

    if (enablePolicy) {
      offering.policy = {
        description: policyDescription.trim() || undefined,
        action: policyAction.trim() || undefined,
        profile: policyProfile.trim() || undefined,
        constraints:
          constraints.length > 0
            ? constraints.map((c) => ({
                leftOperand: c.leftOperand,
                operator: c.operator,
                rightOperand: c.rightOperand,
              }))
            : undefined,
      };
    }

    return offering;
  }, [
    datasetTitle,
    datasetDescription,
    datasetConformsTo,
    datasetCreator,
    catalogId,
    distributionTitle,
    distributionDescription,
    distributionFormat,
    selectedDataServiceId,
    enablePolicy,
    policyDescription,
    policyAction,
    policyProfile,
    constraints,
  ]);

  // Mutation
  const createOfferingMutation = useCreateDatasetOffering({
    mutation: {
      onSuccess: (res) => {
        if (res.status === 201) {
          toast.success("Dataset offering published successfully!");
          setCreatedResult(res.data);
        } else {
          toast.error(`Publication failed: ${(res.data as any)?.error || "Unexpected status"}`);
        }
      },
      onError: (err: any) => {
        toast.error(`Error publishing dataset offering: ${err.message || String(err)}`);
      },
    },
  });

  const handleSubmit = async () => {
    if (!datasetTitle.trim()) {
      toast.error("Dataset title is required");
      setCurrentStep(1);
      return;
    }

    createOfferingMutation.mutate({ data: payload });
  };

  const stepsConfig = [
    { num: 1, label: "Dataset Info", icon: Database },
    { num: 2, label: "Distribution", icon: Layers },
    { num: 3, label: "Governance Policy", icon: ShieldCheck },
    { num: 4, label: "Review & Publish", icon: CheckCircle2 },
  ];

  return (
    <PageLayout>
      <PageHeader title="Publish Dataset Offering">
        <div className="flex items-center justify-between mt-1 text-xs text-muted-foreground">
          <span>
            Register a dataset, its distribution and an ODRL policy into the main catalog.
          </span>
          <Link to="/my-catalog">
            <Button variant="outline" size="sm" className="gap-1.5 text-xs h-7">
              <ArrowLeft className="h-3.5 w-3.5" />
              Back to Catalog
            </Button>
          </Link>
        </div>
      </PageHeader>

      <div className="container mx-auto px-4 py-6 max-w-5xl space-y-6">
        {/* Step Indicator Progress Bar */}
        <div className="bg-background-800/40 border border-ink/10 rounded-xl p-4 shadow-sm backdrop-blur-sm">
          <div className="flex items-center justify-between">
            {stepsConfig.map((s, idx) => {
              const Icon = s.icon;
              const isPassed = currentStep > s.num;
              const isCurrent = currentStep === s.num;

              return (
                <React.Fragment key={s.num}>
                  <button
                    type="button"
                    onClick={() => !createdResult && setCurrentStep(s.num as WizardStep)}
                    disabled={!!createdResult}
                    className="flex flex-col sm:flex-row items-center gap-2 group text-left transition-all disabled:opacity-50"
                  >
                    <div
                      className={`flex h-9 w-9 items-center justify-center rounded-xl text-xs font-semibold transition-all shadow-sm ${
                        isCurrent
                          ? "bg-primary text-white ring-2 ring-brand-sky/40 scale-105"
                          : isPassed
                            ? "bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 border border-emerald-500/30"
                            : "bg-background-700/60 text-muted-foreground border border-ink/5 group-hover:border-ink/20"
                      }`}
                    >
                      {isPassed ? (
                        <Check className="h-4 w-4 stroke-[2.5]" />
                      ) : (
                        <Icon className="h-4 w-4" />
                      )}
                    </div>
                    <div className="hidden sm:block">
                      <p
                        className={`text-xs font-medium leading-none ${
                          isCurrent ? "text-foreground font-semibold" : "text-muted-foreground"
                        }`}
                      >
                        Step {s.num}
                      </p>
                      <p className="text-xs text-muted-foreground/80 mt-1">{s.label}</p>
                    </div>
                  </button>

                  {idx < stepsConfig.length - 1 && (
                    <div
                      className={`flex-1 h-[2px] mx-2 sm:mx-4 transition-all ${
                        currentStep > s.num ? "bg-emerald-500/40" : "bg-ink/10"
                      }`}
                    />
                  )}
                </React.Fragment>
              );
            })}
          </div>
        </div>

        {/* Success Screen if already created */}
        {createdResult ? (
          <Card className="border-emerald-500/30 bg-emerald-50 dark:bg-emerald-950/10 backdrop-blur-md">
            <CardHeader className="text-center pb-4">
              <div className="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-emerald-500/20 text-emerald-700 dark:text-emerald-400 border border-emerald-500/30 mb-2">
                <Sparkles className="h-7 w-7" />
              </div>
              <CardTitle className="text-xl text-emerald-700 dark:text-emerald-400 font-bold">
                Dataset Offering Successfully Published!
              </CardTitle>
              <CardDescription className="text-sm text-muted-foreground max-w-lg mx-auto">
                The dataset, its distribution and ODRL policy have been registered in the main
                catalog.
              </CardDescription>
            </CardHeader>

            <CardContent className="space-y-4 max-w-2xl mx-auto">
              <div className="rounded-xl border border-ink/10 bg-background-800/60 p-4 space-y-3">
                <div className="flex items-center justify-between text-xs pb-2 border-b border-ink/5">
                  <span className="text-muted-foreground">Dataset Identifier:</span>
                  <span className="font-mono text-emerald-700 dark:text-emerald-400 font-medium">
                    {createdResult.dataset?.id || "Created"}
                  </span>
                </div>
                <div className="flex items-center justify-between text-xs pb-2 border-b border-ink/5">
                  <span className="text-muted-foreground">Distribution Identifier:</span>
                  <span className="font-mono text-foreground font-medium">
                    {createdResult.distribution?.id || "Created"}
                  </span>
                </div>
                {createdResult.policy && (
                  <div className="flex items-center justify-between text-xs">
                    <span className="text-muted-foreground">ODRL Governance Policy:</span>
                    <span className="font-mono text-foreground font-medium">
                      {createdResult.policy?.id || "Enforced"}
                    </span>
                  </div>
                )}
              </div>

              <div className="flex items-center justify-center gap-3 pt-4">
                <Link to="/my-catalog">
                  <Button variant="default" className="gap-2">
                    <CheckCircle2 className="h-4 w-4" />
                    Go to Myself&apos;s Catalog
                  </Button>
                </Link>
                <Button
                  variant="outline"
                  onClick={() => {
                    setCreatedResult(null);
                    setDatasetTitle("");
                    setDatasetDescription("");
                    setDistributionTitle("");
                    setCurrentStep(1);
                  }}
                >
                  Publish Another Offering
                </Button>
              </div>
            </CardContent>
          </Card>
        ) : (
          /* Wizard Forms */
          <div className="space-y-6">
            {/* STEP 1: DATASET INFO */}
            {currentStep === 1 && (
              <Card className="border-ink/10 bg-background-800/30">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        <Database className="h-5 w-5 text-primary" />
                        Dataset Identity &amp; Metadata
                      </CardTitle>
                      <CardDescription className="text-xs mt-1">
                        Define high-level DCAT metadata for the dataset offering in the catalog.
                      </CardDescription>
                    </div>
                    {catalogId && (
                      <Badge variant="outline" className="text-xs font-mono">
                        Target: {catalogId.length > 24 ? `${catalogId.slice(0, 24)}...` : catalogId}
                      </Badge>
                    )}
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground flex items-center gap-1">
                      Dataset Title <span className="text-rose-500">*</span>
                    </label>
                    <Input
                      placeholder="e.g. European Renewable Energy Production Q1-2026"
                      value={datasetTitle}
                      onChange={(e) => handleDatasetTitleChange(e.target.value)}
                      className="text-sm"
                    />
                    <p className="text-xs text-muted-foreground">
                      Human-readable name used across dataspace participant catalogs.
                    </p>
                  </div>

                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">
                      Description / Abstract
                    </label>
                    <Textarea
                      placeholder="Detailed overview of dataset contents, temporal scope, sampling rate, and domain context..."
                      rows={3}
                      value={datasetDescription}
                      onChange={(e) => setDatasetDescription(e.target.value)}
                      className="text-sm"
                    />
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-1.5">
                      <label className="text-xs font-semibold text-foreground">
                        Conforms To (DCAT Profile Standard)
                      </label>
                      <Input
                        value={datasetConformsTo}
                        onChange={(e) => setDatasetConformsTo(e.target.value)}
                        placeholder="https://w3id.org/dspace/v0.8/dcat"
                        className="text-xs font-mono"
                      />
                    </div>

                    <div className="space-y-1.5">
                      <label className="text-xs font-semibold text-foreground">
                        Creator / Publisher DID
                      </label>
                      <Input
                        value={datasetCreator}
                        onChange={(e) => setDatasetCreator(e.target.value)}
                        placeholder="e.g. did:web:participant.example.org"
                        className="text-xs font-mono"
                      />
                    </div>
                  </div>
                </CardContent>

                <CardFooter className="flex justify-between border-t border-ink/5 pt-4">
                  <Link to="/my-catalog">
                    <Button variant="ghost" size="sm" className="text-xs">
                      Cancel
                    </Button>
                  </Link>
                  <Button
                    size="sm"
                    variant="default"
                    disabled={!datasetTitle.trim()}
                    onClick={() => setCurrentStep(2)}
                    className="gap-2 text-xs"
                  >
                    Next: Distribution Profile
                    <ArrowRight className="h-3.5 w-3.5" />
                  </Button>
                </CardFooter>
              </Card>
            )}

            {/* STEP 2: DISTRIBUTION */}
            {currentStep === 2 && (
              <Card className="border-ink/10 bg-background-800/30">
                <CardHeader>
                  <CardTitle className="text-lg flex items-center gap-2">
                    <Layers className="h-5 w-5 text-primary" />
                    Distribution &amp; Data Service
                  </CardTitle>
                  <CardDescription className="text-xs mt-1">
                    Configure the technical representation, serialization format, and data service
                    binding.
                  </CardDescription>
                </CardHeader>

                <CardContent className="space-y-4">
                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground flex items-center gap-1">
                      Distribution Title <span className="text-rose-500">*</span>
                    </label>
                    <Input
                      placeholder="e.g. JSON Streaming Distribution"
                      value={distributionTitle}
                      onChange={(e) => setDistributionTitle(e.target.value)}
                      className="text-sm"
                    />
                  </div>

                  <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-foreground">Description</label>
                    <Input
                      placeholder="e.g. High-throughput JSON payload over HTTPS pull endpoint"
                      value={distributionDescription}
                      onChange={(e) => setDistributionDescription(e.target.value)}
                      className="text-sm"
                    />
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div className="space-y-1.5">
                      <label className="text-xs font-semibold text-foreground">
                        Serialization Format / MIME
                      </label>
                      <Select value={distributionFormat} onValueChange={setDistributionFormat}>
                        <SelectTrigger className="text-xs">
                          <SelectValue placeholder="Select format" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="application/json">application/json (JSON)</SelectItem>
                          <SelectItem value="text/csv">text/csv (CSV)</SelectItem>
                          <SelectItem value="application/parquet">
                            application/parquet (Apache Parquet)
                          </SelectItem>
                          <SelectItem value="application/octet-stream">
                            application/octet-stream (Binary)
                          </SelectItem>
                          <SelectItem value="application/xml">application/xml (XML)</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-1.5">
                      <label className="text-xs font-semibold text-foreground flex items-center justify-between">
                        <span>Access Data Service</span>
                        <span className="text-xs text-muted-foreground">
                          Auto-discovered if empty
                        </span>
                      </label>
                      <Select
                        value={selectedDataServiceId}
                        onValueChange={setSelectedDataServiceId}
                      >
                        <SelectTrigger className="text-xs">
                          <SelectValue placeholder="Auto-bind to Main Data Service" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="none">Auto-discover Main Data Service</SelectItem>
                          {dataServices.map((ds) => (
                            <SelectItem key={ds.id} value={ds.id}>
                              {ds.dctTitle || ds.id}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </CardContent>

                <CardFooter className="flex justify-between border-t border-ink/5 pt-4">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setCurrentStep(1)}
                    className="gap-2 text-xs"
                  >
                    <ArrowLeft className="h-3.5 w-3.5" />
                    Back
                  </Button>
                  <Button
                    size="sm"
                    variant="default"
                    disabled={!distributionTitle.trim()}
                    onClick={() => setCurrentStep(3)}
                    className="gap-2 text-xs"
                  >
                    Next: Governance Policy
                    <ArrowRight className="h-3.5 w-3.5" />
                  </Button>
                </CardFooter>
              </Card>
            )}

            {/* STEP 3: GOVERNANCE POLICY */}
            {currentStep === 3 && (
              <Card className="border-ink/10 bg-background-800/30">
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        <ShieldCheck className="h-5 w-5 text-primary" />
                        ODRL Governance &amp; Access Policy
                      </CardTitle>
                      <CardDescription className="text-xs mt-1">
                        Define usage permissions, legal profiles, and contract constraints for
                        dataspace consumers.
                      </CardDescription>
                    </div>
                    <Button
                      variant={enablePolicy ? "default" : "outline"}
                      size="sm"
                      onClick={() => setEnablePolicy(!enablePolicy)}
                      className="text-xs h-7"
                    >
                      {enablePolicy ? "Enforced" : "Disabled"}
                    </Button>
                  </div>
                </CardHeader>

                <CardContent className="space-y-4">
                  {!enablePolicy ? (
                    <div className="rounded-xl border border-dashed border-ink/10 p-6 text-center text-muted-foreground text-xs">
                      Policy enforcement is disabled for this dataset offering.
                    </div>
                  ) : (
                    <>
                      {/* Presets Grid */}
                      <div className="space-y-2">
                        <label className="text-xs font-semibold text-foreground">
                          Quick Presets
                        </label>
                        <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
                          {[
                            { id: "permissive", title: "Permissive Use", desc: "Full odrl:use" },
                            {
                              id: "non-commercial",
                              title: "Non-Commercial",
                              desc: "Research only",
                            },
                            { id: "read-only", title: "Read Only", desc: "odrl:read action" },
                            { id: "custom", title: "Custom", desc: "Manual rules" },
                          ].map((p) => (
                            <button
                              key={p.id}
                              type="button"
                              onClick={() => applyPolicyPreset(p.id as any)}
                              className={`p-3 rounded-xl border text-left transition-all ${
                                policyPreset === p.id
                                  ? "border-primary bg-primary/10 ring-1 ring-primary"
                                  : "border-ink/10 bg-background-800/40 hover:border-ink/20"
                              }`}
                            >
                              <p className="text-xs font-semibold text-foreground">{p.title}</p>
                              <p className="text-xs text-muted-foreground mt-0.5">{p.desc}</p>
                            </button>
                          ))}
                        </div>
                      </div>

                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-1.5">
                          <label className="text-xs font-semibold text-foreground">
                            Permission Action URI
                          </label>
                          <Input
                            value={policyAction}
                            onChange={(e) => setPolicyAction(e.target.value)}
                            placeholder="http://www.w3.org/ns/odrl/2/use"
                            className="text-xs font-mono"
                          />
                        </div>

                        <div className="space-y-1.5">
                          <label className="text-xs font-semibold text-foreground">
                            ODRL Profile Specification
                          </label>
                          <Input
                            value={policyProfile}
                            onChange={(e) => setPolicyProfile(e.target.value)}
                            placeholder="http://www.w3.org/ns/odrl/2/"
                            className="text-xs font-mono"
                          />
                        </div>
                      </div>

                      <div className="space-y-1.5">
                        <label className="text-xs font-semibold text-foreground">
                          Policy Description
                        </label>
                        <Input
                          value={policyDescription}
                          onChange={(e) => setPolicyDescription(e.target.value)}
                          placeholder="e.g. Standard Open Data Commons License"
                          className="text-sm"
                        />
                      </div>

                      {/* Dynamic Constraints List */}
                      <div className="space-y-2 pt-2 border-t border-ink/5">
                        <div className="flex items-center justify-between">
                          <label className="text-xs font-semibold text-foreground">
                            Access Constraints ({constraints.length})
                          </label>
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            onClick={addConstraint}
                            className="h-7 text-xs gap-1"
                          >
                            <Plus className="h-3 w-3" />
                            Add Constraint
                          </Button>
                        </div>

                        {constraints.length === 0 ? (
                          <div className="rounded-lg border border-ink/5 bg-background-800/20 p-3 text-center text-xs text-muted-foreground">
                            No constraint rules defined. Action is unconditionally allowed.
                          </div>
                        ) : (
                          <div className="space-y-2">
                            {constraints.map((c) => (
                              <div
                                key={c.id}
                                className="flex items-center gap-2 p-2.5 rounded-lg border border-ink/10 bg-background-800/60"
                              >
                                <Input
                                  value={c.leftOperand}
                                  onChange={(e) =>
                                    updateConstraint(c.id, "leftOperand", e.target.value)
                                  }
                                  placeholder="Left operand (e.g. odrl:spatial)"
                                  className="h-8 text-xs font-mono flex-1"
                                />
                                <Select
                                  value={c.operator}
                                  onValueChange={(v) => updateConstraint(c.id, "operator", v)}
                                >
                                  <SelectTrigger className="h-8 text-xs font-mono w-32">
                                    <SelectValue />
                                  </SelectTrigger>
                                  <SelectContent>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/eq">
                                      odrl:eq
                                    </SelectItem>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/neq">
                                      odrl:neq
                                    </SelectItem>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/lt">
                                      odrl:lt
                                    </SelectItem>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/lteq">
                                      odrl:lteq
                                    </SelectItem>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/gt">
                                      odrl:gt
                                    </SelectItem>
                                    <SelectItem value="http://www.w3.org/ns/odrl/2/gteq">
                                      odrl:gteq
                                    </SelectItem>
                                  </SelectContent>
                                </Select>
                                <Input
                                  value={c.rightOperand}
                                  onChange={(e) =>
                                    updateConstraint(c.id, "rightOperand", e.target.value)
                                  }
                                  placeholder="Right operand (e.g. Research)"
                                  className="h-8 text-xs font-mono flex-1"
                                />
                                <Button
                                  type="button"
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => removeConstraint(c.id)}
                                  className="h-8 w-8 p-0 text-muted-foreground hover:text-rose-700 dark:hover:text-rose-400"
                                >
                                  <Trash2 className="h-3.5 w-3.5" />
                                </Button>
                              </div>
                            ))}
                          </div>
                        )}
                      </div>
                    </>
                  )}
                </CardContent>

                <CardFooter className="flex justify-between border-t border-ink/5 pt-4">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setCurrentStep(2)}
                    className="gap-2 text-xs"
                  >
                    <ArrowLeft className="h-3.5 w-3.5" />
                    Back
                  </Button>
                  <Button
                    size="sm"
                    variant="default"
                    onClick={() => setCurrentStep(4)}
                    className="gap-2 text-xs"
                  >
                    Review &amp; Publish
                    <ArrowRight className="h-3.5 w-3.5" />
                  </Button>
                </CardFooter>
              </Card>
            )}

            {/* STEP 4: REVIEW & PUBLISH */}
            {currentStep === 4 && (
              <div className="space-y-6">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  {/* Summary Card 1: Dataset */}
                  <Card className="border-ink/10 bg-background-800/30">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-sm font-semibold flex items-center gap-2">
                          <Database className="h-4 w-4 text-brand-sky" />
                          Dataset Metadata
                        </CardTitle>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setCurrentStep(1)}
                          className="text-xs h-6 px-2"
                        >
                          Edit
                        </Button>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-2 text-xs">
                      <div>
                        <span className="text-muted-foreground">Title:</span>{" "}
                        <span className="font-semibold text-foreground">{datasetTitle}</span>
                      </div>
                      {datasetDescription && (
                        <div className="text-muted-foreground text-xs line-clamp-2">
                          {datasetDescription}
                        </div>
                      )}
                      <div className="text-xs font-mono text-muted-foreground break-all">
                        Standard: {datasetConformsTo}
                      </div>
                    </CardContent>
                  </Card>

                  {/* Summary Card 2: Distribution */}
                  <Card className="border-ink/10 bg-background-800/30">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-sm font-semibold flex items-center gap-2">
                          <Layers className="h-4 w-4 text-emerald-700 dark:text-emerald-400" />
                          Distribution Details
                        </CardTitle>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setCurrentStep(2)}
                          className="text-xs h-6 px-2"
                        >
                          Edit
                        </Button>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-2 text-xs">
                      <div>
                        <span className="text-muted-foreground">Title:</span>{" "}
                        <span className="font-semibold text-foreground">{distributionTitle}</span>
                      </div>
                      <div className="flex items-center gap-2">
                        <span className="text-muted-foreground">MIME:</span>
                        <Badge variant="code" className="text-xs">
                          {distributionFormat}
                        </Badge>
                      </div>
                    </CardContent>
                  </Card>

                  {/* Summary Card 3: Policy */}
                  <Card className="border-ink/10 bg-background-800/30">
                    <CardHeader className="pb-3">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-sm font-semibold flex items-center gap-2">
                          <ShieldCheck className="h-4 w-4 text-purple-700 dark:text-purple-400" />
                          ODRL Governance
                        </CardTitle>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => setCurrentStep(3)}
                          className="text-xs h-6 px-2"
                        >
                          Edit
                        </Button>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-2 text-xs">
                      {enablePolicy ? (
                        <>
                          <div className="text-foreground">{policyDescription}</div>
                          <div className="text-xs font-mono text-muted-foreground break-all">
                            Action: {policyAction}
                          </div>
                          <div className="flex items-center gap-2 pt-1">
                            <span className="text-muted-foreground text-xs">Constraints:</span>
                            <Badge variant="code" className="text-xs">
                              {constraints.length} rule{constraints.length === 1 ? "" : "s"}
                            </Badge>
                          </div>
                        </>
                      ) : (
                        <span className="text-muted-foreground text-xs italic">
                          Policy enforcement disabled
                        </span>
                      )}
                    </CardContent>
                  </Card>
                </div>

                {/* Atomic Action Card */}
                <Card className="border-primary/30 bg-primary/5">
                  <CardContent className="p-6 flex flex-col sm:flex-row items-center justify-between gap-4">
                    <div className="space-y-1 text-center sm:text-left">
                      <h4 className="text-sm font-bold text-foreground">
                        Ready to Register in Main Catalog
                      </h4>
                      <p className="text-xs text-muted-foreground">
                        This atomic operation creates all 4 resources and emits a{" "}
                        <code className="text-brand-sky">catalog.dataset.created</code> event.
                      </p>
                    </div>

                    <div className="flex items-center gap-3 w-full sm:w-auto">
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => setCurrentStep(3)}
                        disabled={createOfferingMutation.isPending}
                        className="text-xs"
                      >
                        <ArrowLeft className="h-3.5 w-3.5 mr-1" />
                        Back
                      </Button>
                      <Button
                        variant="default"
                        size="sm"
                        onClick={handleSubmit}
                        disabled={createOfferingMutation.isPending}
                        className="gap-2 text-xs shadow-lg flex-1 sm:flex-none"
                      >
                        {createOfferingMutation.isPending ? (
                          <>
                            <Loader2 className="h-4 w-4 animate-spin" />
                            Publishing Offering...
                          </>
                        ) : (
                          <>
                            <Sparkles className="h-4 w-4" />
                            Publish to Main Catalog
                          </>
                        )}
                      </Button>
                    </div>
                  </CardContent>
                </Card>

                {/* Inspect JSON Payload */}
                <details className="rounded-xl border border-ink/10 bg-background-800/60 p-4 text-xs">
                  <summary className="cursor-pointer font-medium text-muted-foreground hover:text-foreground flex items-center gap-2">
                    <Code2 className="h-4 w-4" />
                    Inspect Generated Offering Payload (JSON)
                  </summary>
                  <pre className="mt-3 p-3 rounded-lg bg-brand-black/90 font-mono text-xs text-brand-sky overflow-x-auto border border-ink/5">
                    {JSON.stringify(payload, null, 2)}
                  </pre>
                </details>
              </div>
            )}
          </div>
        )}
      </div>
    </PageLayout>
  );
}
