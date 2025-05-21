import * as THREE from 'three'
import type { GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js'
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js'
import type { Skin, Cape } from '../skins'
import { determineModelType } from '../skins'
import { reactive } from 'vue'

export interface RenderResult {
  forwards: string
  backwards: string
}

class BatchSkinRenderer {
  private renderer: THREE.WebGLRenderer
  private readonly scene: THREE.Scene
  private readonly camera: THREE.PerspectiveCamera
  private modelCache: Map<string, GLTF> = new Map()
  private textureCache: Map<string, THREE.Texture> = new Map()
  private currentModel: THREE.Group | null = null
  private capeModel: THREE.Object3D | null = null
  private bodyNode: THREE.Object3D | null = null
  private capeAttached: boolean = false

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

  private async loadModel(modelUrl: string): Promise<GLTF> {
    if (this.modelCache.has(modelUrl)) {
      return this.modelCache.get(modelUrl)!
    }

    const loader = new GLTFLoader()
    return new Promise<GLTF>((resolve, reject) => {
      loader.load(
        modelUrl,
        (gltf) => {
          this.modelCache.set(modelUrl, gltf)
          resolve(gltf)
        },
        undefined,
        reject,
      )
    })
  }

  private async loadTexture(textureUrl: string): Promise<THREE.Texture> {
    if (this.textureCache.has(textureUrl)) {
      return this.textureCache.get(textureUrl)!
    }

    return new Promise<THREE.Texture>((resolve) => {
      const textureLoader = new THREE.TextureLoader()
      textureLoader.load(textureUrl, (texture) => {
        texture.colorSpace = THREE.SRGBColorSpace
        texture.flipY = false
        texture.magFilter = THREE.NearestFilter
        texture.minFilter = THREE.NearestFilter

        this.textureCache.set(textureUrl, texture)
        resolve(texture)
      })
    })
  }

  private applyTexture(model: THREE.Object3D, texture: THREE.Texture): void {
    model.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh

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

  private applyCapeTexture(model: THREE.Object3D, texture: THREE.Texture): void {
    model.traverse((child) => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh
        const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]

        materials.forEach((mat: THREE.Material) => {
          if (mat instanceof THREE.MeshStandardMaterial) {
            mat.map = texture
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

  private attachCapeToBody(): void {
    if (!this.bodyNode || !this.capeModel || this.capeAttached) return

    if (this.capeModel.parent) {
      this.capeModel.parent.remove(this.capeModel)
    }

    this.capeModel.position.set(0, -1, -0.01)
    this.capeModel.rotation.set(0, -Math.PI / 2, 0)

    this.bodyNode.add(this.capeModel)
    this.capeAttached = true
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

    this.bodyNode = null
    this.capeAttached = false

    const [gltf, texture] = await Promise.all([
      this.loadModel(modelUrl),
      this.loadTexture(textureUrl),
    ])

    const model = gltf.scene.clone()
    this.applyTexture(model, texture)

    model.traverse((node) => {
      if (node.name === 'Body') {
        this.bodyNode = node
      }
    })

    const group = new THREE.Group()
    group.add(model)
    group.position.set(0, 0.3, 1.95)
    group.scale.set(0.8, 0.8, 0.8)

    this.scene.add(group)
    this.currentModel = group

    if (capeModelUrl && capeUrl) {
      const [capeGltf, capeTexture] = await Promise.all([
        this.loadModel(capeModelUrl),
        this.loadTexture(capeUrl),
      ])

      this.capeModel = capeGltf.scene.clone()
      this.applyCapeTexture(this.capeModel, capeTexture)

      if (this.bodyNode && this.capeModel) {
        this.attachCapeToBody()
      }
    }
  }

  public dispose(): void {
    Array.from(this.textureCache.values()).forEach((texture) => {
      texture.dispose()
    })

    this.renderer.dispose()
    this.textureCache.clear()
    this.modelCache.clear()
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
        skin.texture,
        modelUrl,
        cape?.texture,
        capeModelUrl,
      )

      map.set(key, renderResult)
    }
  } finally {
    renderer.dispose()
  }
}
