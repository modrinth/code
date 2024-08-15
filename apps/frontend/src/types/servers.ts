export interface Mod {
  id: string;
  filename: string;
  modrinth_ids: {
    project_id: string;
    version_id: string;
  };
}

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

export interface Server {
  server_id: string;
  name: string;
  state: string;
  net: {
    ip: string;
    port: number;
    domain: string;
  };
  modpack: string | null;
  project: Project | null;
  game: string;
  loader: string | null;
  loader_version: string | null;
  mc_version: string | null;
  mods: Mod[];
  backup_quota: number;
  used_backup_quota: number;
}

export interface WSAuth {
  url: string;
  token: string;
}

export type ServerState = "running" | "stopped" | "crashed";

export interface WSEvent {
  event: string;
  message: string;
  state: ServerState;
}

export interface Servers {
  servers: Server[];
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

export interface ServerBackup {
  id: string;
  name: string;
  created_at: string;
}
