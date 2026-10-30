import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "@eunomia/ui";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { Form } from "@eunomia/ui";

const vcRequestSchema = z.object({});

export function VCRequestForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof vcRequestSchema>>({
    resolver: zodResolver(vcRequestSchema),
  });

  function onSubmit() {
    ssiAuthContext.requestVCtoAuthority();
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Button
          type="submit"
          isLoading={ssiAuthContext.isLoading.requestVC}
          disabled={!ssiAuthContext.authDid.did || !ssiAuthContext.ownDid}
        >
          Request Dataspace VC
        </Button>
      </form>
    </Form>
  );
}
