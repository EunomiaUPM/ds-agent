
import { createFileRoute } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "shared/src/components/ui/tabs";
import { WalletStep } from "../../components/ssi-auth/steps/WalletStep";
import { AuthorityStep } from "../../components/ssi-auth/steps/AuthorityStep";
import { PeerStep } from "../../components/ssi-auth/steps/PeerStep";

/**
 * Route for listing transfer processes.
 */
export const Route = createFileRoute("/ssi-auth/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <PageLayout>
      <PageHeader title="SSI Auth" />
      <PageSection>
        <Tabs defaultValue="wallet" className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="wallet">Wallet</TabsTrigger>
            <TabsTrigger value="authority">Authority</TabsTrigger>
            <TabsTrigger value="peer">Peer</TabsTrigger>
          </TabsList>

          <TabsContent value="wallet">
            <WalletStep />
          </TabsContent>

          <TabsContent value="authority">
            <AuthorityStep />
          </TabsContent>

          <TabsContent value="peer">
            <PeerStep />
          </TabsContent>
        </Tabs>
      </PageSection>
    </PageLayout>
  );
}
