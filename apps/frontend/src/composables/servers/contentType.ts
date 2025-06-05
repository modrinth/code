import {type ServerNotice, ServersError} from "@modrinth/utils";
import type {FilesystemOp, FSQueuedOp, WSBackupState, WSBackupTask} from "@modrinth/utils.ts";
import type { Project } from "@modrinth/utils";

export interface General {
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
  flows?: {
    intro?: boolean;
  };
}

export interface Allocation {
  port: number;
  name: string;
}

export interface Startup {
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

export interface JWTAuth {
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

export type ContentType = "mod" | "plugin";
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

  endIntro: () => Promise<void>;
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
export type avaliableModules = ("general" | "content" | "backups" | "network" | "startup" | "ws" | "fs")[];
