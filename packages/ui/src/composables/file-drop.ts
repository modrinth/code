import type { ComputedRef, Ref } from 'vue'
import { computed, onMounted, onUnmounted, ref, unref } from 'vue'

import type { NativeFileDropEvent } from '#ui/providers/file-drop'
import { injectFileDrop } from '#ui/providers/file-drop'

type MaybeRef<T> = T | Ref<T> | ComputedRef<T>

export interface UseFileDropTargetOptions {
	target: Ref<HTMLElement | null | undefined>
	disabled?: MaybeRef<boolean>
	onFiles: (files: File[]) => void | Promise<void>
	onError?: (error: unknown) => void
}

function isFileDrag(event: DragEvent) {
	const dataTransfer = event.dataTransfer
	if (!dataTransfer) return false
	if (Array.from(dataTransfer.types).includes('Files')) return true
	return Array.from(dataTransfer.items ?? []).some((item) => item.kind === 'file')
}

function getDroppedFiles(event: DragEvent) {
	const files = Array.from(event.dataTransfer?.files ?? [])
	if (files.length > 0) return files

	return Array.from(event.dataTransfer?.items ?? [])
		.filter((item) => item.kind === 'file')
		.map((item) => item.getAsFile())
		.filter((file): file is File => file !== null)
}

export function useFileDropTarget(options: UseFileDropTargetOptions) {
	const fileDrop = injectFileDrop(null)
	const domDragCounter = ref(0)
	const domDragging = ref(false)
	const nativeDragging = ref(false)

	const disabled = computed(() => unref(options.disabled) ?? false)
	const isDragging = computed(() => domDragging.value || nativeDragging.value)

	function resetDomDrag() {
		domDragCounter.value = 0
		domDragging.value = false
	}

	function isPositionOverTarget(position: NativeFileDropEvent['position']) {
		const element = options.target.value
		if (!element) return false

		const rect = element.getBoundingClientRect()
		return (
			position.x >= rect.left &&
			position.x <= rect.right &&
			position.y >= rect.top &&
			position.y <= rect.bottom
		)
	}

	function canHandleNativeFileDrop(event: NativeFileDropEvent) {
		return event.paths.length > 0 && !disabled.value && isPositionOverTarget(event.position)
	}

	async function handleNativeFileDrop(event: NativeFileDropEvent) {
		if (event.type === 'leave') {
			nativeDragging.value = false
			return
		}

		const canDrop = canHandleNativeFileDrop(event)

		if (event.type === 'enter' || event.type === 'over') {
			nativeDragging.value = canDrop
			return
		}

		nativeDragging.value = false
		if (!canDrop || !fileDrop) return

		try {
			const files = await fileDrop.createFilesFromNativePaths(event.paths)
			await options.onFiles(files)
		} catch (error) {
			options.onError?.(error)
		}
	}

	function handleDragEnter(event: DragEvent) {
		if (disabled.value || !isFileDrag(event)) return

		event.preventDefault()
		domDragCounter.value++
		domDragging.value = true
	}

	function handleDragOver(event: DragEvent) {
		if (disabled.value || !isFileDrag(event)) return

		event.preventDefault()
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'copy'
		}
	}

	function handleDragLeave(event: DragEvent) {
		if (!domDragging.value) return

		event.preventDefault()
		domDragCounter.value--
		if (domDragCounter.value <= 0) {
			resetDomDrag()
		}
	}

	async function handleDrop(event: DragEvent) {
		if (!domDragging.value && !isFileDrag(event)) return

		event.preventDefault()
		resetDomDrag()

		if (disabled.value) return

		const files = getDroppedFiles(event)
		if (files.length === 0) return

		try {
			await options.onFiles(files)
		} catch (error) {
			options.onError?.(error)
		}
	}

	let nativeFileDropUnlisten: (() => void) | null = null
	let unmounted = false

	async function setupNativeFileDrop() {
		if (!fileDrop) return

		let unlisten: () => void
		try {
			unlisten = await fileDrop.listenNativeFileDrop(handleNativeFileDrop)
		} catch {
			return
		}

		if (unmounted) {
			unlisten()
			return
		}

		nativeFileDropUnlisten = unlisten
	}

	onMounted(() => {
		void setupNativeFileDrop()
	})

	onUnmounted(() => {
		unmounted = true
		nativeDragging.value = false
		resetDomDrag()
		if (nativeFileDropUnlisten) {
			nativeFileDropUnlisten()
			nativeFileDropUnlisten = null
		}
	})

	return {
		isDragging,
		dropTargetProps: {
			onDragenter: handleDragEnter,
			onDragover: handleDragOver,
			onDragleave: handleDragLeave,
			onDrop: handleDrop,
		},
	}
}
