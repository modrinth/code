import { ModrinthServerError } from "@modrinth/utils";
import type { JWTAuth, ModuleError, ModuleName } from "@modrinth/utils";
import { useServersFetch } from "./servers-fetch.ts";

import {
  GeneralModule,
  ContentModule,
  BackupsModule,
  NetworkModule,
  StartupModule,
  WSModule,
  FSModule,
} from "./modules/index.ts";

export function handleError(err: any) {
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

export class ModrinthServer {
  readonly serverId: string;
  private errors: Partial<Record<ModuleName, ModuleError>> = {};

  readonly general: GeneralModule;
  readonly content: ContentModule;
  readonly backups: BackupsModule;
  readonly network: NetworkModule;
  readonly startup: StartupModule;
  readonly ws: WSModule;
  readonly fs: FSModule;

  constructor(serverId: string) {
    this.serverId = serverId;

    this.general = new GeneralModule(this);
    this.content = new ContentModule(this);
    this.backups = new BackupsModule(this);
    this.network = new NetworkModule(this);
    this.startup = new StartupModule(this);
    this.ws = new WSModule(this);
    this.fs = new FSModule(this);
  }

  async createMissingFolders(path: string): Promise<void> {
    if (path.startsWith("/")) {
      path = path.substring(1);
    }
    const folders = path.split("/");
    let currentPath = "";

    for (const folder of folders) {
      currentPath += "/" + folder;
      try {
        await this.fs.createFileOrFolder(currentPath, "directory");
      } catch {
        // Folder might already exist, ignore error
      }
    }
  }

  async fetchConfigFile(fileName: string): Promise<any> {
    return await useServersFetch(`servers/${this.serverId}/config/${fileName}`);
  }

  constructServerProperties(properties: any): string {
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

  async processImage(iconUrl: string | undefined): Promise<string | undefined> {
    const sharedImage = useState<string | undefined>(`server-icon-${this.serverId}`);

    if (sharedImage.value) {
      return sharedImage.value;
    }

    try {
      const auth = await useServersFetch<JWTAuth>(`servers/${this.serverId}/fs`);
      try {
        const fileData = await useServersFetch(`/download?path=/server-icon-original.png`, {
          override: auth,
          retry: 1, // Reduce retries for optional resources
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
        if (error instanceof ModrinthServerError) {
          if (error.statusCode && error.statusCode >= 500) {
            console.debug("Service unavailable, skipping icon processing");
            sharedImage.value = undefined;
            return undefined;
          }

          if (error.statusCode === 404 && iconUrl) {
            try {
              const response = await fetch(iconUrl);
              if (!response.ok) throw new Error("Failed to fetch icon");
              const file = await response.blob();
              const originalFile = new File([file], "server-icon-original.png", {
                type: "image/png",
              });

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
                        const scaledFile = new File([blob], "server-icon.png", {
                          type: "image/png",
                        });
                        await useServersFetch(`/create?path=/server-icon.png&type=file`, {
                          method: "POST",
                          contentType: "application/octet-stream",
                          body: scaledFile,
                          override: auth,
                        });
                        await useServersFetch(`/create?path=/server-icon-original.png&type=file`, {
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
            } catch (externalError: any) {
              console.debug("Could not process external icon:", externalError.message);
            }
          }
        } else {
          throw error;
        }
      }
    } catch (error: any) {
      console.debug("Icon processing failed:", error.message);
    }

    sharedImage.value = undefined;
    return undefined;
  }

  async testNodeReachability(): Promise<boolean> {
    if (!this.general?.datacenter) {
      console.warn("No datacenter info available for ping test");
      return false;
    }

    const datacenter = this.general.datacenter;
    const wsUrl = `wss://${datacenter}.nodes.modrinth.com/pingtest`;

    try {
      return await new Promise((resolve) => {
        const socket = new WebSocket(wsUrl);
        const timeout = setTimeout(() => {
          socket.close();
          resolve(false);
        }, 5000);

        socket.onopen = () => {
          clearTimeout(timeout);
          socket.send(performance.now().toString());
        };

        socket.onmessage = () => {
          clearTimeout(timeout);
          socket.close();
          resolve(true);
        };

        socket.onerror = () => {
          clearTimeout(timeout);
          resolve(false);
        };
      });
    } catch (error) {
      console.error(`Failed to ping node ${wsUrl}:`, error);
      return false;
    }
  }

  async refresh(
    modules: ModuleName[] = [],
    options?: {
      preserveConnection?: boolean;
      preserveInstallState?: boolean;
    },
  ): Promise<void> {
    const modulesToRefresh =
      modules.length > 0
        ? modules
        : (["general", "content", "backups", "network", "startup", "ws", "fs"] as ModuleName[]);

    for (const module of modulesToRefresh) {
      this.errors[module] = undefined;

      try {
        switch (module) {
          case "general": {
            if (options?.preserveConnection) {
              const currentImage = this.general.image;
              const currentMotd = this.general.motd;
              const currentStatus = this.general.status;

              await this.general.fetch();

              if (currentImage) {
                this.general.image = currentImage;
              }
              if (currentMotd) {
                this.general.motd = currentMotd;
              }
              if (options.preserveInstallState && currentStatus === "installing") {
                this.general.status = "installing";
              }
            } else {
              await this.general.fetch();
            }
            break;
          }
          case "content":
            await this.content.fetch();
            break;
          case "backups":
            await this.backups.fetch();
            break;
          case "network":
            await this.network.fetch();
            break;
          case "startup":
            await this.startup.fetch();
            break;
          case "ws":
            await this.ws.fetch();
            break;
          case "fs":
            await this.fs.fetch();
            break;
        }
      } catch (error) {
        if (error instanceof ModrinthServerError) {
          if (error.statusCode === 404 && ["fs", "content"].includes(module)) {
            console.debug(`Optional ${module} resource not found:`, error.message);
            continue;
          }

          if (error.statusCode && error.statusCode >= 500) {
            console.debug(`Temporary ${module} unavailable:`, error.message);
            continue;
          }
        }

        this.errors[module] = {
          error:
            error instanceof ModrinthServerError
              ? error
              : new ModrinthServerError("Unknown error", undefined, error as Error),
          timestamp: Date.now(),
        };
      }
    }
  }

  get moduleErrors() {
    return this.errors;
  }
}

export const useModrinthServers = async (
  serverId: string,
  includedModules: ModuleName[] = ["general"],
) => {
  const server = new ModrinthServer(serverId);
  await server.refresh(includedModules);
  return reactive(server);
};
