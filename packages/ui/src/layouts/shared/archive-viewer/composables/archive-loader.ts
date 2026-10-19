import {
	markRaw,
	type MaybeRefOrGetter,
	onScopeDispose,
	ref,
	shallowRef,
	toValue,
	watch,
} from 'vue'

import type { EventSubscription } from '#ui/helpers/toolbox/misc'
import type { ArchiveUpdateTypes, RootZipTreeNode, ZipHandler } from '#ui/helpers/toolbox/zip'

import type { ArchiveLoadProgress, ArchiveLoadStage, ArchiveSource } from '../types'
import { startToolboxLogCapture } from './toolbox-logs'

export interface ArchiveLoaderOptions {
	/** Keeps archives open in the worker once they are left, so reopening them skips reading them. */
	retain?: boolean
}

/** The toolbox is loaded on demand, as it pulls in the workers, zip.js and the decompilers. */
async function getZipHandler(): Promise<ZipHandler> {
	await startToolboxLogCapture()
	const { messengerRegistry, zipHandlerType } = await import('#ui/helpers/toolbox/workers')
	return (await messengerRegistry.getMessenger(zipHandlerType)!).handler
}

/**
 * Opens an archive by URL within the zip worker, reopening it whenever the source's URL changes.
 * Only the archive's index is fetched before its tree is shown; file contents are fetched with
 * range requests as they are opened, while nested archives are read in the background.
 */
export function useArchiveLoader(
	source: MaybeRefOrGetter<ArchiveSource | null>,
	options: ArchiveLoaderOptions = {},
) {
	const stage = ref<ArchiveLoadStage>('idle')
	/** Bytes of the archive fetched so far, which keeps growing as files are opened. */
	const progress = ref<ArchiveLoadProgress>({ loaded: 0, total: null })
	const error = ref<Error | null>(null)
	const root = shallowRef<RootZipTreeNode | null>(null)
	const handler = shallowRef<ZipHandler | null>(null)
	/** Whether every nested archive has been read too. */
	const fullyLoaded = ref(false)

	let progressSubscription: EventSubscription<(update: ArchiveUpdateTypes) => void> | null = null
	/** Identifies the latest load, so an earlier one finishing late is ignored. */
	let generation = 0

	function release(node: RootZipTreeNode | null) {
		if (options.retain || !node || !handler.value) return
		handler.value.closeZipTree(node).catch(() => undefined)
	}

	function trackProgress(zip: ZipHandler, url: string) {
		progressSubscription?.cancel()
		progressSubscription = zip.onArchiveUpdate.subscribe((update) => {
			if (update.type === 'archive_download' && update.url === url) {
				progress.value = { loaded: update.current, total: update.total }
			}
		})
	}

	async function load(target: ArchiveSource | null) {
		const current = ++generation
		release(root.value)
		root.value = null
		error.value = null
		fullyLoaded.value = false
		progressSubscription?.cancel()
		progressSubscription = null

		if (!target || typeof window === 'undefined') {
			stage.value = 'idle'
			return
		}

		stage.value = 'reading'
		progress.value = { loaded: 0, total: target.size ?? null }

		try {
			const zip = await getZipHandler()
			if (current !== generation) return
			handler.value = zip
			trackProgress(zip, target.url)

			const node = await zip.getOrReadZipTree({
				data_type: 'url',
				name: target.name,
				url: target.url,
				sha512Hash: target.sha512Hash,
				rangeOptions: { size: target.size },
			})
			if (current !== generation) return

			root.value = markRaw(node)
			stage.value = 'ready'
			node.onFinishedLoading.then(() => {
				if (current === generation) fullyLoaded.value = true
			})
		} catch (e) {
			if (current !== generation) return
			error.value = e instanceof Error ? e : new Error(String(e))
			stage.value = 'error'
		}
	}

	watch(
		() => toValue(source)?.url,
		() => load(toValue(source)),
		{ immediate: true },
	)

	onScopeDispose(() => {
		generation++
		progressSubscription?.cancel()
		release(root.value)
	})

	return {
		stage,
		progress,
		error,
		root,
		handler,
		fullyLoaded,
		retry: () => load(toValue(source)),
	}
}
