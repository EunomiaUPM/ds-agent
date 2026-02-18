// Custom mutator for Orval that uses a global API gateway configuration
let API_GATEWAY_BASE: string = "";

export const setApiGatewayBase = (url: string) => {
  API_GATEWAY_BASE = url;
};

export type RequestConfig = RequestInit;

// NOTE: Adjusted signature to match Orval's default generation: (url, config)
export const customInstance = <T>(
  url: string,
  options: { method?: string; headers?: any; params?: any; data?: any } & Partial<RequestConfig>
): Promise<T> => {
  const { method, headers, params, data, ...rest } = options || {};

  const config: RequestConfig = {
    ...rest,
    headers: { 
      "Content-Type": "application/json",
      ...headers,
      ...(rest as any)?.headers 
    },
  };

  // Convert params to query string if present
  let targetUrl = url;
  if (params) {
    const searchParams = new URLSearchParams();
    Object.entries(params).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        if (Array.isArray(value)) {
            value.forEach(v => searchParams.append(key, String(v)));
        } else {
            searchParams.append(key, String(value));
        }
      }
    });
    const queryString = searchParams.toString();
    // Handle existing query params in url
    targetUrl += (targetUrl.includes('?') ? '&' : '?') + queryString;
  }

  // Prepend API Gateway URL
  const fullUrl = API_GATEWAY_BASE ? `${API_GATEWAY_BASE}${targetUrl}` : targetUrl;

  const requestMethod = method || 'GET';

  const fetchOptions: RequestInit = {
    ...config,
    method: requestMethod,
  };

  if (data) {
    fetchOptions.body = JSON.stringify(data);
  }

  return fetch(fullUrl, fetchOptions).then(async (response) => {
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

    // Return the response structure expected by Orval generated types
    return {
        status: response.status,
        data: data_1,
        headers: response.headers
    } as T;
  });
};

export type ErrorType<Error> = Error;
export type BodyType<Body> = Body;
