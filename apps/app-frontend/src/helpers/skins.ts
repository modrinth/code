import { arrayBufferToBase64 } from '@modrinth/utils'
import { invoke } from '@tauri-apps/api/core'

export interface Cape {
	id: string
	name: string
	texture: string
	is_equipped: boolean
}

export type SkinModel = 'CLASSIC' | 'SLIM' | 'UNKNOWN'
export type SkinSource = 'default' | 'custom_external' | 'custom'

export interface Skin {
	texture_key: string
	name?: string
	section?: string
	variant: SkinModel
	cape_id?: string
	texture: string
	source: SkinSource
	is_equipped: boolean
}

export interface SkinTextureUrl {
	original: string
	normalized: string
}

export const DEFAULT_MODEL_SORTING = ['Steve', 'Alex'] as string[]

export const DEFAULT_MODELS: Record<string, SkinModel> = {
	Steve: 'CLASSIC',
	Alex: 'SLIM',
	Zuri: 'CLASSIC',
	Sunny: 'CLASSIC',
	Noor: 'SLIM',
	Makena: 'SLIM',
	Kai: 'CLASSIC',
	Efe: 'SLIM',
	Ari: 'CLASSIC',
}

export function filterSavedSkins(list: Skin[]) {
	const customSkins = list.filter((s) => s.source !== 'default')
	fixUnknownSkins(customSkins)
	return customSkins
}

export async function determineModelType(texture: string): Promise<'SLIM' | 'CLASSIC'> {
	return new Promise((resolve, reject) => {
		const canvas = document.createElement('canvas')
		const context = canvas.getContext('2d')

		if (!context) {
			return reject(new Error('Failed to create canvas rendering context.'))
		}

		const image = new Image()
		image.crossOrigin = 'anonymous'
		image.src = texture

		image.onload = () => {
			canvas.width = image.width
			canvas.height = image.height

			context.drawImage(image, 0, 0)

			const armX = 54
			const armY = 20
			const armWidth = 2
			const armHeight = 12
			const imageData = context.getImageData(armX, armY, armWidth, armHeight).data
			for (let alphaIndex = 3; alphaIndex < imageData.length; alphaIndex += 4) {
				if (imageData[alphaIndex] !== 0) {
					resolve('CLASSIC')
					return
				}
			}

			canvas.remove()
			resolve('SLIM')
		}

		image.onerror = () => {
			canvas.remove()
			reject(new Error('Failed to load the image.'))
		}
	})
}

export async function fixUnknownSkins(list: Skin[]) {
	const unknownSkins = list.filter((s) => s.variant === 'UNKNOWN')
	for (const unknownSkin of unknownSkins) {
		unknownSkin.variant = await determineModelType(unknownSkin.texture)
	}
}

export function filterDefaultSkins(list: Skin[]) {
	return list
		.filter(
			(s) =>
				s.source === 'default' &&
				(!s.name || !(s.name in DEFAULT_MODELS) || s.variant === DEFAULT_MODELS[s.name]),
		)
		.sort((a, b) => {
			const aIndex = a.name ? DEFAULT_MODEL_SORTING.indexOf(a.name) : -1
			const bIndex = b.name ? DEFAULT_MODEL_SORTING.indexOf(b.name) : -1
			return (aIndex === -1 ? Infinity : aIndex) - (bIndex === -1 ? Infinity : bIndex)
		})
}

export async function get_available_capes(): Promise<Cape[]> {
	return invoke('plugin:minecraft-skins|get_available_capes', {})
}

export async function get_available_skins(): Promise<Skin[]> {
	return invoke('plugin:minecraft-skins|get_available_skins', {})
}

export async function add_and_equip_custom_skin(
	textureBlob: Uint8Array,
	variant: SkinModel,
	cape?: Cape,
): Promise<void> {
	await invoke('plugin:minecraft-skins|add_and_equip_custom_skin', {
		textureBlob,
		variant,
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

export async function save_custom_skin(
	skin: Skin,
	textureBlob: Uint8Array,
	variant: SkinModel,
	cape: Cape | undefined,
	replaceTexture: boolean,
): Promise<Skin> {
	return await invoke('plugin:minecraft-skins|save_custom_skin', {
		skin,
		textureBlob,
		variant,
		cape,
		replaceTexture,
	})
}

export async function get_normalized_skin_texture(skin: Skin): Promise<string> {
	const data = await normalize_skin_texture(skin.texture)
	const base64 = arrayBufferToBase64(data)
	return `data:image/png;base64,${base64}`
}

export async function normalize_skin_texture(texture: Uint8Array | string): Promise<Uint8Array> {
	return await invoke('plugin:minecraft-skins|normalize_skin_texture', { texture })
}

export async function unequip_skin(): Promise<void> {
	await invoke('plugin:minecraft-skins|unequip_skin')
}

export async function flush_pending_skin_change(): Promise<void> {
	await invoke('plugin:minecraft-skins|flush_pending_skin_change')
}

export async function flush_pending_skin_change_for_profile(profileId: string): Promise<void> {
	await invoke('plugin:minecraft-skins|flush_pending_skin_change_for_profile', {
		profileId,
	})
}

export async function get_dragged_skin_data(path: string): Promise<Uint8Array> {
	const data = await invoke('plugin:minecraft-skins|get_dragged_skin_data', { path })
	return new Uint8Array(data)
}
