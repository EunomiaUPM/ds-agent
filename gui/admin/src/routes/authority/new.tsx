import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "shared/src/components/ui/card";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
  FormDescription,
} from "shared/src/components/ui/form";
import { Input } from "shared/src/components/ui/input";
import { Button } from "shared/src/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Search, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { Badge } from "shared/src/components/ui/badge";
import { customInstance } from "shared/src/data/orval-mutator";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import { getFriendlyVCType } from "shared/src/lib/utils";

const schema = z.object({
  url: z.string().url("Please enter a valid URL"),
  slug: z.string().min(1, "Name is required"),
  vc_type: z.string().min(1, "Please select a VC type"),
  method: z.enum(["oidc4vp", "cert"]),
  auto: z.boolean().default(true),
});

interface DidService {
  id?: string;
  type: string;
  serviceEndpoint: string;
}

type FormValues = z.infer<typeof schema>;

// @ts-ignore
export const Route = createFileRoute("/authority/new")({
  component: NewAuthorityRequest,
});

function NewAuthorityRequest() {
  const navigate = useNavigate();
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [discoveredInfo, setDiscoveredInfo] = useState<{
    id: string;
    vc_types: string[];
    services: DidService[];
  } | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);

  const { data: participantsResponse } = useGetAllParticipants();
  const knownAuthorities =
    participantsResponse?.status === 200
      ? participantsResponse.data.filter((p) => p.participant_type === "Authority")
      : [];

  const form = useForm<FormValues>({
    resolver: zodResolver(schema) as any,
    defaultValues: {
      url: "",
      slug: "",
      vc_type: "",
      method: "cert",
      auto: true,
    },
  });

  const url = form.watch("url");

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

      (navigate as any)({ to: "/authority" });
    } catch (err) {
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <PageLayout>
      <PageHeader title="Request New Credential" />
      <PageSection>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Authority Connection</CardTitle>
                <CardDescription>
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
                          <FormLabel>Authority URL</FormLabel>
                          <div className="flex gap-2">
                            <FormControl>
                              <Input
                                placeholder="https://authority.example.com"
                                list="known-authorities"
                                {...field}
                                onChange={(e) => {
                                  field.onChange(e);
                                  const val = e.target.value;
                                  const known = knownAuthorities.find((a) => a.base_url === val);
                                  if (known) {
                                    if (known.participant_slug) {
                                      form.setValue("slug", known.participant_slug);
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
                              type="button"
                              variant="secondary"
                              onClick={() => handleDiscovery()}
                              disabled={isDiscovering || !url}
                            >
                              {isDiscovering ? (
                                <Loader2 className="animate-spin h-4 w-4 mr-2" />
                              ) : (
                                <Search className="h-4 w-4 mr-2" />
                              )}
                              Discover
                            </Button>
                          </div>
                          <FormMessage />
                          <FormDescription>
                            Example: http://host.docker.internal:1500
                          </FormDescription>
                        </FormItem>
                      )}
                    />

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <FormField
                        control={form.control as any}
                        name="slug"
                        render={({ field }) => (
                          <FormItem>
                            <FormLabel>Friendly Name (Slug)</FormLabel>
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
                            <FormLabel>VC Type</FormLabel>
                            <Select
                              onValueChange={field.onChange}
                              defaultValue={field.value}
                              disabled={!discoveredInfo}
                            >
                              <FormControl>
                                <SelectTrigger>
                                  <SelectValue
                                    placeholder={
                                      discoveredInfo ? "Select VC type" : "Discover first..."
                                    }
                                  />
                                </SelectTrigger>
                              </FormControl>
                              <SelectContent>
                                {discoveredInfo?.vc_types.map((type) => (
                                  <SelectItem key={type} value={type}>
                                    {getFriendlyVCType(type)}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                            <FormMessage />
                          </FormItem>
                        )}
                      />
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                      <FormField
                        control={form.control as any}
                        name="method"
                        render={({ field }: { field: any }) => (
                          <FormItem>
                            <FormLabel>Identity Proof</FormLabel>
                            <FormControl>
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
                                  className={`relative z-10 w-1/2 py-2 px-3 text-xs md:text-sm font-semibold transition-colors duration-300 rounded-md ${
                                    field.value === "cert"
                                      ? "text-primary-foreground"
                                      : "text-foreground/70 hover:text-foreground"
                                  }`}
                                  onClick={() => field.onChange("cert")}
                                >
                                  Certificate
                                </button>

                                <button
                                  type="button"
                                  className={`relative z-10 w-1/2 py-2 px-3 text-xs md:text-sm font-semibold transition-colors duration-300 rounded-md ${
                                    field.value === "oidc4vp"
                                      ? "text-primary-foreground"
                                      : "text-foreground/70 hover:text-foreground"
                                  }`}
                                  onClick={() => field.onChange("oidc4vp")}
                                >
                                  Verifiable Credential
                                </button>
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
                              <FormLabel>Automatic Acceptance</FormLabel>
                              <FormDescription>
                                Automatically claim the VC once the request is approved.
                              </FormDescription>
                            </div>
                          </FormItem>
                        )}
                      />
                    </div>

                    <Button
                      type="submit"
                      className="w-full"
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
                      <Badge variant="infoLighter" className="font-mono text-[10px] break-all p-2">
                        {discoveredInfo.id}
                      </Badge>
                    </div>
                    <div className="space-y-1">
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        Available VC Types
                      </p>
                      <div className="flex flex-wrap gap-2 pt-1">
                        {discoveredInfo.vc_types.map((t) => (
                          <Badge key={t} variant="info" className="text-[10px]">
                            {t}
                          </Badge>
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
                            className="p-2 border rounded bg-background-200/50 text-[10px] space-y-1"
                          >
                            <p className="font-bold text-primary">{s.type}</p>
                            <p className="break-all opacity-70">{s.serviceEndpoint}</p>
                          </div>
                        ))}
                        {discoveredInfo.services.length === 0 && (
                          <p className="text-[10px] italic opacity-50">No services found</p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-green-500 font-medium pt-2">
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
