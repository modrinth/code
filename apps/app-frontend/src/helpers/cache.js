import { invoke } from '@tauri-apps/api/core'

export async function get_project(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_project', { id, cacheBehaviour })
}

export async function get_project_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_project_many', { ids, cacheBehaviour })
}

export async function get_version(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_version', { id, cacheBehaviour })
}

export async function get_version_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_version_many', { ids, cacheBehaviour })
}

export async function get_user(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_user', { id, cacheBehaviour })
}

export async function get_user_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_user_many', { ids, cacheBehaviour })
}

export async function get_team(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_team', { id, cacheBehaviour })
}

export async function get_team_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_team_many', { ids, cacheBehaviour })
}

export async function get_organization(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_organization', { id, cacheBehaviour })
}

export async function get_organization_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_organization_many', { ids, cacheBehaviour })
}

export async function get_search_results(id, cacheBehaviour) {
  return await invoke('plugin:cache|get_search_results', { id, cacheBehaviour })
}

export async function get_search_results_many(ids, cacheBehaviour) {
  return await invoke('plugin:cache|get_search_results_many', { ids, cacheBehaviour })
}

export async function purge_cache_types(cacheTypes) {
  return await invoke('plugin:cache|purge_cache_types', { cacheTypes })
}
