
import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
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

const peerDidSchema = z.object({
  url: z.string().url("Please enter a valid URL"),
});

export function PeerConnectorForm() {
  const ssiAuthContext = useContext(SSIAuthContext);

  const form = useForm<z.infer<typeof peerDidSchema>>({
    resolver: zodResolver(peerDidSchema),
    defaultValues: {
      url: "",
    },
  });

  function onSubmit(values: z.infer<typeof peerDidSchema>) {
    ssiAuthContext.fetchPeerDid(values.url);
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-2">
        <FormField
          control={form.control}
          name="url"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Peer URL</FormLabel>
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
  );
}
