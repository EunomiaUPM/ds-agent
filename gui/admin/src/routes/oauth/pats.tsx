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
  useListPersonalAccessTokens,
  useCreatePersonalAccessToken,
  useRevokePersonalAccessToken,
  getListPersonalAccessTokensQueryKey,
} from "shared/src/data/orval/personal-access-tokens/personal-access-tokens";
import { PatRecord, CreatePatCommandRole, CreatePatResponse } from "shared/src/data/orval/model";
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
import { Plus, Trash2, KeyRound, Copy, Check, ShieldAlert } from "lucide-react";
import { toast } from "sonner";

interface CreatePatDialogProps {
  open: boolean;
  onClose: () => void;
  onCreated: (token: string, name: string) => void;
}

const CreatePatDialog = ({ open, onClose, onCreated }: CreatePatDialogProps) => {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [role, setRole] = useState<CreatePatCommandRole>("Service");
  const [scopesStr, setScopesStr] = useState("events:* transfers:*");
  const [days, setDays] = useState("90");

  const { mutate: createPat, isPending } = useCreatePersonalAccessToken({
    mutation: {
      onSuccess: (res) => {
        queryClient.invalidateQueries({ queryKey: getListPersonalAccessTokensQueryKey() });
        const resData = res.data as CreatePatResponse;
        onCreated(resData.token, resData.pat.name);
        onClose();
        setName("");
      },
      onError: (err) => {
        toast.error(`Failed to create PAT: ${String(err)}`);
      },
    },
  });

  const handleSubmit = () => {
    if (!name.trim()) {
      toast.error("Token description / name is required");
      return;
    }
    const scopes = scopesStr
      .split(/[\s,]+/)
      .map((s) => s.trim())
      .filter(Boolean);

    const expiresInDays = parseInt(days, 10);

    createPat({
      data: {
        name: name.trim(),
        role,
        scopes,
        expires_in_days: isNaN(expiresInDays) ? undefined : expiresInDays,
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>Generate Personal Access Token</DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Token Name / Purpose
            </label>
            <Input
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g. CI/CD Automated Agent"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              RBAC Role
            </label>
            <Select value={role} onValueChange={(val) => setRole(val as CreatePatCommandRole)}>
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select role" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Service">Service</SelectItem>
                <SelectItem value="Admin">Admin</SelectItem>
                <SelectItem value="User">User</SelectItem>
                <SelectItem value="Auditor">Auditor</SelectItem>
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
              placeholder="events:* transfers:*"
            />
          </div>
          <div className="flex flex-col gap-1.5">
            <label className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Expiration (Days)
            </label>
            <Input
              type="number"
              value={days}
              onChange={(e) => setDays(e.target.value)}
              placeholder="90"
              min="1"
              max="365"
            />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={isPending}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isPending}>
            {isPending ? "Generating..." : "Generate Token"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

interface TokenDisplayDialogProps {
  tokenData: { token: string; name: string } | null;
  onClose: () => void;
}

const TokenDisplayDialog = ({ tokenData, onClose }: TokenDisplayDialogProps) => {
  const [copied, setCopied] = useState(false);

  if (!tokenData) return null;

  const handleCopy = () => {
    navigator.clipboard.writeText(tokenData.token);
    setCopied(true);
    toast.success("Token copied to clipboard");
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Dialog open={!!tokenData} onOpenChange={onClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-emerald-500">
            <KeyRound className="h-5 w-5" />
            Personal Access Token Created
          </DialogTitle>
        </DialogHeader>
        <div className="flex flex-col gap-4 py-2">
          <p className="text-sm text-muted-foreground">
            Copy your token now. For security reasons, it will not be displayed again.
          </p>
          <div className="flex flex-col gap-1 rounded bg-muted/50 p-3 text-xs font-mono">
            <div className="text-muted-foreground">Description:</div>
            <div className="text-foreground font-semibold">{tokenData.name}</div>
          </div>
          <div className="flex flex-col gap-1 rounded bg-muted/50 p-3 text-xs font-mono">
            <div className="text-muted-foreground">Token:</div>
            <div className="flex items-center justify-between gap-2 overflow-x-auto select-all text-emerald-700 dark:text-emerald-400 font-semibold">
              <span className="truncate">{tokenData.token}</span>
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

const PatsComponent = () => {
  const queryClient = useQueryClient();
  const [createOpen, setCreateOpen] = useState(false);
  const [tokenData, setTokenData] = useState<{ token: string; name: string } | null>(null);

  const { params: queryParams, apiParams, onQueryChange } = useTableQueryParams({
    defaultLimit: 10,
    defaultSort: "created_at_desc",
    defaultFilters: { status: "all" },
  });

  const { data, isLoading, isError, isFetching } = useListPersonalAccessTokens({
    query: {
      queryKey: ["/oauth/pats", apiParams],
      placeholderData: keepPreviousData,
    },
    request: {
      params: apiParams,
    },
  });
  const pats: PatRecord[] = Array.isArray(data?.data) ? (data.data as PatRecord[]) : [];

  const { mutate: revokePat } = useRevokePersonalAccessToken({
    mutation: {
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: getListPersonalAccessTokensQueryKey() });
        toast.success("PAT revoked");
      },
      onError: (err) => {
        toast.error(`Revocation failed: ${String(err)}`);
      },
    },
  });

  return (
    <div className="flex flex-col gap-6 w-full">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold tracking-tight">Personal Access Tokens (PATs)</h2>
          <p className="text-xs text-muted-foreground">
            Create long-lived API tokens for CLI scripts, Git actions, and external integrations
            without sharing passwords.
          </p>
        </div>
        <Button onClick={() => setCreateOpen(true)} className="gap-2">
          <Plus className="h-4 w-4" /> Generate New PAT
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
            Failed to load Personal Access Tokens.
          </div>
        ) : (
          <DataTable
            className="text-sm"
            data={pats}
            serverSide={true}
            loading={isFetching}
            queryParams={queryParams}
            onQueryChange={onQueryChange}
            keyExtractor={(pat) => pat.id}
            searchPlaceholder="Filter tokens by name, prefix, or role..."
            emptyMessage='No active Personal Access Tokens. Click "Generate New PAT" to create one.'
            defaultSortKey="created_at"
            defaultSortDirection="desc"
            pageSize={10}
            pageSizeOptions={[10, 20, 50, 100]}
            filters={[
              {
                id: "status",
                label: "Status",
                value: queryParams.filters.status ?? "all",
                options: [
                  { label: "All Statuses", value: "all" },
                  { label: "Active", value: "active" },
                  { label: "Revoked", value: "revoked" },
                ],
              },
            ]}
            columns={[
              {
                header: "Name",
                accessorKey: "name",
                cell: (pat) => <span className="font-medium text-sm">{pat.name}</span>,
              },
              {
                header: "Prefix",
                accessorKey: "token_prefix",
                cell: (pat) => <Badge variant="code">{pat.token_prefix}...</Badge>,
              },
              {
                header: "Status",
                sortValue: (pat) => (pat.revoked ? "revoked" : pat.role),
                searchValue: (pat) => (pat.revoked ? "revoked" : pat.role),
                cell: (pat) =>
                  pat.revoked ? (
                    <Badge variant="status" state="revoked">
                      Revoked
                    </Badge>
                  ) : (
                    <Badge variant="role">{pat.role}</Badge>
                  ),
              },
              {
                header: "Scopes",
                sortable: false,
                searchValue: (pat) => pat.scopes.join(" "),
                cell: (pat) => (
                  <div className="flex flex-wrap gap-1 max-w-[240px]">
                    {pat.scopes.map((sc: string) => (
                      <Badge key={sc} variant="infoLighter" size="xs">
                        {sc}
                      </Badge>
                    ))}
                  </div>
                ),
              },
              {
                header: "Created at",
                accessorKey: "created_at",
                sortValue: (pat) => new Date(pat.created_at).getTime(),
                cell: (pat) => <FormatDate date={pat.created_at} />,
              },
              {
                header: "Expires at",
                accessorKey: "expires_at",
                sortValue: (pat) => (pat.expires_at ? new Date(pat.expires_at).getTime() : 0),
                cell: (pat) =>
                  pat.expires_at ? (
                    <FormatDate date={pat.expires_at} />
                  ) : (
                    <span className="text-xs text-muted-foreground">Never</span>
                  ),
              },
              {
                header: "Actions",
                sortable: false,
                searchable: false,
                cell: (pat) =>
                  pat.revoked ? null : (
                    <Button
                      variant="ghost"
                      size="xs"
                      className="gap-1 text-destructive hover:text-destructive hover:bg-destructive/10"
                      onClick={() => {
                        if (confirm(`Revoke token "${pat.name}"?`)) {
                          revokePat({ patId: pat.id });
                        }
                      }}
                    >
                      <Trash2 className="h-3.5 w-3.5" /> Revoke
                    </Button>
                  ),
              },
            ]}
          />
        )}
      </PageSection>

      <CreatePatDialog
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        onCreated={(token, name) => setTokenData({ token, name })}
      />

      <TokenDisplayDialog tokenData={tokenData} onClose={() => setTokenData(null)} />
    </div>
  );
};

export const Route = createFileRoute("/oauth/pats")({
  component: PatsComponent,
});
