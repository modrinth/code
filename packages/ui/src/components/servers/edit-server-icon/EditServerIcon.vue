<template>
	<div class="flex flex-col gap-2.5">
		<span class="text-lg font-semibold text-contrast">Icon</span>
		<div class="group relative w-fit">
			<OverflowMenu
				v-tooltip="'Edit icon'"
				class="m-0 cursor-pointer appearance-none border-none bg-transparent p-0 transition-transform group-active:scale-95"
				:disabled="isIconActionLoading"
				:options="[
					{
						id: 'upload',
						action: () => triggerFileInput(),
					},
					{
						id: 'sync',
						action: () => resetIcon(),
					},
				]"
			>
				<ServerIcon
					class="size-28 transition-[filter] group-hover:brightness-[0.50]"
					:class="isIconActionLoading ? 'brightness-[0.50]' : ''"
					:image="displayIcon"
				/>
				<div
					class="absolute top-0 h-full w-full flex items-center justify-center"
					:class="isIconActionLoading ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'"
				>
					<SpinnerIcon
						v-if="isIconActionLoading"
						aria-hidden="true"
						class="h-10 w-10 animate-spin text-primary"
					/>
					<EditIcon v-else aria-hidden="true" class="h-10 w-10 text-primary" />
				</div>
				<template #upload> <UploadIcon /> Upload icon </template>
				<template #sync> <TransferIcon /> Sync icon </template>
			</OverflowMenu>
		</div>
	</div>
</template>

<script setup lang="ts">
import { EditIcon, SpinnerIcon, TransferIcon, UploadIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import { OverflowMenu, ServerIcon } from '#ui/components'
import { useServerImage } from '#ui/composables'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { serverId, server } = injectModrinthServerContext()
const queryClient = useQueryClient()
const isUploadingIcon = ref(false)
const isSyncingIcon = ref(false)
const isIconActionLoading = computed(() => isUploadingIcon.value || isSyncingIcon.value)

const {
	image: displayIcon,
	refetch: refetchRemoteIcon,
	setImage,
	clearImage,
} = useServerImage(
	serverId,
	computed(() => server.value?.upstream ?? null),
	{
		includeProjectFallback: false,
	},
)

function getStatusCode(error: unknown): number | undefined {
	const err = error as { statusCode?: number; response?: { status?: number } }
	return err.statusCode ?? err.response?.status
}

function isNotFound(error: unknown): boolean {
	return getStatusCode(error) === 404
}

const uploadFile = async (e: Event) => {
	if (isIconActionLoading.value) return

	const file = (e.target as HTMLInputElement).files?.[0]
	if (!file) {
		addNotification({
			type: 'error',
			title: 'No file selected',
			text: 'Please select a file to upload.',
		})
		return
	}

	isUploadingIcon.value = true

	try {
		const scaledFile = await new Promise<File>((resolve, reject) => {
			const canvas = document.createElement('canvas')
			const ctx = canvas.getContext('2d')
			const img = new Image()
			img.onload = () => {
				canvas.width = 64
				canvas.height = 64
				ctx?.drawImage(img, 0, 0, 64, 64)
				canvas.toBlob((blob) => {
					if (blob) {
						resolve(new File([blob], 'server-icon.png', { type: 'image/png' }))
					} else {
						reject(new Error('Canvas toBlob failed'))
					}
				}, 'image/png')
				URL.revokeObjectURL(img.src)
			}
			img.onerror = reject
			img.src = URL.createObjectURL(file)
		})

		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)

		try {
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon.png', scaledFile).promise
		} catch (scaledUploadError) {
			// Node FS may reject create when file already exists. Delete and retry once.
			try {
				await client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, '/server-icon.png', false)
			} catch (deleteError) {
				if (!isNotFound(deleteError)) {
					throw scaledUploadError
				}
			}

			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon.png', scaledFile).promise
		}

		// Keep original file in sync when possible, but don't block icon updates on failures here.
		try {
			await client.kyros.files_v0.deleteFileOrFolderWithAuth(
				fsAuth,
				'/server-icon-original.png',
				false,
			)
		} catch (deleteOriginalError) {
			if (!isNotFound(deleteOriginalError)) {
				// best effort
			}
		}

		try {
			await client.kyros.files_v0.uploadFileWithAuth(fsAuth, '/server-icon-original.png', file)
				.promise
		} catch (originalUploadError) {
			if (!isNotFound(originalUploadError)) {
				// best effort
			}
		}

		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')
		const img = new Image()
		await new Promise<void>((resolve) => {
			img.onload = () => {
				canvas.width = 512
				canvas.height = 512
				ctx?.drawImage(img, 0, 0, 512, 512)
				const dataURL = canvas.toDataURL('image/png')
				setImage(dataURL)
				queryClient.setQueriesData({ queryKey: ['servers', 'detail', serverId, 'icon'] }, dataURL)
				resolve()
				URL.revokeObjectURL(img.src)
			}
			img.src = URL.createObjectURL(file)
		})
		await refetchRemoteIcon()

		addNotification({
			type: 'success',
			title: 'Server icon updated',
			text: 'Your server icon was successfully changed.',
		})
	} catch {
		addNotification({
			type: 'error',
			title: 'Upload failed',
			text: 'Failed to upload server icon.',
		})
	} finally {
		isUploadingIcon.value = false
	}
}

const resetIcon = async () => {
	if (isIconActionLoading.value) return
	isSyncingIcon.value = true

	try {
		const fsAuth = await client.archon.servers_v0.getFilesystemAuth(serverId)
		const deleteResults = await Promise.allSettled([
			client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, '/server-icon.png', false),
			client.kyros.files_v0.deleteFileOrFolderWithAuth(fsAuth, '/server-icon-original.png', false),
		])

		for (const result of deleteResults) {
			if (result.status === 'rejected' && !isNotFound(result.reason)) {
				throw result.reason
			}
		}

		// Force default icon state across all useServerImage instances via the shared query cache.
		// Use `null` (not `undefined`) because TanStack Query v5 treats setQueriesData(undefined)
		// as a no-op. The `null` sentinel is handled by useServerImage's image computed.
		clearImage()
		await queryClient.cancelQueries({ queryKey: ['servers', 'detail', serverId, 'icon'] })
		queryClient.setQueriesData({ queryKey: ['servers', 'detail', serverId, 'icon'] }, null)

		addNotification({
			type: 'success',
			title: 'Server icon reset',
			text: 'Your server icon was successfully reset.',
		})
	} catch {
		addNotification({
			type: 'error',
			title: 'Reset failed',
			text: 'Failed to reset server icon.',
		})
	} finally {
		isSyncingIcon.value = false
	}
}

const triggerFileInput = () => {
	if (isIconActionLoading.value) return

	const input = document.createElement('input')
	input.type = 'file'
	input.id = 'server-icon-field'
	input.accept = 'image/png,image/jpeg,image/gif,image/webp'
	const cleanup = () => {
		input.remove()
		window.removeEventListener('focus', handleWindowFocus)
	}
	const handleWindowFocus = () => {
		// If picker was cancelled there is no change event; clean up on focus return.
		setTimeout(() => {
			if (!input.value) cleanup()
		}, 0)
	}
	input.onchange = async (event) => {
		try {
			await uploadFile(event)
		} finally {
			cleanup()
		}
	}
	document.body.appendChild(input)
	window.addEventListener('focus', handleWindowFocus, { once: true })
	input.click()
}
</script>
