import * as THREE from 'three'
import {
	computed,
	type ComputedRef,
	type CSSProperties,
	onMounted,
	onUnmounted,
	type Ref,
	ref,
	watch,
} from 'vue'

import type {
	SkinPreviewBounds,
	SkinPreviewFitLock,
	SkinPreviewFitPadding,
	SkinPreviewFraming,
	SkinPreviewTuple,
} from './types'

const FRAMING_PRESETS = {
	page: {
		fov: 35,
		zoom: 0.96,
		padding: { top: 0.2, right: 0.14, bottom: 0.3, left: 0.14 },
	},
	modal: {
		fov: 35,
		zoom: 1,
		padding: { top: 0.1, right: 0.1, bottom: 0.18, left: 0.1 },
	},
} satisfies Record<
	SkinPreviewFraming,
	{ fov: number; zoom: number; padding: SkinPreviewFitPadding }
>

const PREVIEW_CONTROLS_FOOT_OFFSET = 64
const SUBTITLE_CONTROLS_OFFSET = 48
const NAMETAG_HEAD_OFFSET = 16

function cloneModelTuple(tuple: SkinPreviewTuple): SkinPreviewTuple {
	return [tuple[0], tuple[1], tuple[2]]
}

function cloneBounds(bounds: SkinPreviewBounds): SkinPreviewBounds {
	return {
		min: cloneModelTuple(bounds.min),
		max: cloneModelTuple(bounds.max),
	}
}

const MODEL_ROTATION_AXIS = new THREE.Vector3(0, 1, 0)

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

type SkinPreviewDebugEntry = Record<string, unknown>

type SkinPreviewDebugWindow = Window & {
	__SKIN_PREVIEW_DEBUG__?: SkinPreviewDebugEntry[]
}

function serializeRect(rect: DOMRect) {
	return {
		bottom: rect.bottom,
		height: rect.height,
		left: rect.left,
		right: rect.right,
		top: rect.top,
		width: rect.width,
		x: rect.x,
		y: rect.y,
	}
}

function serializeElement(element: Element | null) {
	if (!(element instanceof HTMLElement)) return null

	const style = window.getComputedStyle(element)
	return {
		rect: serializeRect(element.getBoundingClientRect()),
		style: {
			bottom: style.bottom,
			height: style.height,
			left: style.left,
			position: style.position,
			top: style.top,
			transform: style.transform,
			width: style.width,
		},
	}
}

function getVisibleMeshDebugBounds(root: THREE.Object3D | null) {
	if (!root) return []

	root.updateWorldMatrix(true, true)
	const entries: SkinPreviewDebugEntry[] = []

	root.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.geometry || mesh.visible === false) return

		const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		if (materials.length && materials.every((material) => material.visible === false)) return

		if (!mesh.geometry.boundingBox) mesh.geometry.computeBoundingBox()
		if (!mesh.geometry.boundingBox) return

		const box = mesh.geometry.boundingBox.clone().applyMatrix4(mesh.matrixWorld)
		entries.push({
			materials: materials.map((material) => ({
				name: material.name,
				side: material.side,
				visible: material.visible,
			})),
			max: box.max.toArray(),
			min: box.min.toArray(),
			name: mesh.name,
			parent: mesh.parent?.name,
			uuid: mesh.uuid,
		})
	})

	return entries
}

export function useSkinPreviewFit({
	containerElement,
	fit,
	lockFit,
	framing,
	fitZoom,
	fitPadding,
	scale,
	fov,
	modelRotation,
	nametag,
	hasSubtitle,
	hasNametagBadge,
	subtitleWrapped,
	modelCenter,
	modelSize,
	scene,
	visibleBounds,
	isModelLoaded,
}: {
	containerElement: MaybeReadonlyRef<HTMLElement | null>
	fit: MaybeReadonlyRef<boolean | undefined>
	lockFit: MaybeReadonlyRef<boolean | undefined>
	framing: MaybeReadonlyRef<SkinPreviewFraming | undefined>
	fitZoom: MaybeReadonlyRef<number | undefined>
	fitPadding: MaybeReadonlyRef<Partial<SkinPreviewFitPadding> | undefined>
	scale: MaybeReadonlyRef<number | undefined>
	fov: MaybeReadonlyRef<number | undefined>
	modelRotation: MaybeReadonlyRef<number>
	nametag: MaybeReadonlyRef<string | undefined>
	hasSubtitle: MaybeReadonlyRef<boolean>
	hasNametagBadge: MaybeReadonlyRef<boolean>
	subtitleWrapped: MaybeReadonlyRef<boolean>
	modelCenter: MaybeReadonlyRef<SkinPreviewTuple>
	modelSize: MaybeReadonlyRef<SkinPreviewTuple>
	scene: MaybeReadonlyRef<THREE.Object3D | null>
	visibleBounds: MaybeReadonlyRef<SkinPreviewBounds>
	isModelLoaded: MaybeReadonlyRef<boolean>
}) {
	const containerSize = ref({ width: 1, height: 1 })
	const fitLock = ref<SkinPreviewFitLock | null>(null)
	let resizeObserver: ResizeObserver | undefined
	let debugAnimationFrame: number | null = null
	const pendingDebugReasons = new Set<string>()

	const fitEnabled = computed(() => {
		if (fit.value !== undefined) return fit.value
		return scale.value === undefined && fov.value === undefined
	})
	const currentFraming = computed<SkinPreviewFraming>(() => framing.value ?? 'page')
	const lockFitEnabled = computed(() => currentFraming.value === 'page' || (lockFit.value ?? true))
	const legacyScale = computed(() => scale.value ?? 1)
	const legacyFov = computed(() => fov.value ?? 40)

	const hasUsableFitSize = computed(
		() => containerSize.value.width > 1 && containerSize.value.height > 1,
	)
	const hasResolvedFit = computed(
		() =>
			!fitEnabled.value || (lockFitEnabled.value ? fitLock.value !== null : hasUsableFitSize.value),
	)

	const fitContainerSize = computed(() =>
		lockFitEnabled.value
			? (fitLock.value?.containerSize ?? containerSize.value)
			: containerSize.value,
	)
	const fitModelCenter = computed(() =>
		lockFitEnabled.value ? (fitLock.value?.modelCenter ?? modelCenter.value) : modelCenter.value,
	)
	const fitModelSize = computed(() =>
		lockFitEnabled.value ? (fitLock.value?.modelSize ?? modelSize.value) : modelSize.value,
	)
	const fitModelRotation = computed(() =>
		lockFitEnabled.value ? (fitLock.value?.rotation ?? modelRotation.value) : modelRotation.value,
	)
	const fitVisibleBounds = computed(() =>
		lockFitEnabled.value
			? (fitLock.value?.visibleBounds ?? visibleBounds.value)
			: visibleBounds.value,
	)

	const resolvedFitPadding = computed<SkinPreviewFitPadding>(() => {
		const preset = FRAMING_PRESETS[currentFraming.value].padding

		return {
			top: Math.max(preset.top, hasNametagBadge.value ? 0.28 : nametag.value ? 0.2 : 0),
			right: preset.right,
			bottom: Math.max(preset.bottom, hasSubtitle.value ? 0.28 : preset.bottom),
			left: preset.left,
			...(fitPadding.value ?? {}),
		}
	})
	const fitResolvedPadding = computed(() =>
		lockFitEnabled.value
			? (fitLock.value?.padding ?? resolvedFitPadding.value)
			: resolvedFitPadding.value,
	)

	const modelOffset = computed<SkinPreviewTuple>(() => {
		if (!fitEnabled.value) return [0, 0, 0]

		const [x, y, z] = fitModelCenter.value
		return [-x, -y, -z]
	})

	const modelGroupPosition = computed<SkinPreviewTuple>(() => {
		if (fitEnabled.value) return [0, 0, 0]
		return [0, -0.05 * legacyScale.value, 1.95]
	})

	const modelGroupScale = computed<SkinPreviewTuple>(() => {
		if (fitEnabled.value) return [1, 1, 1]

		const resolvedScale = 0.8 * legacyScale.value
		return [resolvedScale, resolvedScale, resolvedScale]
	})

	const fittedCamera = computed(() => {
		const width = Math.max(fitContainerSize.value.width, 1)
		const height = Math.max(fitContainerSize.value.height, 1)
		const aspect = width / height
		const preset = FRAMING_PRESETS[currentFraming.value]
		const padding = fitResolvedPadding.value

		const usableWidth = Math.max(width * (1 - padding.left - padding.right), 1)
		const usableHeight = Math.max(height * (1 - padding.top - padding.bottom), 1)

		const [sizeX, sizeY, sizeZ] = fitModelSize.value
		const halfWidth = Math.sqrt((sizeX / 2) ** 2 + (sizeZ / 2) ** 2)
		const halfHeight = sizeY / 2

		const resolvedFov = fov.value ?? preset.fov
		const verticalFov = THREE.MathUtils.degToRad(resolvedFov)
		const horizontalFov = 2 * Math.atan(Math.tan(verticalFov / 2) * aspect)

		const paddedHalfWidth = halfWidth * (width / usableWidth)
		const paddedHalfHeight = halfHeight * (height / usableHeight)
		const zoom = Math.max((fitZoom.value ?? 1) * preset.zoom, 0.01)

		const distance =
			Math.max(
				paddedHalfHeight / Math.tan(verticalFov / 2),
				paddedHalfWidth / Math.tan(horizontalFov / 2),
			) / zoom

		const visibleHalfHeight = distance * Math.tan(verticalFov / 2)
		const targetY = -(padding.bottom - padding.top) * visibleHalfHeight

		return {
			fov: resolvedFov,
			position: [0, targetY, -distance] as SkinPreviewTuple,
			target: [0, targetY, 0] as SkinPreviewTuple,
		}
	})

	const cameraConfig = computed(() => {
		if (fitEnabled.value) return fittedCamera.value

		return {
			fov: legacyFov.value,
			position: [0, 1.5, -3.25] as SkinPreviewTuple,
			target: modelCenter.value,
		}
	})

	const projectedVisibleBounds = computed(() => {
		if (!fitEnabled.value) return null

		const width = Math.max(containerSize.value.width, 1)
		const height = Math.max(containerSize.value.height, 1)
		const { fov: resolvedFov, position, target } = cameraConfig.value
		const camera = new THREE.PerspectiveCamera(resolvedFov, width / height, 0.1, 1000)
		camera.position.set(...position)
		camera.lookAt(...target)
		camera.updateProjectionMatrix()
		camera.updateMatrixWorld(true)

		const bounds = fitVisibleBounds.value
		const [offsetX, offsetY, offsetZ] = modelOffset.value
		let top = Number.POSITIVE_INFINITY
		let bottom = Number.NEGATIVE_INFINITY

		for (const x of [bounds.min[0], bounds.max[0]]) {
			for (const y of [bounds.min[1], bounds.max[1]]) {
				for (const z of [bounds.min[2], bounds.max[2]]) {
					const point = new THREE.Vector3(x + offsetX, y + offsetY, z + offsetZ)
					point.applyAxisAngle(MODEL_ROTATION_AXIS, fitModelRotation.value)
					point.project(camera)

					const screenY = ((1 - point.y) / 2) * height
					top = Math.min(top, screenY)
					bottom = Math.max(bottom, screenY)
				}
			}
		}

		return { bottom, top }
	})

	const modelFeetTop = computed(() => projectedVisibleBounds.value?.bottom ?? null)

	const previewControlsTop = computed(() =>
		modelFeetTop.value === null ? null : modelFeetTop.value + PREVIEW_CONTROLS_FOOT_OFFSET,
	)

	const previewControlsPositionStyle = computed<CSSProperties>(() => {
		if (!fitEnabled.value || currentFraming.value !== 'page' || previewControlsTop.value === null) {
			return {
				bottom: currentFraming.value === 'modal' ? '6%' : 'calc(15% + 64px)',
			}
		}

		return {
			top: `${previewControlsTop.value}px`,
		}
	})

	const subtitlePositionStyle = computed<CSSProperties>(() => {
		if (!fitEnabled.value || currentFraming.value !== 'page' || previewControlsTop.value === null) {
			return {
				bottom:
					currentFraming.value === 'modal'
						? '6%'
						: subtitleWrapped.value
							? 'calc(15% - 32px)'
							: '15%',
			}
		}

		return {
			top: `${previewControlsTop.value + SUBTITLE_CONTROLS_OFFSET}px`,
		}
	})

	const nametagTop = computed(() => {
		if (!fitEnabled.value) return '18%'

		const top = projectedVisibleBounds.value?.top
		return top === undefined ? '18%' : `${top - NAMETAG_HEAD_OFFSET}px`
	})

	const spotlightY = computed(() => {
		if (!fitEnabled.value) return -0.1 * legacyScale.value

		const [, sizeY] = fitModelSize.value
		return -sizeY / 2 - 0.02
	})

	const spotlightPosition = computed<SkinPreviewTuple>(() => [
		0,
		spotlightY.value,
		fitEnabled.value ? 0 : 2,
	])

	const spotlightScale = computed<SkinPreviewTuple>(() => {
		if (!fitEnabled.value) {
			const resolvedScale = 0.75 * legacyScale.value
			return [resolvedScale, resolvedScale, resolvedScale]
		}

		const [sizeX, , sizeZ] = fitModelSize.value
		const radius = Math.max(sizeX, sizeZ, 1) * 0.8
		return [radius, radius, radius]
	})

	function captureDebugSnapshot(reasons: string[]) {
		if (typeof window === 'undefined') return

		const container = containerElement.value
		const canvas = container?.querySelector('canvas') ?? null
		const debugWindow = window as SkinPreviewDebugWindow
		const entries = (debugWindow.__SKIN_PREVIEW_DEBUG__ ??= [])
		const snapshot = JSON.parse(
			JSON.stringify({
				camera: cameraConfig.value,
				currentInputs: {
					isModelLoaded: isModelLoaded.value,
					modelCenter: modelCenter.value,
					modelSize: modelSize.value,
					padding: resolvedFitPadding.value,
					rotation: modelRotation.value,
					visibleBounds: visibleBounds.value,
				},
				dom: {
					canvas: {
						...serializeElement(canvas),
						bufferHeight: canvas instanceof HTMLCanvasElement ? canvas.height : null,
						bufferWidth: canvas instanceof HTMLCanvasElement ? canvas.width : null,
					},
					container: serializeElement(container),
					controls: serializeElement(
						container?.querySelector('[data-skin-preview-debug="controls"]') ?? null,
					),
					nametag: serializeElement(
						container?.querySelector('[data-skin-preview-debug="nametag"]') ?? null,
					),
					subtitle: serializeElement(
						container?.querySelector('[data-skin-preview-debug="subtitle"]') ?? null,
					),
				},
				effectiveFit: {
					containerSize: fitContainerSize.value,
					modelCenter: fitModelCenter.value,
					modelOffset: modelOffset.value,
					modelSize: fitModelSize.value,
					padding: fitResolvedPadding.value,
					rotation: fitModelRotation.value,
					visibleBounds: fitVisibleBounds.value,
				},
				fitEnabled: fitEnabled.value,
				fitLock: fitLock.value,
				lockFitEnabled: lockFitEnabled.value,
				meshBounds: getVisibleMeshDebugBounds(scene.value),
				overlayCalculations: {
					nametagTop: nametagTop.value,
					previewControlsPositionStyle: previewControlsPositionStyle.value,
					projectedVisibleBounds: projectedVisibleBounds.value,
					subtitlePositionStyle: subtitlePositionStyle.value,
				},
				reasons,
				timestamp: new Date().toISOString(),
				window: {
					devicePixelRatio: window.devicePixelRatio,
					innerHeight: window.innerHeight,
					innerWidth: window.innerWidth,
				},
			}),
		) as SkinPreviewDebugEntry

		entries.push(snapshot)
		if (entries.length > 100) entries.splice(0, entries.length - 100)
		console.log('[SkinPreviewDebug]', snapshot)
	}

	function queueDebugSnapshot(reason: string) {
		if (typeof window === 'undefined') return

		pendingDebugReasons.add(reason)
		if (debugAnimationFrame !== null) return

		debugAnimationFrame = window.requestAnimationFrame(() => {
			debugAnimationFrame = null
			const reasons = Array.from(pendingDebugReasons)
			pendingDebugReasons.clear()
			captureDebugSnapshot(reasons)
		})
	}

	function lockFitState() {
		if (!fitEnabled.value || !lockFitEnabled.value || fitLock.value || !isModelLoaded.value) return

		const { width, height } = containerSize.value
		if (width <= 1 || height <= 1) return

		fitLock.value = {
			containerSize: { width, height },
			modelCenter: cloneModelTuple(modelCenter.value),
			modelSize: cloneModelTuple(modelSize.value),
			padding: { ...resolvedFitPadding.value },
			rotation: modelRotation.value,
			visibleBounds: cloneBounds(visibleBounds.value),
		}
		queueDebugSnapshot('fit-lock-created')
	}

	function resetFitLockForLayoutChange() {
		if (!fitEnabled.value || !lockFitEnabled.value) return

		fitLock.value = null
		lockFitState()
		queueDebugSnapshot('fit-lock-reset')
	}

	onMounted(() => {
		const el = containerElement.value
		if (!el) return

		resizeObserver = new ResizeObserver(([entry]) => {
			const { width, height } = entry.contentRect
			const nextContainerSize = {
				width: Math.max(width, 1),
				height: Math.max(height, 1),
			}
			const didContainerSizeChange =
				nextContainerSize.width !== containerSize.value.width ||
				nextContainerSize.height !== containerSize.value.height

			containerSize.value = nextContainerSize

			if (didContainerSizeChange) {
				resetFitLockForLayoutChange()
				queueDebugSnapshot('container-resized')
			}
		})

		resizeObserver.observe(el)
		queueDebugSnapshot('mounted')
	})

	watch(
		() => isModelLoaded.value,
		(loaded) => {
			if (loaded) lockFitState()
		},
	)

	watch(
		() => lockFitEnabled.value,
		() => {
			fitLock.value = null
			lockFitState()
		},
	)

	watch(fitEnabled, () => {
		fitLock.value = null
		lockFitState()
	})

	watch([modelCenter, modelSize, visibleBounds, resolvedFitPadding], () => {
		resetFitLockForLayoutChange()
		queueDebugSnapshot('model-inputs-changed')
	})

	watch(
		projectedVisibleBounds,
		() => {
			queueDebugSnapshot('projection-changed')
		},
		{ deep: true },
	)

	onUnmounted(() => {
		resizeObserver?.disconnect()
		if (debugAnimationFrame !== null) window.cancelAnimationFrame(debugAnimationFrame)
	})

	return {
		cameraConfig,
		currentFraming,
		fitEnabled,
		hasResolvedFit,
		legacyScale,
		modelGroupPosition,
		modelGroupScale,
		modelOffset,
		nametagTop,
		previewControlsPositionStyle,
		spotlightPosition,
		spotlightScale,
		subtitlePositionStyle,
	}
}
