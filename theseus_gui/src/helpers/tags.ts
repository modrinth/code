/**
 * All theseus API calls return serialized values (both return values and errors);
 * So, for example, addDefaultInstance creates a blank Profile object, where the Rust struct is serialized,
 *  and deserialized into a usable JS object.
 */
import { invoke } from '@tauri-apps/api/tauri'

export interface Category {
  name: string
  project_type: string
  header: string
  icon: string
}

export interface Loader {
  name: string
  icon: string
  supported_project_types: string[]
}

export interface DonationPlatform {
  short: string
  name: string
}

export interface GameVersion {
  version: string
  version_type: string
  date: string
  major: boolean
}

export interface Tags {
  categories: Category[]
  loaders: Loader[]
  game_versions: GameVersion[]
  donation_platforms: DonationPlatform[]
  report_types: string[]
}

// Gets tag bundle of all tags
export async function get_tag_bundle(): Promise<Tags> {
  return await invoke('tags_get_tag_bundle')
}

// Gets cached category tags
export async function get_categories(): Promise<Category[]> {
  return await invoke('tags_get_categories')
}

// Gets cached loaders tags
export async function get_loaders(): Promise<Loader[]> {
  return await invoke('tags_get_loaders')
}

// Gets cached game_versions tags
export async function get_game_versions(): Promise<GameVersion[]> {
  return await invoke('tags_get_game_versions')
}

// Gets cached donation_platforms tags
export async function get_donation_platforms(): Promise<DonationPlatform[]> {
  return await invoke('tags_get_donation_platforms')
}

// Gets cached licenses tags
export async function get_report_types(): Promise<string[]> {
  return await invoke('tags_get_report_types')
}
