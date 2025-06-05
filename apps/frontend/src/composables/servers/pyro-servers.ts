import { $fetch, FetchError } from "ofetch";
import { ServersError } from "@modrinth/utils";
import { usePyroFetch, type PyroFetchOptions } from "./pyro-fetch.ts";
import type {
  V1ErrorInfo,
  JWTAuth,
  ServerGeneral,
  Project,
  Mod,
  Backup,
  Allocation,
  Startup,
  DirectoryResponse,
  AutoBackupSettings,
  ContentType,
  PowerAction,
  JDKVersion,
  JDKBuild,
  ModuleError,
  ModuleName,
  FSQueuedOp,
  FilesystemOp
} from "@modrinth/utils";

// Error handling
export function handleError(err: any) {
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
}

async function pyroFetch<T>(
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

  const base = (import.meta.server ? config.pyroBaseUrl : config.public.pyroBaseUrl)?.replace(/\/$/, "");

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
    "User-Agent": "Modrinth (https://modrinth.com)",
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

      throw new ServersError("Unexpected error during fetch operation", undefined, error as Error, module);
    }
  }

  throw lastError || new Error("Maximum retry attempts reached");
}

export class PyroServer {
  private readonly serverId: string;
  private data: Partial<Record<ModuleName, any>> = {};
  private errors: Partial<Record<ModuleName, ModuleError>> = {};

  constructor(serverId: string) {
    this.serverId = serverId;
  }

  async getGeneral(): Promise<ServerGeneral> {
    const data = await pyroFetch<ServerGeneral>(`servers/${this.serverId}`, {}, "general");

    if (data.upstream?.project_id) {
      const project = await $fetch(`https://api.modrinth.com/v2/project/${data.upstream.project_id}`);
      data.project = project as Project;
    }

    if (import.meta.client) {
      data.image = (await this.processImage(data.project?.icon_url)) ?? undefined;
    }

    const motd = await this.getMotd();
    if (motd === "A Minecraft Server") {
      await this.setMotd(`§b${data.project?.title || data.loader + " " + data.mc_version} §f♦ §aModrinth Servers`);
    }
    data.motd = motd;

    this.data.general = data;
    return data;
  }

  async updateName(newName: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/name`, {
      method: "POST",
      body: { name: newName },
    });
  }

  async sendPowerAction(action: PowerAction): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/power`, {
      method: "POST",
      body: { action },
    });

    await new Promise((resolve) => setTimeout(resolve, 1000));
    await this.refresh(["general"]);
  }

  async reinstallServer(
    loader: boolean,
    projectId: string,
    versionId?: string,
    loaderVersionId?: string,
    hardReset: boolean = false,
  ): Promise<void> {
    const hardResetParam = hardReset ? "true" : "false";

    if (loader) {
      if (projectId.toLowerCase() === "neoforge") {
        projectId = "NeoForge";
      }
      await pyroFetch(`servers/${this.serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { loader: projectId, loader_version: loaderVersionId, game_version: versionId },
      });
    } else {
      await pyroFetch(`servers/${this.serverId}/reinstall?hard=${hardResetParam}`, {
        method: "POST",
        body: { project_id: projectId, version_id: versionId },
      });
    }
  }

  async reinstallFromMrpack(mrpack: File, hardReset: boolean = false): Promise<void> {
    const hardResetParam = hardReset ? "true" : "false";
    const auth = await pyroFetch<JWTAuth>(`servers/${this.serverId}/reinstallFromMrpack`);

    const formData = new FormData();
    formData.append("file", mrpack);

    const response = await fetch(`https://${auth.url}/reinstallMrpackMultiparted?hard=${hardResetParam}`, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${auth.token}`,
      },
      body: formData,
      signal: AbortSignal.timeout(30 * 60 * 1000),
    });

    if (!response.ok) {
      throw new Error(`[pyroservers] native fetch err status: ${response.status}`);
    }
  }

  async suspendServer(status: boolean): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/suspend`, {
      method: "POST",
      body: { suspended: status },
    });
  }

  async endIntro(): Promise<void> {
    await usePyroFetch(`servers/${this.serverId}/flows/intro`, {
      method: "DELETE",
      version: 1,
    });
    await this.refresh(["general"]);
  }

  async getMotd(): Promise<string | undefined> {
    try {
      const props = await this.downloadFile("/server.properties");
      if (props) {
        const lines = props.split("\n");
        for (const line of lines) {
          if (line.startsWith("motd=")) {
            return line.slice(5);
          }
        }
      }
    } catch {
      return undefined;
    }
    return undefined;
  }

  async setMotd(motd: string): Promise<void> {
    const props = await this.fetchConfigFile("ServerProperties") as any;
    if (props) {
      props.motd = motd;
      const newProps = this.constructServerProperties(props);
      const octetStream = new Blob([newProps], { type: "application/octet-stream" });
      const auth = await pyroFetch<JWTAuth>(`servers/${this.serverId}/fs`);

      await pyroFetch(`/update?path=/server.properties`, {
        method: "PUT",
        contentType: "application/octet-stream",
        body: octetStream,
        override: auth,
      });
    }
  }

  async getContent(): Promise<Mod[]> {
    const mods = await pyroFetch<Mod[]>(`servers/${this.serverId}/mods`, {}, "content");
    return mods.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? ""));
  }

  async installContent(contentType: ContentType, projectId: string, versionId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/mods`, {
      method: "POST",
      body: {
        rinth_ids: { project_id: projectId, version_id: versionId },
        install_as: contentType,
      },
    });
  }

  async removeContent(path: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/deleteMod`, {
      method: "POST",
      body: { path },
    });
  }

  async reinstallContent(replace: string, projectId: string, versionId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/mods/update`, {
      method: "POST",
      body: { replace, project_id: projectId, version_id: versionId },
    });
  }

  async getBackups(): Promise<Backup[]> {
    return await pyroFetch<Backup[]>(`servers/${this.serverId}/backups`, {}, "backups");
  }

  async createBackup(backupName: string): Promise<string> {
    const response = await pyroFetch<{ id: string }>(`servers/${this.serverId}/backups`, {
      method: "POST",
      body: { name: backupName },
    });
    await this.refresh(["backups"]);
    return response.id;
  }

  async renameBackup(backupId: string, newName: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/rename`, {
      method: "POST",
      body: { name: newName },
    });
    await this.refresh(["backups"]);
  }

  async deleteBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
    await this.refresh(["backups"]);
  }

  async restoreBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/restore`, {
      method: "POST",
    });
    await this.refresh(["backups"]);
  }

  async prepareBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/prepare-download`, {
      method: "POST",
    });
  }

  async lockBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/lock`, {
      method: "POST",
    });
    await this.refresh(["backups"]);
  }

  async unlockBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/unlock`, {
      method: "POST",
    });
    await this.refresh(["backups"]);
  }

  async retryBackup(backupId: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/backups/${backupId}/retry`, {
      method: "POST",
    });
  }

  async updateAutoBackup(autoBackup: "enable" | "disable", interval: number): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/autobackup`, {
      method: "POST",
      body: { set: autoBackup, interval },
    });
  }

  async getAutoBackup(): Promise<AutoBackupSettings> {
    return await pyroFetch(`servers/${this.serverId}/autobackup`);
  }

  async getAllocations(): Promise<Allocation[]> {
    return await pyroFetch<Allocation[]>(`servers/${this.serverId}/allocations`, {}, "network");
  }

  async reserveAllocation(name: string): Promise<Allocation> {
    return await pyroFetch<Allocation>(`servers/${this.serverId}/allocations?name=${name}`, {
      method: "POST",
    });
  }

  async updateAllocation(port: number, name: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/allocations/${port}?name=${name}`, {
      method: "PUT",
    });
  }

  async deleteAllocation(port: number): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/allocations/${port}`, {
      method: "DELETE",
    });
  }

  async checkSubdomainAvailability(subdomain: string): Promise<boolean> {
    const result = await pyroFetch(`subdomains/${subdomain}/isavailable`) as { available: boolean };
    return result.available;
  }

  async changeSubdomain(subdomain: string): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/subdomain`, {
      method: "POST",
      body: { subdomain },
    });
  }

  async getStartup(): Promise<Startup> {
    return await pyroFetch<Startup>(`servers/${this.serverId}/startup`, {}, "startup");
  }

  async updateStartupSettings(invocation: string, jdkVersion: JDKVersion, jdkBuild: JDKBuild): Promise<void> {
    await pyroFetch(`servers/${this.serverId}/startup`, {
      method: "POST",
      body: {
        invocation: invocation || null,
        jdk_version: jdkVersion || null,
        jdk_build: jdkBuild || null,
      },
    });
  }

  // File system operations
  async getFSAuth(): Promise<JWTAuth> {
    return await pyroFetch<JWTAuth>(`servers/${this.serverId}/fs`, {}, "fs");
  }

  async getWSAuth(): Promise<JWTAuth> {
    return await pyroFetch<JWTAuth>(`servers/${this.serverId}/ws`, {}, "ws");
  }

  private async retryWithAuth<T>(requestFn: () => Promise<T>): Promise<T> {
    try {
      return await requestFn();
    } catch (error) {
      if (error instanceof ServersError && error.statusCode === 401) {
        await this.refresh(["fs"]);
        return await requestFn();
      }
      throw error;
    }
  }

  async listDirContents(path: string, page: number, pageSize: number): Promise<DirectoryResponse> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const auth = this.data.fs?.auth || await this.getFSAuth();
      return await pyroFetch(`/list?path=${encodedPath}&page=${page}&page_size=${pageSize}`, {
        override: auth,
        retry: false,
      });
    });
  }

  async createFileOrFolder(path: string, type: "file" | "directory"): Promise<void> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/create?path=${encodedPath}&type=${type}`, {
        method: "POST",
        contentType: "application/octet-stream",
        override: auth,
      });
    });
  }

  async uploadFile(path: string, file: File): Promise<any> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const auth = this.data.fs?.auth || await this.getFSAuth();
      const progressSubject = new EventTarget();
      const abortController = new AbortController();

      const uploadPromise = new Promise((resolve, reject) => {
        const xhr = new XMLHttpRequest();

        xhr.upload.addEventListener("progress", (e) => {
          if (e.lengthComputable) {
            const progress = (e.loaded / e.total) * 100;
            progressSubject.dispatchEvent(
              new CustomEvent("progress", {
                detail: { loaded: e.loaded, total: e.total, progress },
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

        xhr.open("POST", `https://${auth.url}/create?path=${encodedPath}&type=file`);
        xhr.setRequestHeader("Authorization", `Bearer ${auth.token}`);
        xhr.setRequestHeader("Content-Type", "application/octet-stream");
        xhr.send(file);

        abortController.signal.addEventListener("abort", () => xhr.abort());
      });

      return {
        promise: uploadPromise,
        onProgress: (callback: (progress: { loaded: number; total: number; progress: number }) => void) => {
          progressSubject.addEventListener("progress", ((e: CustomEvent) => {
            callback(e.detail);
          }) as EventListener);
        },
        cancel: () => abortController.abort(),
      };
    });
  }

  async renameFileOrFolder(path: string, name: string): Promise<void> {
    const pathName = path.split("/").slice(0, -1).join("/") + "/" + name;
    return this.retryWithAuth(async () => {
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/move`, {
        method: "POST",
        override: auth,
        body: { source: path, destination: pathName },
      });
    });
  }

  async updateFile(path: string, content: string): Promise<void> {
    const octetStream = new Blob([content], { type: "application/octet-stream" });
    return this.retryWithAuth(async () => {
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/update?path=${path}`, {
        method: "PUT",
        contentType: "application/octet-stream",
        body: octetStream,
        override: auth,
      });
    });
  }

  async moveFileOrFolder(path: string, newPath: string): Promise<void> {
    return this.retryWithAuth(async () => {
      await this.createMissingFolders(newPath.substring(0, newPath.lastIndexOf("/")));
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/move`, {
        method: "POST",
        override: auth,
        body: { source: path, destination: newPath },
      });
    });
  }

  async deleteFileOrFolder(path: string, recursive: boolean): Promise<void> {
    const encodedPath = encodeURIComponent(path);
    return this.retryWithAuth(async () => {
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/delete?path=${encodedPath}&recursive=${recursive}`, {
        method: "DELETE",
        override: auth,
      });
    });
  }

  async downloadFile(path: string, raw?: boolean): Promise<any> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const auth = this.data.fs?.auth || await this.getFSAuth();
      const fileData = await pyroFetch(`/download?path=${encodedPath}`, {
        override: auth,
      });

      if (fileData instanceof Blob) {
        return raw ? fileData : await fileData.text();
      }
      return fileData;
    });
  }

  async extractFile(path: string, override = true, dry = false, silentQueue = false): Promise<{ modpack_name: string | null; conflicting_files: string[] }> {
    return this.retryWithAuth(async () => {
      const encodedPath = encodeURIComponent(path);
      const auth = this.data.fs?.auth || await this.getFSAuth();

      if (!silentQueue && this.data.fs?.queuedOps) {
        this.data.fs.queuedOps.push({ op: "unarchive", src: path });
        setTimeout(() => this.removeQueuedOp("unarchive", path), 4000);
      }

      try {
        return await pyroFetch(`/unarchive?src=${encodedPath}&trg=/&override=${override}&dry=${dry}`, {
          method: "POST",
          override: auth,
          version: 1,
        }, undefined, "Error extracting file");
      } catch (err) {
        this.removeQueuedOp("unarchive", path);
        throw err;
      }
    });
  }

  async modifyOp(id: string, action: "dismiss" | "cancel"): Promise<void> {
    return this.retryWithAuth(async () => {
      const auth = this.data.fs?.auth || await this.getFSAuth();
      await pyroFetch(`/ops/${action}?id=${id}`, {
        method: "POST",
        override: auth,
        version: 1,
      }, undefined, `Error ${action === "dismiss" ? "dismissing" : "cancelling"} filesystem operation`);

      if (this.data.fs) {
        this.data.fs.opsQueuedForModification = this.data.fs.opsQueuedForModification?.filter((x: string) => x !== id) || [];
        this.data.fs.ops = this.data.fs.ops?.filter((x: FilesystemOp) => x.id !== id) || [];
      }
    });
  }

  private async processImage(iconUrl: string | undefined): Promise<string | undefined> {
    const sharedImage = useState<string | undefined>(`server-icon-${this.serverId}`);

    if (sharedImage.value) {
      return sharedImage.value;
    }

    try {
      const auth = await pyroFetch<JWTAuth>(`servers/${this.serverId}/fs`);
      try {
        const fileData = await pyroFetch(`/download?path=/server-icon-original.png`, {
          override: auth,
          retry: false,
        });

        if (fileData instanceof Blob && import.meta.client) {
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
      } catch (error) {
        if (error instanceof ServersError && error.statusCode === 404 && iconUrl) {
          // Handle external icon processing
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
                      await pyroFetch(`/create?path=/server-icon.png&type=file`, {
                        method: "POST",
                        contentType: "application/octet-stream",
                        body: scaledFile,
                        override: auth,
                      });
                      await pyroFetch(`/create?path=/server-icon-original.png&type=file`, {
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
  }

  private constructServerProperties(properties: any): string {
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
  }

  private async createMissingFolders(path: string): Promise<void> {
    if (path.startsWith("/")) {
      path = path.substring(1);
    }
    const folders = path.split("/");
    let currentPath = "";

    for (const folder of folders) {
      currentPath += "/" + folder;
      try {
        await this.createFileOrFolder(currentPath, "directory");
      } catch {
        // Folder might already exist, ignore error
      }
    }
  }

  private async fetchConfigFile(fileName: string): Promise<any> {
    return await pyroFetch(`servers/${this.serverId}/config/${fileName}`);
  }

  private removeQueuedOp(op: FSQueuedOp["op"], src: string): void {
    if (this.data.fs?.queuedOps) {
      this.data.fs.queuedOps = this.data.fs.queuedOps.filter((x: FSQueuedOp) => x.op !== op || x.src !== src);
    }
  }

  private clearQueuedOps(): void {
    if (this.data.fs) {
      this.data.fs.queuedOps = [];
    }
  }

  async refresh(modules: ModuleName[] = []): Promise<void> {
    const modulesToRefresh = modules.length > 0 ? modules : Object.keys(this.data) as ModuleName[];

    for (const module of modulesToRefresh) {
      try {
        switch (module) {
          case "general":
            this.data.general = await this.getGeneral();
            break;
          case "content":
            this.data.content = { data: await this.getContent() };
            break;
          case "backups":
            this.data.backups = { data: await this.getBackups() };
            break;
          case "network":
            this.data.network = { allocations: await this.getAllocations() };
            break;
          case "startup":
            this.data.startup = await this.getStartup();
            break;
          case "ws":
            this.data.ws = await this.getWSAuth();
            break;
          case "fs":
            this.data.fs = {
              auth: await this.getFSAuth(),
              ops: [],
              queuedOps: [],
              opsQueuedForModification: [],
            };
            break;
        }
      } catch (error) {
        this.errors[module] = {
          error: error instanceof ServersError ? error : new ServersError("Unknown error", undefined, error as Error),
          timestamp: Date.now(),
        };
      }
    }
  }

  get general() { return this.data.general; }
  get content() { return this.data.content; }
  get backups() { return this.data.backups; }
  get network() { return this.data.network; }
  get startup() { return this.data.startup; }
  get ws() { return this.data.ws; }
  get fs() { return this.data.fs; }
  get moduleErrors() { return this.errors; }
}

export const usePyroServer = async (serverId: string, includedModules: ModuleName[] = ["general"]) => {
  const server = new PyroServer(serverId);
  await server.refresh(includedModules);
  return reactive(server);
};
