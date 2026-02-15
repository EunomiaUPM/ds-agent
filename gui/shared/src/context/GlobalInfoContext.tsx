import { setApiGatewayBase as setOrvalApiGatewayBase } from "../data/orval-mutator";

import { createContext, ReactNode, useEffect, useMemo, useState } from "react";
import React from "react";

export interface GlobalInfoContextType {
  catalog_type: "rainbow" | "datahub" | "both";
  dsrole: string;
  api_gateway_base: string;
  api_gateway: string;
  api_gateway_callback_address: string;
}

type ConfigInfo = {
  config_role: string;
};

export const GlobalInfoContext = createContext<GlobalInfoContextType | null>(null);

export const GlobalInfoContextProvider = ({ children }: { children: ReactNode }) => {
  const [apiGatewayBase, setApiGatewayBase] = useState<string>("/");
  const [configRole, setConfigRole] = useState<string>("");
  const [catalogType] = useState<"rainbow" | "datahub" | "both">("rainbow");
  const [isConfigLoaded, setIsConfigLoaded] = useState(false);

  /**
   * CHANGE ME - Configuration Logic
   * Production vs Development Configuration
   * In production, fetch configuration from server endpoint.
   * In development, use local hardcoded configuration.
   */
  const isProduction = false; // Set to false for local development
  const localConfig = {
    config_role: "Agent",
    gateway_host: "http://127.0.0.1",
    gateway_port: import.meta.env.VITE_GATEWAY_PORT || "1200",
  };

  /**
   * Initialize configuration on mount
   */
  useEffect(() => {
    const initConfig = async () => {
      try {
        if (isProduction) {
          setApiGatewayBase("/");
          const res = await fetch("/admin/api/fe-config");
          if (res.ok) {
            const data = await res.json();
            setConfigRole(data.config_role);
            console.log("Prod Config Loaded. Role:", data.config_role, "Base: (Relative)");
          } else {
            console.error("Error cargando config en prod:", res.status);
          }
        } else {
          const localBase = `${localConfig.gateway_host}:${localConfig.gateway_port}`;
          setApiGatewayBase(localBase);
          setConfigRole(localConfig.config_role);
          console.log("Dev Config Loaded. Base:", localBase);
        }
      } catch (e) {
        console.error("Critical Error initConfig:", e);
      } finally {
        setIsConfigLoaded(true);
      }
    };

    initConfig();
  }, []);


  /**
   * Memoized context value
   */
  /**
   * Update global configuration for Orval when gateway changes
   */
  useEffect(() => {
    const prefix = apiGatewayBase === "/" ? "" : apiGatewayBase;
    const gateway = `${prefix}/admin/api`;
    setOrvalApiGatewayBase(gateway);
  }, [apiGatewayBase]);

  /**
   * Memoized context value
   */
  const contextValue = useMemo<GlobalInfoContextType>(() => {
    const prefix = apiGatewayBase === "/" ? "" : apiGatewayBase;
    const gateway = `${prefix}/admin/api`;

    return {
      catalog_type: catalogType,
      dsrole: configRole,
      api_gateway_base: prefix,
      api_gateway: gateway,
      api_gateway_callback_address: `${prefix}/admin/api/incoming-notification`,
    };
  }, [catalogType, configRole, apiGatewayBase]);

  /**
   * Render when config is loaded
   */
  if (!isConfigLoaded) {
    return <div>Loading...</div>;
  }

  return <GlobalInfoContext.Provider value={contextValue}>{children}</GlobalInfoContext.Provider>;
};
