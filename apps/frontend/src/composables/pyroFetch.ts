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
  if (!authToken) {
    throw new Error("Cannot pyrofetch without auth (10000)");
  }

  const config = useRuntimeConfig();
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
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Headers": "*",
    },
  };

  if ((method === "POST" || method === "PUT" || method === "PATCH") && body) {
    request.headers["Content-Type"] = "application/json";
    request.body = JSON.stringify(body);
  }

  // Known issue when wrapping $fetch
  // https://github.com/unjs/nitro/issues/470
  // https://github.com/nuxt/nuxt/issues/18570
  // @ts-ignore No known fix.
  return await $fetch(fullUrl, request);
}
