// usePyroServer is a composable that interfaces with the REDACTED API to get data and control the users server
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
}

async function PyroFetch<T>(path: string, options: PyroFetchOptions = {}): Promise<T> {
  const config = useRuntimeConfig();
  const auth = await useAuth();
  const authToken = auth.value?.token;

  if (!authToken) {
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

  const headers: HeadersRecord = {
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
    console.error("[PyroServers/PyroFetch]:", error);
    if (error instanceof FetchError) {
      const statusCode = error.response?.status;
      const statusText = error.response?.statusText || "[no status text available]";
      const errorMessages: { [key: number]: string } = {
        400: "Bad Request",
        401: "Unauthorized",
        403: "Forbidden",
        404: "Not Found",
        405: "Method Not Allowed",
        429: "Too Many Requests",
        500: "Internal Server Error",
        502: "Bad Gateway",
        503: "Service Unavailable",
      };
      const message =
        statusCode && statusCode in errorMessages
          ? errorMessages[statusCode]
          : `HTTP Error: ${statusCode || "[unhandled status code]"} ${statusText}`;
      throw new PyroFetchError(`[PyroServers/PyroFetch] ${message}`, statusCode, error);
    }
    throw new PyroFetchError(
      "[PyroServers/PyroFetch] An unexpected error occurred during the fetch operation.",
      undefined,
      error as Error,
    );
  }
}

const internalServerRefrence = ref<any>(null);

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
    | "other"
    | "transferring"
    | "upgrading"
    | "support"
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

interface Backup {
  id: string;
  name: string;
  created_at: string;
  ongoing: boolean;
  locked: boolean;
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

type ContentType = "Mod" | "Plugin";

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
  const image = ref<string | null>(null);
  const auth = await PyroFetch<JWTAuth>(`servers/${internalServerRefrence.value.serverId}/fs`);
  try {
    const fileData = await PyroFetch(`/download?path=/server-icon-original.png`, {
      override: auth,
      retry: false,
    });

    if (fileData instanceof Blob) {
      if (import.meta.client) {
        const canvas = document.createElement("canvas");
        const ctx = canvas.getContext("2d");
        const img = new Image();
        img.src = URL.createObjectURL(fileData);
        await new Promise<void>((resolve) => {
          img.onload = () => {
            canvas.width = 512;
            canvas.height = 512;
            ctx?.drawImage(img, 0, 0, 512, 512);
            const dataURL = canvas.toDataURL("image/png");
            internalServerRefrence.value.general.image = dataURL;
            image.value = dataURL;
            resolve();
          };
        });
      }
    }
  } catch (error) {
    if (error instanceof PyroFetchError && error.statusCode === 404) {
      console.log("[PYROSERVERS] No server icon found");
    } else {
      console.error(error);
    }
  }

  if (image.value === null && iconUrl) {
    console.log("iconUrl", iconUrl);
    try {
      const response = await fetch(iconUrl);
      const file = await response.blob();
      const originalfile = new File([file], "server-icon-original.png", {
        type: "image/png",
      });
      if (import.meta.client) {
        const scaledFile = await new Promise<File>((resolve, reject) => {
          const canvas = document.createElement("canvas");
          const ctx = canvas.getContext("2d");
          const img = new Image();
          img.src = URL.createObjectURL(file);
          img.onload = () => {
            canvas.width = 64;
            canvas.height = 64;
            ctx?.drawImage(img, 0, 0, 64, 64);
            canvas.toBlob((blob) => {
              if (blob) {
                const data = new File([blob], "server-icon.png", { type: "image/png" });
                resolve(data);
              } else {
                reject(new Error("Canvas toBlob failed"));
              }
            }, "image/png");
          };
          img.onerror = reject;
        });
        if (scaledFile) {
          await PyroFetch(`/create?path=/server-icon.png&type=file`, {
            method: "POST",
            contentType: "application/octet-stream",
            body: scaledFile,
            override: auth,
          });

          await PyroFetch(`/create?path=/server-icon-original.png&type=file`, {
            method: "POST",
            contentType: "application/octet-stream",
            body: originalfile,
            override: auth,
          });
        }
      }
    } catch (error) {
      if (error instanceof PyroFetchError && error.statusCode === 404) {
        console.log("[PYROSERVERS] No server icon found");
      } else {
        console.error(error);
      }
    }
  }
  return image.value;
};

// ------------------ GENERAL ------------------ //

const sendPowerAction = async (action: string) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/power`, {
      method: "POST",
      body: { action },
    });

    await new Promise((resolve) => setTimeout(resolve, 1000));
    await internalServerRefrence.value.refresh();
  } catch (error) {
    console.error("Error changing power state:", error);
    throw error;
  }
};

const updateName = async (newName: string) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/name`, {
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
      `servers/${internalServerRefrence.value.serverId}/reinstallFromMrpack`,
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
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/suspend`, {
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
    return await PyroFetch(`servers/${internalServerRefrence.value.serverId}/config/${fileName}`);
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
        `servers/${internalServerRefrence.value.serverId}/fs`,
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
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/mods`, {
      method: "POST",
      body: {
        install_as: contentType,
        rinth_ids: { project_id: projectId, version_id: versionId },
      },
    });
  } catch (error) {
    console.error("Error installing mod:", error);
    throw error;
  }
};

const removeContent = async (contentType: ContentType, contentId: string) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/deleteMod`, {
      method: "POST",
      body: {
        install_as: contentType,
        path: contentId,
      },
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    throw error;
  }
};

const reinstallContent = async (
  contentType: ContentType,
  contentId: string,
  newContentId: string,
) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/mods/${contentId}`, {
      method: "PUT",
      body: { install_as: contentType, version_id: newContentId },
    });
  } catch (error) {
    console.error("Error reinstalling mod:", error);
    throw error;
  }
};

// ------------------ BACKUPS ------------------ //

const createBackup = async (backupName: string) => {
  try {
    const response = (await PyroFetch(`servers/${internalServerRefrence.value.serverId}/backups`, {
      method: "POST",
      body: { name: backupName },
    })) as { id: string };
    return response.id;
  } catch (error) {
    console.error("Error creating backup:", error);
    throw error;
  }
};

const renameBackup = async (backupId: string, newName: string) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/backups/${backupId}/rename`, {
      method: "POST",
      body: { name: newName },
    });
  } catch (error) {
    console.error("Error renaming backup:", error);
    throw error;
  }
};

const deleteBackup = async (backupId: string) => {
  try {
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
  } catch (error) {
    console.error("Error deleting backup:", error);
    throw error;
  }
};

const restoreBackup = async (backupId: string) => {
  try {
    await PyroFetch(
      `servers/${internalServerRefrence.value.serverId}/backups/${backupId}/restore`,
      {
        method: "POST",
      },
    );
  } catch (error) {
    console.error("Error restoring backup:", error);
    throw error;
  }
};

const downloadBackup = async (backupId: string) => {
  try {
    return await PyroFetch(`servers/${internalServerRefrence.value.serverId}/backups/${backupId}`);
  } catch (error) {
    console.error("Error downloading backup:", error);
    throw error;
  }
};

const updateAutoBackup = async (autoBackup: "enable" | "disable", interval: number) => {
  try {
    return await PyroFetch(`servers/${internalServerRefrence.value.serverId}/autobackup`, {
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
    return await PyroFetch(`servers/${internalServerRefrence.value.serverId}/autobackup`);
  } catch (error) {
    console.error("Error getting auto backup settings:", error);
    throw error;
  }
};

const lockBackup = async (backupId: string) => {
  try {
    return await PyroFetch(
      `servers/${internalServerRefrence.value.serverId}/backups/${backupId}/lock`,
      {
        method: "POST",
      },
    );
  } catch (error) {
    console.error("Error locking backup:", error);
    throw error;
  }
};

const unlockBackup = async (backupId: string) => {
  try {
    return await PyroFetch(
      `servers/${internalServerRefrence.value.serverId}/backups/${backupId}/unlock`,
      {
        method: "POST",
      },
    );
  } catch (error) {
    console.error("Error locking backup:", error);
    throw error;
  }
};

// ------------------ NETWORK ------------------ //

const reserveAllocation = async (name: string): Promise<Allocation> => {
  try {
    return await PyroFetch<Allocation>(
      `servers/${internalServerRefrence.value.serverId}/allocations?name=${name}`,
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
      `servers/${internalServerRefrence.value.serverId}/allocations/${port}?name=${name}`,
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
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/allocations/${port}`, {
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
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/subdomain`, {
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
    await PyroFetch(`servers/${internalServerRefrence.value.serverId}/startup`, {
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
    if (error instanceof PyroFetchError && error.statusCode === 401) {
      await internalServerRefrence.value.refresh(["fs"]);
      return await requestFn();
    }

    throw error;
  }
};

const listDirContents = (path: string, page: number, pageSize: number) => {
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    return await PyroFetch(`/list?path=${encodedPath}&page=${page}&page_size=${pageSize}`, {
      override: internalServerRefrence.value.fs.auth,
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
      override: internalServerRefrence.value.fs.auth,
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
        `https://${internalServerRefrence.value.fs.auth.url}/create?path=${encodedPath}&type=file`,
      );
      xhr.setRequestHeader("Authorization", `Bearer ${internalServerRefrence.value.fs.auth.token}`);
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
      override: internalServerRefrence.value.fs.auth,
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
      override: internalServerRefrence.value.fs.auth,
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
      override: internalServerRefrence.value.fs.auth,
      body: {
        source: path,
        destination: newPath,
      },
    });
  });
};

const deleteFileOrFolder = (path: string, recursive: boolean) => {
  const encodedPath = encodeURIComponent(path);
  return retryWithAuth(async () => {
    return await PyroFetch(`/delete?path=${encodedPath}&recursive=${recursive}`, {
      method: "DELETE",
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const downloadFile = (path: string, raw?: boolean) => {
  return retryWithAuth(async () => {
    const encodedPath = encodeURIComponent(path);
    const fileData = await PyroFetch(`/download?path=${encodedPath}`, {
      override: internalServerRefrence.value.fs.auth,
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
        const data = await PyroFetch<General>(`servers/${serverId}`);
        // TODO: temp hack to fix hydration error
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
        internalServerRefrence.value.setError(error);
        return undefined;
      }
    },
    updateName,
    power: sendPowerAction,
    reinstall: reinstallServer,
    reinstallFromMrpack,
    suspend: suspendServer,
    getMotd,
    setMotd,
    fetchConfigFile,
  },
  content: {
    get: async (serverId: string) => {
      try {
        const mods = await PyroFetch<Mod[]>(`servers/${serverId}/mods`);
        return {
          data:
            internalServerRefrence.value.error === undefined
              ? mods.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? ""))
              : [],
        };
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
      }
    },
    install: installContent,
    remove: removeContent,
    reinstall: reinstallContent,
  },
  backups: {
    get: async (serverId: string) => {
      try {
        return { data: await PyroFetch<Backup[]>(`servers/${serverId}/backups`) };
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
      }
    },
    create: createBackup,
    rename: renameBackup,
    delete: deleteBackup,
    restore: restoreBackup,
    download: downloadBackup,
    updateAutoBackup,
    getAutoBackup,
    lock: lockBackup,
    unlock: unlockBackup,
  },
  network: {
    get: async (serverId: string) => {
      try {
        return { allocations: await PyroFetch<Allocation[]>(`servers/${serverId}/allocations`) };
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
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
        return await PyroFetch<Startup>(`servers/${serverId}/startup`);
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
      }
    },
    update: updateStartupSettings,
  },
  ws: {
    get: async (serverId: string) => {
      try {
        return await PyroFetch<JWTAuth>(`servers/${serverId}/ws`);
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
      }
    },
  },
  fs: {
    get: async (serverId: string) => {
      try {
        return { auth: await PyroFetch<JWTAuth>(`servers/${serverId}/fs`) };
      } catch (error) {
        internalServerRefrence.value.setError(error);
        return undefined;
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
   * INTERNAL: Gets the config file of a server.
   * @param fileName - The name of the file.
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
   * @param contentType - The type of content to remove.
   * @param contentId - The ID of the content.
   */
  remove: (contentType: ContentType, contentId: string) => Promise<void>;

  /**
   * Reinstalls a mod to a server.
   * @param contentType - The type of content to reinstall.
   * @param contentId - The ID of the content.
   * @param newContentId - The ID of the new version.
   */
  reinstall: (contentType: ContentType, contentId: string, newContentId: string) => Promise<void>;
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
};

type GeneralModule = General & GeneralFunctions;
type ContentModule = { data: Mod[] } & ContentFunctions;
type BackupsModule = { data: Backup[] } & BackupFunctions;
type NetworkModule = { allocations: Allocation[] } & NetworkFunctions;
type StartupModule = Startup & StartupFunctions;
export type FSModule = { auth: JWTAuth } & FSFunctions;

type ModulesMap = {
  general: GeneralModule;
  content: ContentModule;
  backups: BackupsModule;
  network: NetworkModule;
  startup: StartupModule;
  ws: JWTAuth;
  fs: FSModule;
};

type avaliableModules = ("general" | "content" | "backups" | "network" | "startup" | "ws" | "fs")[];

export type Server<T extends avaliableModules> = {
  [K in T[number]]?: ModulesMap[K];
} & {
  /**
   * Refreshes the included modules of the server
   * @param refreshModules - The modules to refresh.
   */
  refresh: (refreshModules?: avaliableModules) => Promise<void>;
  setError: (error: Error) => void;
  error?: Error;
  serverId: string;
};

export const usePyroServer = async (serverId: string, includedModules: avaliableModules) => {
  const server: Server<typeof includedModules> = reactive({
    refresh: async (refreshModules?: avaliableModules) => {
      const promises: Promise<void>[] = [];
      if (refreshModules) {
        for (const module of refreshModules) {
          promises.push(
            (async () => {
              const mods = modules[module];
              if (mods.get) {
                const data = await mods.get(serverId);
                server[module] = { ...server[module], ...data };
              }
            })(),
          );
        }
      } else {
        for (const module of includedModules) {
          promises.push(
            (async () => {
              const mods = modules[module];
              if (mods.get) {
                const data = await mods.get(serverId);
                server[module] = { ...server[module], ...data };
              }
            })(),
          );
        }
      }
      await Promise.all(promises);
    },
    setError: (error: Error) => {
      server.error = error;
    },
    serverId,
  });

  for (const module of includedModules) {
    const mods = modules[module];
    server[module] = mods;
  }

  internalServerRefrence.value = server;

  await server.refresh();

  return server as Server<typeof includedModules>;
};
