import * as THREE from 'three'
import {Skin, Cape, normalize_skin_texture, get_normalized_skin_texture_url} from '../skins'
import { determineModelType } from '../skins'
import { reactive } from 'vue'
import { setupSkinModel, disposeCaches } from '@modrinth/utils'
import { skinPreviewStorage } from '../storage/skin-preview-storage'

export interface RenderResult {
  forwards: string
  backwards: string
}

class BatchSkinRenderer {
  private renderer: THREE.WebGLRenderer
  private readonly scene: THREE.Scene
  private readonly camera: THREE.PerspectiveCamera
  private currentModel: THREE.Group | null = null

  constructor(width: number = 360, height: number = 504) {
    const canvas = document.createElement('canvas')
    canvas.width = width
    canvas.height = height

    this.renderer = new THREE.WebGLRenderer({
      canvas: canvas,
      antialias: true,
      alpha: true,
      preserveDrawingBuffer: true,
    })

    this.renderer.outputColorSpace = THREE.SRGBColorSpace
    this.renderer.toneMapping = THREE.NoToneMapping
    this.renderer.setClearColor(0x000000, 0)
    this.renderer.setSize(width, height)

    this.scene = new THREE.Scene()
    this.camera = new THREE.PerspectiveCamera(20, width / height, 0.4, 1000)

    const ambientLight = new THREE.AmbientLight(0xffffff, 2)
    this.scene.add(ambientLight)
  }

  public async renderSkin(
    textureUrl: string,
    modelUrl: string,
    capeUrl?: string,
    capeModelUrl?: string,
  ): Promise<RenderResult> {
    await this.setupModel(modelUrl, textureUrl, capeModelUrl, capeUrl)

    const headPart = this.currentModel!.getObjectByName('Head')
    let lookAtTarget: [number, number, number]

    if (headPart) {
      const headPosition = new THREE.Vector3()
      headPart.getWorldPosition(headPosition)
      lookAtTarget = [headPosition.x, headPosition.y - 0.3, headPosition.z]
    } else {
      throw new Error("Failed to find 'Head' object in model.")
    }

    const frontCameraPos: [number, number, number] = [2, 1, -2.5]
    const backCameraPos: [number, number, number] = [2, 1, 6.0]

    const forwards = await this.renderView(frontCameraPos, lookAtTarget)
    const backwards = await this.renderView(backCameraPos, lookAtTarget)

    return { forwards, backwards }
  }

  private async renderView(
    cameraPosition: [number, number, number],
    lookAtPosition: [number, number, number],
  ): Promise<string> {
    this.camera.position.set(...cameraPosition)
    this.camera.lookAt(...lookAtPosition)

    this.renderer.render(this.scene, this.camera)

    return new Promise<string>((resolve, reject) => {
      this.renderer.domElement.toBlob((blob) => {
        if (blob) {
          const url = URL.createObjectURL(blob)
          resolve(url)
        } else {
          reject(new Error('Failed to create blob from canvas'))
        }
      }, 'image/png')
    })
  }

  private async setupModel(
    modelUrl: string,
    textureUrl: string,
    capeModelUrl?: string,
    capeUrl?: string,
  ): Promise<void> {
    if (this.currentModel) {
      this.scene.remove(this.currentModel)
    }

    const { model } = await setupSkinModel(modelUrl, textureUrl, capeModelUrl, capeUrl)

    const group = new THREE.Group()
    group.add(model)
    group.position.set(0, 0.3, 1.95)
    group.scale.set(0.8, 0.8, 0.8)

    this.scene.add(group)
    this.currentModel = group
  }

  public dispose(): void {
    this.renderer.dispose()
    disposeCaches()
  }
}

function getModelUrlForVariant(variant: string): string {
  switch (variant) {
    case 'SLIM':
      return '/src/assets/models/slim_player.gltf'
    case 'CLASSIC':
    case 'UNKNOWN':
    default:
      return '/src/assets/models/classic_player.gltf'
  }
}

export const map = reactive(new Map<string, RenderResult>())
const DEBUG_MODE = false

export async function cleanupUnusedPreviews(skins: Skin[]): Promise<void> {
  const validKeys = new Set<string>()

  for (const skin of skins) {
    const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
    validKeys.add(key)
  }

  try {
    await skinPreviewStorage.cleanupInvalidKeys(validKeys)
  } catch (error) {
    console.warn('Failed to cleanup unused skin previews:', error)
  }
}

export async function generateSkinPreviews(skins: Skin[], capes: Cape[]): Promise<void> {
  const renderer = new BatchSkinRenderer()
  const capeModelUrl = '/src/assets/models/cape.gltf'

  try {
    for (const skin of skins) {
      const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`

      if (map.has(key)) {
        if (DEBUG_MODE) {
          const result = map.get(key)!
          URL.revokeObjectURL(result.forwards)
          URL.revokeObjectURL(result.backwards)
          map.delete(key)
        } else continue
      }

      try {
        const cached = await skinPreviewStorage.retrieve(key)
        if (cached) {
          map.set(key, cached)
          continue
        }
      } catch (error) {
        console.warn('Failed to retrieve cached skin preview:', error)
      }

      let variant = skin.variant
      if (variant === 'UNKNOWN') {
        try {
          variant = await determineModelType(skin.texture)
        } catch (error) {
          console.error(`Failed to determine model type for skin ${key}:`, error)
          variant = 'CLASSIC'
        }
      }

      const modelUrl = getModelUrlForVariant(variant)
      const cape: Cape | undefined = capes.find((_cape) => _cape.id === skin.cape_id)
      const renderResult = await renderer.renderSkin(
        await get_normalized_skin_texture_url(skin),
        modelUrl,
        cape?.texture,
        capeModelUrl,
      )

      map.set(key, renderResult)

      try {
        await skinPreviewStorage.store(key, renderResult)
      } catch (error) {
        console.warn('Failed to store skin preview in persistent storage:', error)
      }
    }
  } finally {
    renderer.dispose()
    await cleanupUnusedPreviews(skins)
  }
}
