import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { type MaybeRefOrGetter, ref, toValue, watch } from 'vue'

let queue: Promise<void> = Promise.resolve()

export function useImageThumbnail(
	path: MaybeRefOrGetter<string | null | undefined>,
	size: MaybeRefOrGetter<number>,
	revision: MaybeRefOrGetter<string | number | undefined> = undefined,
) {
	const thumbnail = ref<string>()
	watch(
		[() => toValue(path), () => toValue(size), () => toValue(revision)],
		([source, pixels], [previousSource, previousPixels], onCleanup) => {
			if (source !== previousSource || pixels !== previousPixels) thumbnail.value = undefined
			if (!source) return
			let active = true
			onCleanup(() => {
				active = false
			})
			queue = queue.then(async () => {
				if (!active) return
				try {
					const url = await invoke<string>('plugin:utils|get_image_thumbnail', {
						path: source,
						size: pixels,
					})
					if (active) thumbnail.value = url
				} catch (error) {
					console.warn('Could not create image thumbnail', error)
					if (active) thumbnail.value = convertFileSrc(source)
				}
			})
		},
		{ immediate: true },
	)
	return thumbnail
}
