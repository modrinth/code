import { useServersFetch, type ServersFetchOptions, type StateStorage } from "@modrinth/ui";
import { useAuth } from "~/composables/auth.js";

/**
 * Used for ModrinthServer constructor.
 */
export class NuxtStateStorage implements StateStorage {
  get(key: string) {
    return useState(key).value;
  }

  set(key: string, value: any) {
    useState(key).value = value;
  }

  getRef(key: string): Ref<any> {
    return useState(key);
  }
}

export async function useServersFetchSimple<T>(
  path: string,
  options: Partial<ServersFetchOptions> = {},
  errorContext?: string,
): Promise<T> {
  const auth = await useAuth();
  const config = useRuntimeConfig();
  const base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  return await useServersFetch<T>(
    path,
    {
      base,
      auth: auth.value.token,
      ...options,
    },
    undefined,
    errorContext,
  );
}
