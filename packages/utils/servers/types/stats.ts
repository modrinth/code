export interface Stats {
  current: {
    cpu_percent: number
    ram_usage_bytes: number
    ram_total_bytes: number
    storage_usage_bytes: number
    storage_total_bytes: number
  }
  past: {
    cpu_percent: number
    ram_usage_bytes: number
    ram_total_bytes: number
    storage_usage_bytes: number
    storage_total_bytes: number
  }
  graph: {
    cpu: number[]
    ram: number[]
  }
}
