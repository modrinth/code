// 0 pre-launch
type TAPIVersion = 0;
type TAccept = "application/json" | "TODO";
type TMethod = "GET" | "PATCH" | "POST" | "PUT" | "DELETE";

export async function usePyroFetch<T>(
  authToken: string,
  path: string,
  version: TAPIVersion = 0,
  method: TMethod = "GET",
  accept: TAccept = "application/json",
  body?: Record<string, any>,
): Promise<T> {
  const timeout = 10000;
  let retryAmount = 3;

  const config = useRuntimeConfig();

  if (!authToken) {
    throw new Error("Cannot pyrofetch without auth (10000)");
  }

  let base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  if (!base) {
    throw new Error(
      "Cannot pyrofetch without base url. Make sure to set a PYRO_BASE_URL in environment variables (10001)",
    );
  }

  if (base.endsWith("/")) {
    base = base.slice(0, -1);
  }

  if (path.startsWith("/")) {
    path = path.slice(1);
  }

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

  if ((method === "POST" || method === "PUT" || method === "PATCH" || method == "DELETE") && body) {
    request.headers["Content-Type"] = "application/json";
    request.body = JSON.stringify(body);
    request.retry = 0;
  }

  // Known issue when wrapping $fetch
  // https://github.com/unjs/nitro/issues/470
  // https://github.com/nuxt/nuxt/issues/18570
  // @ts-ignore No known fix.
  return await $fetch(fullUrl, request);
}
