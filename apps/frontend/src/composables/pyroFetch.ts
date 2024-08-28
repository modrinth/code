// eslint-disable-next-line import/no-unresolved
import { $fetch, FetchError } from "ofetch";

type TAPIVersion = 0;
type TAccept = "application/json" | "TODO";
type TMethod = "GET" | "PATCH" | "POST" | "PUT" | "DELETE";

export class PyroFetchError extends Error {
  constructor(
    message: string,
    public statusCode?: number,
  ) {
    super(message);
    this.name = "PyroFetchError";
  }
}

export async function usePyroFetch<T>(
  authToken: string,
  path: string,
  version: TAPIVersion = 0,
  method: TMethod = "GET",
  accept: TAccept = "application/json",
  body?: Record<string, any>,
): Promise<T> {
  const config = useRuntimeConfig();
  const timeout = 10000;
  const retryAmount = 3;

  if (!authToken) {
    throw new PyroFetchError("Cannot pyrofetch without auth", 10000);
  }

  let base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  if (!base) {
    throw new PyroFetchError(
      "Cannot pyrofetch without base url. Make sure to set a PYRO_BASE_URL in environment variables",
      10001,
    );
  }

  base = base.endsWith("/") ? base.slice(0, -1) : base;
  path = path.startsWith("/") ? path.slice(1) : path;

  const fullUrl: string = `${base}/modrinth/v${version}/${path}`;

  const request: any = {
    method,
    headers: {
      Accept: accept,
      Authorization: `Bearer ${authToken}`,
      "ngrok-skip-browser-warning": "true",
      "X-Pinggy-No-Screen": "true",
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "*",
      "Access-Control-Allow-Headers": "Authorization",
      "User-Agent": "Pyro/1.0 (https://pyro.host)",
      Vary: "Accept",
    },
    timeout,
    retry: retryAmount,
  };

  if (
    (method === "POST" || method === "PUT" || method === "PATCH" || method === "DELETE") &&
    body
  ) {
    request.headers["Content-Type"] = "application/json";
    request.body = JSON.stringify(body);
    request.retry = 0;
  }

  try {
    // @ts-ignore No known fix for the known issue when wrapping $fetch
    const response = await $fetch(fullUrl, request);
    return response as T;
  } catch (error) {
    if (error instanceof FetchError) {
      switch (error.response?.status) {
        case 400:
          throw new PyroFetchError("[PYRO] Bad Request", 400);
        case 401:
          throw new PyroFetchError("[PYRO] Unauthorized", 401);
        case 403:
          throw new PyroFetchError("[PYRO] Forbidden", 403);
        case 404:
          throw new PyroFetchError("[PYRO] Not Found", 404);
        case 500:
          throw new PyroFetchError("[PYRO] Internal Server Error", 500);
        default:
          throw new PyroFetchError(
            `HTTP Error: ${error.response?.status} ${error.response?.statusText}`,
            error.response?.status,
          );
      }
    }
    throw new PyroFetchError("[PYRO] An unexpected error occurred during the fetch operation.");
  }
}
