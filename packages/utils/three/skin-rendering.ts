import * as THREE from 'three'
import type { GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'

export interface SkinRendererConfig {
  textureColorSpace?: THREE.ColorSpace
  textureFlipY?: boolean
  textureMagFilter?: THREE.MagnificationTextureFilter
  textureMinFilter?: THREE.MinificationTextureFilter
}

// Private caches
const modelCache: Map<string, GLTF> = new Map()
const textureCache: Map<string, THREE.Texture> = new Map()

/**
 * Load and cache a GLTF model
 */
export async function loadModel(modelUrl: string): Promise<GLTF> {
  if (modelCache.has(modelUrl)) {
    return modelCache.get(modelUrl)!
  }

  const loader = new GLTFLoader()
  return new Promise<GLTF>((resolve, reject) => {
    loader.load(
      modelUrl,
      (gltf) => {
        modelCache.set(modelUrl, gltf)
        resolve(gltf)
      },
      undefined,
      reject,
    )
  })
}

/**
 * Load and configure a texture with skin-specific settings
 */
export async function loadTexture(
  textureUrl: string,
  config: SkinRendererConfig = {}
): Promise<THREE.Texture> {
  const cacheKey = `${textureUrl}_${JSON.stringify(config)}`

  if (textureCache.has(cacheKey)) {
    return textureCache.get(cacheKey)!
  }

  return new Promise<THREE.Texture>((resolve) => {
    const textureLoader = new THREE.TextureLoader()
    textureLoader.load(textureUrl, (texture) => {
      texture.colorSpace = config.textureColorSpace ?? THREE.SRGBColorSpace
      texture.flipY = config.textureFlipY ?? false
      texture.magFilter = config.textureMagFilter ?? THREE.NearestFilter
      texture.minFilter = config.textureMinFilter ?? THREE.NearestFilter

      textureCache.set(cacheKey, texture)
      resolve(texture)
    })
  })
}

/**
 * Apply skin texture to a model, excluding cape meshes
 */
export function applyTexture(model: THREE.Object3D, texture: THREE.Texture): void {
  model.traverse((child) => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh

      // Skip cape meshes
      if (mesh.name === 'Cape') return

      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

      materials.forEach((mat: THREE.Material) => {
        if (mat instanceof THREE.MeshStandardMaterial) {
          mat.map = texture
          mat.metalness = 0
          mat.color.set(0xffffff)
          mat.toneMapped = false
          mat.roughness = 1
          mat.needsUpdate = true
        }
      })
    }
  })
}

/**
 * Apply cape texture with specific settings for cape materials
 */
export function applyCapeTexture(
  model: THREE.Object3D,
  texture: THREE.Texture | null,
  transparentTexture?: THREE.Texture
): void {
  model.traverse((child) => {
    if ((child as THREE.Mesh).isMesh) {
      const mesh = child as THREE.Mesh
      const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

      materials.forEach((mat: THREE.Material) => {
        if (mat instanceof THREE.MeshStandardMaterial) {
          mat.map = texture || transparentTexture || null
          mat.transparent = transparentTexture ? true : false
          mat.metalness = 0
          mat.color.set(0xffffff)
          mat.toneMapped = false
          mat.roughness = 1
          mat.side = THREE.DoubleSide
          mat.needsUpdate = true
        }
      })
    }
  })
}

/**
 * Attach cape model to the body node with standard positioning
 */
export function attachCapeToBody(
  bodyNode: THREE.Object3D,
  capeModel: THREE.Object3D,
  position = { x: 0, y: -1, z: -0.01 },
  rotation = { x: 0, y: -Math.PI / 2, z: 0 }
): void {
  if (!bodyNode || !capeModel) return

  // Remove cape from current parent if it exists
  if (capeModel.parent) {
    capeModel.parent.remove(capeModel)
  }

  // Set cape position and rotation
  capeModel.position.set(position.x, position.y, position.z)
  capeModel.rotation.set(rotation.x, rotation.y, rotation.z)

  // Attach to body
  bodyNode.add(capeModel)
}

/**
 * Find the body node in a model hierarchy
 */
export function findBodyNode(model: THREE.Object3D): THREE.Object3D | null {
  let bodyNode: THREE.Object3D | null = null

  model.traverse((node) => {
    if (node.name === 'Body') {
      bodyNode = node
    }
  })

  return bodyNode
}

/**
 * Create a transparent texture for use as fallback
 */
export function createTransparentTexture(): THREE.Texture {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = 1
  const ctx = canvas.getContext('2d') as CanvasRenderingContext2D
  ctx.clearRect(0, 0, 1, 1)

  const texture = new THREE.CanvasTexture(canvas)
  texture.needsUpdate = true
  texture.colorSpace = THREE.SRGBColorSpace
  texture.flipY = false
  texture.magFilter = THREE.NearestFilter
  texture.minFilter = THREE.NearestFilter

  return texture
}

/**
 * Setup a complete skin model with optional cape
 */
export async function setupSkinModel(
  modelUrl: string,
  textureUrl: string,
  capeModelUrl?: string,
  capeTextureUrl?: string,
  config: SkinRendererConfig = {}
): Promise<{
  model: THREE.Object3D
  bodyNode: THREE.Object3D | null
  capeModel: THREE.Object3D | null
}> {
  // Load model and texture in parallel
  const [gltf, texture] = await Promise.all([
    loadModel(modelUrl),
    loadTexture(textureUrl, config),
  ])

  const model = gltf.scene.clone()
  applyTexture(model, texture)

  const bodyNode = findBodyNode(model)
  let capeModel: THREE.Object3D | null = null

  // Load cape if provided
  if (capeModelUrl && capeTextureUrl) {
    const [capeGltf, capeTexture] = await Promise.all([
      loadModel(capeModelUrl),
      loadTexture(capeTextureUrl, config),
    ])

    capeModel = capeGltf.scene.clone()
    applyCapeTexture(capeModel, capeTexture)

    if (bodyNode && capeModel) {
      attachCapeToBody(bodyNode, capeModel)
    }
  }

  return { model, bodyNode, capeModel }
}

/**
 * Clear all caches and dispose of cached resources
 */
export function disposeCaches(): void {
  Array.from(textureCache.values()).forEach((texture) => {
    texture.dispose()
  })

  textureCache.clear()
  modelCache.clear()
}
