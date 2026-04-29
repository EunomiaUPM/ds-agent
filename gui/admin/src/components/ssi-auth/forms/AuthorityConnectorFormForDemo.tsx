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

const authDidSchema = z.object({
  url: z.string().url("Please enter a valid URL"),
});

const DEV_URL = "http://127.0.0.1:1500";
const PROD_URL = "https://dev-dataspaces.dit.upm.es:1500";

export function AuthorityConnectorFormForDemo() {
  const ssiAuthContext = useContext(SSIAuthContext);
  const globalInfoContext = useContext(GlobalInfoContext);

  // api_gateway_base is "" in production (proxy mode) and a full URL in development
  const isProduction = globalInfoContext?.api_gateway_base === "";
  const authorityUrl = isProduction ? PROD_URL : DEV_URL;

  const form = useForm<z.infer<typeof authDidSchema>>({
    resolver: zodResolver(authDidSchema),
    defaultValues: {
      url: authorityUrl,
    },
  });

  function onSubmit(values: z.infer<typeof authDidSchema>) {
    ssiAuthContext.fetchAuthDid(values.url);
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-2">
        <FormField
          control={form.control}
          name="url"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Authority URL</FormLabel>
              <div className="flex gap-2">
                <Select onValueChange={field.onChange} value={field.value}>
                  <FormControl>
                    <SelectTrigger>
                      <SelectValue />
                    </SelectTrigger>
                  </FormControl>
                  <SelectContent>
                    <SelectItem value={authorityUrl}>{authorityUrl}</SelectItem>
                  </SelectContent>
                </Select>
                <Button type="submit" isLoading={ssiAuthContext.isLoading.fetchAuthDid}>
                  Fetch DID
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
