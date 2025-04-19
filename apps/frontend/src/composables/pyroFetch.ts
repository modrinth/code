import { $fetch, FetchError } from "ofetch";

interface PyroFetchOptions {
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  contentType?: string;
  body?: Record<string, any>;
  version?: number;
  override?: {
    url?: string;
    token?: string;
  };
  retry?: boolean;
  bypassAuth?: boolean;
}

export class PyroFetchError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
    public originalError?: Error,
  ) {
    super(message);
    this.name = "PyroFetchError";
  }
}

export async function usePyroFetch<T>(path: string, options: PyroFetchOptions = {}): Promise<T> {
  const config = useRuntimeConfig();
  const auth = await useAuth();
  const authToken = auth.value?.token;

  if (!authToken && !options.bypassAuth) {
    throw new PyroFetchError("Cannot pyrofetch without auth", 10000);
  }

  const { method = "GET", contentType = "application/json", body, version = 0, override } = options;

  const base = (import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl)?.replace(
    /\/$/,
    "",
  );

  if (!base) {
    throw new PyroFetchError(
      "Cannot pyrofetch without base url. Make sure to set a PYRO_BASE_URL in environment variables",
      10001,
    );
  }

  const fullUrl = override?.url
    ? `https://${override.url}/${path.replace(/^\//, "")}`
    : `${base}/modrinth/v${version}/${path.replace(/^\//, "")}`;

  type HeadersRecord = Record<string, string>;

  const authHeader: HeadersRecord = options.bypassAuth
    ? {}
    : {
        Authorization: `Bearer ${override?.token ?? authToken}`,
        "Access-Control-Allow-Headers": "Authorization",
      };

  const headers: HeadersRecord = {
    ...authHeader,
    "User-Agent": "Pyro/1.0 (https://pyro.host)",
    Vary: "Accept, Origin",
    "Content-Type": contentType,
  };

  if (import.meta.client && typeof window !== "undefined") {
    headers.Origin = window.location.origin;
  }

  try {
    const response = await $fetch<T>(fullUrl, {
      method,
      headers,
      body: body && contentType === "application/json" ? JSON.stringify(body) : body ?? undefined,
      timeout: 10000,
      retry: options.retry !== false ? (method === "GET" ? 3 : 0) : 0,
    });
    return response;
  } catch (error) {
    console.error("Fetch error:", error);
    if (error instanceof FetchError) {
      const statusCode = error.response?.status;
      const statusText = error.response?.statusText || "Unknown error";
      const errorMessages: { [key: number]: string } = {
        400: "Bad Request",
        401: "Unauthorized",
        403: "Forbidden",
        404: "Not Found",
        405: "Method Not Allowed",
        429: "Too Many Requests",
        500: "Internal Server Error",
        502: "Bad Gateway",
      };
      const message =
        statusCode && statusCode in errorMessages
          ? errorMessages[statusCode]
          : `HTTP Error: ${statusCode || "unknown"} ${statusText}`;
      throw new PyroFetchError(`[PYROFETCH][PYRO] ${message}`, statusCode, error);
    }
    throw new PyroFetchError(
      "[PYROFETCH][PYRO] An unexpected error occurred during the fetch operation.",
      undefined,
      error as Error,
    );
  }
}
