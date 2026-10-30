import * as react_jsx_runtime from 'react/jsx-runtime';
import React__default from 'react';

interface WalletLayoutProps {
    /** Base route prefix for wallet tabs; defaults to "/wallet" */
    basePath?: string;
    /** Optional controlled linked state (e.g., from SSIAuthContext in DS-Agent) */
    onboarded?: boolean;
    /** Optional controlled link callback */
    onLink?: () => void | Promise<void>;
    /** Optional controlled linking spinner state */
    isLinking?: boolean;
    children?: React__default.ReactNode;
}
declare function WalletLayout({ basePath, onboarded: controlledOnboarded, onLink: controlledOnLink, isLinking: controlledIsLinking, children, }: WalletLayoutProps): react_jsx_runtime.JSX.Element;

interface WalletKeyRef {
    internal: string;
    fragment: string;
}
interface WalletDidModel {
    id: string;
    did: string;
    alias: string;
    default: boolean;
    type: string;
    keys: WalletKeyRef[];
    default_key: WalletKeyRef;
    did_document: unknown;
    service?: unknown[] | null;
}
interface WalletInfoData {
    id: string;
    name: string;
    createdOn: string;
    addedOn: string;
    permission: string;
    dids: WalletDidModel[];
}
interface WalletInfoResponse {
    status: number;
    data: WalletInfoData;
}
declare function WalletInfoView({ didTabHref }: {
    didTabHref?: string;
}): react_jsx_runtime.JSX.Element;

declare function WalletDidView(): react_jsx_runtime.JSX.Element;

interface WalletKeyItem {
    id: string;
    alias?: string | null;
    kty: string;
    crv?: string | null;
    created_at?: string | null;
}
declare const WALLET_KEYS_QUERY_KEY: readonly ["/wallet/keys"];
/** Private keys held by the wallet, the DIDs that use each one, and PEM import. */
declare function WalletKeysView(): react_jsx_runtime.JSX.Element;

interface WalletVcRecord {
    id: string;
    added_on?: string | null;
    addedOn?: string | null;
    valid_until?: string | null;
    parsed_document?: {
        type?: string[];
        issuer?: {
            name?: string;
            id?: string;
        };
        validUntil?: string;
        expirationDate?: string;
        [key: string]: unknown;
    };
    [key: string]: unknown;
}
interface WalletCredentialsViewProps {
    /** Whether to show the Gaia-X 4-step compliance pipeline; defaults to true */
    showGaiaCompliance?: boolean;
    /** Route builder for requesting a VC from an authority; defaults to `/authority/new?recommended=...` */
    requestVcHref?: (recommended: string) => string;
}
declare function WalletCredentialsView({ showGaiaCompliance, requestVcHref, }?: WalletCredentialsViewProps): react_jsx_runtime.JSX.Element;

declare function WalletOidc4vpView(): react_jsx_runtime.JSX.Element;

declare function WalletOidc4vciView(): react_jsx_runtime.JSX.Element;

export { WALLET_KEYS_QUERY_KEY, WalletCredentialsView, type WalletCredentialsViewProps, type WalletDidModel, WalletDidView, type WalletInfoData, type WalletInfoResponse, WalletInfoView, type WalletKeyItem, type WalletKeyRef, WalletKeysView, WalletLayout, type WalletLayoutProps, WalletOidc4vciView, WalletOidc4vpView, type WalletVcRecord };
