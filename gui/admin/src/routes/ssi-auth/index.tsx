import { useContext } from "react";
import { createFileRoute } from "@tanstack/react-router";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Button } from "shared/src/components/ui/button";
import { Input } from "shared/src/components/ui/input";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "shared/src/components/ui/form";
import { DataTable, Column } from "shared/src/components/DataTable";
import { Loader2, Circle } from "lucide-react";


/**
 * Route for listing transfer processes.
 */
export const Route = createFileRoute("/ssi-auth/")({
  component: RouteComponent,
});

const authDidSchema = z.object({
  url: z.string().url("Please enter a valid URL"),
});


const peerDidSchema = z.object({
  url: z.string().url("Please enter a valid URL"),
});

const vcRequestSchema = z.object({});
const acceptVcSchema = z.object({});
const grantRequestSchema = z.object({});
const presentVpSchema = z.object({});

const onboardSchema = z.object({});
function RouteComponent() {
  const ssiAuthContext = useContext(SSIAuthContext)

  const authForm = useForm<z.infer<typeof authDidSchema>>({
    resolver: zodResolver(authDidSchema),
    defaultValues: {
      url: "",
    },
  });

  const peerForm = useForm<z.infer<typeof peerDidSchema>>({
    resolver: zodResolver(peerDidSchema),
    defaultValues: {
      url: "",
    },
  });

  const onboardForm = useForm<z.infer<typeof onboardSchema>>({
    resolver: zodResolver(onboardSchema),
  });

  const vcRequestForm = useForm<z.infer<typeof vcRequestSchema>>({
    resolver: zodResolver(vcRequestSchema),
  });

  const acceptVcForm = useForm<z.infer<typeof acceptVcSchema>>({
    resolver: zodResolver(acceptVcSchema),
  });

  const grantRequestForm = useForm<z.infer<typeof grantRequestSchema>>({
    resolver: zodResolver(grantRequestSchema),
  });

  const presentVpForm = useForm<z.infer<typeof presentVpSchema>>({
    resolver: zodResolver(presentVpSchema),
  });

  function onAuthSubmit(values: z.infer<typeof authDidSchema>) {
    ssiAuthContext.fetchAuthDid(values.url);
  }

  function onPeerSubmit(values: z.infer<typeof peerDidSchema>) {
    ssiAuthContext.fetchPeerDid(values.url);
  }

  function onOnboardSubmit() {
    ssiAuthContext.onboardInWallet();
  }

  function onVcRequestSubmit() {
    ssiAuthContext.requestVCtoAuthority();
  }

  function onAcceptVcSubmit() {
    ssiAuthContext.saveOidc4VciRequestUri();
  }

  function onGrantRequestSubmit() {
    ssiAuthContext.setOidc4VpRequestUri();
  }

  function onPresentVpSubmit() {
    ssiAuthContext.presentVPtoPeer();
  }

  const vcRequestColumns: Column<any>[] = [
    { header: "ID", accessorKey: "id", cell: (item) => <span className="font-mono text-xs">{item.id}</span> },
    { header: "Status", accessorKey: "status" },
    { header: "Type", accessorKey: "vc_type" },
    { header: "Created At", accessorKey: "created_at" },
  ];

  return (
    <PageLayout>
      <PageHeader title="SSI Auth" />
      <PageSection>
        <div className="flex flex-col gap-2 mb-2">
          First, lets checkout if you are onboarded in a wallet.
        </div>
        <div className="w-full">
          <Form {...onboardForm}>
            <form onSubmit={onboardForm.handleSubmit(onOnboardSubmit)} className="space-y-4">
              <Button type="submit" isLoading={ssiAuthContext.isLoading.onboard}>
                Wallet Onboard
              </Button>
            </form>
          </Form>
          <div className="mt-2">
            <div>
              {ssiAuthContext.ownDid && (
                <div className="text-sm font-mono mt-2 p-2 bg-muted rounded break-all">
                  This is your did {ssiAuthContext.ownDid}
                </div>
              )}
            </div>
          </div>
        </div>
        <hr className="my-4" />
        <div>
          Second, let's fetch the authority did.
        </div>
        <div className="w-full">
          <Form {...authForm}>
            <form onSubmit={authForm.handleSubmit(onAuthSubmit)} className="space-y-4">
              <FormField
                control={authForm.control}
                name="url"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Authority DID Url</FormLabel>
                    <div className="flex gap-2">
                      <FormControl>
                        <Input placeholder="http://host.docker.internal:1500" {...field} />
                      </FormControl>
                      <Button
                        type="submit"
                        isLoading={ssiAuthContext.isLoading.fetchAuthDid}
                      >
                        Fetch Authority DID
                      </Button>
                    </div>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </form>
          </Form>
          <div>
            {ssiAuthContext.authDid.did && (
              <div className="text-sm font-mono mt-2 p-2 bg-muted rounded break-all">
                {ssiAuthContext.authDid.did}
              </div>
            )}
          </div>
        </div>
        <div>
          Third, let's your peer did.
        </div>
        <div className="w-full">
          <Form {...peerForm}>
            <form onSubmit={peerForm.handleSubmit(onPeerSubmit)} className="space-y-4">
              <FormField
                control={peerForm.control}
                name="url"
                render={({ field }) => (
                  <FormItem>
                    <FormLabel>Peer DID Url</FormLabel>
                    <div className="flex gap-2">
                      <FormControl>
                        <Input placeholder="http://host.docker.internal:2000" {...field} />
                      </FormControl>
                      <Button
                        type="submit"
                        isLoading={ssiAuthContext.isLoading.fetchPeerDid}
                      >
                        Fetch Peer DID
                      </Button>
                    </div>
                    <FormMessage />
                  </FormItem>
                )}
              />
            </form>
          </Form>
          <div>
            {ssiAuthContext.tempPeer.did && (
              <div className="text-sm font-mono mt-2 p-2 bg-muted rounded break-all">
                {ssiAuthContext.tempPeer.did}
              </div>
            )}
          </div>
        </div>

        <hr className="my-4" />
        <div>
          Let's ask the dataspace authority for a Dataspace VC.
        </div>
        <div className="flex gap-2 flex-col items-start w-full">
          <Form {...vcRequestForm}>
            <form onSubmit={vcRequestForm.handleSubmit(onVcRequestSubmit)} className="space-y-4">
              <Button type="submit" isLoading={ssiAuthContext.isLoading.requestVC}>
                Please issue me a Dataspace VC
              </Button>
            </form>
          </Form>

          <div className="w-full mt-4">
            <div className="flex items-center gap-2 mb-2">
              <h3 className="text-lg font-medium">List of requests</h3>
              {ssiAuthContext.authRequestsPollInterval > 0 && (
                <Loader2 className="h-4 w-4 animate-spin text-orange-500" />
              )}
              {ssiAuthContext.authRequestsPollInterval === 0 && Array.isArray(ssiAuthContext.authRequests) && ssiAuthContext.authRequests.length > 0 && ssiAuthContext.authRequests[0].status === 'Approved' && (
                <Circle className="h-4 w-4 text-green-500" />
              )}
            </div>
            <div className="rounded-md border">
              <DataTable
                columns={vcRequestColumns}
                data={Array.isArray(ssiAuthContext.authRequests) ? ssiAuthContext.authRequests : []}
                keyExtractor={(item) => item.id}
              />
            </div>
          </div>
          <div>
            {ssiAuthContext.oidc4vciRequestUri && (
              <div className="text-sm font-mono mt-2 p-2 bg-muted rounded break-all">
                {ssiAuthContext.oidc4vciRequestUri}
              </div>
            )}
          </div>
        </div>
        <div>
          <Form {...acceptVcForm}>
            <form onSubmit={acceptVcForm.handleSubmit(onAcceptVcSubmit)} className="space-y-4">
              <Button type="submit" isLoading={ssiAuthContext.isLoading.oidc4vci}>
                Accept Dataspace VC
              </Button>
            </form>
          </Form>
        </div>
        <hr className="my-4" />
        <div>
          Ask our peer for grant request
        </div>
        <div>
          <Form {...grantRequestForm}>
            <form onSubmit={grantRequestForm.handleSubmit(onGrantRequestSubmit)} className="space-y-4">
              <Button type="submit" isLoading={ssiAuthContext.isLoading.fetchAuthRequests}>
                Request Grant
              </Button>
            </form>
          </Form>
        </div>
        <div>
          Grant here: {ssiAuthContext.oidc4vpRequestUri}
        </div>
        <hr className="my-4" />
        <div>
          Present VP to peer
        </div>
        <div>
          <Form {...presentVpForm}>
            <form onSubmit={presentVpForm.handleSubmit(onPresentVpSubmit)} className="space-y-4">
              <Button type="submit" isLoading={ssiAuthContext.isLoading.oidc4vp}>
                Present VP
              </Button>
            </form>
          </Form>
        </div>
        <div>
          {ssiAuthContext.oidc4vpSuccess && (
            <div>
              Finish!!!! go checkout participant list, fetch peer catalog and transfer!!!!
            </div>
          )}
        </div>
      </PageSection>
    </PageLayout>
  );
}
