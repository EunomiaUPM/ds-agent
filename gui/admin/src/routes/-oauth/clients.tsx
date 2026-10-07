/*
 * Copyright (C) 2026 - Universidad Politécnica de Madrid - UPM
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <https://www.gnu.org/licenses/>.
 */

import { createFileRoute } from "@tanstack/react-router";
import { keepPreviousData, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  useListOAuthClients,
  useRegisterOAuthClient,
  useDeleteOAuthClient,
  getListOAuthClientsQueryKey,
} from "shared/src/data/orval/oauth-clients/oauth-clients";
import { OAuthClient, CreateClientCommandRole } from "shared/src/data/orval/model";
import { PageSection } from "shared/src/components/layout/PageSection";
import { DataTable } from "shared/src/components/DataTable";
import { useTableQueryParams } from "shared/src/hooks/useTableQueryParams";
import { FormatDate } from "shared/src/components/ui/format-date";
import { Skeleton } from "shared/src/components/ui/skeleton";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import { Input } from "shared/src/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "shared/src/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "shared/src/components/ui/select";
import { Plus, Trash2, Copy, Check, Shield } from "lucide-react";
import { toast } from "sonner";

interface RegisterClientDialogProps {
  open: boolean;
  onClose: () => void;
  onRegistered: (clientName: string, clientId: string, clientSecret: string) => void;
}

// The server stores the secret it is given and never returns it, so it is generated here.
const generateClientSecret = () => {
  const bytes = new Uint8Array(32);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
};

const slugify = (value: string) =>
  value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

const RegisterClientDialog = ({ open, onClose, onRegistered }: RegisterClientDialogProps) => {
  const queryClient = useQueryClient();
  const [clientName, setClientName] = useState("");
  const [clientId, setClientId] = useState("");
  const [role, setRole] = useState<CreateClientCommandRole>("owner");
  const [scopesStr, setScopesStr] = useState("events:read transfers:read");

  const { mutate: register, isPending } = useRegisterOAuthClient({
    mutation: {
      onSuccess: (res, variables) => {
        queryClient.invalidateQueries({ queryKey: getListOAuthClientsQueryKey() });
        if (res.status === 201) {
          onRegistered(res.data.clientName, res.data.clientId, variables.data.clientSecret);
        }
        onClose();
        setClientName("");
        setClientId("");
      },
      onError: (err) => {
        toast.error(`Failed to register client: ${String(err)}`);
      },
    },
  });

  const handleSubmit = () => {
    if (!clientName.trim()) {
      toast.error("Client name is required");
      return;
    }
    const scopes = scopesStr
      .split(/[\s,]+/)
      .map((s) => s.trim())
      .filter(Boolean);

    register({
      data: {
        clientId: clientId.trim() || slugify(clientName),
        clientName: clientName.trim(),
        clientSecret: generateClientSecret(),
        role,
        scopes,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Register OAuth 2.0 Client</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Client Name
            </label>
            <Input
              value={clientName}
              onChange={(e) => setClientName(e.target.value)}
              placeholder="e.g. Data Plane Agent"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Client ID
            </label>
            <Input
              value={clientId}
              onChange={(e) => setClientId(e.target.value)}
              placeholder={slugify(clientName) || "e.g. data-plane-agent"}
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              RBAC Role
            </label>
            <Select value={role} onValueChange={(val) => setRole(val as CreateClientCommandRole)}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="owner">Owner</SelectItem>
                <SelectItem value="reader">Reader</SelectItem>
                <SelectItem value="admin">Admin</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Scopes (space-separated)
            </label>
            <Input
              value={scopesStr}
              onChange={(e) => setScopesStr(e.target.value)}
              placeholder="events:read transfers:read"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isPending}>
            {isPending ? "Registering..." : "Register"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

interface SecretDisplayDialogProps {
  data: { clientName: string; clientId: string; clientSecret: string } | null;
  onClose: () => void;
}

const SecretDisplayDialog = ({ data, onClose }: SecretDisplayDialogProps) => {
  const [copied, setCopied] = useState(false);

  if (!data) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(data.clientSecret);
    setCopied(true);
    toast.success("Secret copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Dialog open={!!data} onOpenChange={onClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-emerald-500">
            <Shield className="h-5 w-5" />
            Client Credentials Generated
          </DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <p className="text-sm text-muted-foreground">
            Save this client secret now. For security reasons, it cannot be recovered later.
          </p>
          <div className="flex flex-col gap-1 rounded bg-muted/50 p-3 text-xs font-mono">
            <div className="text-muted-foreground">Client ID:</div>
            <div className="select-all text-foreground font-semibold">{data.clientId}</div>
          </div>
          <div className="flex flex-col gap-1 rounded bg-muted/50 p-3 text-xs font-mono">
            <div className="text-muted-foreground">Client Secret:</div>
            <div className="flex items-center justify-between gap-2 overflow-x-auto select-all text-emerald-700 dark:text-emerald-400 font-semibold">
              <span className="truncate">{data.clientSecret}</span>
              <Button size="icon" variant="ghost" className="h-6 w-6 shrink-0" onClick={handleCopy}>
                {copied ? (
                  <Check className="h-3.5 w-3.5 text-emerald-700 dark:text-emerald-400" />
                ) : (
                  <Copy className="h-3.5 w-3.5" />
                )}
              </Button>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button onClick={onClose}>Done</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

const ClientsComponent = () => {
  const queryClient = useQueryClient();
  const [registerOpen, setRegisterOpen] = useState(false);
  const [secretData, setSecretData] = useState<{
    clientName: string;
    clientId: string;
    clientSecret: string;
  } | null>(null);

  const {
    params: queryParams,
    apiParams,
    onQueryChange,
  } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { role: "all" },
  });

  const { data, isLoading, isError, isFetching } = useListOAuthClients(apiParams, {
    query: {
      queryKey: ["/oauth/clients", apiParams],
      placeholderData: keepPreviousData,
    },
  });

  const { mutate: deleteClient } = useDeleteOAuthClient({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListOAuthClientsQueryKey() });
        toast.success("Client deleted");
      },
      onError: (err) => {
        toast.error(`Delete failed: ${String(err)}`);
      },
    },
  });

  const clients: OAuthClient[] = data?.status === 200 ? data.data.items : [];

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">Registered Applications</h2>
          <p className="text-xs text-muted-foreground">
            Manage client IDs and secrets for machine-to-machine, daemon, and service
            authentication.
          </p>
        </div>
        <Button onClick={() => setRegisterOpen(true)} className="gap-2">
          <Plus className="h-4 w-4" /> Register Client
        </Button>
      </div>

      <PageSection>
        {isLoading && !data ? (
          <div className="flex flex-col gap-3 p-4">
            <Skeleton className="h-12 w-full" />
            <Skeleton className="h-12 w-full" />
          </div>
        ) : isError && !data ? (
          <div className="p-8 text-center text-sm text-destructive">
            Failed to load registered OAuth clients.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={clients}
            serverSide={true}
            loading={isFetching}
            queryParams={queryParams}
            onQueryChange={onQueryChange}
            keyExtractor={(client) => client.clientId}
            searchPlaceholder="Filter clients by name, ID, or role..."
            emptyMessage='No OAuth clients registered yet. Click "Register Client" to add one.'
            defaultSortKey="createdAt"
            defaultSortDirection="desc"
            pageSize={10}
            pageSizeOptions={[10, 20, 50, 100]}
            filters={[
              {
                id: "role",
                label: "Role",
                value: queryParams.filters.role ?? "all",
                options: [
                  { label: "All Roles", value: "all" },
                  { label: "Owner", value: "owner" },
                  { label: "Reader", value: "reader" },
                  { label: "Admin", value: "admin" },
                ],
              },
            ]}
            columns={[
              {
                header: "Name",
                accessorKey: "clientName",
                cell: (client) => <span className="font-medium text-sm">{client.clientName}</span>,
              },
              {
                header: "Client ID",
                accessorKey: "clientId",
                cell: (client) => <Badge variant="info">{client.clientId}</Badge>,
              },
              {
                header: "Role",
                accessorKey: "role",
                cell: (client) => <Badge variant="role">{client.role}</Badge>,
              },
              {
                header: "Scopes",
                sortable: false,
                searchValue: (client) => client.scopes.join(" "),
                cell: (client) => (
                  <div className="flex flex-wrap gap-1 max-w-[240px]">
                    {client.scopes.map((sc: string) => (
                      <Badge key={sc} variant="infoLighter" size="xs">
                        {sc}
                      </Badge>
                    ))}
                  </div>
                ),
              },
              {
                header: "Created at",
                accessorKey: "createdAt",
                sortValue: (client) => new Date(client.createdAt).getTime(),
                cell: (client) => <FormatDate date={client.createdAt} />,
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (client) => (
                  <div className="flex items-center gap-2">
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-7 w-7 text-destructive hover:text-destructive hover:bg-destructive/10"
                      onClick={() => {
                        if (confirm(`Delete client "${client.clientName}"?`)) {
                          deleteClient({ clientId: client.clientId });
                        }
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </Button>
                  </div>
                ),
              },
            ]}
          />
        )}
      </PageSection>

      <RegisterClientDialog
        open={registerOpen}
        onClose={() => setRegisterOpen(false)}
        onRegistered={(clientName, clientId, clientSecret) =>
          setSecretData({ clientName, clientId, clientSecret })
        }
      />

      <SecretDisplayDialog data={secretData} onClose={() => setSecretData(null)} />
    </div>
  );
};

export const Route = createFileRoute("/oauth/clients")({
  component: ClientsComponent,
});
