// am i winning? :smilew:

// very wip, but it works i know stuff is missing and broken, dont worry, i'll fix it
const internalServerRefrence = ref<any>(null);
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
  } | null;
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
  const auth = await await usePyroFetch<FSAuth>(
    `servers/${internalServerRefrence.value.serverId}/fs`,
  );
  try {
    const fileData = await usePyroFetch(`/download?path=/server-icon-original.png`, {
      override: auth,
    });

    if (fileData instanceof Blob) {
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
  } catch (error) {
    console.error("Error processing server image:", error);
  }

  if (image.value === null && iconUrl) {
    console.log("iconUrl", iconUrl);
    try {
      const response = await fetch(iconUrl);
      const file = await response.blob();
      const originalfile = new File([file], "server-icon-original.png", {
        type: "image/png",
      });
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
        await usePyroFetch(`/create?path=/server-icon.png&type=file`, {
          method: "POST",
          contentType: "application/octet-stream",
          body: scaledFile,
          override: auth,
        });

        await usePyroFetch(`/create?path=/server-icon-original.png&type=file`, {
          method: "POST",
          contentType: "application/octet-stream",
          body: originalfile,
          override: auth,
        });
      }
    } catch (error) {
      console.error("Error processing server image:", error);
    }
  }
  return image.value;
};

// ------------------ GENERAL ------------------ //

const sendPowerAction = async (action: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/power`, {
      method: "POST",
      body: { action },
    });
  } catch (error) {
    console.error("Error changing power state:", error);
    throw error;
  }
};

const updateName = async (newName: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/name`, {
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

// lord forgive me, for i have sinned and should not be forgiven -maddie
const reinstallServer = async (
  serverId: string,
  loader: boolean,
  projectId: string,
  versionId?: string,
  loaderVersionId?: string,
) => {
  // launcher-meta.modrinth.com/{forge,neo,minecraft,quilt,fabric}/v0/manifest.json
  try {
    if (loader) {
      await usePyroFetch(`servers/${serverId}/reinstall`, {
        method: "POST",
        body: { loader: projectId, loader_version: loaderVersionId, game_version: versionId },
      });
    } else {
      await usePyroFetch(`servers/${serverId}/reinstall`, {
        method: "POST",
        body: { project_id: projectId, version_id: versionId },
      });
    }
  } catch (error) {
    console.error("Error reinstalling server:", error);
    throw error;
  }
};

const suspendServer = async (status: boolean) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/suspend`, {
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
    return await usePyroFetch(
      `servers/${internalServerRefrence.value.serverId}/config/${fileName}`,
    );
  } catch (error) {
    console.error("Error fetching config file:", error);
    throw error;
  }
};

const getMotd = async () => {
  try {
    const props = (await fetchConfigFile("ServerProperties")) as any;
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
      const auth = await await usePyroFetch<FSAuth>(
        `servers/${internalServerRefrence.value.serverId}/fs`,
      );

      return await usePyroFetch(`/update?path=/server.properties`, {
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

// ------------------ MODS ------------------ //

const installMod = async (projectId: string, versionId: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/mods`, {
      method: "POST",
      body: { rinth_ids: { project_id: projectId, version_id: versionId } },
    });
  } catch (error) {
    console.error("Error installing mod:", error);
    throw error;
  }
};

const removeMod = async (modId: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/deleteMod`, {
      method: "POST",
      body: {
        path: modId,
      },
    });
  } catch (error) {
    console.error("Error removing mod:", error);
    throw error;
  }
};

const reinstallMod = async (modId: string, versionId: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/mods/${modId}`, {
      method: "PUT",
      body: { version_id: versionId },
    });
  } catch (error) {
    console.error("Error reinstalling mod:", error);
    throw error;
  }
};

// ------------------ BACKUPS ------------------ //

const createBackup = async (backupName: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/backups`, {
      method: "POST",
      body: { name: backupName },
    });
  } catch (error) {
    console.error("Error creating backup:", error);
    throw error;
  }
};

const renameBackup = async (backupId: string, newName: string) => {
  try {
    await usePyroFetch(
      `servers/${internalServerRefrence.value.serverId}/backups/${backupId}/rename`,
      {
        method: "POST",
        body: { name: newName },
      },
    );
  } catch (error) {
    console.error("Error renaming backup:", error);
    throw error;
  }
};

const deleteBackup = async (backupId: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/backups/${backupId}`, {
      method: "DELETE",
    });
  } catch (error) {
    console.error("Error deleting backup:", error);
    throw error;
  }
};

const restoreBackup = async (backupId: string) => {
  try {
    await usePyroFetch(
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
    return await usePyroFetch(
      `servers/${internalServerRefrence.value.serverId}/backups/${backupId}`,
    );
  } catch (error) {
    console.error("Error downloading backup:", error);
    throw error;
  }
};

// ------------------ NETWORK ------------------ //
const reserveAllocation = async (name: string): Promise<Allocation> => {
  try {
    return await usePyroFetch<Allocation>(
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
    await usePyroFetch(
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
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/allocations/${port}`, {
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

const changeSubdomain = async (subdomain: string) => {
  try {
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/subdomain`, {
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
    await usePyroFetch(`servers/${internalServerRefrence.value.serverId}/startup`, {
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

const refreshFileApiInfo = () => {
  internalServerRefrence.value.refresh(["fs"]);
};

const retryWithAuth = async (requestFn: () => Promise<any>) => {
  try {
    return await requestFn();
  } catch {
    await refreshFileApiInfo();
    return await requestFn();
  }
};

const listDirContents = (path: string, page: number, pageSize: number) => {
  return retryWithAuth(async () => {
    return await usePyroFetch(`/list?path=${path}&page=${page}&page_size=${pageSize}`, {
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const createFileOrFolder = (
  path: string,
  // name: string,
  type: "file" | "directory",
) => {
  return retryWithAuth(async () => {
    return await usePyroFetch(`/create?path=${path}&type=${type}`, {
      method: "POST",
      contentType: "application/octet-stream",
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const uploadFile = (path: string, file: File) => {
  return retryWithAuth(async () => {
    return await usePyroFetch(`/create?path=${path}&type=file`, {
      method: "POST",
      contentType: "application/octet-stream",
      body: file,
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const renameFileOrFolder = (path: string, name: string) => {
  const pathName = path.split("/").slice(0, -1).join("/") + "/" + name;
  return retryWithAuth(async () => {
    return await usePyroFetch(`/move`, {
      method: "POST",
      override: internalServerRefrence.value.fs.auth,
      body: {
        source: path,
        destination: pathName,
      },
    });
  });
};

const updateFile = (path: string, content: string) => {
  const octetStream = new Blob([content], { type: "application/octet-stream" });
  return retryWithAuth(async () => {
    return await usePyroFetch(`/update?path=${path}`, {
      method: "PUT",
      contentType: "application/octet-stream",
      body: octetStream,
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const moveFileOrFolder = (path: string, newPath: string) => {
  return retryWithAuth(async () => {
    return await usePyroFetch(`/move`, {
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
  return retryWithAuth(async () => {
    return await usePyroFetch(`/delete?path=${path}&recursive=${recursive}`, {
      method: "DELETE",
      override: internalServerRefrence.value.fs.auth,
    });
  });
};

const downloadFile = (path: string) => {
  return retryWithAuth(async () => {
    const fileData = await usePyroFetch(`/download?path=${path}`, {
      override: internalServerRefrence.value.fs.auth,
    });

    if (fileData instanceof Blob) {
      return fileData.text();
    }
  });
};

const modules: any = {
  general: {
    get: async (serverId: string) => {
      const data = await usePyroFetch<General>(`servers/${serverId}`);
      if (data.upstream?.project_id) {
        data.project = await fetchProject(data.upstream.project_id);
      }
      data.image = (await processImage(data.project?.icon_url)) ?? undefined;
      return data;
    },
    updateName,
    power: sendPowerAction,
    reinstall: reinstallServer,
    suspend: suspendServer,
    getMotd,
    setMotd,
    fetchConfigFile,
  },
  mods: {
    get: async (serverId: string) => {
      const mods = await usePyroFetch<Mod[]>(`servers/${serverId}/mods`);
      return { data: mods.sort((a, b) => (a?.name ?? "").localeCompare(b?.name ?? "")) };
    },
    install: installMod,
    remove: removeMod,
    reinstall: reinstallMod,
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
    update: updateStartupSettings,
  },
  ws: {
    get: async (serverId: string) => await usePyroFetch<WSAuth>(`servers/${serverId}/ws`),
  },
  fs: {
    get: async (serverId: string) => {
      return { auth: await usePyroFetch<FSAuth>(`servers/${serverId}/fs`) };
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
   */
  reinstall: (
    serverId: string,
    loader: boolean,
    projectId: string,
    versionId?: string,
    loaderVersionId?: string,
  ) => Promise<void>;

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

type ModFunctions = {
  /**
   * INTERNAL: Gets the mods of a server.
   * @param serverId - The ID of the server.
   * @returns
   */
  get: (serverId: string) => Promise<Mod[]>;

  /**
   * Installs a mod to a server.
   * @param projectId - The ID of the project.
   * @param versionId - The ID of the version.
   */
  install: (projectId: string, versionId: string) => Promise<void>;

  /**
   * Removes a mod from a server.
   * @param modId - The ID of the mod.
   */
  remove: (modId: string) => Promise<void>;

  /**
   * Reinstalls a mod to a server.
   * @param modId - The ID of the mod.
   * @param versionId - The ID of the version.
   */
  reinstall: (modId: string, versionId: string) => Promise<void>;
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
  get: (serverId: string) => Promise<FSAuth>;

  /**
   * INTERNAL: Lists the contents of a directory.
   * @param path
   * @param page
   * @param pageSize
   * @returns
   */
  listDirContents: (path: string, page: number, pageSize: number) => Promise<any>;

  /**
   * INTERNAL: Creates a file or folder.
   * @param path
   * @param type
   * @returns
   */
  createFileOrFolder: (path: string, type: "file" | "directory") => Promise<any>;

  /**
   * INTERNAL: Uploads a file.
   * @param path
   * @param file
   * @returns
   */
  uploadFile: (path: string, file: File) => Promise<any>;

  /**
   * INTERNAL: Renames a file or folder.
   * @param path
   * @param name
   * @returns
   */
  renameFileOrFolder: (path: string, name: string) => Promise<any>;

  /**
   * INTERNAL: Updates a file.
   * @param path
   * @param content
   * @returns
   */
  updateFile: (path: string, content: string) => Promise<any>;

  /**
   * INTERNAL: Moves a file or folder.
   * @param path
   * @param newPath
   * @returns
   */
  moveFileOrFolder: (path: string, newPath: string) => Promise<any>;

  /**
   * INTERNAL: Deletes a file or folder.
   * @param path
   * @param recursive
   * @returns
   */
  deleteFileOrFolder: (path: string, recursive: boolean) => Promise<any>;

  /**
   * INTERNAL: Downloads a file.
   * @param serverId
   * @param path
   * @returns
   */
  downloadFile: (path: string) => Promise<any>;
};

type GeneralModule = General & GeneralFunctions;
type ModsModule = { data: Mod[] } & ModFunctions;
type BackupsModule = { data: Backup[] } & BackupFunctions;
type NetworkModule = { allocations: Allocation[] } & NetworkFunctions;
type StartupModule = Startup & StartupFunctions;
type FSModule = { auth: FSAuth } & FSFunctions;

type ModulesMap = {
  general: GeneralModule;
  mods: ModsModule;
  backups: BackupsModule;
  network: NetworkModule;
  startup: StartupModule;
  ws: WSAuth;
  fs: FSModule;
};

type avaliableModules = ("general" | "mods" | "backups" | "network" | "startup" | "ws" | "fs")[];

export type Server<T extends avaliableModules> = {
  [K in T[number]]?: ModulesMap[K];
} & {
  /**
   * Refreshes the included modules of the server
   * @param refreshModules - The modules to refresh.
   */
  refresh: (refreshModules?: avaliableModules) => Promise<void>;
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
