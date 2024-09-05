import { $fetch, FetchError } from "ofetch";

interface PyroFetchOptions {
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  body?: Record<string, any>;
  accept?: "application/json" | (string & {});
  version?: number;
}

export class PyroFetchError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
  ) {
    super(message);
    this.name = "PyroFetchError";
  }
}

export async function usePyroFetch<T>(path: string, options: PyroFetchOptions = {}): Promise<T> {
  const config = useRuntimeConfig();
  const auth = await useAuth();
  const authToken = auth.value.token;

  if (!authToken) {
    throw new PyroFetchError("Cannot pyrofetch without auth", 10000);
  }

  const { method = "GET", body, accept = "application/json", version = 0 } = options;

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

  const fullUrl = `${base}/modrinth/v${version}/${path.replace(/^\//, "")}`;

  const headers: any = {
    Accept: accept,
    Authorization: `Bearer ${authToken}`,
    "Access-Control-Allow-Headers": "Authorization",
    "User-Agent": "Pyro/1.0 (https://pyro.host)",
    Vary: "Accept, Origin",
  };

  if (import.meta.client) {
    if (typeof window !== "undefined") {
      headers.Origin = window.location.origin;
    }
  }

  if (["POST", "PUT", "PATCH", "DELETE"].includes(method) && body) {
    headers["Content-Type"] = "application/json";
  }

  try {
    const response = await $fetch<T>(fullUrl, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
      timeout: 10000,
      retry: method === "GET" ? 3 : 0,
    });
    return response;
  } catch (error) {
    if (error instanceof FetchError) {
      console.log(error);
      const statusCode = error.response?.status ?? "unknown";
      const statusText = error.response?.statusText ?? "unknown error";
      const errorMessages: { [key: number]: string } = {
        400: "Bad Request",
        401: "Unauthorized",
        403: "Forbidden",
        404: "Not Found",
        500: "Internal Server Error",
      };
      const message =
        typeof statusCode === "number" && statusCode in errorMessages
          ? errorMessages[statusCode]
          : `HTTP Error: ${statusCode !== "unknown" ? statusCode : "unknown status code"} ${statusText !== "unknown error" ? statusText : "unknown status text"}`;
      throw new PyroFetchError(
        `[PYRO] ${message}`,
        typeof statusCode === "number" ? statusCode : undefined,
      );
    }
    throw new PyroFetchError("[PYRO] An unexpected error occurred during the fetch operation.");
  }
}
