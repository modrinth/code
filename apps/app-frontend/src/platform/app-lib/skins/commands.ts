import type { SkinEarsData } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'

import type { Skin } from '@/helpers/skins'

/** Returns the asset URL of the skin's full-body preview, rendering it if it is not cached. */
export function renderSkinPreview(skin: Skin, capeTexture?: string): Promise<string> {
	return invoke('plugin:minecraft-skins|render_skin_preview', { skin, capeTexture })
}

/** Returns the asset URL of the skin's head image, rendering it if it is not cached. */
export function renderSkinHead(skin: Skin): Promise<string> {
	return invoke('plugin:minecraft-skins|render_skin_head', { skin })
}

/** Resolves to `null` if the texture has no Ears data. */
export function getSkinEars(texture: string, slim: boolean): Promise<SkinEarsData | null> {
	return invoke('plugin:minecraft-skins|get_skin_ears', { texture, slim })
}
