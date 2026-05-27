import * as THREE from 'three'
import { computed, onMounted, onUnmounted, ref, watch, type ComputedRef, type Ref } from 'vue'

import type {
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
} satisfies Record<SkinPreviewFraming, { fov: number; zoom: number; padding: SkinPreviewFitPadding }>

function cloneModelTuple(tuple: SkinPreviewTuple): SkinPreviewTuple {
	return [tuple[0], tuple[1], tuple[2]]
}

type MaybeReadonlyRef<T> = Ref<T> | ComputedRef<T>

export function useSkinPreviewFit({
	containerElement,
	fit,
	lockFit,
	framing,
	fitZoom,
	fitPadding,
	scale,
	fov,
	nametag,
	hasSubtitle,
	hasNametagBadge,
	modelCenter,
	modelSize,
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
	nametag: MaybeReadonlyRef<string | undefined>
	hasSubtitle: MaybeReadonlyRef<boolean>
	hasNametagBadge: MaybeReadonlyRef<boolean>
	modelCenter: MaybeReadonlyRef<SkinPreviewTuple>
	modelSize: MaybeReadonlyRef<SkinPreviewTuple>
	isModelLoaded: MaybeReadonlyRef<boolean>
}) {
	const containerSize = ref({ width: 1, height: 1 })
	const fitLock = ref<SkinPreviewFitLock | null>(null)
	let resizeObserver: ResizeObserver | undefined

	const fitEnabled = computed(() => {
		if (fit.value !== undefined) return fit.value
		return scale.value === undefined && fov.value === undefined
	})
	const currentFraming = computed<SkinPreviewFraming>(() => framing.value ?? 'page')
	const lockFitEnabled = computed(() => lockFit.value ?? true)
	const legacyScale = computed(() => scale.value ?? 1)
	const legacyFov = computed(() => fov.value ?? 40)
	const previewControlsPositionClass = computed(() =>
		currentFraming.value === 'modal' ? 'bottom-[calc(6%)]' : 'bottom-[calc(15%+64px)]',
	)
	const subtitlePositionClass = computed(() =>
		currentFraming.value === 'modal' ? 'bottom-[6%]' : 'bottom-[calc(15%)]',
	)

	const hasUsableFitSize = computed(
		() => containerSize.value.width > 1 && containerSize.value.height > 1,
	)
	const hasResolvedFit = computed(
		() =>
			!fitEnabled.value ||
			(lockFitEnabled.value ? fitLock.value !== null : hasUsableFitSize.value),
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

	const resolvedFitPadding = computed<SkinPreviewFitPadding>(() => {
		const preset = FRAMING_PRESETS[currentFraming.value].padding

		return {
			top: Math.max(preset.top, nametag.value ? (hasNametagBadge.value ? 0.28 : 0.2) : 0),
			right: preset.right,
			bottom: Math.max(preset.bottom, hasSubtitle.value ? 0.28 : preset.bottom),
			left: preset.left,
			...(fitPadding.value ?? {}),
		}
	})

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
		const padding = resolvedFitPadding.value

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

	const nametagTop = computed(() => {
		if (!fitEnabled.value) return '18%'

		const [, sizeY] = fitModelSize.value
		const { fov: resolvedFov, position, target } = cameraConfig.value
		const distance = Math.max(Math.abs(position[2] - target[2]), 0.001)
		const verticalFov = THREE.MathUtils.degToRad(resolvedFov)
		const modelTopY = sizeY / 2
		const projectedY =
			(modelTopY - target[1]) / distance / Math.max(Math.tan(verticalFov / 2), 0.001)
		const topPercent = THREE.MathUtils.clamp(((1 - projectedY) / 2) * 100, 8, 40)

		return `${topPercent - 2}%`
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

	function lockFitState() {
		if (!fitEnabled.value || !lockFitEnabled.value || fitLock.value || !isModelLoaded.value) return

		const { width, height } = containerSize.value
		if (width <= 1 || height <= 1) return

		fitLock.value = {
			containerSize: { width, height },
			modelCenter: cloneModelTuple(modelCenter.value),
			modelSize: cloneModelTuple(modelSize.value),
		}
	}

	onMounted(() => {
		const el = containerElement.value
		if (!el) return

		resizeObserver = new ResizeObserver(([entry]) => {
			const { width, height } = entry.contentRect
			containerSize.value = {
				width: Math.max(width, 1),
				height: Math.max(height, 1),
			}

			if (lockFitEnabled.value) {
				lockFitState()
			}
		})

		resizeObserver.observe(el)
	})

	watch(
		() => isModelLoaded.value,
		(loaded) => {
			if (!loaded) {
				fitLock.value = null
				return
			}

			lockFitState()
		},
	)

	watch(
		() => lockFit.value,
		() => {
			fitLock.value = null
			lockFitState()
		},
	)

	watch(fitEnabled, () => {
		fitLock.value = null
		lockFitState()
	})

	onUnmounted(() => {
		resizeObserver?.disconnect()
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
		previewControlsPositionClass,
		spotlightPosition,
		spotlightScale,
		subtitlePositionClass,
	}
}
