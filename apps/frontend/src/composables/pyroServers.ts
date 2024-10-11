// am i winning? :smilew:

// very wip, but it works i know stuff is missing and broken, dont worry, i'll fix it

const config = true;

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
  suspension_reason: string;
  loader: string;
  loader_version: string;
  mc_version: string;
  upstream: {
    kind: "modpack" | "mod" | "resourcepack";
    version_id: string;
    project_id: string;
  };
  motd?: string;
  image?: string;
  project?: Project;
}

interface Allocation {
  port: number;
  name: string;
}

interface Startup {
  invocation: string;
  jdk_version: "lts8" | "lts11" | "lts17" | "lts21";
  jdk_build: "corretto" | "temurin" | "graal";
}

interface Mod {
  filename: string;
  project_id: string;
  version_id: string;
  name: string;
  version_number: string;
  icon_url: string;
  disabled: boolean;
}

interface Backup {
  id: string;
  name: string;
  created_at: string;
}

interface WSAuth {
  url: string;
  token: string;
}

interface FSAuth {
  url: string;
  token: string;
}

// eslint-disable-next-line @typescript-eslint/no-unused-vars
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

// ------------------ GENERAL ------------------ //

const sendPowerAction = async (serverId: string, action: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/power`, {
      method: "POST",
      body: { action },
    });
  } catch (error) {
    console.error("Error changing power state:", error);
    throw error;
  }
};

const updateName = async (serverId: string, newName: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/name`, {
      method: "POST",
      body: { name: newName },
    });
  } catch (error) {
    console.error("Error updating server name:", error);
    throw error;
  }
};

const fetchProject = async (projectId: string): Promise<Project> => {
  try {
    return (await toRaw(useBaseFetch(`project/${projectId}`, {}, false, config))) as Project;
  } catch (error) {
    console.error("Error fetching project:", error);
    throw error;
  }
};

const reinstallServer = async (
  serverId: string,
  loader: boolean,
  projectId: string,
  versionId?: string,
) => {
  try {
    await usePyroFetch(`servers/${serverId}/reinstall`, {
      method: "POST",
      body: loader
        ? { loader: projectId, version: projectId }
        : { project_id: projectId, version_id: versionId },
    });
  } catch (error) {
    console.error("Error reinstalling server:", error);
    throw error;
  }
};

const suspendServer = async (serverId: string, status: boolean) => {
  try {
    await usePyroFetch(`servers/${serverId}/suspend`, {
      method: "POST",
      body: { suspended: status },
    });
  } catch (error) {
    console.error("Error suspending server:", error);
    throw error;
  }
};

// ------------------ MODS ------------------ //

// ------------------ BACKUPS ------------------ //

const createBackup = async (serverId: string, backupName: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/backups`, {
      method: "POST",
      body: { name: backupName },
    });
  } catch (error) {
    console.error("Error creating backup:", error);
    throw error;
  }
};

const renameBackup = async (serverId: string, backupId: string, newName: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/backups/${backupId}/rename`, {
      method: "POST",
      body: { name: newName },
    });
  } catch (error) {
    console.error("Error renaming backup:", error);
    throw error;
  }
};

const deleteBackup = async (serverId: string, backupId: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
  } catch (error) {
    console.error("Error deleting backup:", error);
    throw error;
  }
};

const restoreBackup = async (serverId: string, backupId: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/backups/${backupId}/restore`, {
      method: "POST",
    });
  } catch (error) {
    console.error("Error restoring backup:", error);
    throw error;
  }
};

const downloadBackup = async (serverId: string, backupId: string) => {
  try {
    return await usePyroFetch(`servers/${serverId}/backups/${backupId}`);
  } catch (error) {
    console.error("Error downloading backup:", error);
    throw error;
  }
};

// ------------------ NETWORK ------------------ //
const reserveAllocation = async (serverId: string, name: string): Promise<Allocation> => {
  try {
    return await usePyroFetch<Allocation>(`servers/${serverId}/allocations?name=${name}`, {
      method: "POST",
    });
  } catch (error) {
    console.error("Error reserving new allocation:", error);
    throw error;
  }
};

const updateAllocation = async (serverId: string, port: number, name: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/allocations/${port}?name=${name}`, {
      method: "PUT",
    });
  } catch (error) {
    console.error("Error updating allocations:", error);
    throw error;
  }
};

const deleteAllocation = async (serverId: string, port: number) => {
  try {
    await usePyroFetch(`servers/${serverId}/allocations/${port}`, {
      method: "DELETE",
    });
  } catch (error) {
    console.error("Error deleting allocation:", error);
    throw error;
  }
};

const checkSubdomainAvailability = async (subdomain: string): Promise<{ available: boolean }> => {
  try {
    return (await usePyroFetch(`subdomains/${subdomain}/isavailable`)) as { available: boolean };
  } catch (error) {
    console.error("Error checking subdomain availability:", error);
    throw error;
  }
};

const changeSubdomain = async (serverId: string, subdomain: string) => {
  try {
    await usePyroFetch(`servers/${serverId}/subdomain`, {
      method: "POST",
      body: { subdomain },
    });
  } catch (error) {
    console.error("Error changing subdomain:", error);
    throw error;
  }
};

const modules: any = {
  general: {
    get: async (serverId: string) => {
      const data = await usePyroFetch<General>(`servers/${serverId}`);
      data.project = await fetchProject(data.upstream?.project_id);
      return data;
    },
    updateName,
    power: sendPowerAction,
    reinstall: reinstallServer,
    suspend: suspendServer,
  },
  mods: {
    get: async (serverId: string) => await usePyroFetch<Mod[]>(`servers/${serverId}/mods`),
  },
  backups: {
    get: async (serverId: string) => {
      return { data: await usePyroFetch<Backup[]>(`servers/${serverId}/backups`) };
    },
    create: createBackup,
    rename: renameBackup,
    delete: deleteBackup,
    restore: restoreBackup,
    download: downloadBackup,
  },
  network: {
    get: async (serverId: string) => {
      return { allocations: await usePyroFetch<Allocation[]>(`servers/${serverId}/allocations`) };
    },
    reserveAllocation,
    updateAllocation,
    deleteAllocation,
    checkSubdomainAvailability,
    changeSubdomain,
  },
  startup: {
    get: async (serverId: string) => await usePyroFetch<Startup>(`servers/${serverId}/startup`),
  },
  ws: {
    get: async (serverId: string) => await usePyroFetch<WSAuth>(`servers/${serverId}/ws`),
  },
  fs: {
    get: async (serverId: string) => await usePyroFetch<FSAuth>(`servers/${serverId}/fs`),
  },
};

type GeneralFunctions = {
  /**
   * Updates the name of the server.
   * @param serverId - The ID of the server.
   * @param newName - The new name for the server.
   */
  updateName: (serverId: string, newName: string) => Promise<void>;

  /**
   * Sends a power action to the server.
   * @param serverId - The ID of the server.
   * @param action - The power action to send (e.g., "start", "stop", "restart").
   */
  power: (serverId: string, action: string) => Promise<void>;

  /**
   * Reinstalls the server with the specified project and version.
   * @param serverId - The ID of the server.
   * @param loader - Whether to use a loader.
   * @param projectId - The ID of the project.
   * @param versionId - Optional version ID.
   */
  reinstall: (
    serverId: string,
    loader: boolean,
    projectId: string,
    versionId?: string,
  ) => Promise<void>;

  /**
   * Suspends or resumes the server.
   * @param serverId - The ID of the server.
   * @param status - True to suspend the server, false to resume.
   */
  suspend: (serverId: string, status: boolean) => Promise<void>;
};

type BackupFunctions = {
  /**
   * Creates a new backup for the server.
   * @param serverId - The ID of the server.
   * @param backupName - The name of the backup.
   */
  create: (serverId: string, backupName: string) => Promise<void>;

  /**
   * Renames a backup for the server.
   * @param serverId - The ID of the server.
   * @param backupId - The ID of the backup.
   * @param newName - The new name for the backup.
   */
  rename: (serverId: string, backupId: string, newName: string) => Promise<void>;

  /**
   * Deletes a backup for the server.
   * @param serverId - The ID of the server.
   * @param backupId - The ID of the backup.
   */
  delete: (serverId: string, backupId: string) => Promise<void>;

  /**
   * Restores a backup for the server.
   * @param serverId - The ID of the server.
   * @param backupId - The ID of the backup.
   */
  restore: (serverId: string, backupId: string) => Promise<void>;

  /**
   * Downloads a backup for the server.
   * @param serverId - The ID of the server.
   * @param backupId - The ID of the backup.
   */
  download: (serverId: string, backupId: string) => Promise<void>;
};

type NetworkFunctions = {
  /**
   * Reserves a new allocation for the server.
   * @param serverId - The ID of the server.
   * @param name - The name of the allocation.
   * @returns The allocated network port details.
   */
  reserveAllocation: (serverId: string, name: string) => Promise<Allocation>;

  /**
   * Updates the allocation for the server.
   * @param serverId - The ID of the server.
   * @param port - The port to update.
   * @param name - The new name for the allocation.
   */
  updateAllocation: (serverId: string, port: number, name: string) => Promise<void>;

  /**
   * Deletes an allocation for the server.
   * @param serverId - The ID of the server.
   * @param port - The port to delete.
   */
  deleteAllocation: (serverId: string, port: number) => Promise<void>;

  /**
   * Checks if a subdomain is available.
   * @param subdomain - The subdomain to check.
   * @returns True if the subdomain is available, otherwise false.
   */
  checkSubdomainAvailability: (subdomain: string) => Promise<boolean>;

  /**
   * Changes the subdomain of the server.
   * @param serverId - The ID of the server.
   * @param subdomain - The new subdomain.
   */
  changeSubdomain: (serverId: string, subdomain: string) => Promise<void>;
};

type GeneralModule = General & GeneralFunctions;
type BackupsModule = { data: Backup[] } & BackupFunctions;
type NetworkModule = { allocations: Allocation[] } & NetworkFunctions;

type ModulesMap = {
  general: GeneralModule;
  mods: Mod[];
  backups: BackupsModule;
  network: NetworkModule;
  ws: WSAuth;
  fs: FSAuth;
};

export type Server<T extends ("general" | "mods" | "backups" | "network" | "ws" | "fs")[]> = {
  [K in T[number]]?: ModulesMap[K];
} & {
  refresh: () => void;
};

export const usePyroServer = async (
  serverId: string,
  includedModules: ("general" | "mods" | "backups" | "network" | "ws" | "fs")[],
) => {
  const server: Server<typeof includedModules> = reactive({
    /**
     * Refreshes the included modules of the server
     */
    refresh: async () => {
      for (const module of includedModules) {
        const mods = modules[module];
        if (mods.get) {
          const data = await mods.get(serverId);
          server[module] = { ...server[module], ...data };
        }
      }
    },
  });

  for (const module of includedModules) {
    const mods = modules[module];
    server[module] = mods;
  }

  await server.refresh();

  return server as Server<typeof includedModules>;
};
