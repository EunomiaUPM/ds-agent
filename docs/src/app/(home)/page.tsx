import Link from 'next/link';
import {
    ArrowRight,
    Terminal,
    Cpu,
    ShieldCheck,
    Zap,
    Database,
    Network,
    FileSignature,
    Layers,
    GitBranch,
    Server
} from 'lucide-react';

export default function HomePage() {
    return (
        <div className="relative flex flex-col items-center justify-center overflow-hidden bg-background text-foreground">

            <main className="container relative z-10 flex flex-col items-center px-4 py-16 md:py-24 lg:py-32">

                {/* 2. HERO SECTION */}
                <div className="flex max-w-[980px] flex-col items-center gap-4 text-center">
                    {/* Badge de Origen */}
                    <div className="inline-flex items-center rounded-full border border-muted bg-muted/50 px-3 py-1 text-sm font-medium backdrop-blur-sm mb-2">
                        <Link href="https://ging.github.io/"><span className="inline-flex items-center">
                            <span className="flex h-2 w-2 rounded-full bg-indigo-500 mr-2"></span>
                            Built by GING @ UPM
                            </span>
                        </Link>
                    </div>

                    {/* Título Principal */}
                    <h1 className="text-4xl font-bold leading-tight tracking-tighter md:text-6xl lg:leading-[1.1]">
                        Eunomia DS-Agent <br className="hidden md:block" />
                        <span className="bg-gradient-to-r from-indigo-400 via-purple-400 to-pink-400 bg-clip-text text-transparent">
              Dataspace Agent
            </span>
                    </h1>

                    <p className="max-w-[750px] text-lg text-muted-foreground sm:text-xl mt-4">
                        A Rust-native, multi-protocol implementation for the Next Generation Internet.
                        Secure, decentralized, and interoperable.
                    </p>

                    <div className="flex flex-wrap items-center justify-center gap-4 mt-8">
                        <Link
                            href="/docs"
                            className="inline-flex h-10 items-center justify-center rounded-md bg-primary px-8 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        >
                            Get Started
                        </Link>
                        <Link
                            href="https://github.com/EunomiaUPM/ds-agent" // Asumo la URL basada en el contexto, ajústala si es necesario
                            target="_blank"
                            rel="noreferrer"
                            className="inline-flex h-10 items-center justify-center rounded-md border border-input bg-background px-8 text-sm font-medium shadow-sm transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
                        >
                            GitHub
                        </Link>
                    </div>
                </div>

                {/* 3. TERMINAL / QUICK START */}
                <div className="w-full max-w-3xl mt-16 perspective-1000">
                    <div className="relative overflow-hidden rounded-xl border bg-card/50 shadow-2xl backdrop-blur-sm transition-all hover:bg-card/80">
                        <div className="flex items-center justify-between border-b bg-muted/50 px-4 py-3">
                            <div className="flex items-center gap-2">
                                <div className="h-3 w-3 rounded-full bg-red-500/20"></div>
                                <div className="h-3 w-3 rounded-full bg-yellow-500/20"></div>
                                <div className="h-3 w-3 rounded-full bg-green-500/20"></div>
                            </div>
                            <div className="text-xs text-muted-foreground font-mono">bash</div>
                        </div>
                        <div className="p-6 font-mono text-sm overflow-x-auto">
                            {/* Docker example */}
                            <div className="flex gap-2">
                                <span className="text-primary font-bold">➜</span>
                                <span className="text-foreground">docker pull quay.io/eunomia_upm/ds-agent</span>
                            </div>
                            <div className="flex gap-2 mt-2 text-muted-foreground">
                                <span>Using default tag: latest...</span>
                            </div>
                            <div className="flex gap-2 mt-2 text-green-500 font-bold">
                                <span>✔ Pulled image successfully</span>
                            </div>

                            {/* Script example */}
                            <div className="flex gap-2 mt-4">
                                <span className="text-primary font-bold">➜</span>
                                <span className="text-foreground">./scripts/bash/auto-setup.sh</span>
                            </div>
                            <div className="flex gap-2 mt-2 text-muted-foreground">
                                <span>Initializing SSI wallets & catalogs...</span>
                            </div>
                            <div className="flex gap-2 mt-4">
                                <span className="text-primary font-bold">➜</span>
                                <span className="text-foreground cursor-blink">_</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* 4. KEY CONCEPTS GRID */}
                <div className="mt-24 w-full max-w-6xl">
                    <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl text-center mb-12">
                        Key Concepts
                    </h2>
                    <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
                        <FeatureCard
                            icon={<ShieldCheck className="h-6 w-6 text-emerald-500" />}
                            title="SSI Authentication"
                            description="Self-Sovereign Identity based authentication using verifiable credentials and decentralized identifiers."
                        />
                        <FeatureCard
                            icon={<Database className="h-6 w-6 text-blue-500" />}
                            title="Catalog Management"
                            description="DCAT3-compatible catalog system for efficient dataset and data service discovery."
                        />
                        <FeatureCard
                            icon={<FileSignature className="h-6 w-6 text-purple-500" />}
                            title="Contract Negotiation"
                            description="Full implementation of the Dataspace Protocol's contract negotiation flow (ODRL policies)."
                        />
                        <FeatureCard
                            icon={<Network className="h-6 w-6 text-orange-500" />}
                            title="Datahub Proxy"
                            description="Seamless integration layer acting as a proxy for external data hubs and repositories."
                        />
                        <FeatureCard
                            icon={<Zap className="h-6 w-6 text-yellow-500" />}
                            title="Data Transfer"
                            description="Robust control plane and data plane for secure, policy-compliant data transfers."
                        />
                        <FeatureCard
                            icon={<Layers className="h-6 w-6 text-pink-500" />}
                            title="Dynamic Stack"
                            description="Designed with a multi-protocol orientation and flexible architecture."
                        />
                    </div>
                </div>

                {/* 5. TECH HIGHLIGHTS */}
                <div className="mt-24 flex flex-col items-center w-full max-w-5xl">
                    <div className="grid grid-cols-2 md:grid-cols-4 gap-8 w-full text-center">
                        <div className="flex flex-col items-center gap-2">
                            <Cpu className="h-8 w-8 text-primary mb-2" />
                            <h3 className="font-bold">Rust Native</h3>
                            <p className="text-sm text-muted-foreground">Async Tokio Runtime</p>
                        </div>
                        <div className="flex flex-col items-center gap-2">
                            <Server className="h-8 w-8 text-primary mb-2" />
                            <h3 className="font-bold">HTTP APIs</h3>
                            <p className="text-sm text-muted-foreground">Axum & SeaORM</p>
                        </div>
                        <div className="flex flex-col items-center gap-2">
                            <Network className="h-8 w-8 text-primary mb-2" />
                            <h3 className="font-bold">gRPC Support</h3>
                            <p className="text-sm text-muted-foreground">Inter-service comms</p>
                        </div>
                        <div className="flex flex-col items-center gap-2">
                            <Terminal className="h-8 w-8 text-primary mb-2" />
                            <h3 className="font-bold">Low Footprint</h3>
                            <p className="text-sm text-muted-foreground">Minimal memory usage</p>
                        </div>
                    </div>
                </div>

                {/* 6. CRATE ORGANIZATION (Architecture) */}
                <div className="mt-24 w-full max-w-4xl">
                    <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl text-center mb-8">
                        Modular Architecture
                    </h2>
                    <div className="rounded-lg border bg-card text-card-foreground shadow-sm">
                        <div className="p-6 grid gap-6 md:grid-cols-3">
                            <div className="space-y-4">
                                <div className="flex items-center gap-2 font-semibold text-lg text-indigo-400">
                                    <Cpu className="h-5 w-5" /> Core
                                </div>
                                <ul className="text-sm text-muted-foreground space-y-2 list-disc pl-4">
                                    <li>core</li>
                                    <li>common</li>
                                    <li>db</li>
                                    <li>events</li>
                                </ul>
                            </div>

                            <div className="space-y-4">
                                <div className="flex items-center gap-2 font-semibold text-lg text-purple-400">
                                    <GitBranch className="h-5 w-5" /> Protocol
                                </div>
                                <ul className="text-sm text-muted-foreground space-y-2 list-disc pl-4">
                                    <li>catalog</li>
                                    <li>contracts</li>
                                    <li>transfer</li>
                                    <li>dataplane</li>
                                </ul>
                            </div>

                            <div className="space-y-4">
                                <div className="flex items-center gap-2 font-semibold text-lg text-pink-400">
                                    <Network className="h-5 w-5" /> Gateway
                                </div>
                                <ul className="text-sm text-muted-foreground space-y-2 list-disc pl-4">
                                    <li>auth</li>
                                    <li>authority</li>
                                    <li>fe-gateway</li>
                                    <li>datahub</li>
                                </ul>
                            </div>
                        </div>
                    </div>
                </div>

                {/* 7. CTA BOTTOM */}
                <div className="mt-32 flex flex-col items-center gap-4 text-center pb-16">
                    <h2 className="text-3xl font-bold tracking-tighter sm:text-4xl">
                        Ready to integrate?
                    </h2>
                    <p className="max-w-[600px] text-muted-foreground md:text-xl">
                        Join the Next Generation Internet data economy.
                    </p>
                    <div className="flex gap-4 mt-4">
                        <Link
                            href="/docs"
                            className="inline-flex h-11 items-center justify-center rounded-md bg-primary px-8 text-sm font-medium text-primary-foreground shadow transition-colors hover:bg-primary/90"
                        >
                            Documentation
                        </Link>
                    </div>

                </div>

            </main>
        </div>
    );
}

// Subcomponente simple para las cards
function FeatureCard({ icon, title, description }: { icon: React.ReactNode, title: string, description: string }) {
    return (
        <div className="group relative overflow-hidden rounded-lg border bg-background p-6 hover:shadow-md transition-all hover:border-primary/50">
            <div className="mb-4 flex h-10 w-10 items-center justify-center rounded-lg border bg-muted/50 group-hover:bg-muted/80 transition-colors">
                {icon}
            </div>
            <h3 className="mb-2 font-semibold tracking-tight text-lg">{title}</h3>
            <p className="text-sm text-muted-foreground leading-relaxed">{description}</p>
        </div>
    );
}