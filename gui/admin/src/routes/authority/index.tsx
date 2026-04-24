import { createFileRoute } from "@tanstack/react-router";
import { PageLayout } from "shared/src/components/layout/PageLayout";
import { PageHeader } from "shared/src/components/layout/PageHeader";
import { PageSection } from "shared/src/components/layout/PageSection";
import { AuthorityStep } from "../../components/ssi-auth/steps/AuthorityStep";

/**
 * Authority index page.
 * Reuses the existing AuthorityStep component for connection and credential requests.
 */
const AuthorityPage = () => {
  return (
    <PageLayout>
      <PageHeader title="Authority" />
      <PageSection>
        <AuthorityStep />
      </PageSection>
    </PageLayout>
  );
};

export const Route = createFileRoute("/authority/")({
  component: AuthorityPage,
});
