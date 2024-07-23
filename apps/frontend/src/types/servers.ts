export interface Mod {
  id: string;
  filename: string;
  modrinth_ids: {
    project_id: string;
    version_id: string;
  };
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
  game: string;
  loader: string | null;
  loader_version: string | null;
  mc_version: string | null;
  mods: Mod[];
}

export interface WSAuth {
  url: string;
  token: string;
}

export interface WSEvent {
  event: string;
  message: string;
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
  },
  past: {
    cpu_percent: number;
    ram_usage_bytes: number;
    ram_total_bytes: number;
    storage_usage_bytes: number;
    storage_total_bytes: number;
  },
  graph: {
    cpu: number[];
    ram: number[];
  }
}
