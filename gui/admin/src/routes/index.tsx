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

import React from "react";
import { createFileRoute, Link } from "@tanstack/react-router";
import {
  ArrowLeftRight,
  BookOpen,
  FileText,
  HandshakeIcon,
  ShieldCheck,
  Users,
  Radio,
  Lock,
  KeyRound,
  ArrowRight,
  Zap,
  Activity,
  CheckCircle2,
} from "lucide-react";
import { Button } from "shared/src/components/ui/button";
import { Badge } from "shared/src/components/ui/badge";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardFooter,
} from "shared/src/components/ui/card";

const coreModules = [
  {
    icon: Radio,
    title: "Pub/Sub Event Bus",
    tag: "Real-time Streaming",
    tagColor: "text-emerald-700 dark:text-emerald-400 bg-emerald-500/10 border-emerald-500/20",
    description:
      "High-throughput internal pub/sub event bus with exponential backoff retries, webhook dispatch, and Dead Letter Queue.",
    link: "/events/feed",
    action: "View Event Stream",
  },
  {
    icon: Lock,
    title: "OAuth 2.0 & Access Security",
    tag: "Identity & M2M",
    tagColor: "text-sky-700 dark:text-sky-400 bg-sky-500/10 border-sky-500/20",
    description:
      "Modern OAuth 2.0 authorization server supporting RFC 7523 JWT Profile, PKCE, Client Credentials, and Personal Access Tokens.",
    link: "/oauth/clients",
    action: "Manage Clients & PATs",
  },
  {
    icon: BookOpen,
    title: "Dataspace Catalog",
    tag: "DCAT-AP",
    tagColor: "text-amber-700 dark:text-amber-400 bg-amber-500/10 border-amber-500/20",
    description:
      "Publish and explore dataspace datasets, distributions, ODRL policies, and federated peer catalog discovery.",
    link: "/catalog",
    action: "Browse Offerings",
  },
  {
    icon: HandshakeIcon,
    title: "Contract Negotiations",
    tag: "Dataspace Protocol",
    tagColor: "text-purple-700 dark:text-purple-400 bg-purple-500/10 border-purple-500/20",
    description:
      "Automated negotiation protocol execution between consumers and providers with cryptographic agreement generation.",
    link: "/contract-negotiation",
    action: "Review Negotiations",
  },
  {
    icon: ArrowLeftRight,
    title: "Transfers & Dataplane",
    tag: "Data Movement",
    tagColor: "text-indigo-700 dark:text-indigo-400 bg-indigo-500/10 border-indigo-500/20",
    description:
      "End-to-end management of control-plane transfer states, data proxies, authorization tokens, and transfer audit logs.",
    link: "/transfer-process",
    action: "Monitor Transfers",
  },
  {
    icon: KeyRound,
    title: "Keystore & Config",
    tag: "Secrets Management",
    tagColor: "text-rose-700 dark:text-rose-400 bg-rose-500/10 border-rose-500/20",
    description:
      "Secure key-value parameter store, masked secret configuration, and distributed runtime node settings.",
    link: "/keystore/parameters",
    action: "Configure Keystore",
  },
];

const Index = () => {
  return (
    <div className="flex flex-1 flex-col gap-8 w-full max-w-6xl mx-auto py-4">
      {/* Hero Banner */}
      <div className="relative overflow-hidden rounded-2xl border border-ink/10 bg-gradient-to-br from-card/80 via-card/40 to-background p-8 sm:p-10 shadow-xl backdrop-blur-sm">
        <div className="absolute top-0 right-0 -mt-8 -mr-8 h-64 w-64 rounded-full bg-primary/5 blur-3xl pointer-events-none" />
        <div className="absolute bottom-0 left-1/3 -mb-8 h-48 w-48 rounded-full bg-emerald-500/5 blur-3xl pointer-events-none" />

        <div className="relative z-10 flex flex-col gap-4 max-w-2xl">
          <div className="flex items-center gap-2">
            <Badge variant="code" className="text-xs font-mono">
              Eunomia DS-Agent • Enterprise BFF Gateway
            </Badge>
            <span className="flex items-center gap-1 text-emerald-700 dark:text-emerald-400 text-xs font-mono font-medium">
              <CheckCircle2 className="h-3.5 w-3.5" /> All Services Operational
            </span>
          </div>

          <h1 className="text-3xl sm:text-4xl font-bold tracking-tight text-foreground">
            Sovereign Dataspace Control Center
          </h1>

          <p className="text-sm sm:text-base text-muted-foreground leading-relaxed">
            Unified management gateway for the Dataspace Protocol ecosystem. Orchestrate catalogs,
            automated contract negotiations, secure dataplane transfers, OAuth 2.0 client
            credentials, and reactive event streaming from a single pane of glass.
          </p>

          <div className="flex flex-wrap items-center gap-3 pt-2">
            <Link to="/catalog">
              <Button className="gap-2 shadow-sm">
                Explore Catalog <ArrowRight className="h-4 w-4" />
              </Button>
            </Link>
            <Link to="/events/feed">
              <Button variant="outline" className="gap-2">
                <Activity className="h-4 w-4 text-emerald-700 dark:text-emerald-400" />
                Live Event Bus
              </Button>
            </Link>
            <Link to="/oauth/clients">
              <Button variant="outline" className="gap-2">
                <Zap className="h-4 w-4 text-sky-700 dark:text-sky-400" />
                OAuth Applications
              </Button>
            </Link>
          </div>
        </div>
      </div>

      {/* Grid of Core Modules */}
      <div className="flex flex-col gap-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold tracking-tight text-foreground">
            Platform Capabilities
          </h2>
          <span className="text-xs text-muted-foreground font-mono">DSP 2026 Compatible</span>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {coreModules.map((mod) => {
            const Icon = mod.icon;
            return (
              <Card
                key={mod.title}
                variant="interactive"
                className="group flex flex-col justify-between"
              >
                <CardHeader className="pb-3 space-y-3">
                  <div className="flex items-center justify-between">
                    <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 text-brand-sky group-hover:scale-105 transition-transform">
                      <Icon className="h-4 w-4" />
                    </div>
                    <span
                      className={`text-xs font-mono font-medium px-2 py-0.5 rounded-full border ${mod.tagColor}`}
                    >
                      {mod.tag}
                    </span>
                  </div>

                  <div>
                    <CardTitle className="text-base font-semibold mb-1 group-hover:text-brand-sky transition-colors">
                      {mod.title}
                    </CardTitle>
                    <CardDescription className="line-clamp-2">{mod.description}</CardDescription>
                  </div>
                </CardHeader>

                <CardFooter className="pt-3 border-t border-ink/5 bg-background-800/20 rounded-b-xl">
                  <Link
                    to={mod.link}
                    className="inline-flex items-center gap-1.5 text-xs font-medium text-primary hover:underline transition-colors"
                  >
                    <span>{mod.action}</span>
                    <ArrowRight className="h-3 w-3 transition-transform group-hover:translate-x-0.5" />
                  </Link>
                </CardFooter>
              </Card>
            );
          })}
        </div>
      </div>
    </div>
  );
};

export const Route = createFileRoute("/")({
  component: Index,
});

export default Index;
