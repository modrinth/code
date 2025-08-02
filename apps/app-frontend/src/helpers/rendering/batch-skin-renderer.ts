import * as THREE from 'three'
import type { Skin, Cape } from '../skins'
import { get_normalized_skin_texture, determineModelType } from '../skins'
import { reactive } from 'vue'
import {
  setupSkinModel,
  disposeCaches,
  loadTexture,
  applyCapeTexture,
  createTransparentTexture,
} from '@modrinth/utils'
import { skinPreviewStorage } from '../storage/skin-preview-storage'
import { headStorage } from '../storage/head-storage'
import { ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'

export interface RenderResult {
  forwards: string
  backwards: string
}

export interface RawRenderResult {
  forwards: Blob
  backwards: Blob
}

class BatchSkinRenderer {
  private renderer: THREE.WebGLRenderer | null = null
  private scene: THREE.Scene | null = null
  private camera: THREE.PerspectiveCamera | null = null
  private currentModel: THREE.Group | null = null
  private readonly width: number
  private readonly height: number

  constructor(width: number = 360, height: number = 504) {
    this.width = width
    this.height = height
  }

  private initializeRenderer(): void {
    if (this.renderer) return

    const canvas = document.createElement('canvas')
    canvas.width = this.width
    canvas.height = this.height

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
    this.renderer.setSize(this.width, this.height)

    this.scene = new THREE.Scene()
    this.camera = new THREE.PerspectiveCamera(20, this.width / this.height, 0.4, 1000)

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
  ): Promise<RawRenderResult> {
    this.initializeRenderer()

    this.clearScene()

    await this.setupModel(modelUrl, textureUrl, capeUrl)

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
  ): Promise<Blob> {
    if (!this.camera || !this.renderer || !this.scene) {
      throw new Error('Renderer not initialized')
    }

    this.camera.position.set(...cameraPosition)
    this.camera.lookAt(...lookAtPosition)

    this.renderer.render(this.scene, this.camera)

    const dataUrl = this.renderer.domElement.toDataURL('image/webp', 0.9)
    const response = await fetch(dataUrl)
    return await response.blob()
  }

  private async setupModel(modelUrl: string, textureUrl: string, capeUrl?: string): Promise<void> {
    if (!this.scene) {
      throw new Error('Renderer not initialized')
    }

    const { model } = await setupSkinModel(modelUrl, textureUrl)

    if (capeUrl) {
      const capeTexture = await loadTexture(capeUrl)
      applyCapeTexture(model, capeTexture)
    } else {
      const transparentTexture = createTransparentTexture()
      applyCapeTexture(model, null, transparentTexture)
    }

    const group = new THREE.Group()
    group.add(model)
    group.position.set(0, 0.3, 1.95)
    group.scale.set(0.8, 0.8, 0.8)

    this.scene.add(group)
    this.currentModel = group
  }

  private clearScene(): void {
    if (!this.scene) return

    while (this.scene.children.length > 0) {
      const child = this.scene.children[0]
      this.scene.remove(child)

      if (child instanceof THREE.Mesh) {
        if (child.geometry) child.geometry.dispose()
        if (child.material) {
          if (Array.isArray(child.material)) {
            child.material.forEach((material) => material.dispose())
          } else {
            child.material.dispose()
          }
        }
      }
    }

    const ambientLight = new THREE.AmbientLight(0xffffff, 2)
    const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2)
    directionalLight.castShadow = true
    directionalLight.position.set(2, 4, 3)
    this.scene.add(ambientLight)
    this.scene.add(directionalLight)

    this.currentModel = null
  }

  public dispose(): void {
    if (this.renderer) {
      this.renderer.dispose()
    }
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

export const skinBlobUrlMap = reactive(new Map<string, RenderResult>())
export const headBlobUrlMap = reactive(new Map<string, string>())
const DEBUG_MODE = false

let sharedRenderer: BatchSkinRenderer | null = null
function getSharedRenderer(): BatchSkinRenderer {
  if (!sharedRenderer) {
    sharedRenderer = new BatchSkinRenderer()
  }
  return sharedRenderer
}

export function disposeSharedRenderer(): void {
  if (sharedRenderer) {
    sharedRenderer.dispose()
    sharedRenderer = null
  }
}

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
    await headStorage.cleanupInvalidKeys(validHeadKeys)
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

        outputCanvas.toBlob(
          (blob) => {
            if (blob) {
              resolve(blob)
            } else {
              reject(new Error('Failed to create blob from canvas'))
            }
          },
          'image/webp',
          0.9,
        )
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

  if (headBlobUrlMap.has(headKey)) {
    if (DEBUG_MODE) {
      const url = headBlobUrlMap.get(headKey)!
      URL.revokeObjectURL(url)
      headBlobUrlMap.delete(headKey)
    } else {
      return headBlobUrlMap.get(headKey)!
    }
  }

  const skinUrl = await get_normalized_skin_texture(skin)
  const headBlob = await generatePlayerHeadBlob(skinUrl, 64)
  const headUrl = URL.createObjectURL(headBlob)

  headBlobUrlMap.set(headKey, headUrl)

  try {
    await headStorage.store(headKey, headBlob)
  } catch (error) {
    console.warn('Failed to store head render in persistent storage:', error)
  }

  return headUrl
}

export async function getPlayerHeadUrl(skin: Skin): Promise<string> {
  return await generateHeadRender(skin)
}

export async function generateSkinPreviews(skins: Skin[], capes: Cape[]): Promise<void> {
  try {
    const skinKeys = skins.map(
      (skin) => `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`,
    )
    const headKeys = skins.map((skin) => `${skin.texture_key}-head`)

    const [cachedSkinPreviews, cachedHeadPreviews] = await Promise.all([
      skinPreviewStorage.batchRetrieve(skinKeys),
      headStorage.batchRetrieve(headKeys),
    ])

    for (let i = 0; i < skins.length; i++) {
      const skinKey = skinKeys[i]
      const headKey = headKeys[i]

      const rawCached = cachedSkinPreviews[skinKey]
      if (rawCached) {
        const cached: RenderResult = {
          forwards: URL.createObjectURL(rawCached.forwards),
          backwards: URL.createObjectURL(rawCached.backwards),
        }
        skinBlobUrlMap.set(skinKey, cached)
      }

      const cachedHead = cachedHeadPreviews[headKey]
      if (cachedHead) {
        headBlobUrlMap.set(headKey, URL.createObjectURL(cachedHead))
      }
    }

    for (const skin of skins) {
      const key = `${skin.texture_key}+${skin.variant}+${skin.cape_id ?? 'no-cape'}`

      if (skinBlobUrlMap.has(key)) {
        if (DEBUG_MODE) {
          const result = skinBlobUrlMap.get(key)!
          URL.revokeObjectURL(result.forwards)
          URL.revokeObjectURL(result.backwards)
          skinBlobUrlMap.delete(key)
        } else continue
      }

      const renderer = getSharedRenderer()

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
      const rawRenderResult = await renderer.renderSkin(
        await get_normalized_skin_texture(skin),
        modelUrl,
        cape?.texture,
      )

      const renderResult: RenderResult = {
        forwards: URL.createObjectURL(rawRenderResult.forwards),
        backwards: URL.createObjectURL(rawRenderResult.backwards),
      }

      skinBlobUrlMap.set(key, renderResult)

      try {
        await skinPreviewStorage.store(key, rawRenderResult)
      } catch (error) {
        console.warn('Failed to store skin preview in persistent storage:', error)
      }

      const headKey = `${skin.texture_key}-head`
      if (!headBlobUrlMap.has(headKey)) {
        await generateHeadRender(skin)
      }
    }
  } finally {
    disposeSharedRenderer()
    await cleanupUnusedPreviews(skins)

    await skinPreviewStorage.debugCalculateStorage()
    await headStorage.debugCalculateStorage()
  }
}
