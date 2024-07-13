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
  net: {
    ip: string;
    port: number;
    domain: string;
  };
  modpack: string;
  game: string;
  loader: string;
  version: string;
  mods: Mod[];
}

export interface Servers {
  servers: Server[];
}

export interface Stats {
  cpu_percent: number;
  ram_usage_bytes: number;
  ram_total_bytes: number;
  storage_usage_bytes: number;
  storage_total_bytes: number;
}
