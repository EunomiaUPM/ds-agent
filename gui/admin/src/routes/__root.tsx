import { QueryClient } from "@tanstack/react-query";
import {
  createRootRouteWithContext,
  Outlet,
  useLocation,
  useNavigate,
} from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { SidebarInset, SidebarProvider } from "../../../shared/src/components/ui/sidebar";
import React, { useEffect, useState } from "react";
import { AppSidebar } from "shared/src/components/AppSidebar.tsx";
import { Header } from "shared/src/components/header.tsx";
import { GeneralErrorComponent } from "../components/GeneralErrorComponent";
import { GlobalLoadingIndicator } from "../components/GlobalLoadingIndicator";
import { Input } from "shared/src/components/ui/input";
import { Button } from "shared/src/components/ui/button";
import { Eye, EyeOff } from "lucide-react";
import { clearSession, isSessionActive, setSession } from "../lib/session";
import eunomiaLogo from "shared/src/img/eunomia_logo_lg_light.svg";
import { DevMode } from "shared/src/components/DevMode";

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
  api_gateway: string;
}>()({
  component: RootComponent,
  errorComponent: GeneralErrorComponent,
});

function RootComponent() {
  const [authed, setAuthed] = useState(isSessionActive);
  const { pathname } = useLocation();
  const navigate = useNavigate();

  useEffect(() => {
    if (!authed && !pathname.startsWith("/login")) {
      navigate({ to: "/login/" });
    }
    if (authed && pathname.startsWith("/login")) {
      navigate({ to: "/" });
    }
  }, [authed, pathname]);

  if (!authed) {
    return <LoginForm onSuccess={() => setAuthed(true)} />;
  }

  return (
    <>
      <SidebarProvider>
        <DevMode />
        <AppSidebar />
        <SidebarInset>
          <Header
            onSignOut={() => {
              clearSession();
              setAuthed(false);
            }}
          />
          <div className="flex flex-1 flex-col gap-4 p-8 items-start justify-start w-full h-full min-w-0">
            <Outlet />
          </div>
        </SidebarInset>
      </SidebarProvider>
      <GlobalLoadingIndicator />
      <TanStackRouterDevtools />
    </>
  );
}

const VALID_USER = "eunomia";
const VALID_PASSWORD = "eunomia";

function LoginForm({ onSuccess }: { onSuccess: () => void }) {
  const navigate = useNavigate();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (username === VALID_USER && password === VALID_PASSWORD) {
      setSession();
      onSuccess();
      navigate({ to: "/" });
    } else {
      setError("Invalid username or password.");
    }
  };

  return (
    <div className="min-h-screen flex">
      {/* Left panel — Eunomia branding */}
      <div className="hidden lg:flex w-1/2 flex-col items-center justify-center bg-background border-r border-white/10 px-16 gap-8">
        <img src={eunomiaLogo} alt="Eunomia" className="w-72" />
        <p className="text-sm text-center text-muted-foreground max-w-xs">
          Trusted data spaces for a sovereign digital economy.
        </p>
      </div>

      {/* Right panel — login form */}
      <div className="flex flex-1 flex-col items-center justify-center bg-background px-8">
        <div className="w-full max-w-sm flex flex-col gap-6">
          <div className="flex flex-col gap-1">
            <h1 className="text-2xl font-semibold tracking-tight">Eunomia DS-Agent Admin</h1>
            <p className="text-sm text-muted-foreground">Sign in to continue</p>
          </div>

          <form onSubmit={handleSubmit} className="flex flex-col gap-4">
            <div className="flex flex-col gap-1.5">
              <label htmlFor="username" className="text-sm font-medium">
                Username
              </label>
              <Input
                id="username"
                type="text"
                autoComplete="username"
                value={username}
                onChange={(e) => {
                  setUsername(e.target.value);
                  setError(null);
                }}
                placeholder="agent"
              />
            </div>

            <div className="flex flex-col gap-1.5">
              <label htmlFor="password" className="text-sm font-medium">
                Password
              </label>
              <div className="relative">
                <Input
                  id="password"
                  type={showPassword ? "text" : "password"}
                  autoComplete="current-password"
                  value={password}
                  onChange={(e) => {
                    setPassword(e.target.value);
                    setError(null);
                  }}
                  placeholder="••••••••"
                  className="pr-10"
                />
                <button
                  type="button"
                  onClick={() => setShowPassword((v) => !v)}
                  className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
                  tabIndex={-1}
                >
                  {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
            </div>

            {error && <p className="text-sm text-destructive">{error}</p>}

            <Button type="submit" className="w-full">
              Sign in
            </Button>
          </form>
        </div>
      </div>
    </div>
  );
}
