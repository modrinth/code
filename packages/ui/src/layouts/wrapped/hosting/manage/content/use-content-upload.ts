import { ModrinthApiError } from '@modrinth/api-client'
import { nextTick, type Ref, ref } from 'vue'

import type UnknownFileWarningModal from '#ui/components/modal/UnknownFileWarningModal.vue'
import { useUploadSessionUpload } from '#ui/composables/hosting/kyros-session-upload'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import type { ContentType } from '#ui/layouts/shared/content-tab/types'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const SKIP_UNKNOWN_FILE_WARNING_KEY = 'hosting-skip-unknown-file-warning'

const messages = defineMessages({
	failedToUpload: {
		id: 'hosting.content.failed-to-upload',
		defaultMessage: 'Failed to upload file',
	},
})

export function useContentUpload(options: {
	primaryType: Readonly<Ref<ContentType>>
	actionDisabled: Readonly<Ref<boolean>>
}) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { worldId, uploadState, cancelUpload } = injectModrinthServerContext()
	const uploadSession = useUploadSessionUpload({
		client,
		scope: 'content',
		worldId,
		uploadState,
		cancelUpload,
	})

	const unknownFileWarningModal = ref<InstanceType<typeof UnknownFileWarningModal> | null>()
	const unknownFileName = ref('')
	let resolveUnknownFileConfirmation: ((confirmed: boolean) => void) | null = null

	async function isFileOnModrinth(file: File) {
		const digest = await crypto.subtle.digest('SHA-1', await file.arrayBuffer())
		const hash = Array.from(new Uint8Array(digest), (byte) =>
			byte.toString(16).padStart(2, '0'),
		).join('')

		try {
			await client.labrinth.versions_v2.getVersionFromFileHash(hash, 'sha1')
			return true
		} catch (error) {
			return !(error instanceof ModrinthApiError && error.statusCode === 404)
		}
	}

	function confirmUnknownFile(fileName: string) {
		if (localStorage.getItem(SKIP_UNKNOWN_FILE_WARNING_KEY) === 'true') {
			return Promise.resolve(true)
		}

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

	function handleUnknownFileContinue(dontShowAgain: boolean) {
		if (dontShowAgain) localStorage.setItem(SKIP_UNKNOWN_FILE_WARNING_KEY, 'true')
		resolveUnknownFileWarning(true)
	}

	async function uploadFiles(files: File[]) {
		try {
			const recognized = await Promise.all(files.map(isFileOnModrinth))
			const confirmedFiles: File[] = []
			for (const [index, file] of files.entries()) {
				if (recognized[index] || (await confirmUnknownFile(file.name))) confirmedFiles.push(file)
			}
			if (confirmedFiles.length === 0) return

			await uploadSession.uploadFiles(confirmedFiles.map((file) => ({ file, filename: file.name })))
		} catch (err) {
			addNotification({
				type: 'error',
				title: formatMessage(messages.failedToUpload),
				text: err instanceof Error ? err.message : undefined,
			})
		}
	}

	function upload() {
		if (options.actionDisabled.value) return
		const input = document.createElement('input')
		input.type = 'file'
		input.multiple = true
		input.accept = options.primaryType.value === 'datapack' ? '.zip' : '.jar'
		input.onchange = () => {
			if (!input.files || !worldId.value) return
			void uploadFiles(Array.from(input.files))
		}
		input.click()
	}

	return {
		upload,
		unknownFileWarningModal,
		unknownFileName,
		resolveUnknownFileWarning,
		handleUnknownFileContinue,
	}
}
