import type { Labrinth } from '@modrinth/api-client'
import type { Ref } from 'vue'
import { computed, nextTick, ref, watch } from 'vue'

import { formatLoaderLabel } from '#ui/utils/loaders'

import type { ContentUpdaterModal } from '../../content-tab'
import type ContentDiffModal from '../components/ContentDiffModal.vue'
import type { InstallationSettingsContext } from '../providers/installation-settings'
import type { ContentDiffPreview } from '../types'

export function useInstallationForm(
	ctx: InstallationSettingsContext,
	updaterModalRef: Ref<InstanceType<typeof ContentUpdaterModal> | null | undefined>,
	contentDiffModalRef?: Ref<InstanceType<typeof ContentDiffModal> | null | undefined>,
) {
	const isEditing = ref(false)
	const selectedPlatform = ctx.editingPlatformRef ?? ref(ctx.currentPlatform.value)
	const selectedGameVersion = ctx.editingGameVersionRef ?? ref(ctx.currentGameVersion.value)
	const selectedLoaderVersion = ref(0)
	const showSnapshots = ref(false)
	const isSaving = ref(false)
	const isVerifying = ref(false)
	const pendingPreview = ref<ContentDiffPreview | null>(null)
	let abortController: AbortController | null = null

	const gameVersionOptions = computed(() =>
		ctx.resolveGameVersions(selectedPlatform.value, showSnapshots.value),
	)

	const loaderVersionEntries = computed(() =>
		ctx.resolveLoaderVersions(selectedPlatform.value, selectedGameVersion.value),
	)

	const loaderVersionOptions = computed(() =>
		loaderVersionEntries.value.map((v, index) => ({ value: index, label: v.id })),
	)

	const loaderVersionDisplayValue = computed(() => {
		const idx = selectedLoaderVersion.value
		return idx >= 0 && loaderVersionEntries.value[idx] ? loaderVersionEntries.value[idx].id : ''
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
		selectedLoaderVersion.value = 0
	})
	watch(selectedGameVersion, () => {
		selectedLoaderVersion.value = 0
	})

	async function save() {
		isSaving.value = true
		try {
			const isModded = ctx.currentPlatform.value !== 'vanilla'
			const gameVersionChanged = selectedGameVersion.value !== ctx.currentGameVersion.value

			if (ctx.previewSave && isModded && gameVersionChanged) {
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
					pendingPreview.value = preview
					await nextTick()
					contentDiffModalRef?.value?.show()
					return
				}
			}

			await performSave()
		} catch {
			isSaving.value = false
		}
	}

	async function performSave() {
		try {
			const loaderVersionId =
				selectedPlatform.value !== 'vanilla'
					? (loaderVersionEntries.value[selectedLoaderVersion.value]?.id ?? null)
					: null
			await ctx.save(selectedPlatform.value, selectedGameVersion.value, loaderVersionId)
			if (ctx.afterSave) await ctx.afterSave()
			isEditing.value = false
		} finally {
			isSaving.value = false
		}
	}

	async function confirmSave() {
		pendingPreview.value = null
		try {
			await performSave()
		} catch {
			// Error handled in performSave
		}
	}

	function cancelPreview() {
		pendingPreview.value = null
		isSaving.value = false
	}

	function cancelEditing() {
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
	}

	// Modpack updater state
	const updatingModpack = ref(false)
	const updatingProjectVersions = ref<Labrinth.Versions.v2.Version[]>([])
	const loadingVersions = ref(false)
	const loadingChangelog = ref(false)

	async function handleChangeModpackVersion() {
		updatingModpack.value = true
		loadingChangelog.value = false

		const cached = ctx.getCachedModpackVersions()
		if (cached) {
			updatingProjectVersions.value = [...cached].sort(
				(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
			)
			loadingVersions.value = false
		} else {
			updatingProjectVersions.value = []
			loadingVersions.value = true
		}

		await nextTick()
		updaterModalRef.value?.show(ctx.updaterModalProps.value.currentVersionId || undefined)

		if (!cached) {
			try {
				const versions = await ctx.fetchModpackVersions()
				versions.sort(
					(a, b) => new Date(b.date_published).getTime() - new Date(a.date_published).getTime(),
				)
				updatingProjectVersions.value = versions
			} catch {
				// Error handled by context
			} finally {
				loadingVersions.value = false
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
		if (version.changelog) return
		try {
			const full = await ctx.getVersionChangelog(version.id)
			if (full) spliceVersion(full)
		} catch {
			// silent
		}
	}

	function resetUpdateState() {
		updatingModpack.value = false
		updatingProjectVersions.value = []
		loadingVersions.value = false
		loadingChangelog.value = false
	}

	async function handleUpdaterConfirm(version: Labrinth.Versions.v2.Version) {
		try {
			await ctx.onModpackVersionConfirm(version)
		} finally {
			resetUpdateState()
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
		confirmSave,
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
