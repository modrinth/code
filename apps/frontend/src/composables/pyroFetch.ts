function normalizeUrl(base: string, path: string): string {
  const normalizedBase = base.endsWith("/") ? base.slice(0, -1) : base;
  const normalizedUrl = path.startsWith("/") ? path.slice(1) : path;
  return `${normalizedBase}/${normalizedUrl}`;
}

// 0 pre-launch
type TAPIVersion = 0;
type TMethod = "GET" | "POST" | "PUT" | "DELETE";

export async function usePyroFetch<T>(
  version: TAPIVersion = 0,
  path: string,
  method: TMethod = "GET",
  authToken?: string | undefined,
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  options?: any,
): Promise<T> {
  const config = useRuntimeConfig();
  const base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  const fullUrl: string = normalizeUrl(`${base}modrinth/v${version}`, path);

  return await $fetch(fullUrl, {
    method,
    headers: {
      Accept: "application/json",
      Authorization: authToken ? `${authToken}` : "",
      "ngrok-skip-browser-warning": "true",
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Headers": "*",
    },
  });
}
