
import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "shared/src/components/ui/button";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { Form } from "shared/src/components/ui/form";

const grantRequestSchema = z.object({});

export function GrantRequestForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof grantRequestSchema>>({
    resolver: zodResolver(grantRequestSchema),
  });

  function onSubmit() {
    ssiAuthContext.setOidc4VpRequestUri();
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Button
          type="submit"
          isLoading={ssiAuthContext.isLoading.fetchAuthRequests}
          disabled={!ssiAuthContext.tempPeer.did}
        >
          Request Grant
        </Button>
      </form>
    </Form>
  );
}
