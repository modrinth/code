import { invoke } from '@tauri-apps/api/core'

export interface Cape {
  id: string
  name: string
  texture: string
  is_default: boolean
  is_equipped: boolean
}

export type SkinModel = 'Classic' | 'Slim' | 'Unknown'
export type SkinSource = 'Default' | 'CustomExternal' | 'Custom'

export interface Skin {
  texture_key: string
  name?: string
  variant: SkinModel
  cape_id?: string
  texture: string
  source: SkinSource
  is_equipped: boolean
}

export async function get_available_capes(): Promise<Cape[]> {
  return await invoke('plugin:minecraft-skins|get_available_capes', {})
}

export async function get_available_skins(): Promise<Skin[]> {
  return await invoke('plugin:minecraft-skins|get_available_skins', {})
}

export async function add_and_equip_custom_skin(
  texture_blob: Uint8Array,
  variant: SkinModel,
  cape_override?: Cape,
): Promise<void> {
  await invoke('plugin:minecraft-skins|add_and_equip_custom_skin', {
    texture_blob,
    variant,
    cape_override,
  })
}

export async function set_default_cape(cape?: Cape): Promise<void> {
  await invoke('plugin:minecraft-skins|set_default_cape', {
    cape,
  })
}

export async function equip_skin(skin: Skin): Promise<void> {
  await invoke('plugin:minecraft-skins|equip_skin', {
    skin,
  })
}

export async function remove_custom_skin(skin: Skin): Promise<void> {
  await invoke('plugin:minecraft-skins|remove_custom_skin', {
    skin,
  })
}

export async function unequip_skin(): Promise<void> {
  await invoke('plugin:minecraft-skins|unequip_skin')
}
