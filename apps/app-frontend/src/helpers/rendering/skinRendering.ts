import * as THREE from 'three';
import { GLTFLoader, GLTF } from 'three/examples/jsm/loaders/GLTFLoader.js';

interface CameraSettings {
  fov: number;
  position: THREE.Vector3 | [number, number, number];
  lookAt?: THREE.Vector3 | [number, number, number];
}

/**
 * Creates a screenshot of a 3D skin model with the given texture and returns a blob URL
 * Uses OffscreenCanvas for rendering and specifically handles GLTF models
 *
 * @param skinTexture - URL to the skin texture
 * @param gltfModelUrl - URL to the GLTF/GLB model
 * @param cameraSettings - Camera settings including fov and position
 * @param width - Width of the output image
 * @param height - Height of the output image
 * @returns Promise that resolves to a blob URL of the screenshot
 */
export async function bakeSkinRender(
  skinTexture: string,
  gltfModelUrl: string,
  cameraSettings: CameraSettings,
  width: number = 512,
  height: number = 512
): Promise<string> {
  if (typeof OffscreenCanvas === 'undefined') {
    throw new Error('OffscreenCanvas is not supported in this browser');
  }

  let canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;

  const renderer = new THREE.WebGLRenderer({
    canvas: canvas,
    antialias: false,
    alpha: true,
    preserveDrawingBuffer: true
  });

  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.toneMapping = THREE.NoToneMapping;
  renderer.setSize(width, height);

  const scene = new THREE.Scene();

  const camera = new THREE.PerspectiveCamera(
    cameraSettings.fov,
    width / height,
    0.1,
    1000
  );

  if (Array.isArray(cameraSettings.position)) {
    camera.position.set(...cameraSettings.position);
  } else {
    camera.position.copy(cameraSettings.position);
  }

  if (cameraSettings.lookAt) {
    if (Array.isArray(cameraSettings.lookAt)) {
      camera.lookAt(new THREE.Vector3(...cameraSettings.lookAt));
    } else {
      camera.lookAt(cameraSettings.lookAt);
    }
  } else {
    camera.lookAt(0, 0, 0);
  }

  const ambientLight = new THREE.AmbientLight(0xffffff, 2);
  scene.add(ambientLight);

  const [gltfResult, texture] = await Promise.all([
    loadGLTFModel(gltfModelUrl),
    loadTexture(skinTexture)
  ]);

  const model = gltfResult.scene;

  applyTextureSettings(texture);

  applyTextureToModel(model, texture);

  const group = new THREE.Group();
  group.add(model);

  group.position.set(0, -0.05, 1.95);
  group.scale.set(0.8, 0.8, 0.8);
  scene.add(group);

  renderer.render(scene, camera);

  return new Promise<string>((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob) {
        const url = URL.createObjectURL(blob);
        resolve(url);
      } else {
        canvas.remove();
        reject(new Error("Failed to make blob."));
      }
    }, 'image/png');
  });
}

async function loadGLTFModel(url: string): Promise<GLTF> {
  return new Promise((resolve, reject) => {
    const loader = new GLTFLoader();
    loader.load(
      url,
      (gltf) => resolve(gltf),
      undefined,
      (error) => reject(error)
    );
  });
}

async function loadTexture(url: string): Promise<THREE.Texture> {
  return new Promise((resolve) => {
    const textureLoader = new THREE.TextureLoader();
    textureLoader.load(url, resolve);
  });
}

function applyTextureSettings(texture: THREE.Texture): void {
  texture.colorSpace = THREE.SRGBColorSpace;
  texture.flipY = false;
  texture.magFilter = THREE.NearestFilter;
  texture.minFilter = THREE.NearestFilter;
}

function applyTextureToModel(root: THREE.Object3D, texture: THREE.Texture): void {
  if (!root) return;

  root.traverse(child => {
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

// TODO: temp remove
export async function testWithSteve() {
  const result = await bakeSkinRender(
    "/src/assets/skins/steve.png",
    "/src/assets/models/classic_player.gltf",
    {
      fov: 40,
      position: [0, 0, -3.25]
    },
    800, 600
  );

  console.log(result);

  return result;
}
