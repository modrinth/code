import { invoke } from '@tauri-apps/api/core'
import {handleError} from "@/store/notifications";

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

export const DEFAULT_MODEL_SORTING = ['Steve', 'Alex'] as string[]

export const DEFAULT_MODELS: Record<string, SkinModel> = {
  Steve: 'Classic',
  Alex: 'Slim',
  Zuri: 'Classic',
  Sunny: 'Classic',
  Noor: 'Slim',
  Makena: 'Slim',
  Kai: 'Classic',
  Efe: 'Slim',
  Ari: 'Classic',
}

export function filterSavedSkins(list: Skin[]) {
  const customSkins = list.filter((s) => s.source !== 'Default');
  console.log(customSkins[0]);
  fixUnknownSkins(customSkins).catch(handleError);
  return customSkins;
}

export async function determineModelType(texture: string): Promise<'Slim' | 'Classic'> {
  return new Promise((resolve, reject) => {
    const canvas = document.createElement('canvas');
    const context = canvas.getContext('2d');

    if (!context) {
      return reject(new Error('Failed to create canvas rendering context.'));
    }

    const image = new Image();
    image.crossOrigin = 'anonymous';
    image.src = texture;

    image.onload = () => {
      canvas.width = image.width;
      canvas.height = image.height;

      context.drawImage(image, 0, 0);

      const armX = 44;
      const armY = 16;
      const armWidth = 4;
      const armHeight = 12;

      const imageData = context.getImageData(armX, armY, armWidth, armHeight).data;

      for (let y = 0; y < armHeight; y++) {
        const alphaIndex = (3 + y * armWidth) * 4 + 3;
        if (imageData[alphaIndex] !== 0) {
          resolve('Classic');
          return;
        }
      }

      canvas.remove();
      resolve('Slim');
    };

    image.onerror = () => {
      canvas.remove();
      reject(new Error('Failed to load the image.'));
    };
  });
}

export async function fixUnknownSkins(list: Skin[]) {
  const unknownSkins = list.filter((s) => s.variant === "Unknown");
  for (let unknownSkin of unknownSkins) {
    console.log(unknownSkin.texture);
    const modelType = await determineModelType(unknownSkin.texture);
    unknownSkin.variant = modelType;
  }
}

export function filterDefaultSkins(list: Skin[]) {
  console.log(list);
  return list
    .filter(
      (s) =>
        s.source === 'Default' &&
        (!s.name || s.variant === DEFAULT_MODELS[s.name]),
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
