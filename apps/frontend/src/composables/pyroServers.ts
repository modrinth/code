// usePyroServer is a composable that interfaces with the REDACTED API to get data and control the users server
import { $fetch, FetchError } from "ofetch";
import type { ServerNotice } from "@modrinth/utils";
import type { FilesystemOp, FSQueuedOp, WSBackupState, WSBackupTask } from "~/types/servers.ts";

interface PyroFetchOptions {
  method?: "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
  contentType?: string;
  body?: Record<string, any>;
  version?: number;
  override?: {
    url?: string;
    token?: string;
  };
  retry?: number | boolean;
}

class PyroServerError extends Error {
  public readonly errors: Map<string, Error> = new Map();
  public readonly timestamp: number = Date.now();

  constructor(message?: string) {
    super(message || "Multiple errors occurred");
    this.name = "PyroServerError";
  }

  addError(module: string, error: Error) {
    this.errors.set(module, error);
    this.message = this.buildErrorMessage();
  }

  hasErrors() {
    return this.errors.size > 0;
  }

  private buildErrorMessage(): string {
    return Array.from(this.errors.entries())
      .map(([_module, error]) => error.message)
      .join("\n");
  }
}

type V1ErrorInfo = {
  context?: string;
  error: string;
  description: string;
};

export class ServersError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    public readonly originalError?: Error,
    public readonly module?: string,
    public readonly v1Error?: V1ErrorInfo,
  ) {
    let errorMessage = message;
    let method = "GET";
    let path = "";

    if (originalError instanceof FetchError) {
      const matches = message.match(/\[([A-Z]+)\]\s+"([^"]+)":/);
      if (matches) {
        method = matches[1];
        path = matches[2].replace(/https?:\/\/[^/]+\/[^/]+\/v\d+\//, "");
      }

      const statusMessage = (() => {
        if (!statusCode) return "Unknown Error";
        switch (statusCode) {
          case 400:
            return "Bad Request";
          case 401:
            return "Unauthorized";
          case 403:
            return "Forbidden";
          case 404:
            return "Not Found";
          case 408:
            return "Request Timeout";
          case 429:
            return "Too Many Requests";
          case 500:
            return "Internal Server Error";
          case 502:
            return "Bad Gateway";
          case 503:
            return "Service Unavailable";
          case 504:
            return "Gateway Timeout";
          default:
            return `HTTP ${statusCode}`;
        }
      })();

      errorMessage = `[${method}] ${statusMessage} (${statusCode}) while fetching ${path}${module ? ` in ${module}` : ""}`;
    } else {
      errorMessage = `${message}${statusCode ? ` (${statusCode})` : ""}${module ? ` in ${module}` : ""}`;
    }

    super(errorMessage);
    this.name = "PyroServersFetchError";
  }
}

export const handleError = (err: any) => {
  if (err instanceof ServersError && err.v1Error) {
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
};

async function PyroFetch<T>(
  path: string,
  options: PyroFetchOptions = {},
  module?: string,
  errorContext?: string,
): Promise<T> {
  const config = useRuntimeConfig();
  const auth = await useAuth();
  const authToken = auth.value?.token;

  if (!authToken) {
    throw new ServersError("Missing auth token", 401, undefined, module);
  }

  const {
    method = "GET",
    contentType = "application/json",
    body,
    version = 0,
    override,
    retry = method === "GET" ? 3 : 0,
  } = options;

  const base = (import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl)?.replace(
    /\/$/,
    "",
  );

  if (!base) {
    throw new ServersError("Configuration error: Missing PYRO_BASE_URL", 500, undefined, module);
  }

  const versionString = `v${version}`;

  let newOverrideUrl = override?.url;
  if (newOverrideUrl && newOverrideUrl.includes("v0") && version !== 0) {
    newOverrideUrl = newOverrideUrl.replace("v0", versionString);
  }

  const fullUrl = newOverrideUrl
    ? `https://${newOverrideUrl}/${path.replace(/^\//, "")}`
    : `${base}/modrinth/v${version}/${path.replace(/^\//, "")}`;

  const headers: Record<string, string> = {
    Authorization: `Bearer ${override?.token ?? authToken}`,
    "Access-Control-Allow-Headers": "Authorization",
    "User-Agent": "Pyro/1.0 (https://pyro.host)",
    Vary: "Accept, Origin",
  };

  if (contentType !== "none") {
    headers["Content-Type"] = contentType;
  }

  if (import.meta.client && typeof window !== "undefined") {
    headers.Origin = window.location.origin;
  }

  let attempts = 0;
  const maxAttempts = (typeof retry === "boolean" ? (retry ? 1 : 0) : retry) + 1;
  let lastError: Error | null = null;

  while (attempts < maxAttempts) {
    try {
      const response = await $fetch<T>(fullUrl, {
        method,
        headers,
        body: body && contentType === "application/json" ? JSON.stringify(body) : body ?? undefined,
        timeout: 10000,
      });

      return response;
    } catch (error) {
      lastError = error as Error;
      attempts++;

      if (error instanceof FetchError) {
        let v1Error: V1ErrorInfo | undefined;

        if (error.data.error && error.data.description) {
          v1Error = {
            context: errorContext,
            ...error.data,
          };
        }

        const statusCode = error.response?.status;
        const isRetryable = statusCode ? [408, 429, 500, 502, 503, 504].includes(statusCode) : true;

        if (!isRetryable || attempts >= maxAttempts) {
          throw new ServersError(error.message, statusCode, error, module, v1Error);
        }

        const delay = Math.min(1000 * Math.pow(2, attempts - 1) + Math.random() * 1000, 10000);
        await new Promise((resolve) => setTimeout(resolve, delay));
        continue;
      }

      throw new ServersError(
        "Unexpected error during fetch operation",
        undefined,
        error as Error,
        module,
      );
    }
  }

  throw lastError || new Error("Maximum retry attempts reached");
}

const internalServerReference = ref<any>(null);

interface License {
  id: string;
  name: string;
  url: string;
}

interface DonationUrl {
  id: string;
  platform: string;
  url: string;
}

interface GalleryItem {
  url: string;
  featured: boolean;
  title: string;
  description: string;
  created: string;
  ordering: number;
}

export interface Project {
  slug: string;
  title: string;
  description: string;
  categories: string[];
  client_side: "required" | "optional";
  server_side: "required" | "optional";
  body: string;
  status: "approved" | "pending" | "rejected";
  requested_status: "approved" | "pending" | "rejected";
  additional_categories: string[];
  issues_url: string;
  source_url: string;
  wiki_url: string;
  discord_url: string;
  donation_urls: DonationUrl[];
  project_type: "mod" | "resourcepack" | "map" | "plugin";
  downloads: number;
  icon_url: string;
  color: number;
  thread_id: string;
  monetization_status: "monetized" | "non-monetized";
  id: string;
  team: string;
  body_url: string | null;
  moderator_message: string | null;
  published: string;
  updated: string;
  approved: string;
  queued: string;
  followers: number;
  license: License;
  versions: string[];
  game_versions: string[];
  loaders: string[];
  gallery: GalleryItem[];
}

interface General {
  server_id: string;
  name: string;
  net: {
    ip: string;
    port: number;
    domain: string;
  };
  game: string;
  backup_quota: number;
  used_backup_quota: number;
  status: string;
  suspension_reason:
    | "moderated"
    | "paymentfailed"
    | "cancelled"
    | "upgrading"
    | "other"
    | (string & {});
  loader: string;
  loader_version: string;
  mc_version: string;
  upstream: {
    kind: "modpack" | "mod" | "resourcepack";
    version_id: string;
    project_id: string;
  } | null;
  motd?: string;
  image?: string;
  project?: Project;
  sftp_username: string;
  sftp_password: string;
  sftp_host: string;
  datacenter?: string;
  notices?: ServerNotice[];
  node: {
    token: string;
    instance: string;
  };
}

interface Allocation {
  port: number;
  name: string;
}

interface Startup {
  invocation: string;
  original_invocation: string;
  jdk_version: "lts8" | "lts11" | "lts17" | "lts21";
  jdk_build: "corretto" | "temurin" | "graal";
}

export interface Mod {
  filename: string;
  project_id: string | undefined;
  version_id: string | undefined;
  name: string | undefined;
  version_number: string | undefined;
  icon_url: string | undefined;
  owner: string | undefined;
  disabled: boolean;
  installing: boolean;
}

export interface Backup {
  id: string;
  name: string;
  created_at: string;
  locked: boolean;
  automated: boolean;
  interrupted: boolean;
  ongoing: boolean;
  task: {
    [K in WSBackupTask]?: {
      progress: number;
      state: WSBackupState;
    };
  };
}

interface AutoBackupSettings {
  enabled: boolean;
  interval: number;
}

interface JWTAuth {
  url: string;
  token: string;
}

export interface DirectoryItem {
  name: string;
  type: "directory" | "file";
  count?: number;
  modified: number;
  created: number;
  path: string;
}

export interface DirectoryResponse {
  items: DirectoryItem[];
  total: number;
  current?: number;
}

type ContentType = "mod" | "plugin";

const constructServerProperties = (properties: any): string => {
  let fileContent = `#Minecraft server properties\n#${new Date().toUTCString()}\n`;

  for (const [key, value] of Object.entries(properties)) {
    if (typeof value === "object") {
      fileContent += `${key}=${JSON.stringify(value)}\n`;
    } else if (typeof value === "boolean") {
      fileContent += `${key}=${value ? "true" : "false"}\n`;
    } else {
      fileContent += `${key}=${value}\n`;
    }
  }

  return fileContent;
};

const processImage = async (iconUrl: string | undefined) => {
  const sharedImage = useState<string | undefined>(
    `server-icon-${internalServerReference.value.serverId}`,
  );

  if (sharedImage.value) {
    return sharedImage.value;
  }

  try {
    const auth = await PyroFetch<JWTAuth>(`servers/${internalServerReference.value.serverId}/fs`);
    try {
      const fileData = await PyroFetch(`/download?path=/server-icon-original.png`, {
        override: auth,
        retry: false,
      });

      if (fileData instanceof Blob) {
        if (import.meta.client) {
          const dataURL = await new Promise<string>((resolve) => {
            const canvas = document.createElement("canvas");
            const ctx = canvas.getContext("2d");
            const img = new Image();
            img.onload = () => {
              canvas.width = 512;
              canvas.height = 512;
              ctx?.drawImage(img, 0, 0, 512, 512);
              const dataURL = canvas.toDataURL("image/png");
              sharedImage.value = dataURL;
              resolve(dataURL);
              URL.revokeObjectURL(img.src);
            };
            img.src = URL.createObjectURL(fileData);
          });
          return dataURL;
        }
      }
    } catch (error) {
      if (error instanceof ServersError && error.statusCode === 404 && iconUrl) {
        try {
          const response = await fetch(iconUrl);
          if (!response.ok) throw new Error("Failed to fetch icon");
          const file = await response.blob();
          const originalFile = new File([file], "server-icon-original.png", { type: "image/png" });

          if (import.meta.client) {
            const dataURL = await new Promise<string>((resolve) => {
              const canvas = document.createElement("canvas");
              const ctx = canvas.getContext("2d");
              const img = new Image();
              img.onload = () => {
                canvas.width = 64;
                canvas.height = 64;
                ctx?.drawImage(img, 0, 0, 64, 64);
                canvas.toBlob(async (blob) => {
                  if (blob) {
                    const scaledFile = new File([blob], "server-icon.png", { type: "image/png" });
                    await PyroFetch(`/create?path=/server-icon.png&type=file`, {
                      method: "POST",
                      contentType: "application/octet-stream",
                      body: scaledFile,
                      override: auth,
                    });
                    await PyroFetch(`/create?path=/server-icon-original.png&type=file`, {
                      method: "POST",
                      contentType: "application/octet-stream",
                      body: originalFile,
                      override: auth,
                    });
                  }
                }, "image/png");
                const dataURL = canvas.toDataURL("image/png");
                sharedImage.value = dataURL;
                resolve(dataURL);
                URL.revokeObjectURL(img.src);
              };
              img.src = URL.createObjectURL(file);
            });
            return dataURL;
          }
        } catch (error) {
          console.error("Failed to process external icon:", error);
        }
      }
    }
  } catch (error) {
    console.error("Failed to process server icon:", error);
  }

  sharedImage.value = undefined;
  return undefined;
};

// ------------------ GENERAL ------------------ //

const sendPowerAction = async (action: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/power`, {
      method: "POST",
      body: { action },
    });

    await new Promise((resolve) => setTimeout(resolve, 1000));
    await internalServerReference.value.refresh();
  } catch (error) {
    console.error("Error changing power state:", error);
    throw error;
  }
};

const updateName = async (newName: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/name`, {
      method: "POST",
      body: { name: newName },
    });
  } catch (error) {
    console.error("Error updating server name:", error);
    throw error;
  }
};

const reinstallServer = async (
  serverId: string,
  loader: boolean,
  projectId: string,
  versionId?: string,
  loaderVersionId?: string,
  hardReset: boolean = false,
) => {
  try {
    const hardResetParam = hardReset ? "true" : "false";
    if (loader) {
      if (projectId.toLowerCase() === "neoforge") {
        projectId = "NeoForge";
      }
      await PyroFetch(`servers/${serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { loader: projectId, loader_version: loaderVersionId, game_version: versionId },
      });
    } else {
      await PyroFetch(`servers/${serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { project_id: projectId, version_id: versionId },
      });
    }
  } catch (error) {
    console.error("Error reinstalling server:", error);
    throw error;
  }
};

const reinstallFromMrpack = async (mrpack: File, hardReset: boolean = false) => {
  const hardResetParam = hardReset ? "true" : "false";
  try {
    const auth = await PyroFetch<JWTAuth>(
      `servers/${internalServerReference.value.serverId}/reinstallFromMrpack`,
    );

    const formData = new FormData();
    formData.append("file", mrpack);

    const response = await fetch(
      `https://${auth.url}/reinstallMrpackMultiparted?hard=${hardResetParam}`,
      {
        method: "POST",
        headers: {
          Authorization: `Bearer ${auth.token}`,
        },
        body: formData,
        signal: AbortSignal.timeout(30 * 60 * 1000),
      },
    );

    if (!response.ok) {
      throw new Error(`[pyroservers] native fetch err status: ${response.status}`);
    }
  } catch (error) {
    console.error("Error reinstalling from mrpack:", error);
    throw error;
  }
};

const suspendServer = async (status: boolean) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/suspend`, {
      method: "POST",
      body: { suspended: status },
    });
  } catch (error) {
    console.error("Error suspending server:", error);
    throw error;
  }
};

const fetchConfigFile = async (fileName: string) => {
  try {
    return await PyroFetch(`servers/${internalServerReference.value.serverId}/config/${fileName}`);
  } catch (error) {
    console.error("Error fetching config file:", error);
    throw error;
  }
};

const getMotd = async () => {
  try {
    const props = await downloadFile("/server.properties");
    if (props) {
      const lines = props.split("\n");
      for (const line of lines) {
        if (line.startsWith("motd=")) {
          return line.slice(5);
        }
      }
    }
  } catch {
    return null;
  }
};

const setMotd = async (motd: string) => {
  try {
    const props = (await fetchConfigFile("ServerProperties")) as any;
    if (props) {
      props.motd = motd;
      const newProps = constructServerProperties(props);
      const octetStream = new Blob([newProps], { type: "application/octet-stream" });
      const auth = await await PyroFetch<JWTAuth>(
        `servers/${internalServerReference.value.serverId}/fs`,
      );

      return await PyroFetch(`/update?path=/server.properties`, {
        method: "PUT",
        contentType: "application/octet-stream",
        body: octetStream,
        override: auth,
      });
    }
  } catch (error) {
    console.error("Error setting motd:", error);
  }
};

// ------------------ CONTENT ------------------ //

const installContent = async (contentType: ContentType, projectId: string, versionId: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/mods`, {
      method: "POST",
      body: {
        rinth_ids: { project_id: projectId, version_id: versionId },
        install_as: contentType,
      },
    });
  } catch (error) {
    console.error("Error installing mod:", error);
    throw error;
  }
};

const removeContent = async (path: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/deleteMod`, {
      method: "POST",
      body: {
        path,
      },
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    throw error;
  }
};

const reinstallContent = async (replace: string, projectId: string, versionId: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/mods/update`, {
      method: "POST",
      body: { replace, project_id: projectId, version_id: versionId },
    });
  } catch (error) {
    console.error("Error reinstalling mod:", error);
    throw error;
  }
};

// ------------------ BACKUPS ------------------ //

const createBackup = async (backupName: string) => {
  try {
    const response = await PyroFetch<{ id: string }>(
      `servers/${internalServerReference.value.serverId}/backups`,
      {
        method: "POST",
        body: { name: backupName },
      },
    );
    await internalServerReference.value.refresh(["backups"]);
    return response.id;
  } catch (error) {
    console.error("Error creating backup:", error);
    throw error;
  }
};

const renameBackup = async (backupId: string, newName: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerReference.value.serverId}/backups/${backupId}/rename`,
      {
        method: "POST",
        body: { name: newName },
      },
    );
    await internalServerReference.value.refresh(["backups"]);
  } catch (error) {
    console.error("Error renaming backup:", error);
    throw error;
  }
};

const deleteBackup = async (backupId: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
    await internalServerReference.value.refresh(["backups"]);
  } catch (error) {
    console.error("Error deleting backup:", error);
    throw error;
  }
};

const restoreBackup = async (backupId: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerReference.value.serverId}/backups/${backupId}/restore`,
      {
        method: "POST",
      },
    );
    await internalServerReference.value.refresh(["backups"]);
  } catch (error) {
    console.error("Error restoring backup:", error);
    throw error;
  }
};

const prepareBackup = async (backupId: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerReference.value.serverId}/backups/${backupId}/prepare-download`,
      {
        method: "POST",
      },
    );
  } catch (error) {
    console.error("Error preparing backup:", error);
    throw error;
  }
};

const updateAutoBackup = async (autoBackup: "enable" | "disable", interval: number) => {
  try {
    return await PyroFetch(`servers/${internalServerReference.value.serverId}/autobackup`, {
      method: "POST",
      body: { set: autoBackup, interval },
    });
  } catch (error) {
    console.error("Error updating auto backup:", error);
    throw error;
  }
};

const getAutoBackup = async () => {
  try {
    return await PyroFetch(`servers/${internalServerReference.value.serverId}/autobackup`);
  } catch (error) {
    console.error("Error getting auto backup settings:", error);
    throw error;
  }
};

const lockBackup = async (backupId: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/backups/${backupId}/lock`, {
      method: "POST",
    });
    await internalServerReference.value.refresh(["backups"]);
  } catch (error) {
    console.error("Error locking backup:", error);
    throw error;
  }
};

const unlockBackup = async (backupId: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerReference.value.serverId}/backups/${backupId}/unlock`,
      {
        method: "POST",
      },
    );
    await internalServerReference.value.refresh(["backups"]);
  } catch (error) {
    console.error("Error unlocking backup:", error);
    throw error;
  }
};

const retryBackup = async (backupId: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/backups/${backupId}/retry`, {
      method: "POST",
    });
  } catch (error) {
    console.error("Error retrying backup:", error);
    throw error;
  }
};

// ------------------ NETWORK ------------------ //

const reserveAllocation = async (name: string): Promise<Allocation> => {
  try {
    return await PyroFetch<Allocation>(
      `servers/${internalServerReference.value.serverId}/allocations?name=${name}`,
      {
        method: "POST",
      },
    );
  } catch (error) {
    console.error("Error reserving new allocation:", error);
    throw error;
  }
};

const updateAllocation = async (port: number, name: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerReference.value.serverId}/allocations/${port}?name=${name}`,
      {
        method: "PUT",
      },
    );
  } catch (error) {
    console.error("Error updating allocations:", error);
    throw error;
  }
};

const deleteAllocation = async (port: number) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/allocations/${port}`, {
      method: "DELETE",
    });
  } catch (error) {
    console.error("Error deleting allocation:", error);
    throw error;
  }
};

const checkSubdomainAvailability = async (subdomain: string): Promise<{ available: boolean }> => {
  try {
    return (await PyroFetch(`subdomains/${subdomain}/isavailable`)) as { available: boolean };
  } catch (error) {
    console.error("Error checking subdomain availability:", error);
    throw error;
  }
};

const changeSubdomain = async (subdomain: string) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/subdomain`, {
      method: "POST",
      body: { subdomain },
    });
  } catch (error) {
    console.error("Error changing subdomain:", error);
    throw error;
  }
};

// ------------------ STARTUP ------------------ //

const updateStartupSettings = async (
  invocation: string,
  jdkVersion: "lts8" | "lts11" | "lts17" | "lts21",
  jdkBuild: "corretto" | "temurin" | "graal",
) => {
  try {
    await PyroFetch(`servers/${internalServerReference.value.serverId}/startup`, {
      method: "POST",
      body: {
        invocation: invocation || null,
        jdk_version: jdkVersion || null,
        jdk_build: jdkBuild || null,
      },
    });
  } catch (error) {
    console.error("Error updating startup settings:", error);
    throw error;
  }
};

// ------------------ FS ------------------ //

const retryWithAuth = async (requestFn: () => Promise<any>) => {
  try {
    return await requestFn();
  } catch (error) {
    if (error instanceof ServersError && error.statusCode === 401) {
      await internalServerReference.value.refresh(["fs"]);
      return await requestFn();
    }

    throw error;
  }
};

const listDirContents = (path: string, page: number, pageSize: number) => {
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    return await PyroFetch(`/list?path=${encodedPath}&page=${page}&page_size=${pageSize}`, {
      override: internalServerReference.value.fs.auth,
      retry: false,
    });
  });
};

const createFileOrFolder = (path: string, type: "file" | "directory") => {
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    return await PyroFetch(`/create?path=${encodedPath}&type=${type}`, {
      method: "POST",
      contentType: "application/octet-stream",
      override: internalServerReference.value.fs.auth,
    });
  });
};

const uploadFile = (path: string, file: File) => {
  // eslint-disable-next-line require-await
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    const progressSubject = new EventTarget();
    const abortController = new AbortController();

    const uploadPromise = new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.upload.addEventListener("progress", (e) => {
        if (e.lengthComputable) {
          const progress = (e.loaded / e.total) * 100;
          progressSubject.dispatchEvent(
            new CustomEvent("progress", {
              detail: {
                loaded: e.loaded,
                total: e.total,
                progress,
              },
            }),
          );
        }
      });

      xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          resolve(xhr.response);
        } else {
          reject(new Error(`Upload failed with status ${xhr.status}`));
        }
      };

      xhr.onerror = () => reject(new Error("Upload failed"));
      xhr.onabort = () => reject(new Error("Upload cancelled"));

      xhr.open(
        "POST",
        `https://${internalServerReference.value.fs.auth.url}/create?path=${encodedPath}&type=file`,
      );
      xhr.setRequestHeader(
        "Authorization",
        `Bearer ${internalServerReference.value.fs.auth.token}`,
      );
      xhr.setRequestHeader("Content-Type", "application/octet-stream");
      xhr.send(file);

      abortController.signal.addEventListener("abort", () => {
        xhr.abort();
      });
    });

    return {
      promise: uploadPromise,
      onProgress: (
        callback: (progress: { loaded: number; total: number; progress: number }) => void,
      ) => {
        progressSubject.addEventListener("progress", ((e: CustomEvent) => {
          callback(e.detail);
        }) as EventListener);
      },
      cancel: () => {
        abortController.abort();
      },
    };
  });
};

const renameFileOrFolder = (path: string, name: string) => {
  const pathName = path.split("/").slice(0, -1).join("/") + "/" + name;
  return retryWithAuth(async () => {
    await PyroFetch(`/move`, {
      method: "POST",
      override: internalServerReference.value.fs.auth,
      body: {
        source: path,
        destination: pathName,
      },
    });
    return true;
  });
};

const updateFile = (path: string, content: string) => {
  const octetStream = new Blob([content], { type: "application/octet-stream" });
  return retryWithAuth(async () => {
    return await PyroFetch(`/update?path=${path}`, {
      method: "PUT",
      contentType: "application/octet-stream",
      body: octetStream,
      override: internalServerReference.value.fs.auth,
    });
  });
};

const createMissingFolders = async (path: string) => {
  if (path.startsWith("/")) {
    path = path.substring(1);
  }
  const folders = path.split("/");
  console.log(folders);
  let currentPath = "";

  for (const folder of folders) {
    currentPath += "/" + folder;
    try {
      await createFileOrFolder(currentPath, "directory");
    } catch {}
  }
};

const moveFileOrFolder = (path: string, newPath: string) => {
  return retryWithAuth(async () => {
    console.log(path);
    console.log(newPath);
    console.log(newPath.substring(0, newPath.lastIndexOf("/")));
    await createMissingFolders(newPath.substring(0, newPath.lastIndexOf("/")));

    return await PyroFetch(`/move`, {
      method: "POST",
      override: internalServerReference.value.fs.auth,
      body: {
        source: path,
        destination: newPath,
      },
    });
  });
};

const clearQueuedOps = () => {
  internalServerReference.value.fs.queuedOps = [];
};

const removeQueuedOp = (op: FSQueuedOp["op"], src: string) => {
  internalServerReference.value.fs.queuedOps = internalServerReference.value.fs.queuedOps.filter(
    (x: FSQueuedOp) => x.op !== op || x.src !== src,
  );
};

const extractFile = (path: string, override = true, dry = false, silentQueue = false) =>
  retryWithAuth(async () => {
    console.log(
      `Extracting: ${path}` + (dry ? " (dry run)" : "") + (silentQueue ? " (silent)" : ""),
    );

    const encodedPath = encodeURIComponent(path);

    if (!silentQueue) {
      internalServerReference.value.fs.queuedOps.push({
        op: "unarchive",
        src: path,
      });

      setTimeout(() => internalServerReference.value.fs.removeQueuedOp("unarchive", path), 4000);
    }

    return (await PyroFetch(
      `/unarchive?src=${encodedPath}&trg=/&override=${override}&dry=${dry}`,
      {
        method: "POST",
        override: internalServerReference.value.fs.auth,
        version: 1,
      },
      undefined,
      "Error extracting file",
    ).catch((err) => {
      removeQueuedOp("unarchive", path);
      throw err;
    })) as { modpack_name: string | null };
  });

const modifyOp = (id: string, action: "dismiss" | "cancel") =>
  retryWithAuth(async () => {
    return await PyroFetch(
      `/ops/${action}?id=${id}`,
      {
        method: "POST",
        override: internalServerReference.value.fs.auth,
        version: 1,
      },
      undefined,
      `Error ${action === "dismiss" ? "dismissing" : "cancelling"} filesystem operation`,
    ).then(() => {
      internalServerReference.value.fs.opsQueuedForModification =
        internalServerReference.value.fs.opsQueuedForModification.filter((x: string) => x !== id);
      internalServerReference.value.fs.ops = internalServerReference.value.fs.ops.filter(
        (x: FilesystemOp) => x.id !== id,
      );
    });
  });

const deleteFileOrFolder = (path: string, recursive: boolean) => {
  const encodedPath = encodeURIComponent(path);
  return retryWithAuth(async () => {
    return await PyroFetch(`/delete?path=${encodedPath}&recursive=${recursive}`, {
      method: "DELETE",
      override: internalServerReference.value.fs.auth,
    });
  });
};

const downloadFile = (path: string, raw?: boolean) => {
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    const fileData = await PyroFetch(`/download?path=${encodedPath}`, {
      override: internalServerReference.value.fs.auth,
    });

    if (fileData instanceof Blob) {
      if (raw) {
        return fileData;
      } else {
        return await fileData.text();
      }
    }
  });
};

const modules: any = {
  general: {
    get: async (serverId: string) => {
      try {
        const data = await PyroFetch<General>(`servers/${serverId}`, {}, "general");
        if (data.upstream?.project_id) {
          const res = await $fetch(
            `https://api.modrinth.com/v2/project/${data.upstream.project_id}`,
          );
          data.project = res as Project;
        }

        if (import.meta.client) {
          data.image = (await processImage(data.project?.icon_url)) ?? undefined;
        }

        const motd = await getMotd();
        if (motd === "A Minecraft Server") {
          await setMotd(
            `§b${data.project?.title || data.loader + " " + data.mc_version} §f♦ §aModrinth Servers`,
          );
        }
        data.motd = motd;
        return data;
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          status: "error",
          server_id: serverId,
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    updateName,
    power: sendPowerAction,
    reinstall: reinstallServer,
    reinstallFromMrpack,
    suspend: suspendServer,
    getMotd,
    setMotd,
  },
  content: {
    get: async (serverId: string) => {
      try {
        const mods = await PyroFetch<Mod[]>(`servers/${serverId}/mods`, {}, "content");
        return {
          data: mods.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? "")),
        };
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          data: [],
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    install: installContent,
    remove: removeContent,
    reinstall: reinstallContent,
  },
  backups: {
    get: async (serverId: string) => {
      try {
        return {
          data: await PyroFetch<Backup[]>(`servers/${serverId}/backups`, {}, "backups"),
        };
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          data: [],
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    create: createBackup,
    rename: renameBackup,
    delete: deleteBackup,
    restore: restoreBackup,
    prepare: prepareBackup,
    updateAutoBackup,
    getAutoBackup,
    lock: lockBackup,
    unlock: unlockBackup,
    retry: retryBackup,
  },
  network: {
    get: async (serverId: string) => {
      try {
        return {
          allocations: await PyroFetch<Allocation[]>(
            `servers/${serverId}/allocations`,
            {},
            "network",
          ),
        };
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          allocations: [],
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    reserveAllocation,
    updateAllocation,
    deleteAllocation,
    checkSubdomainAvailability,
    changeSubdomain,
  },
  startup: {
    get: async (serverId: string) => {
      try {
        return await PyroFetch<Startup>(`servers/${serverId}/startup`, {}, "startup");
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    update: updateStartupSettings,
  },
  ws: {
    get: async (serverId: string) => {
      try {
        return await PyroFetch<JWTAuth>(`servers/${serverId}/ws`, {}, "ws");
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
  },
  fs: {
    queuedOps: [],
    opsQueuedForModification: [],
    get: async (serverId: string) => {
      try {
        return { auth: await PyroFetch<JWTAuth>(`servers/${serverId}/fs`, {}, "fs") };
      } catch (error) {
        const fetchError =
          error instanceof ServersError
            ? error
            : new ServersError("Unknown error occurred", undefined, error as Error);

        return {
          auth: undefined,
          error: {
            error: fetchError,
            timestamp: Date.now(),
          },
        };
      }
    },
    listDirContents,
    createFileOrFolder,
    uploadFile,
    renameFileOrFolder,
    updateFile,
    moveFileOrFolder,
    deleteFileOrFolder,
    downloadFile,
    extractFile,
    removeQueuedOp,
    clearQueuedOps,
    modifyOp,
  },
};

type GeneralFunctions = {
  /**
   * INTERNAL: Gets the general settings of a server.
   * @param serverId - The ID of the server.
   */
  get: (serverId: string) => Promise<General>;

  /**
   * Updates the name of the server.
   * @param newName - The new name for the server.
   */
  updateName: (newName: string) => Promise<void>;

  /**
   * Sends a power action to the server.

   * @param action - The power action to send (e.g., "start", "stop", "restart").
   */
  power: (action: string) => Promise<void>;

  /**
   * Reinstalls the server with the specified project and version.
   * @param loader - Whether to use a loader.
   * @param projectId - The ID of the project.
   * @param versionId - Optional version ID.
   * @param loaderVersionId - Optional loader version ID.
   * @param hardReset - Whether to perform a hard reset.
   */
  reinstall: (
    serverId: string,
    loader: boolean,
    projectId: string,
    versionId?: string,
    loaderVersionId?: string,
    hardReset?: boolean,
  ) => Promise<void>;

  /**
   * Reinstalls the server from a mrpack.
   * @param mrpack - The mrpack file.
   * @param hardReset - Whether to perform a hard reset.
   */
  reinstallFromMrpack: (mrpack: File, hardReset?: boolean) => Promise<void>;

  /**
   * Suspends or resumes the server.
   * @param status - True to suspend the server, false to resume.
   */
  suspend: (status: boolean) => Promise<void>;

  /**
   * INTERNAL: Gets the general settings of a server.
   */
  getMotd: () => Promise<string>;

  /**
   * INTERNAL: Updates the general settings of a server.
   * @param motd - The new motd.
   */
  setMotd: (motd: string) => Promise<void>;

  /**
   * @deprecated Use fs.downloadFile instead
   */
  fetchConfigFile: (fileName: string) => Promise<any>;
};

type ContentFunctions = {
  /**
   * INTERNAL: Gets the list content of a server.
   * @param serverId - The ID of the server.
   * @returns
   */
  get: (serverId: string) => Promise<Mod[]>;

  /**
   * Installs a mod to a server.
   * @param contentType - The type of content to install.
   * @param projectId - The ID of the project.
   * @param versionId - The ID of the version.
   */
  install: (contentType: ContentType, projectId: string, versionId: string) => Promise<void>;

  /**
   * Removes a mod from a server.
   * @param path - The path of the mod file.
   */
  remove: (path: string) => Promise<void>;

  /**
   * Reinstalls a mod to a server.
   * @param replace - The path of the mod to replace.
   * @param projectId - The ID of the content.
   * @param versionId - The ID of the new version.
   */
  reinstall: (replace: string, projectId: string, versionId: string) => Promise<void>;
};

type BackupFunctions = {
  /**
   * INTERNAL: Gets the backups of a server.
   * @param serverId - The ID of the server.
   * @returns
   */
  get: (serverId: string) => Promise<Backup[]>;

  /**
   * Creates a new backup for the server.
   * @param backupName - The name of the backup.
   * @returns The ID of the backup.
   */
  create: (backupName: string) => Promise<void>;

  /**
   * Renames a backup for the server.
   * @param backupId - The ID of the backup.
   * @param newName - The new name for the backup.
   */
  rename: (backupId: string, newName: string) => Promise<void>;

  /**
   * Deletes a backup for the server.
   * @param backupId - The ID of the backup.
   */
  delete: (backupId: string) => Promise<void>;

  /**
   * Restores a backup for the server.
   * @param serverId - The ID of the server.
   * @param backupId - The ID of the backup.
   */
  restore: (backupId: string) => Promise<void>;

  /**
   * Downloads a backup for the server.
   * @param backupId - The ID of the backup.
   */
  download: (backupId: string) => Promise<void>;

  /**
   * Prepare a backup for the server.
   * @param backupId - The ID of the backup.
   */
  prepare: (backupId: string) => Promise<void>;

  /**
   * Updates the auto backup settings of the server.
   * @param autoBackup - Whether to enable auto backup.
   * @param interval - The interval to backup at (in Hours).
   */
  updateAutoBackup: (autoBackup: "enable" | "disable", interval: number) => Promise<void>;

  /**
   * Gets the auto backup settings of the server.
   */
  getAutoBackup: () => Promise<AutoBackupSettings>;

  /**
   * Locks a backup for the server.
   * @param backupId - The ID of the backup.
   */
  lock: (backupId: string) => Promise<void>;

  /**
   * Unlocks a backup for the server.
   * @param backupId - The ID of the backup.
   */
  unlock: (backupId: string) => Promise<void>;

  /**
   * Retries a failed backup for the server.
   * @param backupId - The ID of the backup.
   */
  retry: (backupId: string) => Promise<void>;
};

type NetworkFunctions = {
  /**
   * INTERNAL: Gets the network settings of a server.
   * @param serverId - The ID of the server.
   * @returns
   */
  get: (serverId: string) => Promise<Allocation[]>;

  /**
   * Reserves a new allocation for the server.
   * @param name - The name of the allocation.
   * @returns The allocated network port details.
   */
  reserveAllocation: (name: string) => Promise<Allocation>;

  /**
   * Updates the allocation for the server.
   * @param port - The port to update.
   * @param name - The new name for the allocation.
   */
  updateAllocation: (port: number, name: string) => Promise<void>;

  /**
   * Deletes an allocation for the server.
   * @param port - The port to delete.
   */
  deleteAllocation: (port: number) => Promise<void>;

  /**
   * Checks if a subdomain is available.
   * @param subdomain - The subdomain to check.
   * @returns True if the subdomain is available, otherwise false.
   */
  checkSubdomainAvailability: (subdomain: string) => Promise<boolean>;

  /**
   * Changes the subdomain of the server.
   * @param subdomain - The new subdomain.
   */
  changeSubdomain: (subdomain: string) => Promise<void>;
};

type StartupFunctions = {
  /**
   * INTERNAL: Gets the startup settings of a server.
   * @param serverId - The ID of the server.
   * @returns
   */
  get: (serverId: string) => Promise<Startup>;

  /**
   * Updates the startup settings of a server.
   * @param invocation - The invocation of the server.
   * @param jdkVersion - The version of the JDK.
   * @param jdkBuild - The build of the JDK.
   */
  update: (
    invocation: string,
    jdkVersion: "lts8" | "lts11" | "lts17" | "lts21",
    jdkBuild: "corretto" | "temurin" | "graal",
  ) => Promise<void>;
};

type FSFunctions = {
  /**
   * INTERNAL: Gets the file system settings of a server.
   * @param serverId
   * @returns
   */
  get: (serverId: string) => Promise<JWTAuth>;

  /**
   * @param path - The path to list the contents of.
   * @param page - The page to list.
   * @param pageSize - The page size to list.
   * @returns
   */
  listDirContents: (path: string, page: number, pageSize: number) => Promise<DirectoryResponse>;

  /**
   * @param path - The path to create the file or folder at.
   * @param type - The type of file or folder to create.
   * @returns
   */
  createFileOrFolder: (path: string, type: "file" | "directory") => Promise<any>;

  /**
   * @param path - The path to upload the file to.
   * @param file - The file to upload.
   * @returns
   */
  uploadFile: (path: string, file: File) => Promise<any>;

  /**
   * @param path - The path to rename the file or folder at.
   * @param name - The new name for the file or folder.
   * @returns
   */
  renameFileOrFolder: (path: string, name: string) => Promise<any>;

  /**
   * @param path - The path to update the file at.
   * @param content - The new content for the file.
   * @returns
   */
  updateFile: (path: string, content: string) => Promise<any>;

  /**
   * @param path - The path to move the file or folder at.
   * @param newPath - The new path for the file or folder.
   * @returns
   */
  moveFileOrFolder: (path: string, newPath: string) => Promise<any>;

  /**
   * @param path - The path to delete the file or folder at.
   * @param recursive - Whether to delete the file or folder recursively.
   * @returns
   */
  deleteFileOrFolder: (path: string, recursive: boolean) => Promise<any>;

  /**
   * @param serverId - The ID of the server.
   * @param path - The path to download the file from.
   * @param raw - Whether to return the raw blob.
   * @returns
   */
  downloadFile: (path: string, raw?: boolean) => Promise<any>;

  /**
   * @param path - The path of the file to extract
   * @returns
   */
  extractFile: (
    path: string,
    override?: boolean,
    dry?: boolean,
    silentQueue?: boolean,
  ) => Promise<{
    modpack_name: string | null;
    conflicting_files: string[];
  }>;

  removeQueuedOp: (op: FSQueuedOp["op"], src: string) => void;
  clearQueuedOps: () => void;

  modifyOp: (id: string, action: "dismiss" | "cancel") => Promise<any>;
};

type ModuleError = {
  error: ServersError;
  timestamp: number;
};

type GeneralModule = General &
  GeneralFunctions & {
    error?: ModuleError;
  };

type ContentModule = {
  data: Mod[];
  error?: ModuleError;
} & ContentFunctions;

type BackupsModule = {
  data: Backup[];
  error?: ModuleError;
} & BackupFunctions;

type NetworkModule = {
  allocations: Allocation[];
  error?: ModuleError;
} & NetworkFunctions;

type StartupModule = Startup &
  StartupFunctions & {
    error?: ModuleError;
  };

type WSModule = JWTAuth & {
  error?: ModuleError;
};

export type FSModule = {
  auth: JWTAuth;
  ops: FilesystemOp[];
  queuedOps: FSQueuedOp[];
  opsQueuedForModification: string[];
  error?: ModuleError;
} & FSFunctions;

type ModulesMap = {
  general: GeneralModule;
  content: ContentModule;
  backups: BackupsModule;
  network: NetworkModule;
  startup: StartupModule;
  ws: WSModule;
  fs: FSModule;
};

type avaliableModules = ("general" | "content" | "backups" | "network" | "startup" | "ws" | "fs")[];

export type Server<T extends avaliableModules> = {
  [K in T[number]]?: ModulesMap[K];
} & {
  /**
   * Refreshes the included modules of the server
   * @param refreshModules - The modules to refresh.
   * @param options - The options to use when refreshing the modules.
   */
  refresh: (
    refreshModules?: avaliableModules,
    options?: {
      preserveConnection?: boolean;
      preserveInstallState?: boolean;
    },
  ) => Promise<void>;
  loadModules: (modulesToLoad: avaliableModules) => Promise<void>;
  setError: (error: Error) => void;
  error?: Error;
  serverId: string;
};

export const usePyroServer = async (serverId: string, includedModules: avaliableModules) => {
  const server: Server<typeof includedModules> = reactive({
    refresh: async (
      refreshModules?: avaliableModules,
      options?: {
        preserveConnection?: boolean;
        preserveInstallState?: boolean;
      },
    ) => {
      if (server.general?.status === "installing" && !refreshModules) {
        return;
      }

      const modulesToRefresh = [...new Set(refreshModules || includedModules)];
      const serverError = new PyroServerError();

      const modulePromises = modulesToRefresh.map(async (module) => {
        try {
          const mods = modules[module];
          if (!mods?.get) return;

          const data = await mods.get(serverId);
          if (!data) return;

          if (module === "general" && options?.preserveConnection) {
            server[module] = {
              ...server[module],
              ...data,
              image: server[module]?.image || data.image,
              motd: server[module]?.motd || data.motd,
              status:
                options.preserveInstallState && server[module]?.status === "installing"
                  ? "installing"
                  : data.status,
            };
          } else {
            server[module] = { ...server[module], ...data };
          }
        } catch (error) {
          console.error(`Failed to refresh module ${module}:`, error);
          if (error instanceof Error) {
            serverError.addError(module, error);
          }
        }
      });

      await Promise.allSettled(modulePromises);

      if (serverError.hasErrors()) {
        if (server.error && server.error instanceof PyroServerError) {
          serverError.errors.forEach((error, module) => {
            (server.error as PyroServerError).addError(module, error);
          });
        } else {
          server.setError(serverError);
        }
      }
    },
    loadModules: async (modulesToLoad: avaliableModules) => {
      const newModules = modulesToLoad.filter((module) => !server[module]);
      if (newModules.length === 0) return;

      newModules.forEach((module) => {
        server[module] = modules[module];
      });

      await server.refresh(newModules);
    },
    setError: (error: Error) => {
      if (!server.error) {
        server.error = error;
      } else if (error instanceof PyroServerError) {
        if (!(server.error instanceof PyroServerError)) {
          const newError = new PyroServerError();
          newError.addError("previous", server.error);
          server.error = newError;
        }
        error.errors.forEach((err, module) => {
          (server.error as PyroServerError).addError(module, err);
        });
      }
    },

    serverId,
  });

  const initialModules = includedModules.filter((module) => ["general", "ws"].includes(module));
  const deferredModules = includedModules.filter((module) => !["general", "ws"].includes(module));

  initialModules.forEach((module) => {
    server[module] = modules[module];
  });

  internalServerReference.value = server;
  await server.refresh(initialModules);

  if (deferredModules.length > 0) {
    await server.loadModules(deferredModules);
  }

  return server as Server<typeof includedModules>;
};
