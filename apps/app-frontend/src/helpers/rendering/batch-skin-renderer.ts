import * as THREE from 'three'
import type { Skin, Cape } from '../skins'
import { get_normalized_skin_texture, determineModelType } from '../skins'
import { reactive } from 'vue'
import { setupSkinModel, disposeCaches } from '@modrinth/utils'
import { skinPreviewStorage } from '../storage/skin-preview-storage'
import { CapeModel, ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'

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
    this.renderer.toneMappingExposure = 10.0
    this.renderer.setClearColor(0x000000, 0)
    this.renderer.setSize(width, height)

    this.scene = new THREE.Scene()
    this.camera = new THREE.PerspectiveCamera(20, width / height, 0.4, 1000)

    const ambientLight = new THREE.AmbientLight(0xffffff, 2)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2)
    directionalLight.castShadow = true
    directionalLight.position.set(2, 4, 3)
    this.scene.add(ambientLight)
    this.scene.add(directionalLight)
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

    const frontCameraPos: [number, number, number] = [-1.3, 1, 6.3]
    const backCameraPos: [number, number, number] = [-1.3, 1, -2.5]

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
      return SlimPlayerModel
    case 'CLASSIC':
    case 'UNKNOWN':
    default:
      return ClassicPlayerModel
  }
}

export const map = reactive(new Map<string, RenderResult>())
export const headMap = reactive(new Map<string, string>())
const DEBUG_MODE = false

export async function cleanupUnusedPreviews(skins: Skin[]): Promise<void> {
  const validKeys = new Set<string>()
  const validHeadKeys = new Set<string>()

  for (const skin of skins) {
    const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`
    const headKey = `${skin.texture_key}-head`
    validKeys.add(key)
    validHeadKeys.add(headKey)
  }

  try {
    await skinPreviewStorage.cleanupInvalidKeys(validKeys)
    await skinPreviewStorage.cleanupInvalidKeys(validHeadKeys)
  } catch (error) {
    console.warn('Failed to cleanup unused skin previews:', error)
  }
}

export async function generatePlayerHeadBlob(skinUrl: string, size: number = 64): Promise<Blob> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.crossOrigin = 'anonymous'

    img.onload = () => {
      try {
        const sourceCanvas = document.createElement('canvas')
        const sourceCtx = sourceCanvas.getContext('2d')

        if (!sourceCtx) {
          throw new Error('Could not get 2D context from source canvas')
        }

        sourceCanvas.width = img.width
        sourceCanvas.height = img.height

        sourceCtx.drawImage(img, 0, 0)

        const outputCanvas = document.createElement('canvas')
        const outputCtx = outputCanvas.getContext('2d')

        if (!outputCtx) {
          throw new Error('Could not get 2D context from output canvas')
        }

        outputCanvas.width = size
        outputCanvas.height = size

        outputCtx.imageSmoothingEnabled = false

        const headImageData = sourceCtx.getImageData(8, 8, 8, 8)

        const headCanvas = document.createElement('canvas')
        const headCtx = headCanvas.getContext('2d')

        if (!headCtx) {
          throw new Error('Could not get 2D context from head canvas')
        }

        headCanvas.width = 8
        headCanvas.height = 8
        headCtx.putImageData(headImageData, 0, 0)

        outputCtx.drawImage(headCanvas, 0, 0, 8, 8, 0, 0, size, size)

        const hatImageData = sourceCtx.getImageData(40, 8, 8, 8)

        const hatCanvas = document.createElement('canvas')
        const hatCtx = hatCanvas.getContext('2d')

        if (!hatCtx) {
          throw new Error('Could not get 2D context from hat canvas')
        }

        hatCanvas.width = 8
        hatCanvas.height = 8
        hatCtx.putImageData(hatImageData, 0, 0)

        const hatPixels = hatImageData.data
        let hasHat = false

        for (let i = 3; i < hatPixels.length; i += 4) {
          if (hatPixels[i] > 0) {
            hasHat = true
            break
          }
        }

        if (hasHat) {
          outputCtx.drawImage(hatCanvas, 0, 0, 8, 8, 0, 0, size, size)
        }

        outputCanvas.toBlob((blob) => {
          if (blob) {
            resolve(blob)
          } else {
            reject(new Error('Failed to create blob from canvas'))
          }
        }, 'image/png')
      } catch (error) {
        reject(error)
      }
    }

    img.onerror = () => {
      reject(new Error('Failed to load skin texture image'))
    }

    img.src = skinUrl
  })
}

async function generateHeadRender(skin: Skin): Promise<string> {
  const headKey = `${skin.texture_key}-head`

  if (headMap.has(headKey)) {
    if (DEBUG_MODE) {
      const url = headMap.get(headKey)!
      URL.revokeObjectURL(url)
      headMap.delete(headKey)
    } else {
      return headMap.get(headKey)!
    }
  }

  try {
    const cached = await skinPreviewStorage.retrieve(headKey)
    if (cached && typeof cached === 'string') {
      headMap.set(headKey, cached)
      return cached
    }
  } catch (error) {
    console.warn('Failed to retrieve cached head render:', error)
  }

  const skinUrl = await get_normalized_skin_texture(skin)
  const headBlob = await generatePlayerHeadBlob(skinUrl, 64)
  const headUrl = URL.createObjectURL(headBlob)

  headMap.set(headKey, headUrl)

  try {
    // @ts-expect-error - skinPreviewStorage.store expects a RenderResult, but we are storing a string url.
    await skinPreviewStorage.store(headKey, headUrl)
  } catch (error) {
    console.warn('Failed to store head render in persistent storage:', error)
  }

  return headUrl
}

export async function getPlayerHeadUrl(skin: Skin): Promise<string> {
  return await generateHeadRender(skin)
}

export async function generateSkinPreviews(skins: Skin[], capes: Cape[]): Promise<void> {
  const renderer = new BatchSkinRenderer()

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
        await get_normalized_skin_texture(skin),
        modelUrl,
        cape?.texture,
        CapeModel,
      )

      map.set(key, renderResult)

      try {
        await skinPreviewStorage.store(key, renderResult)
      } catch (error) {
        console.warn('Failed to store skin preview in persistent storage:', error)
      }

      await generateHeadRender(skin)
    }
  } finally {
    renderer.dispose()
    await cleanupUnusedPreviews(skins)
  }
}
