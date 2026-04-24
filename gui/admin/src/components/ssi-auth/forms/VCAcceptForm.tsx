import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "shared/src/components/ui/button";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { Form } from "shared/src/components/ui/form";

const acceptVcSchema = z.object({});

export function VCAcceptForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof acceptVcSchema>>({
    resolver: zodResolver(acceptVcSchema),
  });

  function onSubmit() {
    ssiAuthContext.saveOidc4VciRequestUri();
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Button type="submit" isLoading={ssiAuthContext.isLoading.oidc4vci} variant="default">
          Accept Credential (Sign & Store)
        </Button>
      </form>
    </Form>
  );
}
