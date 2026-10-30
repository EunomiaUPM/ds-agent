import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "@eunomia/ui";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { Form } from "@eunomia/ui";

const presentVpSchema = z.object({});

export function VPPresentationForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof presentVpSchema>>({
    resolver: zodResolver(presentVpSchema),
  });

  function onSubmit() {
    ssiAuthContext.presentVPtoPeer();
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)}>
        <Button type="submit" isLoading={ssiAuthContext.isLoading.oidc4vp}>
          Present VP
        </Button>
      </form>
    </Form>
  );
}
