import type { CSSProperties, Ref } from 'vue'
import { nextTick, onUnmounted, ref, watch } from 'vue'

export type AnchoredTeleportPlacement =
	| 'bottom-start'
	| 'bottom-end'
	| 'bottom-center'
	| 'top-start'
	| 'top-end'
	| 'top-center'
	| 'right-start'
	| 'right-end'
	| 'left-start'
	| 'left-end'
export type AnchoredTeleportSide = 'top' | 'right' | 'bottom' | 'left'

const viewportPadding = 8
const anchorPadding = 18
const defaultDistance = ref(8)

export function useAnchoredTeleport(
	trigger: Readonly<Ref<HTMLElement | null>>,
	panel: Readonly<Ref<HTMLElement | null>>,
	placement: Readonly<Ref<AnchoredTeleportPlacement>>,
	distance: Readonly<Ref<number>> = defaultDistance,
) {
	const isOpen = ref(false)
	const panelStyle = ref<CSSProperties>({
		top: '0px',
		left: '0px',
		visibility: 'hidden',
	})
	const anchorStyle = ref<CSSProperties>({})
	const resolvedSide = ref<AnchoredTeleportSide>('bottom')

	let resizeObserver: ResizeObserver | undefined

	function updatePosition() {
		if (!isOpen.value || !trigger.value || !panel.value) return

		const triggerRect = trigger.value.getBoundingClientRect()
		const scrollRegion =
			panel.value.querySelector<HTMLElement>('[data-anchored-scroll-region]') ?? panel.value
		const panelRect = { width: panel.value.offsetWidth, height: scrollRegion.scrollHeight }
		const offset = distance.value
		const alignsEnd = placement.value.endsWith('end')
		const alignsCenter = placement.value.endsWith('center')
		const isHorizontal = placement.value.startsWith('right') || placement.value.startsWith('left')
		let idealTop: number
		let idealLeft: number
		let maxHeight: number | undefined

		if (isHorizontal) {
			const prefersRight = placement.value.startsWith('right')
			const spaceRight = window.innerWidth - triggerRect.right - viewportPadding
			const spaceLeft = triggerRect.left - viewportPadding
			const opensRight = prefersRight
				? panelRect.width + offset <= spaceRight || spaceRight > spaceLeft
				: panelRect.width + offset > spaceLeft && spaceRight > spaceLeft

			resolvedSide.value = opensRight ? 'right' : 'left'
			idealTop = alignsEnd ? triggerRect.bottom - panelRect.height : triggerRect.top
			idealLeft = opensRight
				? triggerRect.right + offset
				: triggerRect.left - panelRect.width - offset
		} else {
			const prefersTop = placement.value.startsWith('top')
			const spaceBelow = window.innerHeight - triggerRect.bottom - viewportPadding
			const spaceAbove = triggerRect.top - viewportPadding
			const opensAbove = prefersTop
				? panelRect.height + offset <= spaceAbove || spaceAbove > spaceBelow
				: panelRect.height + offset > spaceBelow && spaceAbove > spaceBelow

			resolvedSide.value = opensAbove ? 'top' : 'bottom'
			const availableHeight = opensAbove ? spaceAbove : spaceBelow
			if (panelRect.height + offset > availableHeight) {
				maxHeight = Math.max(0, availableHeight - offset)
			}
			idealTop = opensAbove
				? triggerRect.top - (maxHeight ?? panelRect.height) - offset
				: triggerRect.bottom + offset
			if (alignsCenter) {
				const centered = triggerRect.left + triggerRect.width / 2 - panelRect.width / 2
				if (centered < viewportPadding) {
					idealLeft = triggerRect.left
				} else if (centered + panelRect.width > window.innerWidth - viewportPadding) {
					idealLeft = triggerRect.right - panelRect.width
				} else {
					idealLeft = centered
				}
			} else {
				idealLeft = alignsEnd ? triggerRect.right - panelRect.width : triggerRect.left
			}
		}

		const effectiveHeight = maxHeight ?? panelRect.height
		const maxTop = Math.max(viewportPadding, window.innerHeight - effectiveHeight - viewportPadding)
		const maxLeft = Math.max(viewportPadding, window.innerWidth - panelRect.width - viewportPadding)
		const panelTop = Math.min(Math.max(idealTop, viewportPadding), maxTop)
		const panelLeft = Math.min(Math.max(idealLeft, viewportPadding), maxLeft)

		panelStyle.value = {
			top: `${panelTop}px`,
			left: `${panelLeft}px`,
			visibility: 'visible',
			...(maxHeight !== undefined ? { maxHeight: `${maxHeight}px` } : {}),
		}
		anchorStyle.value = isHorizontal
			? {
					top: `${Math.min(
						Math.max(triggerRect.top + triggerRect.height / 2 - panelTop, anchorPadding),
						Math.max(anchorPadding, panelRect.height - anchorPadding),
					)}px`,
				}
			: {
					left: `${Math.min(
						Math.max(triggerRect.left + triggerRect.width / 2 - panelLeft, anchorPadding),
						Math.max(anchorPadding, panelRect.width - anchorPadding),
					)}px`,
				}
	}

	function handlePointerDown(event: PointerEvent) {
		const target = event.target as Node | null
		if (!target || trigger.value?.contains(target) || panel.value?.contains(target)) return
		close()
	}

	function addListeners() {
		document.addEventListener('pointerdown', handlePointerDown)
		window.addEventListener('resize', updatePosition)
		window.addEventListener('scroll', updatePosition, true)

		resizeObserver = new ResizeObserver(updatePosition)
		if (trigger.value) resizeObserver.observe(trigger.value)
		if (panel.value) resizeObserver.observe(panel.value)
	}

	function removeListeners() {
		document.removeEventListener('pointerdown', handlePointerDown)
		window.removeEventListener('resize', updatePosition)
		window.removeEventListener('scroll', updatePosition, true)
		resizeObserver?.disconnect()
		resizeObserver = undefined
	}

	async function open() {
		if (isOpen.value) return
		panelStyle.value = { top: '0px', left: '0px', visibility: 'hidden' }
		isOpen.value = true
		await nextTick()
		updatePosition()
		addListeners()
	}

	function close(restoreFocus = false) {
		if (!isOpen.value) return
		isOpen.value = false
		removeListeners()
		if (restoreFocus) nextTick(() => trigger.value?.focus())
	}

	watch([placement, distance], updatePosition)
	watch(panel, () => {
		if (!isOpen.value) return
		resizeObserver?.disconnect()
		if (trigger.value) resizeObserver?.observe(trigger.value)
		if (panel.value) resizeObserver?.observe(panel.value)
		updatePosition()
	})

	onUnmounted(removeListeners)

	return {
		isOpen,
		panelStyle,
		anchorStyle,
		resolvedSide,
		open,
		close,
	}
}
