import * as THREE from 'three';
import { GLTFLoader, GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';
import {Skin, determineModelType, get_available_skins} from '../skins';
import {reactive} from "vue";

interface RenderResult {
  forwards: string;
  backwards: string;
}

class BatchSkinRenderer {
  private renderer: THREE.WebGLRenderer;
  private readonly scene: THREE.Scene;
  private readonly camera: THREE.PerspectiveCamera;
  private modelCache: Map<string, GLTF> = new Map();
  private textureCache: Map<string, THREE.Texture> = new Map();
  private readonly width: number;
  private readonly height: number;
  private currentModel: THREE.Group | null = null;

  constructor(width: number = 215, height: number = 645) {
    this.width = width;
    this.height = height;

    // Create canvas and renderer
    const canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;

    this.renderer = new THREE.WebGLRenderer({
      canvas: canvas,
      antialias: true,
      alpha: true,
      preserveDrawingBuffer: true
    });

    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.NoToneMapping;
    this.renderer.setClearColor(0x000000, 0);
    this.renderer.setSize(width, height);

    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(40, width/height, 0.1, 1000);

    const ambientLight = new THREE.AmbientLight(0xffffff, 2);
    this.scene.add(ambientLight);
  }

  /**
   * Loads a GLTF model with caching
   */
  private async loadModel(modelUrl: string): Promise<GLTF> {
    if (this.modelCache.has(modelUrl)) {
      return this.modelCache.get(modelUrl)!;
    }

    const loader = new GLTFLoader();
    return new Promise<GLTF>((resolve, reject) => {
      loader.load(
        modelUrl,
        (gltf) => {
          this.modelCache.set(modelUrl, gltf);
          resolve(gltf);
        },
        undefined,
        reject
      );
    });
  }

  /**
   * Loads a texture with caching
   */
  private async loadTexture(textureUrl: string): Promise<THREE.Texture> {
    if (this.textureCache.has(textureUrl)) {
      return this.textureCache.get(textureUrl)!;
    }

    return new Promise<THREE.Texture>((resolve) => {
      const textureLoader = new THREE.TextureLoader();
      textureLoader.load(textureUrl, (texture) => {
        // Apply texture settings
        texture.colorSpace = THREE.SRGBColorSpace;
        texture.flipY = false;
        texture.magFilter = THREE.NearestFilter;
        texture.minFilter = THREE.NearestFilter;

        this.textureCache.set(textureUrl, texture);
        resolve(texture);
      });
    });
  }

  /**
   * Applies a texture to all meshes in a model
   */
  private applyTexture(model: THREE.Object3D, texture: THREE.Texture): void {
    model.traverse(child => {
      if ((child as THREE.Mesh).isMesh) {
        const mesh = child as THREE.Mesh;
        const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];

        materials.forEach((mat: THREE.Material) => {
          if (mat instanceof THREE.MeshStandardMaterial) {
            mat.map = texture;
            mat.metalness = 0;
            mat.color.set(0xffffff);
            mat.toneMapped = false;
            mat.roughness = 1;
            mat.needsUpdate = true;
          }
        });
      }
    });
  }

  /**
   * Renders both forward and backward views of a skin
   */
  public async renderSkin(textureUrl: string, modelUrl: string): Promise<RenderResult> {
    await this.setupModel(modelUrl, textureUrl);

    // Create a bounding box for the entire model
    const bbox = new THREE.Box3().setFromObject(this.currentModel!);

    // Calculate model dimensions
    const boxCenter = bbox.getCenter(new THREE.Vector3());
    const boxSize = bbox.getSize(new THREE.Vector3());

    // Calculate optimal camera distance based on model height and canvas aspect ratio
    const aspectRatio = this.height / this.width;
    const verticalFOVRadians = this.camera.fov * Math.PI / 180;
    const cameraDistance = (boxSize.y * 1.5) / Math.tan(verticalFOVRadians / 2);

    // Position camera perfectly front and back, with no side angle
    const frontCameraPos: [number, number, number] = [
      0, // No x-offset for straight-on view
      boxCenter.y + (boxSize.y * 0.1), // Slightly above center to frame face better
      boxCenter.z - cameraDistance // Front view (negative z)
    ];

    const backCameraPos: [number, number, number] = [
      0, // No x-offset for straight-on view
      boxCenter.y + (boxSize.y * 0.1), // Same height as front
      boxCenter.z + cameraDistance // Back view (positive z)
    ];

    // Look at the center of the model (vertically centered on face/upper torso)
    const lookAtPos: [number, number, number] = [
      boxCenter.x,
      boxCenter.y - (boxSize.y * 0.7),
      boxCenter.z
    ];

    // Pass these positions to renderView with the lookAt target
    const [forwards, backwards] = await Promise.all([
      this.renderView(frontCameraPos, lookAtPos),
      this.renderView(backCameraPos, lookAtPos)
    ]);

    return { forwards, backwards };
  }

  /**
   * Renders a view of the model and returns a blob URL
   * Updated to accept lookAt position
   */
  private async renderView(cameraPosition: [number, number, number], lookAtPosition: [number, number, number]): Promise<string> {
    this.camera.position.set(...cameraPosition);
    this.camera.lookAt(...lookAtPosition);

    this.renderer.render(this.scene, this.camera);

    return new Promise<string>((resolve, reject) => {
      this.renderer.domElement.toBlob((blob) => {
        if (blob) {
          const url = URL.createObjectURL(blob);
          resolve(url);
        } else {
          reject(new Error("Failed to create blob from canvas"));
        }
      }, 'image/png');
    });
  }

  /**
   * Sets up a model with texture in the scene
   */
  private async setupModel(modelUrl: string, textureUrl: string): Promise<void> {
    // Clean up previous model if it exists
    if (this.currentModel) {
      this.scene.remove(this.currentModel);
    }

    const [gltf, texture] = await Promise.all([
      this.loadModel(modelUrl),
      this.loadTexture(textureUrl)
    ]);

    // Clone the model to avoid modifying the cached one
    const model = gltf.scene.clone();
    this.applyTexture(model, texture);

    // Setup group and positioning
    const group = new THREE.Group();
    group.add(model);
    group.position.set(0, -0.5, 1.95);
    group.scale.set(0.8, 0.8, 0.8);

    this.scene.add(group);
    this.currentModel = group;
  }

  /**
   * Cleanup resources
   */
  public dispose(): void {
    Array.from(this.textureCache.values()).forEach(texture => {
      texture.dispose();
    });

    this.renderer.dispose();
    this.textureCache.clear();
    this.modelCache.clear();
  }
}

/**
 * Gets the appropriate model URL based on skin variant
 */
function getModelUrlForVariant(variant: string): string {
  switch (variant) {
    case 'SLIM':
      return '/src/assets/models/slim_player.gltf';
    case 'CLASSIC':
    case 'UNKNOWN':
    default:
      return '/src/assets/models/classic_player.gltf';
  }
}

export const map = reactive(new Map<string, RenderResult>());

/**
 * Generates skin previews for an array of skins
 * Renders both front and back views for each skin
 *
 * @param skins - Array of Skin objects to render
 * @returns A map of skin texture keys to their rendered front and back views
 */
export async function generateSkinPreviews(skins: Skin[]): Promise<void> {
  const renderer = new BatchSkinRenderer(215, 645);

  try {
    // Process each skin
    for (const skin of skins) {
      // Skip if already in result map
      if (map.has(skin.texture_key)) {
        continue;
      }

      // Determine model variant if unknown
      let variant = skin.variant;
      if (variant === 'UNKNOWN') {
        try {
          variant = await determineModelType(skin.texture);
        } catch (error) {
          console.error(`Failed to determine model type for skin ${skin.texture_key}:`, error);
          variant = 'CLASSIC'; // Fall back to classic
        }
      }

      const modelUrl = getModelUrlForVariant(variant);

      // Render the skin
      const renderResult = await renderer.renderSkin(skin.texture, modelUrl);

      // Store in result map and cache
      map.set(skin.texture_key, renderResult);
    }
  } finally {
    // Clean up renderer resources
    renderer.dispose();
  }
}


