// export interface Mod {
//   id: string;
//   filename: string;
//   modrinth_ids: {
//     project_id: string;
//     version_id: string;
//   };
// }

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

export interface ServerBackup {
  id: string;
  name: string;
  created_at: string;
}

export interface Allocation {
  name: string;
  port: number;
}

export interface Server {
  server_id: string;
  name: string;
  status: string;
  net: {
    ip: string;
    port: number;
    domain: string;
    allocations: Allocation[];
  };
  game: string;
  loader: string | null;
  loader_version: string | null;
  mc_version: string | null;
  backup_quota: number;
  used_backup_quota: number;
  backups: ServerBackup[];
  mods: Mod[];
  project: Project | null;
  suspension_reason: string | null;
  image: string | null;
  upstream?: {
    kind: "modpack";
    project_id: string;
    version_id: string;
  };
  motd: string;
}

export interface Stats {
  current: {
    cpu_percent: number;
    ram_usage_bytes: number;
    ram_total_bytes: number;
    storage_usage_bytes: number;
    storage_total_bytes: number;
  };
  past: {
    cpu_percent: number;
    ram_usage_bytes: number;
    ram_total_bytes: number;
    storage_usage_bytes: number;
    storage_total_bytes: number;
  };
  graph: {
    cpu: number[];
    ram: number[];
  };
}

export interface WSAuth {
  url: string;
  token: string;
}

export type ServerState = "running" | "stopped" | "crashed";
// export type WebsocketEventType =
//   | "log"
//   | "auth"
//   | "stats"
//   | "power-state"
//   | "auth-expiring"
//   | "auth-incorrect"
//   | "installation-result"
//   | (string & {});

// export interface WSEvent {
//   event: WebsocketEventType;
//   message: string;
//   state: ServerState;
// }

export type Loaders =
  | "Fabric"
  | "Quilt"
  | "Forge"
  | "NeoForge"
  | "Paper"
  | "Spigot"
  | "Bukkit"
  | "Vanilla"
  | "Purpur";

export interface WSLogEvent {
  event: "log";
  message: string;
}

type CurrentStats = Stats["current"];

export interface WSStatsEvent extends CurrentStats {
  event: "stats";
}

export interface WSAuthExpiringEvent {
  event: "auth-expiring";
}

export interface WSPowerStateEvent {
  event: "power-state";
  state: ServerState;
  // if state "crashed"
  oom_killed?: boolean;
  exit_code?: number;
}

export interface WSAuthIncorrectEvent {
  event: "auth-incorrect";
}

export interface WSInstallationResultOkEvent {
  event: "installation-result";
  result: "ok";
}

export interface WSInstallationResultErrEvent {
  event: "installation-result";
  result: "err";
  reason: string;
}
export type WSInstallationResultEvent = WSInstallationResultOkEvent | WSInstallationResultErrEvent;

export interface WSAuthOkEvent {
  event: "auth-ok";
}

export interface WSUptimeEvent {
  event: "uptime";
  uptime: number; // seconds
}

export interface WSNewModEvent {
  event: "new-mod";
}

export type WSBackupTask = "file" | "create" | "restore";
export type WSBackupState = "ongoing" | "done" | "failed" | "cancelled" | "unchanged";

export interface WSBackupProgressEvent {
  event: "backup-progress";
  task: WSBackupTask;
  id: string;
  progress: number; // percentage
  state: WSBackupState;
  ready: boolean;
}

export type FSQueuedOpUnarchive = {
  op: "unarchive";
  src: string;
};

export type FSQueuedOp = FSQueuedOpUnarchive;

export type FSOpUnarchive = {
  op: "unarchive";
  progress: number; // Note: 1 does not mean it's done
  id: string; // UUID

  mime: string;
  src: string;
  state:
    | "queued"
    | "ongoing"
    | "cancelled"
    | "done"
    | "failed-corrupted"
    | "failed-invalid-path"
    | "failed-cf-no-serverpack"
    | "failed-cf-not-available"
    | "failed-not-reachable";

  current_file: string | null;
  failed_path?: string;
  bytes_processed: number;
  files_processed: number;
  started: string;
};

export type FilesystemOp = FSOpUnarchive;

export interface WSFilesystemOpsEvent {
  event: "filesystem-ops";
  all: FilesystemOp[];
}

export type WSEvent =
  | WSLogEvent
  | WSStatsEvent
  | WSPowerStateEvent
  | WSAuthExpiringEvent
  | WSAuthIncorrectEvent
  | WSInstallationResultEvent
  | WSAuthOkEvent
  | WSUptimeEvent
  | WSNewModEvent
  | WSBackupProgressEvent
  | WSFilesystemOpsEvent;

export interface Servers {
  servers: Server[];
}
