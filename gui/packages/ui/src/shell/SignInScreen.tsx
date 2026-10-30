import React from "react";
import { signIn } from "../lib/session";
import { BubbleBackground } from "../ui/bubble-background";
import { Button } from "../ui/button";
import eunomiaLogo from "../assets/eunomia_logo_lg_light.svg";

export interface SignInScreenProps {
  title?: string;
  description?: string;
  tagline?: string;
  pillars?: string;
  logoSrc?: string;
  onSignIn?: () => void;
}

export function SignInScreen({
  title = "Eunomia Console",
  description = "Your session has ended or has not started. Sign in to access the console.",
  tagline = "Trusted data spaces for a sovereign digital economy.",
  pillars = "Dataspace Protocol • Identity & Authority • Access Control",
  logoSrc = eunomiaLogo,
  onSignIn = signIn,
}: SignInScreenProps) {
  return (
    <div className="min-h-screen flex">
      {/* Left panel — Eunomia branding with interactive Bubble Background */}
      <div className="dark hidden lg:flex w-1/2 relative border-r border-ink/10 overflow-hidden bg-background">
        <BubbleBackground
          className="flex flex-col items-center justify-center px-12 gap-8 text-center"
          interactive={true}
        >
          <div className="relative z-10 flex flex-col items-center gap-6 p-8 rounded-2xl bg-sunken/40 backdrop-blur-xl border border-ink/15 shadow-2xl max-w-md">
            <img src={logoSrc} alt="Eunomia" className="w-64 drop-shadow-md" />
            <div className="space-y-2">
              <p className="text-sm font-medium text-foreground/90 leading-relaxed">{tagline}</p>
              <p className="text-xs text-brand-sky/80 font-mono">{pillars}</p>
            </div>
          </div>
        </BubbleBackground>
      </div>

      {/* Right panel — sign in at the identity provider */}
      <div className="flex flex-1 flex-col items-center justify-center bg-background px-8">
        <div className="w-full max-w-sm flex flex-col gap-6">
          <div className="flex flex-col gap-1.5">
            <h1 className="text-2xl font-semibold tracking-tight">{title}</h1>
            <p className="text-sm text-muted-foreground">{description}</p>
          </div>
          <Button className="w-full" onClick={onSignIn}>
            Sign in
          </Button>
        </div>
      </div>
    </div>
  );
}
