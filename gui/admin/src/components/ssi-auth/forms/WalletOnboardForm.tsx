
import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "shared/src/components/ui/button";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { Form } from "shared/src/components/ui/form";

const onboardSchema = z.object({});

export function WalletOnboardForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof onboardSchema>>({
    resolver: zodResolver(onboardSchema),
  });

  function onSubmit() {
    ssiAuthContext.onboardInWallet();
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="flex items-center gap-4">
        <Button type="submit" isLoading={ssiAuthContext.isLoading.onboard}>
          Wallet Onboard
        </Button>
        {ssiAuthContext.ownWalletOnboarded && (
          <span className="text-green-600 text-sm font-medium">Onboarded!</span>
        )}
      </form>
    </Form>
  );
}
