// Requests carry no token: the session is a cookie of oauth2-proxy, which adds the token on the
// way to the agents (or there is none, in `static` mode). See lib/session.ts.

/** Event fired when a call finds no session (401). */
export const UNAUTHORIZED_EVENT = "eunomia:unauthorized";

let API_GATEWAY_BASE: string = "";

export const setApiGatewayBase = (url: string) => {
  API_GATEWAY_BASE = url;
};

export const getApiGatewayBase = (): string => API_GATEWAY_BASE;

export type RequestConfig = RequestInit;

export interface CustomInstanceOptions extends Partial<RequestConfig> {
  method?: string;
  headers?: HeadersInit;
  params?: Record<string, unknown>;
  data?: unknown;
  /** A 401 here does not end the session (used to ask whether there is one). */
  _noSessionRedirect?: boolean;
}

export const customInstance = async <T>(
  url: string,
  options?: CustomInstanceOptions,
): Promise<T> => {
  const { method, headers, params, data, _noSessionRedirect, ...rest } = options || {};

  const normalizedHeaders: Record<string, string> = {
    "Content-Type": "application/json",
  };
  if (headers) {
    if (headers instanceof Headers) {
      headers.forEach((v, k) => {
        normalizedHeaders[k] = v;
      });
    } else if (Array.isArray(headers)) {
      headers.forEach(([k, v]) => {
        normalizedHeaders[k] = v;
      });
    } else {
      Object.entries(headers).forEach(([k, v]) => {
        if (v !== undefined) {
          normalizedHeaders[k] = Array.isArray(v) ? v.join(", ") : String(v);
        }
      });
    }
  }

  const config: RequestConfig = {
    ...rest,
    headers: normalizedHeaders,
  };

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
    if (queryString) {
      targetUrl += (targetUrl.includes("?") ? "&" : "?") + queryString;
    }
  }

  const fullUrl = API_GATEWAY_BASE ? `${API_GATEWAY_BASE}${targetUrl}` : targetUrl;
  const requestMethod = method || "GET";

  const fetchOptions: RequestInit = {
    ...config,
    method: requestMethod,
  };

  if (data !== undefined) {
    fetchOptions.body = JSON.stringify(data);
  }

  const response = await fetch(fullUrl, fetchOptions);
  let parsedData: unknown;

  if (response.status === 204) {
    parsedData = {};
  } else {
    try {
      parsedData = await response.clone().json();
    } catch {
      parsedData = await response.text();
    }
  }

  if (response.status === 401 && !_noSessionRedirect && typeof window !== "undefined") {
    window.dispatchEvent(new Event(UNAUTHORIZED_EVENT));
  }

  // If response is a Paginated envelope ({ items: [...], total }), attach envelope properties to the array
  if (
    parsedData &&
    typeof parsedData === "object" &&
    !Array.isArray(parsedData) &&
    Array.isArray((parsedData as { items?: unknown[] }).items)
  ) {
    const envelope = parsedData as {
      items: unknown[];
      total?: number;
      nextCursor?: string | null;
    };
    const arr = [...envelope.items] as unknown[] & {
      items: unknown[];
      total?: number;
      nextCursor?: string | null;
    };
    arr.items = envelope.items;
    arr.total = envelope.total;
    arr.nextCursor = envelope.nextCursor;
    parsedData = arr;
  }

  return {
    status: response.status,
    data: parsedData,
    headers: response.headers,
  } as T;
};

export type ErrorType<Error> = Error;
export type BodyType<Body> = Body;
