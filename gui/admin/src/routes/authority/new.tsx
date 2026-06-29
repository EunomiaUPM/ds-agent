import { AlertCircle, CheckCircle2, Info, Loader2, Search } from 'lucide-react';
import { useEffect, useRef, useState } from 'react';
import { useForm } from 'react-hook-form';
import { PageHeader } from 'shared/src/components/layout/PageHeader';
import { PageLayout } from 'shared/src/components/layout/PageLayout';
import { PageSection } from 'shared/src/components/layout/PageSection';
import { Badge } from 'shared/src/components/ui/badge';
import { Button } from 'shared/src/components/ui/button';
import {
    Card, CardContent, CardDescription, CardHeader, CardTitle
} from 'shared/src/components/ui/card';
import {
    Form, FormControl, FormDescription, FormField, FormItem, FormLabel, FormMessage
} from 'shared/src/components/ui/form';
import { Input } from 'shared/src/components/ui/input';
import {
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue
} from 'shared/src/components/ui/select';
import {
    Tooltip, TooltipContent, TooltipProvider, TooltipTrigger
} from 'shared/src/components/ui/tooltip';
import WizardDialog from 'shared/src/components/WizardDialog';
import { customInstance } from 'shared/src/data/orval-mutator';
import { useGetAllParticipants } from 'shared/src/data/orval/participants/participants';
import { useFederatedCatalog } from 'shared/src/data/useFederatedCatalog';
import { formatIdentifier, getFriendlyVCType } from 'shared/src/lib/utils';
import * as z from 'zod';

import { zodResolver } from '@hookform/resolvers/zod';
import { createFileRoute, useNavigate } from '@tanstack/react-router';

const schema = z.object({
  url: z.string().url("Please enter a valid URL"),
  nick: z.string().min(1, "Name is required"),
  vc_type: z.string().min(1, "Please select a VC type"),
  method: z.enum(["oidc4vp", "cert"]),
  auto: z.boolean().default(true),
});

interface DidService {
  id?: string;
  type: string;
  serviceEndpoint: string;
}

const DEMO_AUTHORITY_URL = "https://dev-dataspaces.dit.upm.es";
const DEMO_AUTHORITY_NICK = "Heimdall";
const DEMO_VC_TYPE_ID = "DataSpaceParticipant_jwt_vc_json";
const DEMO_VC_TYPE_LABEL = "Data Space Participant (JWT)";

type FormValues = z.infer<typeof schema>;

function LabelWithInfo({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center gap-1.5">
      <FormLabel>{label}</FormLabel>
      <TooltipProvider delayDuration={150}>
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              type="button"
              className="text-muted-foreground hover:text-foreground transition-colors"
              aria-label={`What is ${label}?`}
            >
              <Info className="h-3.5 w-3.5" />
            </button>
          </TooltipTrigger>
          <TooltipContent
            side="top"
            className="max-w-xs bg-background-300 border border-secondary-800 text-white text-xs leading-relaxed p-3"
          >
            {children}
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    </div>
  );
}

// @ts-ignore
export const Route = createFileRoute("/authority/new")({
  validateSearch: (search: Record<string, unknown>) => ({
    recommended:
      typeof search?.recommended === "string" ? (search.recommended as string) : undefined,
  }),
  component: NewAuthorityRequest,
});

function NewAuthorityRequest() {
  const navigate = useNavigate();
  const search = Route.useSearch();
  const recommended = search.recommended ?? DEMO_VC_TYPE_ID;
  const recommendedMethod: "cert" | "oidc4vp" = recommended.startsWith(
    "gx_LabelCredential",
  )
    ? "oidc4vp"
    : "cert";
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [discoveredInfo, setDiscoveredInfo] = useState<{
    id: string;
    vc_types: string[];
    services: DidService[];
  } | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);
  const federated = useFederatedCatalog();

  // wizard state: 0 = closed, 1 = URL/Nickname, 2 = Discover, 3 = VC type
  const [wizardStep, setWizardStep] = useState<0 | 1 | 2 | 3>(1);
  const vcTypeLabelRef = useRef<HTMLElement | null>(null);
  const discoverButtonRef = useRef<HTMLButtonElement | null>(null);

  const { data: participantsResponse } = useGetAllParticipants();
  const knownAuthorities =
    participantsResponse?.status === 200
      ? participantsResponse.data.filter((p) => p.participant_type === "Authority")
      : [];

  const form = useForm<FormValues>({
    resolver: zodResolver(schema) as any,
    defaultValues: {
      url: "",
      nick: "",
      vc_type: "",
      method: recommendedMethod,
      auto: true,
    },
  });

  const url = form.watch("url");
  const nick = form.watch("nick");
  // refs for positioning the tooltip over the "Authority URL" label
  const labelURLRef = useRef<HTMLElement | null>(null);
  const vcType = form.watch("vc_type");

  // advance wizard from "Discover" step to "VC type" step when discovery succeeds
  useEffect(() => {
    if (discoveredInfo && wizardStep === 2) {
      setWizardStep(3);
    }
  }, [discoveredInfo, wizardStep]);

  // close wizard once user picks a VC type
  useEffect(() => {
    if (vcType && wizardStep === 3) {
      setWizardStep(0);
    }
  }, [vcType, wizardStep]);

  const handleDiscovery = async (optionalUrl?: string) => {
    const targetUrl = typeof optionalUrl === "string" ? optionalUrl : url;
    if (!targetUrl || !targetUrl.startsWith("http")) return;

    setIsDiscovering(true);
    setDiscoveryError(null);
    try {
      const cleanUrl = targetUrl.replace(/\/$/, "");

      const didResponse = await fetch(`${cleanUrl}/.well-known/did.json`);
      if (!didResponse.ok) throw new Error("Failed to fetch DID document");
      const didJson = await didResponse.json();
      const id = didJson.id;

      const issuerResponse = await fetch(`${cleanUrl}/.well-known/openid-credential-issuer`);
      if (!issuerResponse.ok) throw new Error("Failed to fetch Credential Issuer configuration");
      const issuerJson = await issuerResponse.json();

      const vcTypes = Object.keys(issuerJson.credential_configurations_supported || {});
      const services = (didJson.service || []) as DidService[];

      setDiscoveredInfo({ id, vc_types: vcTypes, services });
    } catch (err: any) {
      console.error(err);
      setDiscoveryError(err.message || "Could not discover authority info");
    } finally {
      setIsDiscovering(false);
    }
  };

  const onSubmit = async (values: FormValues) => {
    if (!discoveredInfo) {
      return;
    }

    setIsSubmitting(true);
    try {
      const issuerService = discoveredInfo.services.find((s) => s.type === "CredentialIssuer");
      const targetUrl = issuerService?.serviceEndpoint || values.url;

      await customInstance(`/vc-request/beg`, {
        method: "POST",
        data: {
          ...values,
          id: discoveredInfo.id,
          url: targetUrl,
        },
      });

      // mark that the user just authenticated via the new-authority flow
      try {
        sessionStorage.setItem("justJoinedDataspace", "true");
      } catch (e) {
        /* ignore if unavailable */
      }

      (navigate as any)({ to: "/authority" });
    } catch (err) {
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  // federated presence check (kept for clarity)

  let highlightRingClasses =
    knownAuthorities.length === 0 ? "ring-2 ring-secondary-400 shadow-md animate-pulse" : "";
  let highlightButtonClasses =
    knownAuthorities.length === 0
      ? "animate-pulse bg-secondary-600 hover:bg-secondary-500 ring-2 ring-secondary-400"
      : "";

  return (
    <PageLayout>
      <PageHeader title="Request New Credential" />
      {knownAuthorities.length === 0 ? (
        <>
          <WizardDialog
            open={wizardStep === 1}
            step={"1 of 3"}
            onClose={() => setWizardStep(0)}
            anchorRef={labelURLRef}
            align="left"
            title="Connect to a Dataspace"
            sectionTitle="Dataspace Sign Up Tutorial"
            content={<>Introduce the URL of the authority of the dataspace you want to join.</>}
          >
            <div className="flex justify-end mt-3">
              <Button
                type="button"
                size="sm"
                onClick={() => {
                  form.setValue("url", DEMO_AUTHORITY_URL);
                  form.setValue("nick", DEMO_AUTHORITY_NICK);
                  setWizardStep(2);
                }}
              >
                Insert predefined data
              </Button>
            </div>
          </WizardDialog>

          <WizardDialog
            open={wizardStep === 2}
            step={"2 of 3"}
            onClose={() => setWizardStep(0)}
            anchorRef={discoverButtonRef as React.RefObject<HTMLElement>}
            align="left"
            title="Discover the authority"
            sectionTitle="Dataspace Sign Up Tutorial"
            content={
              <>
                Click <strong>Find authority</strong> to fetch the authority's DID document and the
                list of credentials it can issue.
                <br />
                Once it succeeds, the panel on the right will show the authority's{" "}
                <strong>DID</strong>, its <strong>available VC types</strong> and its{" "}
                <strong>services</strong>.
              </>
            }
          />

          <WizardDialog
            open={wizardStep === 3}
            sectionTitle="Dataspace Sign Up Tutorial"
            onClose={() => setWizardStep(0)}
            anchorRef={vcTypeLabelRef}
            align="left"
            step={"3 of 3"}
            title="Choose a credential type"
            content={
              <>
                Select which Verifiable Credential type you want to request. For the demo, pick{" "}
                <span className="font-mono text-sky-600">{DEMO_VC_TYPE_LABEL}</span>.
                <br />
                If none are available, try a different authority or contact the provider.
              </>
            }
          />
        </>
      ) : null}
      <PageSection>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Authority Connection</CardTitle>
                <CardDescription className="">
                  Enter the authority base URL to discover its profile and available credentials.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Form {...form}>
                  <form onSubmit={form.handleSubmit(onSubmit as any)} className="space-y-6">
                    <FormField
                      control={form.control as any}
                      name="url"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel ref={(el) => (labelURLRef.current = el as any)}>
                            Authority URL
                          </FormLabel>
                          <div className="flex items-center gap-2">
                            <FormControl className="flex-1">
                              <Input
                                placeholder="https://authority.example.com"
                                list="known-authorities"
                                {...field}
                                className={`${!url ? highlightRingClasses : ""}`}
                                onChange={(e) => {
                                  field.onChange(e);
                                  const val = e.target.value;
                                  const known = knownAuthorities.find((a) => a.base_url === val);
                                  if (known) {
                                    if (known.participant_slug) {
                                      form.setValue("nick", known.participant_slug);
                                    }
                                    handleDiscovery(val);
                                  }
                                }}
                              />
                            </FormControl>
                            <datalist id="known-authorities">
                              {knownAuthorities.map((authority) => (
                                <option key={authority.participant_id} value={authority.base_url}>
                                  {authority.participant_slug || authority.participant_id}
                                </option>
                              ))}
                            </datalist>
                            <Button
                              ref={discoverButtonRef as any}
                              type="button"
                              variant="secondary"
                              size={"sm"}
                              onClick={() => handleDiscovery()}
                              disabled={isDiscovering || !url}
                            >
                              {isDiscovering ? (
                                <Loader2 className="animate-spin h-4 w-4 mr-2" />
                              ) : (
                                <Search />
                              )}
                              Find authority
                            </Button>
                          </div>
                          <FormMessage />
                          <FormDescription>
                            Example: http://host.docker.internal:1500
                          </FormDescription>
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control as any}
                      name="nick"
                      render={({ field }) => (  
                        <FormItem>
                          <FormLabel>Nickname</FormLabel>
                          <FormControl>
                            <Input placeholder="Heimdall" {...field} />
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control as any}
                      name="vc_type"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel ref={(el) => (vcTypeLabelRef.current = el as any)}>
                            Verifiable Credential Type
                          </FormLabel>

                          <Select
                            onValueChange={field.onChange}
                            defaultValue={field.value}
                            disabled={!discoveredInfo}
                          >
                            <FormControl>
                              <SelectTrigger
                                className={discoveredInfo && !vcType ? highlightRingClasses : ""}
                              >
                                <SelectValue
                                  placeholder={
                                    discoveredInfo ? "Select VC type" : "Discover first..."
                                  }
                                />
                              </SelectTrigger>
                            </FormControl>
                            <SelectContent>
                              {discoveredInfo?.vc_types.map((type: string) => {
                                const isRecommended = type === recommended;
                                return (
                                  <SelectItem
                                    key={type}
                                    value={type}
                                    className={
                                      isRecommended
                                        ? "bg-secondary-700/30 ring-1 ring-secondary-400 my-1"
                                        : ""
                                    }
                                  >
                                    <span className="flex items-center gap-2">
                                      {getFriendlyVCType(type)}
                                      {isRecommended && (
                                        <Badge variant="info" className="text-[10px]">
                                          Recommended
                                        </Badge>
                                      )}
                                    </span>
                                  </SelectItem>
                                );
                              })}
                            </SelectContent>
                          </Select>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control as any}
                      name="method"
                      render={({ field }: { field: any }) => (
                        <FormItem>
                          <LabelWithInfo label="Identity Proof">
                            The authority needs to verify who you are before issuing a credential.
                            <br />
                            Use a <strong>Certificate</strong> if this is your first onboarding (a
                            pre-issued X.509). Switch to <strong>Verifiable Credential</strong>{" "}
                            later, once your wallet holds something the authority can check.
                          </LabelWithInfo>
                          <FormControl>
                            <div className="space-y-2">
                              <div className="relative flex p-1 bg-muted/50 border border-primary/40 rounded-lg w-full">
                                {/* Sliding pill */}
                                <div
                                  className={`absolute top-1 bottom-1 w-[calc(50%-0.25rem)] bg-primary rounded-md transition-transform duration-300 ease-in-out shadow-sm ${
                                    field.value === "cert"
                                      ? "translate-x-0"
                                      : "translate-x-[calc(100%+0.25rem)]"
                                  }`}
                                />

                                <button
                                  type="button"
                                  className={`relative z-10 w-1/2 py-2 px-3 text-xs md:text-sm font-semibold transition-all duration-300 rounded-md ${
                                    field.value === "cert"
                                      ? "text-primary-foreground"
                                      : "text-foreground/70 hover:text-foreground"
                                  } ${
                                    recommendedMethod === "cert" && field.value !== "cert"
                                      ? "ring-2 ring-secondary-400"
                                      : ""
                                  }`}
                                  onClick={() => field.onChange("cert")}
                                >
                                  Certificate
                                </button>

                                <button
                                  type="button"
                                  className={`relative z-10 w-1/2 py-2 px-3 text-xs md:text-sm font-semibold transition-all duration-300 rounded-md ${
                                    field.value === "oidc4vp"
                                      ? "text-primary-foreground"
                                      : "text-foreground/70 hover:text-foreground"
                                  } ${
                                    recommendedMethod === "oidc4vp" && field.value !== "oidc4vp"
                                      ? "ring-2 ring-secondary-400"
                                      : ""
                                  }`}
                                  onClick={() => field.onChange("oidc4vp")}
                                >
                                  Verifiable Credential
                                </button>
                              </div>

                              {/* Recommendation hint (fades, no layout shift) */}
                              <div className="h-5 flex items-center justify-end gap-2">
                                <span
                                  className={`text-[10px] uppercase tracking-widest text-muted-foreground transition-opacity duration-500 ${
                                    field.value !== recommendedMethod
                                      ? "opacity-100"
                                      : "opacity-0"
                                  }`}
                                >
                                  Recommended:
                                </span>
                                <Badge
                                  variant="info"
                                  className={`text-[10px] transition-opacity duration-500 ${
                                    field.value !== recommendedMethod
                                      ? "opacity-100"
                                      : "opacity-0"
                                  }`}
                                >
                                  {recommendedMethod === "cert"
                                    ? "Certificate"
                                    : "Verifiable Credential"}
                                </Badge>
                              </div>
                            </div>
                          </FormControl>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control as any}
                      name="auto"
                      render={({ field }: { field: any }) => (
                        <FormItem className="flex flex-row items-start space-x-3 space-y-0 rounded-md border p-4">
                          <FormControl>
                            <input
                              type="checkbox"
                              checked={field.value}
                              onChange={field.onChange}
                              className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                            />
                          </FormControl>
                          <div className="space-y-1 leading-none">
                            <LabelWithInfo label="Automatic Acceptance">
                              When the authority approves the request, the credential is{" "}
                              <strong>claimed into your wallet automatically</strong>. No follow-up
                              needed.
                            </LabelWithInfo>
                            <FormDescription>
                              Automatically claim the VC once the request is approved.
                            </FormDescription>
                          </div>
                        </FormItem>
                      )}
                    />

                    <Button
                      type="submit"
                      className={`w-full ${vcType ? highlightButtonClasses : ""}`}
                      disabled={!discoveredInfo || isSubmitting}
                    >
                      {isSubmitting ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          Submitting...
                        </>
                      ) : (
                        "Submit Request"
                      )}
                    </Button>
                  </form>
                </Form>
              </CardContent>
            </Card>
          </div>

          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Discovery Info</CardTitle>
                <CardDescription>Details retrieved from the authority.</CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                {discoveryError && (
                  <div className="flex items-center gap-2 p-3 text-sm text-destructive bg-destructive/10 rounded-md">
                    <AlertCircle className="h-4 w-4" />
                    <span>{discoveryError}</span>
                  </div>
                )}

                {discoveredInfo ? (
                  <div className="space-y-4">
                    <div className="space-y-1">
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        Authority DID
                      </p>
                      <Badge variant="infoLighter" className=" break-all">
                        {formatIdentifier(discoveredInfo.id, 0, true, 40)}
                      </Badge>
                    </div>
                    <div className="space-y-1">
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        Available VC Types
                      </p>
                      <div className="flex flex-wrap gap-2 pt-1">
                        {discoveredInfo.vc_types.map((t) => (
                          <Badge key={t}>{getFriendlyVCType(t)}</Badge>
                        ))}
                      </div>
                    </div>
                    <div className="space-y-1">
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        Services
                      </p>
                      <div className="space-y-2 pt-1">
                        {discoveredInfo.services.map((s, idx) => (
                          <div
                            key={idx}
                            className="p-2 border rounded bg-background-200/30 text-sm space-y-1"
                          >
                            <p className="font-medium text-brand-sky">
                              {s.type.replace(/([a-z])([A-Z])/g, "$1 $2")}
                            </p>
                            <p className="break-all text-xs text-white/70">{s.serviceEndpoint}</p>
                          </div>
                        ))}
                        {discoveredInfo.services.length === 0 && (
                          <p className="text-sm italic opacity-50">No services found</p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-success-400 font-medium pt-2">
                      <CheckCircle2 className="h-4 w-4" />
                      Authority verified
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8 text-muted-foreground italic text-sm">
                    Enter a URL and click Discover to see authority details.
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </PageSection>
    </PageLayout>
  );
}
