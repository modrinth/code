import type { Labrinth } from '@modrinth/api-client'
import type { Ref } from 'vue'
import { computed, nextTick, ref, watch } from 'vue'

import { useDebugLogger } from '#ui/composables/debug-logger'
import { formatLoaderLabel } from '#ui/utils/loaders'

import type { ContentUpdaterModal } from '../../content-tab'
import type ContentDiffModal from '../components/ContentDiffModal.vue'
import type IncompatibleContentModal from '../components/IncompatibleContentModal.vue'
import type { InstallationSettingsContext } from '../providers/installation-settings'
import type { ContentDiffPreview } from '../types'

export type IncompatibleContentVariant = 'loader-change' | 'game-version-change'

export function useInstallationForm(
	ctx: InstallationSettingsContext,
	updaterModalRef: Ref<InstanceType<typeof ContentUpdaterModal> | null | undefined>,
	contentDiffModalRef?: Ref<InstanceType<typeof ContentDiffModal> | null | undefined>,
	incompatibleContentModalRef?: Ref<
		InstanceType<typeof IncompatibleContentModal> | null | undefined
	>,
) {
	const debug = useDebugLogger('InstallationSettingsForm')
	const isEditing = ref(false)
	const selectedPlatform = ctx.editingPlatformRef ?? ref(ctx.currentPlatform.value)
	const selectedGameVersion = ctx.editingGameVersionRef ?? ref(ctx.currentGameVersion.value)
	const selectedLoaderVersion = ref(0)
	const showSnapshots = ref(false)
	const isSaving = ref(false)
	const isVerifying = ref(false)
	const pendingPreview = ref<ContentDiffPreview | null>(null)
	const incompatibleContentVariant = ref<IncompatibleContentVariant | null>(null)
	let abortController: AbortController | null = null

	const gameVersionOptions = computed(() =>
		ctx.resolveGameVersions(selectedPlatform.value, showSnapshots.value),
	)

	const loaderVersionEntries = computed(() =>
		ctx.resolveLoaderVersions(selectedPlatform.value, selectedGameVersion.value),
	)

	const loaderVersionOptions = computed(() =>
		loaderVersionEntries.value.map((v, index) => ({
			value: index,
			label: v.label ?? v.id,
		})),
	)

	const loaderVersionDisplayValue = computed(() => {
		const idx = selectedLoaderVersion.value
		const e = loaderVersionEntries.value[idx]
		return idx >= 0 && e ? (e.label ?? e.id) : ''
	})

	const hasSnapshots = computed(() => ctx.resolveHasSnapshots(selectedPlatform.value))

	const formattedLoaderName = computed(() => formatLoaderLabel(selectedPlatform.value))

	const isValid = computed(() => {
		if (!selectedGameVersion.value) return false
		if (selectedPlatform.value !== 'vanilla') {
			return selectedLoaderVersion.value >= 0 && loaderVersionEntries.value.length > 0
		}
		return true
	})

	const hasChanges = computed(() => {
		if (selectedPlatform.value !== ctx.currentPlatform.value) return true
		if (selectedGameVersion.value !== ctx.currentGameVersion.value) return true
		if (
			selectedPlatform.value !== 'vanilla' &&
			loaderVersionEntries.value[selectedLoaderVersion.value]?.id !== ctx.currentLoaderVersion.value
		) {
			return true
		}
		return false
	})

	watch(selectedPlatform, () => {
		debug('selectedPlatform watch:', {
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
		})
		selectedLoaderVersion.value = 0
	})
	watch(selectedGameVersion, () => {
		debug('selectedGameVersion watch:', {
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
		})
		selectedLoaderVersion.value = 0
	})

	watch(
		[
			isEditing,
			isSaving,
			isVerifying,
			pendingPreview,
			incompatibleContentVariant,
		],
		(value, oldValue) => {
			debug('state watch:', {
				oldValue,
				value,
				selectedPlatform: selectedPlatform.value,
				selectedGameVersion: selectedGameVersion.value,
				selectedLoaderVersion: selectedLoaderVersion.value,
				isValid: isValid.value,
				hasChanges: hasChanges.value,
			})
		},
	)

	async function save() {
		debug('save: start', {
			isBusy: ctx.isBusy.value,
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
			isValid: isValid.value,
			hasChanges: hasChanges.value,
		})
		if (ctx.isBusy.value) {
			debug('save: ignored busy')
			return
		}
		isSaving.value = true
		try {
			const platformChanged = selectedPlatform.value !== ctx.currentPlatform.value
			const isModded = ctx.currentPlatform.value !== 'vanilla'
			const gameVersionChanged = selectedGameVersion.value !== ctx.currentGameVersion.value

			if (platformChanged && ctx.disableAllContent) {
				debug('save: platform changed, showing incompatible modal', {
					currentPlatform: ctx.currentPlatform.value,
					selectedPlatform: selectedPlatform.value,
				})
				isSaving.value = false
				incompatibleContentVariant.value = 'loader-change'
				await nextTick()
				debug('save: incompatible modal ref before show', {
					hasRef: !!incompatibleContentModalRef?.value,
				})
				incompatibleContentModalRef?.value?.show()
				return
			}

			if (isModded && gameVersionChanged && ctx.disableIncompatibleContent) {
				debug('save: game version changed, showing incompatible modal', {
					currentGameVersion: ctx.currentGameVersion.value,
					selectedGameVersion: selectedGameVersion.value,
				})
				isSaving.value = false
				incompatibleContentVariant.value = 'game-version-change'
				await nextTick()
				debug('save: incompatible modal ref before show', {
					hasRef: !!incompatibleContentModalRef?.value,
				})
				incompatibleContentModalRef?.value?.show()
				return
			}

			if (ctx.previewSave && isModded && gameVersionChanged) {
				debug('save: previewSave start')
				isVerifying.value = true
				abortController = new AbortController()
				const loaderVersionId =
					selectedPlatform.value !== 'vanilla'
						? (loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? null)
						: null

				let preview: ContentDiffPreview | null
				try {
					preview = await ctx.previewSave(
						selectedPlatform.value,
						selectedGameVersion.value,
						loaderVersionId,
						abortController.signal,
					)
				} finally {
					isVerifying.value = false
					abortController = null
				}

				if (preview && (preview.diffs.length > 0 || preview.hasUnknownContent)) {
					debug('save: preview returned diffs, showing content diff modal', {
						diffs: preview.diffs.length,
						hasUnknownContent: preview.hasUnknownContent,
					})
					pendingPreview.value = preview
					await nextTick()
					debug('save: content diff modal ref before show', {
						hasRef: !!contentDiffModalRef?.value,
					})
					contentDiffModalRef?.value?.show()
					return
				}
			}

			await performSave()
		} catch {
			debug('save: caught error, resetting isSaving')
			isSaving.value = false
		}
	}

	async function performSave() {
		debug('performSave: start', {
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
		})
		try {
			const loaderVersionId =
				selectedPlatform.value !== 'vanilla'
					? (loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? null)
					: null
			await ctx.save(selectedPlatform.value, selectedGameVersion.value, loaderVersionId)
			if (ctx.afterSave) await ctx.afterSave()
			isEditing.value = false
			debug('performSave: success')
		} finally {
			isSaving.value = false
			debug('performSave: finally', { isSaving: isSaving.value, isEditing: isEditing.value })
		}
	}

	async function confirmLoaderChange() {
		debug('confirmLoaderChange: start', { isBusy: ctx.isBusy.value })
		if (ctx.isBusy.value) {
			debug('confirmLoaderChange: ignored busy')
			return
		}
		try {
			if (ctx.disableAllContent) {
				await ctx.disableAllContent()
			}
			incompatibleContentVariant.value = null
			await performSave()
		} catch {
			incompatibleContentVariant.value = null
			isSaving.value = false
		}
	}

	async function confirmAutoFix() {
		debug('confirmAutoFix: start', { isBusy: ctx.isBusy.value })
		if (ctx.isBusy.value) {
			debug('confirmAutoFix: ignored busy')
			return
		}
		try {
			if (ctx.previewSave) {
				isVerifying.value = true
				abortController = new AbortController()
				const loaderVersionId =
					selectedPlatform.value !== 'vanilla'
						? (loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? null)
						: null

				let preview: ContentDiffPreview | null
				try {
					preview = await ctx.previewSave(
						selectedPlatform.value,
						selectedGameVersion.value,
						loaderVersionId,
						abortController.signal,
					)
				} finally {
					isVerifying.value = false
					abortController = null
				}

				if (preview && (preview.diffs.length > 0 || preview.hasUnknownContent)) {
					debug('confirmAutoFix: preview returned diffs', {
						diffs: preview.diffs.length,
						hasUnknownContent: preview.hasUnknownContent,
					})
					pendingPreview.value = preview
					incompatibleContentVariant.value = null
					await nextTick()
					await nextTick()
					debug('confirmAutoFix: content diff modal ref before show', {
						hasRef: !!contentDiffModalRef?.value,
					})
					contentDiffModalRef?.value?.show()
					return
				}
			}

			incompatibleContentVariant.value = null
			await performSave()
		} catch {
			incompatibleContentVariant.value = null
			isSaving.value = false
		}
	}

	async function confirmDisableConflicts() {
		debug('confirmDisableConflicts: start', { isBusy: ctx.isBusy.value })
		if (ctx.isBusy.value) {
			debug('confirmDisableConflicts: ignored busy')
			return
		}
		try {
			if (ctx.disableIncompatibleContent) {
				await ctx.disableIncompatibleContent(selectedGameVersion.value)
			}

			incompatibleContentVariant.value = null
			if (ctx.saveWithoutAutoFix) {
				const loaderVersionId =
					selectedPlatform.value !== 'vanilla'
						? (loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? null)
						: null
				await ctx.saveWithoutAutoFix(
					selectedPlatform.value,
					selectedGameVersion.value,
					loaderVersionId,
				)
				if (ctx.afterSave) await ctx.afterSave()
				isEditing.value = false
				isSaving.value = false
			} else {
				await performSave()
			}
		} catch {
			incompatibleContentVariant.value = null
			isSaving.value = false
		}
	}

	async function confirmSave() {
		debug('confirmSave: start', { isBusy: ctx.isBusy.value, hasPendingPreview: !!pendingPreview.value })
		if (ctx.isBusy.value) {
			debug('confirmSave: ignored busy')
			return
		}
		pendingPreview.value = null
		try {
			await performSave()
		} catch {
			// Error handled in performSave
		}
	}

	function cancelPreview() {
		debug('cancelPreview: start', {
			hasPendingPreview: !!pendingPreview.value,
			incompatibleContentVariant: incompatibleContentVariant.value,
			isSaving: isSaving.value,
		})
		pendingPreview.value = null
		incompatibleContentVariant.value = null
		isSaving.value = false
		debug('cancelPreview: done')
	}

	function cancelEditing() {
		debug('cancelEditing: start', {
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
			currentPlatform: ctx.currentPlatform.value,
			currentGameVersion: ctx.currentGameVersion.value,
			currentLoaderVersion: ctx.currentLoaderVersion.value,
			isSaving: isSaving.value,
			isVerifying: isVerifying.value,
		})
		abortController?.abort()
		abortController = null
		isVerifying.value = false
		isSaving.value = false
		pendingPreview.value = null
		selectedPlatform.value = ctx.currentPlatform.value
		selectedGameVersion.value = ctx.currentGameVersion.value
		const currentId = ctx.currentLoaderVersion.value
		const entries = ctx.resolveLoaderVersions(
			ctx.currentPlatform.value,
			ctx.currentGameVersion.value,
		)
		selectedLoaderVersion.value = Math.max(
			entries.findIndex((e) => e.id === currentId),
			0,
		)
		isEditing.value = false
		debug('cancelEditing: done', {
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
			selectedLoaderVersion: selectedLoaderVersion.value,
			entries: entries.length,
			isEditing: isEditing.value,
		})
	}

	// Modpack updater state
	const updatingModpack = ref(false)
	const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
	const loadingVersions = ref(false)
	const loadingChangelog = ref(false)

	watch([updatingModpack, loadingVersions, loadingChangelog], (value, oldValue) => {
		debug('updater state watch:', {
			oldValue,
			value,
			versions: updatingProjectVersions.value.length,
			selectedPlatform: selectedPlatform.value,
			selectedGameVersion: selectedGameVersion.value,
		})
	})

	async function handleChangeModpackVersion() {
		debug('handleChangeModpackVersion: start', {
			isBusy: ctx.isBusy.value,
			currentVersionId: ctx.updaterModalProps.value.currentVersionId,
			hasUpdaterRef: !!updaterModalRef.value,
		})
		if (ctx.isBusy.value) {
			debug('handleChangeModpackVersion: ignored busy')
			return
		}
		updatingModpack.value = true
		loadingChangelog.value = false

		const cached = ctx.getCachedModpackVersions()
		if (cached) {
			debug('handleChangeModpackVersion: using cached versions', { count: cached.length })
			updatingProjectVersions.value = [...cached].sort(
				(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
			)
			loadingVersions.value = false
		} else {
			debug('handleChangeModpackVersion: no cached versions')
			updatingProjectVersions.value = []
			loadingVersions.value = true
		}

		await nextTick()
		debug('handleChangeModpackVersion: showing updater modal', {
			hasUpdaterRef: !!updaterModalRef.value,
			versions: updatingProjectVersions.value.length,
		})
		updaterModalRef.value?.show(ctx.updaterModalProps.value.currentVersionId || undefined)

		if (!cached) {
			try {
				debug('handleChangeModpackVersion: fetching versions')
				const versions = await ctx.fetchModpackVersions()
				versions.sort(
					(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
				)
				updatingProjectVersions.value = versions
				debug('handleChangeModpackVersion: fetched versions', { count: versions.length })
			} catch {
				// Error handled by context
			} finally {
				loadingVersions.value = false
				debug('handleChangeModpackVersion: fetch done', {
					loadingVersions: loadingVersions.value,
					versions: updatingProjectVersions.value.length,
				})
			}
		}
	}

	function spliceVersion(full: Labrinth.Versions.v2.Version) {
		const i = updatingProjectVersions.value.findIndex((v) => v.id === full.id)
		if (i !== -1) {
			const arr = [...updatingProjectVersions.value]
			arr[i] = full
			updatingProjectVersions.value = arr
		}
	}

	async function handleUpdaterVersionSelect(version: Labrinth.Versions.v2.Version) {
		debug('handleUpdaterVersionSelect:', {
			versionId: version.id,
			hasChangelog: !!version.changelog,
		})
		if (version.changelog) return
		loadingChangelog.value = true
		try {
			const full = await ctx.getVersionChangelog(version.id)
			if (full) spliceVersion(full)
		} finally {
			loadingChangelog.value = false
		}
	}

	async function handleUpdaterVersionHover(version: Labrinth.Versions.v2.Version) {
		debug('handleUpdaterVersionHover:', {
			versionId: version.id,
			hasChangelog: !!version.changelog,
		})
		if (version.changelog) return
		try {
			const full = await ctx.getVersionChangelog(version.id)
			if (full) spliceVersion(full)
		} catch {
			// silent
		}
	}

	function resetUpdateState() {
		debug('resetUpdateState: start', {
			updatingModpack: updatingModpack.value,
			versions: updatingProjectVersions.value.length,
			loadingVersions: loadingVersions.value,
			loadingChangelog: loadingChangelog.value,
		})
		updatingModpack.value = false
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
		debug('resetUpdateState: done')
	}

	async function handleUpdaterConfirm(version: Labrinth.Versions.v2.Version) {
		debug('handleUpdaterConfirm: start', { versionId: version.id, isBusy: ctx.isBusy.value })
		if (ctx.isBusy.value) {
			debug('handleUpdaterConfirm: ignored busy')
			return
		}
		try {
			await ctx.onModpackVersionConfirm(version)
		} finally {
			resetUpdateState()
			debug('handleUpdaterConfirm: done')
		}
	}

	return {
		isEditing,
		selectedPlatform,
		selectedGameVersion,
		selectedLoaderVersion,
		showSnapshots,
		isSaving,
		isVerifying,
		gameVersionOptions,
		loaderVersionOptions,
		loaderVersionDisplayValue,
		hasSnapshots,
		formattedLoaderName,
		isValid,
		hasChanges,
		save,
		pendingPreview,
		incompatibleContentVariant,
		confirmSave,
		confirmLoaderChange,
		confirmAutoFix,
		confirmDisableConflicts,
		cancelPreview,
		cancelEditing,
		updatingModpack,
		updatingProjectVersions,
		loadingVersions,
		loadingChangelog,
		handleChangeModpackVersion,
		handleUpdaterVersionSelect,
		handleUpdaterVersionHover,
		handleUpdaterConfirm,
		resetUpdateState,
	}
}
