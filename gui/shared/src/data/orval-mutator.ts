import { refreshOAuthToken, clearSession } from "../lib/session";
import { getActingTenant } from "../lib/tenant";

// Custom mutator for Orval that uses a global API gateway configuration
let API_GATEWAY_BASE: string = "";

export const setApiGatewayBase = (url: string) => {
  API_GATEWAY_BASE = url;
};

export const getApiGatewayBase = (): string => API_GATEWAY_BASE;

export type RequestConfig = RequestInit;

let refreshPromise: Promise<string | null> | null = null;

// NOTE: Adjusted signature to match Orval's default generation: (url, config)
export const customInstance = async <T>(
  url: string,
  options: {
    method?: string;
    headers?: any;
    params?: any;
    data?: any;
    _isRetry?: boolean;
    // Skips the acting tenant, for admin views that span every tenant.
    _allTenants?: boolean;
  } & Partial<RequestConfig>,
): Promise<T> => {
  const { method, headers, params, data, _isRetry, _allTenants, ...rest } = options || {};

  const token =
    (headers as any)?.Authorization?.replace("Bearer ", "") ||
    (typeof localStorage !== "undefined"
      ? localStorage.getItem("eunomia_token") ||
        localStorage.getItem("access_token") ||
        localStorage.getItem("pat_token")
      : null);

  const authHeader = token ? { Authorization: `Bearer ${token}` } : {};
  const actingTenant = _allTenants ? null : getActingTenant();
  const tenantHeader = actingTenant ? { "x-tenant-id": actingTenant } : {};

  const config: RequestConfig = {
    ...rest,
    headers: {
      "Content-Type": "application/json",
      ...authHeader,
      ...tenantHeader,
      ...headers,
      ...(rest as any)?.headers,
    },
  };

  // Convert params to query string if present
  let targetUrl = url;
  if (params) {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
          value.forEach((v) => searchParams.append(key, String(v)));
        } else {
          searchParams.append(key, String(value));
        }
      }
    });
    const queryString = searchParams.toString();
    // Handle existing query params in url
    targetUrl += (targetUrl.includes("?") ? "&" : "?") + queryString;
  }

  // Prepend API Gateway URL
  const fullUrl = API_GATEWAY_BASE ? `${API_GATEWAY_BASE}${targetUrl}` : targetUrl;

  const requestMethod = method || "GET";

  const fetchOptions: RequestInit = {
    ...config,
    method: requestMethod,
  };

  if (data) {
    fetchOptions.body = JSON.stringify(data);
  }

  const response = await fetch(fullUrl, fetchOptions);
  let data_1: any;

  // Handle empty responses
  if (response.status === 204) {
    data_1 = {};
  } else {
    try {
      data_1 = await response.clone().json();
    } catch (error) {
      // If JSON parsing fails (e.g. text response), return text
      data_1 = await response.text();
    }
  }

  // Automatically refresh token on 401 Unauthorized or expired token error
  const isUnauthorized =
    response.status === 401 ||
    (data_1 &&
      typeof data_1 === "object" &&
      (data_1.error_code === 4200 || data_1.message === "Unauthorized Error"));

  const isAuthEndpoint =
    url.includes("/oauth/refresh") ||
    url.includes("/oauth/token") ||
    url.includes("/oauth/login");

  if (isUnauthorized && !isAuthEndpoint && !_isRetry) {
    if (!refreshPromise) {
      refreshPromise = refreshOAuthToken(API_GATEWAY_BASE).finally(() => {
        refreshPromise = null;
      });
    }

    const newToken = await refreshPromise;
    if (newToken) {
      return customInstance<T>(url, {
        ...options,
        headers: {
          ...options.headers,
          Authorization: `Bearer ${newToken}`,
        },
        _isRetry: true,
      });
    } else {
      clearSession();
      if (typeof window !== "undefined") {
        window.dispatchEvent(new Event("eunomia:unauthorized"));
      }
    }
  }

  // If response is a Paginated envelope ({ items: [...], total }), attach envelope properties to the array
  if (
    data_1 &&
    typeof data_1 === "object" &&
    !Array.isArray(data_1) &&
    Array.isArray(data_1.items)
  ) {
    const arr = [...data_1.items] as any;
    arr.items = data_1.items;
    arr.total = data_1.total;
    arr.nextCursor = data_1.nextCursor;
    data_1 = arr;
  }

  // Return the response structure expected by Orval generated types
  return {
    status: response.status,
    data: data_1,
    headers: response.headers,
  } as T;
};

export type ErrorType<Error> = Error;
export type BodyType<Body> = Body;
