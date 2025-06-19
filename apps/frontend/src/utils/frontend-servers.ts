import {
  useModrinthServers,
  useServersFetch,
  type ServersFetchOptions,
  type StateStorage,
} from "@modrinth/ui";

import { ModrinthServerError, type ModuleName } from "@modrinth/utils";
import { addNotification } from "~/composables/notifs.js";
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

export function handleServersError(err: any) {
  if (err instanceof ModrinthServerError && err.v1Error) {
    addNotification({
      title: err.v1Error?.context ?? `An error occurred`,
      type: "error",
      text: err.v1Error.description,
      errorCode: err.v1Error.error,
    });
  } else {
    addNotification({
      title: "An error occurred",
      type: "error",
      text: err.message ?? (err.data ? err.data.description : err),
    });
  }
}

export async function useModrinthServersSimple(serverId: string, modules: ModuleName[]) {
  const auth = await useAuth();
  const config = useRuntimeConfig();
  const base = import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl;

  return await useModrinthServers(
    serverId,
    auth.value.token,
    base,
    handleServersError,
    new NuxtStateStorage(),
    modules,
  );
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
