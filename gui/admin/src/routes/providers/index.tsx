import { createFileRoute } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { PeerStep } from "../../components/ssi-auth/steps/PeerStep";

/**
 * Providers index page.
 * Reuses the existing PeerStep component for connecting with other providers.
 */
const ProvidersPage = () => {
  return (
    <PageLayout>
      <PageHeader title="Providers" />
      <PageSection>
        <PeerStep />
      </PageSection>
    </PageLayout>
  );
};

export const Route = createFileRoute("/providers/")({
  component: ProvidersPage,
});
