import { invoke } from '@tauri-apps/api/tauri'

export async function get_project(id) {
  return await invoke('plugin:cache|get_project', { id })
}

export async function get_project_many(ids) {
  return await invoke('plugin:cache|get_project_many', { ids })
}

export async function get_version(id) {
  return await invoke('plugin:cache|get_version', { id })
}

export async function get_version_many(ids) {
  return await invoke('plugin:cache|get_version_many', { ids })
}

export async function get_user(id) {
  return await invoke('plugin:cache|get_user', { id })
}

export async function get_user_many(ids) {
  return await invoke('plugin:cache|get_user_many', { ids })
}

export async function get_team(id) {
  return await invoke('plugin:cache|get_team', { id })
}

export async function get_team_many(ids) {
  return await invoke('plugin:cache|get_team_many', { ids })
}

export async function get_organization(id) {
  return await invoke('plugin:cache|get_organization', { id })
}

export async function get_organization_many(ids) {
  return await invoke('plugin:cache|get_organization_many', { ids })
}

export async function get_search_results(id) {
  return await invoke('plugin:cache|get_search_results', { id })
}

export async function get_search_results_many(ids) {
  return await invoke('plugin:cache|get_search_results_many', { ids })
}
