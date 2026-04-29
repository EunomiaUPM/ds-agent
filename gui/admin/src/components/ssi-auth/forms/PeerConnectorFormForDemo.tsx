import { useContext } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Button } from "shared/src/components/ui/button";
import { SSIAuthContext } from "shared/src/context/SSIAuthContext";
import { GlobalInfoContext } from "shared/src/context/GlobalInfoContext";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "shared/src/components/ui/form";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";

const peerDidSchema = z.object({
  url: z.string().url("Please enter a valid URL"),
});

const DEV_URL = "http://127.0.0.1:1200";
const PROD_URL = "https://dev-dataspaces.dit.upm.es:1200";

export function PeerConnectorFormForDemo() {
  const ssiAuthContext = useContext(SSIAuthContext);
  const globalInfoContext = useContext(GlobalInfoContext);

  // api_gateway_dsp_base contains the full URL in both dev and prod
  const isProduction = globalInfoContext?.api_gateway_base === "";
  const peerUrl = isProduction ? PROD_URL : DEV_URL;

  const form = useForm<z.infer<typeof peerDidSchema>>({
    resolver: zodResolver(peerDidSchema),
    defaultValues: {
      url: peerUrl,
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
                <Select onValueChange={field.onChange} value={field.value}>
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value={peerUrl}>{peerUrl}</SelectItem>
                  </SelectContent>
                </Select>
                <Button type="submit" isLoading={ssiAuthContext.isLoading.fetchPeerDid}>
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
