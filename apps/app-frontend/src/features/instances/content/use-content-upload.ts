import {
	type ContentItem,
	defineMessages,
	injectNotificationManager,
	type UnknownFileWarningModal,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { type ComputedRef, nextTick, onMounted, onUnmounted, type Ref, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { add_project_from_path, is_file_on_modrinth } from '@/helpers/instance'
import { listenNativeDragDrop } from '@/helpers/native-drag-drop'
import {
	type AppSettings,
	appSettingsKeys,
	get as getSettings,
	set as setSettings,
} from '@/helpers/settings'
import type { CacheBehaviour, GameInstance } from '@/helpers/types'
import { pickFiles } from '@/platform/dialogs'

const messages = defineMessages({
	successfullyUploaded: {
		id: 'app.instance.mods.successfully-uploaded',
		defaultMessage: 'Successfully uploaded',
	},
	projectWasAdded: {
		id: 'app.instance.mods.project-was-added',
		defaultMessage: '"{name}" was added',
	},
	projectsWereAdded: {
		id: 'app.instance.mods.projects-were-added',
		defaultMessage: '{count, plural, one {# project was added} other {# projects were added}}',
	},
})

export interface ContentUploadOptions {
	instance: Ref<GameInstance>
	items: Readonly<Ref<ContentItem[]>>
	isInstanceBusy: ComputedRef<boolean>
	refetchContent: (cacheBehaviour?: CacheBehaviour) => Promise<void>
}

export function useContentUpload({
	instance,
	items,
	isInstanceBusy,
	refetchContent,
}: ContentUploadOptions) {
	const { formatMessage } = useVIntl()
	const { handleError, addNotification } = injectNotificationManager()
	const appSettings = useAppSettings()
	const queryClient = useQueryClient()

	const unknownFileWarningModal = ref<InstanceType<typeof UnknownFileWarningModal> | null>()
	const unknownFileName = ref('')
	let resolveUnknownFileConfirmation: ((confirmed: boolean) => void) | null = null

	function confirmUnknownFileInstallation(fileName: string) {
		if (!appSettings.warnOnUnknownModpacks) return Promise.resolve(true)

		unknownFileName.value = fileName
		return new Promise<boolean>((resolve) => {
			resolveUnknownFileConfirmation = resolve
			void nextTick(() => unknownFileWarningModal.value?.show())
		})
	}

	function resolveUnknownFileWarning(confirmed: boolean) {
		const resolve = resolveUnknownFileConfirmation
		resolveUnknownFileConfirmation = null
		unknownFileName.value = ''
		resolve?.(confirmed)
	}

	const disableUnknownFileWarningMutation = useMutation({
		networkMode: 'always',
		mutationKey: appSettingsKeys.update,
		scope: { id: 'app-settings' },
		mutationFn: async (_previousWarnOnUnknownModpacks: boolean) => {
			await setSettings({ ...(await getSettings()), warn_on_unknown_modpacks: false })
		},
		onMutate: async () => {
			await queryClient.cancelQueries({ queryKey: appSettingsKeys.all })
			const previous = queryClient.getQueryData<AppSettings>(appSettingsKeys.all)
			queryClient.setQueryData<AppSettings>(appSettingsKeys.all, (settings) =>
				settings ? { ...settings, warn_on_unknown_modpacks: false } : settings,
			)
			return { previous }
		},
		onError: (error, previousWarnOnUnknownModpacks, context) => {
			if (context?.previous) queryClient.setQueryData(appSettingsKeys.all, context.previous)
			appSettings.warnOnUnknownModpacks = previousWarnOnUnknownModpacks
			handleError(error)
		},
		onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
	})

	function handleUnknownFileContinue(dontShowAgain: boolean) {
		if (dontShowAgain) {
			const previousWarnOnUnknownModpacks = appSettings.warnOnUnknownModpacks
			appSettings.warnOnUnknownModpacks = false
			disableUnknownFileWarningMutation.mutate(previousWarnOnUnknownModpacks)
		}
		resolveUnknownFileWarning(true)
	}

	async function upload() {
		if (!instance.value || instance.value.content_locked) return
		await installFiles(await pickFiles())
	}

	async function installFiles(paths: string[]) {
		if (!instance.value || instance.value.content_locked) return
		const selectedFiles = paths.map((path) => ({
			path,
			filename: path.split(/[\\/]/).pop() ?? path,
		}))
		if (selectedFiles.length === 0) return

		const fileRecognition = await Promise.all(
			selectedFiles.map(({ path }) => is_file_on_modrinth(path).catch(() => true)),
		)

		const confirmedFiles: typeof selectedFiles = []
		for (const [index, file] of selectedFiles.entries()) {
			if (!fileRecognition[index] && !(await confirmUnknownFileInstallation(file.filename))) {
				continue
			}
			confirmedFiles.push(file)
		}

		const addedFiles = (
			await Promise.all(
				confirmedFiles.map(async ({ path, filename }) => {
					try {
						const installedPath = await add_project_from_path(instance.value.id, path)
						return { filename, installedPath }
					} catch (error) {
						handleError(error as Error)
						return null
					}
				}),
			)
		).filter((result): result is { filename: string; installedPath: string } => result !== null)
		const uniqueAddedFiles = [
			...new Map(addedFiles.map((file) => [file.installedPath, file])).values(),
		]

		await refetchContent('must_revalidate')

		if (uniqueAddedFiles.length === 0) return
		const names = uniqueAddedFiles.map(({ filename, installedPath }) => {
			const item = items.value.find((project) => project.file_path === installedPath)
			return item?.project?.title ?? filename
		})
		addNotification({
			type: 'success',
			title: formatMessage(messages.successfullyUploaded),
			text:
				names.length === 1
					? formatMessage(messages.projectWasAdded, { name: names[0] })
					: formatMessage(messages.projectsWereAdded, { count: names.length }),
		})
	}

	let unmounted = false
	let unlistenDragDrop: UnlistenFn | null = null

	onMounted(() => {
		void listenNativeDragDrop(async (event) => {
			if (event.type !== 'drop' || isInstanceBusy.value) return
			await installFiles(event.paths.filter((path) => !path.endsWith('.mrpack')))
		})
			.then((unlisten) => {
				if (unmounted) {
					unlisten()
					return
				}
				unlistenDragDrop = unlisten
			})
			.catch((error) => handleError(error as Error))
	})

	onUnmounted(() => {
		unmounted = true
		unlistenDragDrop?.()
	})

	return {
		unknownFileWarningModal,
		unknownFileName,
		upload,
		resolveUnknownFileWarning,
		handleUnknownFileContinue,
	}
}
