import { AlertCircle, ArrowLeft, CheckCircle2, Loader2, Search } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { useForm } from "react-hook-form";
import { formatIdentifier } from "shared/lib/utils";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Badge } from "shared/src/components/ui/badge";
import { Button } from "shared/src/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "shared/src/components/ui/card";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "shared/src/components/ui/form";
import { Input } from "shared/src/components/ui/input";
import { customInstance } from "shared/src/data/orval-mutator";
import { useFederatedCatalog } from "shared/src/data/useFederatedCatalog";
import * as z from "zod";

import { zodResolver } from "@hookform/resolvers/zod";
import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { useGetAllParticipants } from "shared/src/data/orval/participants/participants";
import WizardDialog from "shared/src/components/WizardDialog";

const schema = z.object({
  url: z.string().url("Please enter a valid URL"),
  nick: z.string().min(1, "Name is required"),
  auto: z.boolean().default(true),
  actions: z.array(z.string()).min(1, "Select at least one action"),
});

interface DidService {
  id?: string;
  type: string;
  serviceEndpoint: string;
}

type FormValues = z.infer<typeof schema>;

interface NewConnectionSearch {
  url?: string;
  nick?: string;
}

// @ts-ignore
export const Route = createFileRoute("/connections/sent/new")({
  component: NewSentConnection,
  validateSearch: (search: Record<string, unknown>): NewConnectionSearch => ({
    url: typeof search.url === "string" ? search.url : undefined,
    nick: typeof search.nick === "string" ? search.nick : undefined,
  }),
});

function NewSentConnection() {
  const navigate = useNavigate();
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [discoveredInfo, setDiscoveredInfo] = useState<{
    id: string;
    services: DidService[];
  } | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);
  const { data: participantsResponse } = useGetAllParticipants();
  const localParticipants = participantsResponse?.status === 200 ? participantsResponse.data : [];

  const federated = useFederatedCatalog();
  const knownProviders = federated.state === "ok" ? federated.agents : [];

  const labelURLRef = useRef<HTMLElement | null>(null);
  const [wizardURLOpen, setWizardURLOpen] = useState(true);

  const localParticipantsIds = new Set(localParticipants?.map((p) => p.participant_id));
  const onboardedWithKnownProvider = knownProviders.find((prov) =>
    localParticipantsIds?.has(prov.participant_id),
  );

  const search = Route.useSearch();

  const form = useForm<FormValues>({
    resolver: zodResolver(schema) as any,
    defaultValues: {
      url: search.url ?? "",
      nick: search.nick ?? "",
      auto: true,
      actions: ["talk"],
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
      const services = (didJson.service || []) as DidService[];

      setDiscoveredInfo({ id, services });
    } catch (err: any) {
      console.error(err);
      setDiscoveryError(err.message || "Could not discover provider info");
    } finally {
      setIsDiscovering(false);
    }
  };

  const [didPrefill, setDidPrefill] = useState(false);
  useEffect(() => {
    if (didPrefill || !search.url) return;
    form.setValue("url", search.url);
    if (search.nick) form.setValue("nick", search.nick);
    handleDiscovery(search.url);
    setDidPrefill(true);
  }, [search.url, search.nick, didPrefill]);

  const onSubmit = async (values: FormValues) => {
    if (!discoveredInfo) return;

    setIsSubmitting(true);
    try {
      const authService = discoveredInfo.services.find((s) => s.type === "AuthorizationServer");
      const targetUrl = authService?.serviceEndpoint || values.url;

      await customInstance(`/peer-connection/connect`, {
        method: "POST",
        data: {
          id: discoveredInfo.id,
          nick: values.nick,
          url: targetUrl,
          actions: values.actions,
          auto: values.auto,
        },
      });

      try {
        sessionStorage.setItem("JustAuthenticatedProvider", "true");
      } catch (e) {
        /* ignore if unavailable */
      }

      (navigate as any)({ to: "/connections/sent" });
    } catch (err) {
      console.error(err);
    } finally {
      setIsSubmitting(false);
    }
  };

  let highlightButtonClasses = !onboardedWithKnownProvider
    ? " animate-pulse bg-secondary-600 hover:bg-secondary-500 ring-2 ring-secondary-400"
    : "";

  return (
    <>
      <div className="flex items-center justify-between mb-4">
        <Link to="/connections/sent">
          <Button variant="ghost" size="sm">
            <ArrowLeft className="h-4 w-4 mr-2" />
            Back to Sent
          </Button>
        </Link>
      </div>
      {!onboardedWithKnownProvider && (
        <WizardDialog
          open={wizardURLOpen}
          onClose={() => setWizardURLOpen(false)}
          anchorRef={labelURLRef}
          align="left"
          sectionTitle={"Connection with Participant Tutorial"}
          step="3 of 3"
          title="New Connection with catalog owner"
          content={
            <>
              The URL of the participant has already been filled, so you can send the participant a
              connection request.
              <br />
            </>
          }
        />
      )}
      <PageSection>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Participant Connection</CardTitle>
                <CardDescription>
                  Enter the participant base URL to discover its DID and initiate onboarding.
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
                            Participant URL
                          </FormLabel>
                          <div className="flex items-center gap-2">
                            <FormControl className="flex-1">
                              <Input
                                placeholder="https://provider.example.com"
                                list="known-providers"
                                {...field}
                                onChange={(e) => {
                                  field.onChange(e);
                                  const val = e.target.value;
                                  const known = knownProviders.find((a) => a.base_url === val);
                                  if (known) {
                                    if (known.participant_nick) {
                                      form.setValue("nick", known.participant_nick);
                                    }
                                    handleDiscovery(val);
                                  }
                                }}
                              />
                            </FormControl>
                            <datalist id="known-providers">
                              {knownProviders.map((provider) => (
                                <option key={provider.participant_id} value={provider.base_url}>
                                  {provider.participant_nick || provider.participant_id}
                                </option>
                              ))}
                            </datalist>
                            <Button
                              type="button"
                              variant="secondary"
                              size="sm"
                              onClick={() => handleDiscovery()}
                              disabled={isDiscovering || !url}
                            >
                              {isDiscovering ? (
                                <Loader2 className="animate-spin h-4 w-4 mr-2" />
                              ) : (
                                <Search />
                              )}
                              Find provider
                            </Button>
                          </div>
                          <FormMessage />
                          <FormDescription>
                            Example: http://host.docker.internal:2000
                          </FormDescription>
                        </FormItem>
                      )}
                    />

                    <FormField
                      control={form.control as any}
                      name="nick"
                      render={({ field }) => (
                        <FormItem>
                          <FormLabel>Friendly Name (Nick)</FormLabel>
                          <FormControl>
                            <Input placeholder="Acme Provider" {...field} />
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
                            <FormLabel>Automatic Authentication</FormLabel>
                            <FormDescription>
                              Automatically handle the authentication flow.
                            </FormDescription>
                          </div>
                        </FormItem>
                      )}
                    />

                    <Button
                      type="submit"
                      className={`w-full   ${highlightButtonClasses}`}
                      disabled={!discoveredInfo || isSubmitting}
                    >
                      {isSubmitting ? (
                        <>
                          <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                          Onboarding...
                        </>
                      ) : (
                        "Initiate Onboarding"
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
                <CardDescription>Details retrieved from the provider.</CardDescription>
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
                        Provider DID
                      </p>
                      <Badge variant="infoLighter">
                        {formatIdentifier(discoveredInfo.id, 0, true, 40)}
                      </Badge>
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
                            <p className="font-medium text-brand-sky">{s.type}</p>
                            <p className="break-all text-xs text-white/70">{s.serviceEndpoint}</p>
                          </div>
                        ))}
                        {discoveredInfo.services.length === 0 && (
                          <p className="text-[10px] italic opacity-50">No services found</p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-2 text-sm text-success-400 font-medium pt-2">
                      <CheckCircle2 className="h-4 w-4" />
                      Provider verified
                    </div>
                  </div>
                ) : (
                  <div className="text-center py-8 text-muted-foreground italic text-sm">
                    Enter a URL and click Discover to see provider details.
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </PageSection>
    </>
  );
}
