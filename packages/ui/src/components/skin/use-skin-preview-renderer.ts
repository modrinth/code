import * as THREE from 'three'
import { type ComputedRef, onMounted, onUnmounted, type Ref } from 'vue'

import type { SkinPreviewTuple } from '#ui/composables/skin-rendering'

import { createRadialSpotlightShader } from './skin-preview-shader'

const MIN_PIXEL_RATIO = 1
const MAX_PIXEL_RATIO = 1.5

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

function getOwnerWindow(element: HTMLElement) {
	return element.ownerDocument.defaultView ?? window
}

export function useSkinPreviewRenderer({
	canvasElement,
	scene,
	cameraConfig,
	modelGroupRotation,
	modelGroupPosition,
	modelGroupScale,
	modelOffset,
	spotlightPosition,
	spotlightScale,
	onBeforeRender,
}: {
	canvasElement: MaybeReadonlyRef<HTMLCanvasElement | null>
	scene: MaybeReadonlyRef<THREE.Object3D | null>
	cameraConfig: MaybeReadonlyRef<{
		fov: number
		position: SkinPreviewTuple
		target: SkinPreviewTuple
	}>
	modelGroupRotation: MaybeReadonlyRef<SkinPreviewTuple>
	modelGroupPosition: MaybeReadonlyRef<SkinPreviewTuple>
	modelGroupScale: MaybeReadonlyRef<SkinPreviewTuple>
	modelOffset: MaybeReadonlyRef<SkinPreviewTuple>
	spotlightPosition: MaybeReadonlyRef<SkinPreviewTuple>
	spotlightScale: MaybeReadonlyRef<SkinPreviewTuple>
	onBeforeRender: () => void
}) {
	const root = new THREE.Scene()
	const camera = new THREE.PerspectiveCamera()
	const modelGroup = new THREE.Group()
	const modelOffsetGroup = new THREE.Group()
	const spotlight = new THREE.Mesh(
		new THREE.CircleGeometry(1, 128),
		new THREE.ShaderMaterial(createRadialSpotlightShader()),
	)
	const directionalLight = new THREE.DirectionalLight(0xffffff, 1.2)
	const drawingBufferSize = new THREE.Vector2()

	spotlight.rotation.x = -Math.PI / 2
	directionalLight.position.set(-3, 4, -2)
	modelGroup.add(modelOffsetGroup)
	root.add(modelGroup, spotlight, camera, new THREE.AmbientLight(0xffffff, 2), directionalLight)

	let renderer: THREE.WebGLRenderer | null = null
	let resizeObserver: ResizeObserver | undefined
	let mountedModel: THREE.Object3D | null = null
	let canvasSize = { width: 0, height: 0 }
	let frameWindow: Window | null = null
	let frameHandle: number | null = null

	function syncDrawingBuffer(activeRenderer: THREE.WebGLRenderer, canvas: HTMLCanvasElement) {
		const pixelRatio = THREE.MathUtils.clamp(
			getOwnerWindow(canvas).devicePixelRatio,
			MIN_PIXEL_RATIO,
			MAX_PIXEL_RATIO,
		)
		if (pixelRatio !== activeRenderer.getPixelRatio()) {
			activeRenderer.setPixelRatio(pixelRatio)
		}

		const { x: width, y: height } = activeRenderer.getSize(drawingBufferSize)
		if (width !== canvasSize.width || height !== canvasSize.height) {
			activeRenderer.setSize(canvasSize.width, canvasSize.height, false)
			camera.aspect = canvasSize.width / canvasSize.height
		}
	}

	function syncSceneGraph() {
		const model = scene.value
		if (model !== mountedModel) {
			if (mountedModel) modelOffsetGroup.remove(mountedModel)
			if (model) modelOffsetGroup.add(model)
			mountedModel = model
		}

		modelGroup.rotation.set(...modelGroupRotation.value)
		modelGroup.position.set(...modelGroupPosition.value)
		modelGroup.scale.set(...modelGroupScale.value)
		modelOffsetGroup.position.set(...modelOffset.value)
		spotlight.position.set(...spotlightPosition.value)
		spotlight.scale.set(...spotlightScale.value)

		const { fov, position, target } = cameraConfig.value
		camera.fov = fov
		camera.position.set(...position)
		camera.lookAt(...target)
		camera.updateProjectionMatrix()
	}

	function renderFrame() {
		const canvas = canvasElement.value
		if (!renderer || !canvas || canvasSize.width === 0 || canvasSize.height === 0) return

		syncDrawingBuffer(renderer, canvas)
		syncSceneGraph()
		renderer.render(root, camera)
	}

	function requestFrame() {
		const canvas = canvasElement.value
		if (!canvas) return

		frameWindow = getOwnerWindow(canvas)
		frameHandle = frameWindow.requestAnimationFrame(() => {
			onBeforeRender()
			renderFrame()
			requestFrame()
		})
	}

	onMounted(() => {
		const canvas = canvasElement.value
		if (!canvas) return

		renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true })
		renderer.outputColorSpace = THREE.SRGBColorSpace
		renderer.toneMapping = THREE.NoToneMapping
		renderer.toneMappingExposure = 10

		resizeObserver = new ResizeObserver(([entry]) => {
			canvasSize = { width: entry.contentRect.width, height: entry.contentRect.height }
			renderFrame()
		})
		resizeObserver.observe(canvas)

		requestFrame()
	})

	onUnmounted(() => {
		if (frameHandle !== null) frameWindow?.cancelAnimationFrame(frameHandle)
		resizeObserver?.disconnect()

		if (mountedModel) modelOffsetGroup.remove(mountedModel)
		mountedModel = null
		spotlight.geometry.dispose()
		spotlight.material.dispose()

		renderer?.dispose()
		renderer?.forceContextLoss()
		renderer = null
	})
}
