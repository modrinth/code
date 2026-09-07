import { ClassicPlayerModel, SlimPlayerModel } from '@modrinth/assets'
import { loadSkinRendering } from '@modrinth/ui'
import * as THREE from 'three'

import type { Cape, Skin } from '../skins'
import { determineModelType, get_normalized_skin_texture } from '../skins'

export interface RenderResult {
	forwards: string
}

export interface RawRenderResult {
	forwards: Blob
}

class BatchSkinRenderer {
	private renderer: THREE.WebGLRenderer | null = null
	private scene: THREE.Scene | null = null
	private camera: THREE.PerspectiveCamera | null = null
	private currentModel: THREE.Group | null = null
	private transparentTexture: THREE.Texture | null = null
	private readonly width: number
	private readonly height: number

	constructor(
		private readonly rendering: Awaited<ReturnType<typeof loadSkinRendering>>,
		width: number = 360,
		height: number = 504,
	) {
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
		this.renderer.shadowMap.enabled = false
		this.renderer.toneMapping = THREE.NoToneMapping
		this.renderer.toneMappingExposure = 10.0
		this.renderer.setClearColor(0x000000, 0)
		this.renderer.setSize(this.width, this.height)

		this.scene = new THREE.Scene()
		this.camera = new THREE.PerspectiveCamera(20, this.width / this.height, 0.4, 1000)

		const ambientLight = new THREE.AmbientLight(0xffffff, 2)
		const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2)
		directionalLight.castShadow = false
		directionalLight.position.set(2, 4, 3)
		this.scene.add(ambientLight)
		this.scene.add(directionalLight)
	}

	public async renderSkin(
		textureUrl: string,
		modelUrl: string,
		capeUrl?: string,
		earsTextureUrl?: string,
	): Promise<RawRenderResult> {
		this.initializeRenderer()

		this.clearScene()

		await this.setupModel(modelUrl, textureUrl, capeUrl, earsTextureUrl)

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
		const forwards = await this.renderView(frontCameraPos, lookAtTarget)

		return { forwards }
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

		return await new Promise<Blob>((resolve, reject) => {
			this.renderer!.domElement.toBlob(
				(blob) => {
					if (blob) {
						resolve(blob)
					} else {
						reject(new Error('Failed to create blob from rendered canvas'))
					}
				},
				'image/webp',
				0.9,
			)
		})
	}

	private async setupModel(
		modelUrl: string,
		textureUrl: string,
		capeUrl?: string,
		earsTextureUrl?: string,
	): Promise<void> {
		if (!this.scene) {
			throw new Error('Renderer not initialized')
		}

		const [{ model }, earsTexture] = await Promise.all([
			this.rendering.setupSkinModel(modelUrl, textureUrl, capeUrl),
			earsTextureUrl ? this.rendering.loadTexture(earsTextureUrl) : Promise.resolve(null),
		])

		if (!capeUrl) {
			this.rendering.applyCapeTexture(model, null, this.getTransparentTexture())
		}

		if (earsTexture) {
			this.rendering.applyEarsMod(model, earsTexture)
		}

		const group = new THREE.Group()
		group.add(model)
		group.position.set(0, 0.3, 1.95)
		group.scale.set(0.8, 0.8, 0.8)

		this.scene.add(group)
		this.currentModel = group
	}

	private getTransparentTexture(): THREE.Texture {
		if (!this.transparentTexture) {
			this.transparentTexture = this.rendering.createTransparentTexture()
		}

		return this.transparentTexture
	}

	private clearScene(): void {
		if (!this.scene || !this.currentModel) return

		this.rendering.removeEarsMod(this.currentModel)
		this.scene.remove(this.currentModel)
		this.currentModel.clear()
		this.currentModel = null
	}

	public dispose(): void {
		this.clearScene()

		if (this.transparentTexture) {
			this.transparentTexture.dispose()
			this.transparentTexture = null
		}

		if (this.renderer) {
			this.renderer.dispose()
			this.renderer.forceContextLoss()
		}

		this.renderer = null
		this.scene = null
		this.camera = null

		this.rendering.disposeCaches()
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

let sharedRenderer: BatchSkinRenderer | null = null

export function disposeSharedRenderer(): void {
	sharedRenderer?.dispose()
	sharedRenderer = null
}

export async function renderSkinPreview(skin: Skin, capes: Cape[]): Promise<RawRenderResult> {
	sharedRenderer ??= new BatchSkinRenderer(await loadSkinRendering())
	const variant =
		skin.variant === 'UNKNOWN'
			? await determineModelType(skin.texture).catch(() => 'CLASSIC')
			: skin.variant
	return sharedRenderer.renderSkin(
		await get_normalized_skin_texture(skin),
		getModelUrlForVariant(variant),
		capes.find((cape) => cape.id === skin.cape_id)?.texture,
		skin.texture,
	)
}
