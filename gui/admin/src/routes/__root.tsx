import { QueryClient } from "@tanstack/react-query";
import { createRootRouteWithContext, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/router-devtools";
import { SidebarInset, SidebarProvider } from "../../../shared/src/components/ui/sidebar";
import { useEffect, useState } from "react";
import { AppSidebar } from "shared/src/components/AppSidebar.tsx";
import { Header } from "shared/src/components/header.tsx";
import { GeneralErrorComponent } from "../components/GeneralErrorComponent";
import { GlobalLoadingIndicator } from "../components/GlobalLoadingIndicator";
import { Button } from "shared/src/components/ui/button";
import { Loader2 } from "lucide-react";
import {
  loadSession,
  signIn,
  signOut,
  type SessionUser,
  UNAUTHORIZED_EVENT,
} from "shared/src/lib/session";
import eunomiaLogo from "shared/src/img/eunomia_logo_lg_light.svg";
import { Toaster } from "shared/src/components/ui/sonner";
import { BubbleBackground } from "shared/src/components/ui/bubble-background";

export const Route = createRootRouteWithContext<{
  queryClient: QueryClient;
  api_gateway: string;
}>()({
  component: RootComponent,
  errorComponent: GeneralErrorComponent,
});

/** `undefined` while asking the gateway; `null` when there is no session. */
type SessionState = SessionUser | null | undefined;

function RootComponent() {
  const [user, setUser] = useState<SessionState>(undefined);

  useEffect(() => {
    loadSession().then(setUser);
    const handleUnauthorized = () => setUser(null);
    window.addEventListener(UNAUTHORIZED_EVENT, handleUnauthorized);
    return () => window.removeEventListener(UNAUTHORIZED_EVENT, handleUnauthorized);
  }, []);

  if (user === undefined) {
    return (
      <div className="flex h-screen w-full items-center justify-center">
        <Loader2 className="h-8 w-8 animate-spin text-brand-sky" />
      </div>
    );
  }

  if (user === null) {
    return <SignInScreen />;
  }

  return (
    <>
      <SidebarProvider>
        <AppSidebar />
        <SidebarInset>
          <Header user={user} onSignOut={signOut} />
          <div className="flex flex-1 flex-col gap-4 p-8 items-start justify-start w-full h-full min-w-0">
            <Outlet />
          </div>
        </SidebarInset>
      </SidebarProvider>
      <Toaster />
      <GlobalLoadingIndicator />
      <TanStackRouterDevtools />
    </>
  );
}

/** No session: sign-in happens at the identity provider, through the gateway's proxy. */
function SignInScreen() {
  return (
    <div className="min-h-screen flex">
      {/* Left panel — Eunomia branding with interactive Bubble Background */}
      <div className="dark hidden lg:flex w-1/2 relative border-r border-ink/10 overflow-hidden bg-background">
        <BubbleBackground
          className="flex flex-col items-center justify-center px-12 gap-8 text-center"
          interactive={true}
        >
          <div className="relative z-10 flex flex-col items-center gap-6 p-8 rounded-2xl bg-sunken/40 backdrop-blur-xl border border-ink/15 shadow-2xl max-w-md">
            <img src={eunomiaLogo} alt="Eunomia" className="w-64 drop-shadow-md" />
            <div className="space-y-2">
              <p className="text-sm font-medium text-foreground/90 leading-relaxed">
                Trusted data spaces for a sovereign digital economy.
              </p>
              <p className="text-xs text-brand-sky/80 font-mono">
                Dataspace Protocol • Identity & Authority • Access Control
              </p>
            </div>
          </div>
        </BubbleBackground>
      </div>

      {/* Right panel — sign in at the identity provider */}
      <div className="flex flex-1 flex-col items-center justify-center bg-background px-8">
        <div className="w-full max-w-sm flex flex-col gap-6">
          <div className="flex flex-col gap-1.5">
            <h1 className="text-2xl font-semibold tracking-tight">Eunomia DS-Agent Admin</h1>
            <p className="text-sm text-muted-foreground">
              Your session has ended or has not started. Sign in to access the console.
            </p>
          </div>
          <Button className="w-full" onClick={signIn}>
            Sign in
          </Button>
        </div>
      </div>
    </div>
  );
}

// Password login against the former built-in OAuth module, now disabled (sign-in happens at
// the identity provider). Kept as it was, out of use.
// function LoginForm({ onSuccess }: { onSuccess: () => void }) {
//   const navigate = useNavigate();
//   const [username, setUsername] = useState("");
//   const [password, setPassword] = useState("");
//   const [showPassword, setShowPassword] = useState(false);
//   const [loading, setLoading] = useState(false);
//   const [error, setError] = useState<string | null>(null);
//
//   const handleSubmit = async (e: React.FormEvent) => {
//     e.preventDefault();
//     if (!username.trim() || !password) {
//       setError("Please provide username and password.");
//       return;
//     }
//
//     setLoading(true);
//     setError(null);
//
//     try {
//       const res = await loginWithOAuth(username.trim(), password);
//       if (res.success) {
//         onSuccess();
//         navigate({ to: "/" });
//       } else {
//         setError(res.error || "Authentication failed. Please verify your credentials.");
//       }
//     } catch (err: any) {
//       setError(err?.message || "An unexpected error occurred during login.");
//     } finally {
//       setLoading(false);
//     }
//   };
//
//   return (
//     <div className="min-h-screen flex">
//       {/* Left panel — Eunomia branding with interactive Bubble Background */}
//       <div className="dark hidden lg:flex w-1/2 relative border-r border-ink/10 overflow-hidden bg-background">
//         <BubbleBackground
//           className="flex flex-col items-center justify-center px-12 gap-8 text-center"
//           interactive={true}
//         >
//           {/* Glassmorphic card overlay for crisp brand presentation */}
//           <div className="relative z-10 flex flex-col items-center gap-6 p-8 rounded-2xl bg-sunken/40 backdrop-blur-xl border border-ink/15 shadow-2xl max-w-md">
//             <img src={eunomiaLogo} alt="Eunomia" className="w-64 drop-shadow-md" />
//             <div className="space-y-2">
//               <p className="text-sm font-medium text-foreground/90 leading-relaxed">
//                 Trusted data spaces for a sovereign digital economy.
//               </p>
//               <p className="text-xs text-brand-sky/80 font-mono">
//                 Dataspace Protocol • Identity & Authority • Access Control
//               </p>
//             </div>
//             <div className="flex items-center gap-2 pt-3 border-t border-ink/10 w-full justify-center">
//               <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
//               <span className="text-xs font-mono text-muted-foreground">Eunomia DS-Agent v0.4.0</span>
//             </div>
//           </div>
//         </BubbleBackground>
//       </div>
//
//       {/* Right panel — login form */}
//       <div className="flex flex-1 flex-col items-center justify-center bg-background px-8">
//         <div className="w-full max-w-sm flex flex-col gap-6">
//           <div className="flex flex-col gap-1.5">
//             <div className="flex items-center gap-2">
//               <span className="h-2 w-2 rounded-full bg-emerald-400 animate-pulse" />
//               <span className="text-xs font-mono uppercase tracking-wider text-muted-foreground">
//                 OAuth 2.0 / OIDC Authentication
//               </span>
//             </div>
//             <h1 className="text-2xl font-semibold tracking-tight">Eunomia DS-Agent Admin</h1>
//             <p className="text-sm text-muted-foreground">
//               Sign in with your operator credentials to access the console
//             </p>
//           </div>
//
//           <form onSubmit={handleSubmit} className="flex flex-col gap-4">
//             <div className="flex flex-col gap-1.5">
//               <label htmlFor="username" className="text-sm font-medium">
//                 Username / Email
//               </label>
//               <Input
//                 id="username"
//                 type="text"
//                 autoComplete="username"
//                 value={username}
//                 onChange={(e) => {
//                   setUsername(e.target.value);
//                   setError(null);
//                 }}
//                 placeholder="admin or admin@admin.local"
//                 required
//               />
//             </div>
//
//             <div className="flex flex-col gap-1.5">
//               <label htmlFor="password" className="text-sm font-medium">
//                 Password
//               </label>
//               <div className="relative">
//                 <Input
//                   id="password"
//                   type={showPassword ? "text" : "password"}
//                   autoComplete="current-password"
//                   value={password}
//                   onChange={(e) => {
//                     setPassword(e.target.value);
//                     setError(null);
//                   }}
//                   placeholder="••••••••"
//                   className="pr-10"
//                   required
//                 />
//                 <button
//                   type="button"
//                   onClick={() => setShowPassword((v) => !v)}
//                   className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
//                   tabIndex={-1}
//                 >
//                   {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
//                 </button>
//               </div>
//             </div>
//
//             {error && (
//               <div className="p-3 rounded-lg bg-danger-50 dark:bg-danger-950/40 border border-danger-700/50 text-xs text-danger-700 dark:text-danger-300">
//                 {error}
//               </div>
//             )}
//
//             <Button type="submit" className="w-full" isLoading={loading}>
//               Sign in
//             </Button>
//           </form>
//
//           <p className="text-xs text-muted-foreground/60 text-center font-mono">
//             Secured with RFC 6749 OAuth 2.0 & RFC 7519 JWT Bearer Tokens
//           </p>
//         </div>
//       </div>
//     </div>
//   );
// }
