import type { CSSProperties, Ref } from 'vue'
import { nextTick, onUnmounted, ref, watch } from 'vue'

import type { TeleportPlacement } from './types'

const viewportPadding = 8

export function useAnchoredTeleport(
	trigger: Readonly<Ref<HTMLElement | null>>,
	panel: Readonly<Ref<HTMLElement | null>>,
	placement: Readonly<Ref<TeleportPlacement>>,
) {
	const isOpen = ref(false)
	const panelStyle = ref<CSSProperties>({
		top: '0px',
		left: '0px',
		visibility: 'hidden',
	})

	let resizeObserver: ResizeObserver | undefined

	function updatePosition() {
		if (!isOpen.value || !trigger.value || !panel.value) return

		const triggerRect = trigger.value.getBoundingClientRect()
		const panelRect = panel.value.getBoundingClientRect()
		const offset = 8
		const prefersTop = placement.value.startsWith('top')
		const alignsEnd = placement.value.endsWith('end')
		const spaceBelow = window.innerHeight - triggerRect.bottom - viewportPadding
		const spaceAbove = triggerRect.top - viewportPadding
		const opensAbove = prefersTop
			? panelRect.height + offset <= spaceAbove || spaceAbove > spaceBelow
			: panelRect.height + offset > spaceBelow && spaceAbove > spaceBelow

		const idealTop = opensAbove
			? triggerRect.top - panelRect.height - offset
			: triggerRect.bottom + offset
		const idealLeft = alignsEnd ? triggerRect.right - panelRect.width : triggerRect.left
		const maxTop = Math.max(viewportPadding, window.innerHeight - panelRect.height - viewportPadding)
		const maxLeft = Math.max(viewportPadding, window.innerWidth - panelRect.width - viewportPadding)

		panelStyle.value = {
			top: `${Math.min(Math.max(idealTop, viewportPadding), maxTop)}px`,
			left: `${Math.min(Math.max(idealLeft, viewportPadding), maxLeft)}px`,
			visibility: 'visible',
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

	watch(placement, updatePosition)
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
		open,
		close,
	}
}
