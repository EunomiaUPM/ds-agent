import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "shared/src/components/ui/card";
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage, FormDescription } from "shared/src/components/ui/form";
import { Input } from "shared/src/components/ui/input";
import { Button } from "shared/src/components/ui/button";
import { Search, Loader2, CheckCircle2, AlertCircle } from "lucide-react";
import { Badge } from "shared/src/components/ui/badge";
import { customInstance } from "shared/src/data/orval-mutator";

const schema = z.object({
  url: z.string().url("Please enter a valid URL"),
  slug: z.string().min(1, "Name is required"),
  auto: z.boolean().default(true),
  actions: z.array(z.string()).min(1, "Select at least one action"),
});

interface DidService {
  id?: string;
  type: string;
  serviceEndpoint: string;
}

type FormValues = z.infer<typeof schema>;

// @ts-ignore
export const Route = createFileRoute("/providers/new")({
  component: NewProviderOnboard,
});

function NewProviderOnboard() {
  const navigate = useNavigate();
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [discoveredInfo, setDiscoveredInfo] = useState<{ id: string; services: DidService[] } | null>(null);
  const [discoveryError, setDiscoveryError] = useState<string | null>(null);

  const form = useForm<FormValues>({
    resolver: zodResolver(schema) as any,
    defaultValues: {
      url: "",
      slug: "",
      auto: true,
      actions: ["talk"],
    },
  });

  const url = form.watch("url");

  const handleDiscovery = async () => {
    if (!url || !url.startsWith("http")) return;

    setIsDiscovering(true);
    setDiscoveryError(null);
    try {
      const cleanUrl = url.replace(/\/$/, "");
      
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

  const onSubmit = async (values: FormValues) => {
    if (!discoveredInfo) return;

    try {
      const authService = discoveredInfo.services.find(s => s.type === "AuthorizationServer");
      const targetUrl = authService?.serviceEndpoint || values.url;

      await customInstance(`/onboard/provider`, {
        method: "POST",
        data: {
          ...values,
          id: discoveredInfo.id,
          url: targetUrl,
        },
      });

      (navigate as any)({ to: "/providers" });
    } catch (err) {
      console.error(err);
    }
  };

  return (
    <PageLayout>
      <PageHeader title="New Provider Onboarding" />
      <PageSection>
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2 space-y-6">
            <Card>
              <CardHeader>
                <CardTitle>Provider Connection</CardTitle>
                <CardDescription>
                  Enter the provider base URL to discover its DID and initiate onboarding.
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
                          <FormLabel>Provider URL</FormLabel>
                          <div className="flex gap-2">
                            <FormControl>
                              <Input placeholder="https://provider.example.com" {...field} />
                            </FormControl>
                            <Button 
                              type="button" 
                              variant="secondary" 
                              onClick={handleDiscovery}
                              disabled={isDiscovering || !url}
                            >
                              {isDiscovering ? <Loader2 className="animate-spin h-4 w-4 mr-2" /> : <Search className="h-4 w-4 mr-2" />}
                              Discover
                            </Button>
                          </div>
                          <FormMessage />
                          <FormDescription>
                            Example: http://host.docker.internal:2000
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
                              <FormLabel>
                                Automatic Authentication
                              </FormLabel>
                              <FormDescription>
                                Automatically handle the authentication flow.
                              </FormDescription>
                            </div>
                          </FormItem>
                        )}
                      />
                    </div>

                    <FormField
                      control={form.control as any}
                      name="actions"
                      render={() => (
                        <FormItem>
                          <div className="mb-4">
                            <FormLabel>Requested Actions</FormLabel>
                            <FormDescription>
                              Select the actions you want to perform with this provider.
                            </FormDescription>
                          </div>
                          <div className="flex flex-wrap gap-4">
                            {["talk", "pull", "push"].map((action) => (
                              <FormField
                                key={action}
                                control={form.control as any}
                                name="actions"
                                render={({ field }: { field: any }) => {
                                  return (
                                    <FormItem
                                      key={action}
                                      className="flex flex-row items-start space-x-3 space-y-0"
                                    >
                                      <FormControl>
                                        <input
                                          type="checkbox"
                                          checked={field.value?.includes(action)}
                                          onChange={(e) => {
                                            const checked = e.target.checked;
                                            return checked
                                              ? field.onChange([...field.value, action])
                                              : field.onChange(
                                                  field.value?.filter(
                                                    (value: string) => value !== action
                                                  )
                                                );
                                          }}
                                          className="h-4 w-4 rounded border-gray-300 text-primary focus:ring-primary"
                                        />
                                      </FormControl>
                                      <FormLabel className="font-normal capitalize">
                                        {action}
                                      </FormLabel>
                                    </FormItem>
                                  );
                                }}
                              />
                            ))}
                          </div>
                          <FormMessage />
                        </FormItem>
                      )}
                    />

                    <Button type="submit" className="w-full" disabled={!discoveredInfo}>
                      Initiate Onboarding
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
                <CardDescription>
                  Details retrieved from the provider.
                </CardDescription>
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
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Provider DID</p>
                      <Badge variant="infoLighter" className="font-mono text-[10px] break-all p-2">
                        {discoveredInfo.id}
                      </Badge>
                    </div>
                    <div className="space-y-1">
                      <p className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Services</p>
                      <div className="space-y-2 pt-1">
                        {discoveredInfo.services.map((s, idx) => (
                          <div key={idx} className="p-2 border rounded bg-background-200/50 text-[10px] space-y-1">
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
    </PageLayout>
  );
}
