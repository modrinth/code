<template>
	<div class="relative h-full w-full overflow-y-auto">
		<div class="flex h-full w-full flex-col gap-2">
			<div class="card flex flex-col gap-4">
				<label for="server-name-field" class="flex flex-col gap-2">
					<span class="text-lg font-bold text-contrast">Server name</span>
					<span>This name is only visible on Modrinth.</span>
				</label>
				<div class="flex flex-col gap-2">
					<input
						id="server-name-field"
						v-model="serverName"
						class="w-full md:w-[50%]"
						maxlength="48"
						minlength="1"
						@keyup.enter="saveGeneral"
					/>
					<span v-if="serverName.length === 0" class="text-sm text-red">
						Server name must be at least 1 character long.
					</span>
				</div>
			</div>

			<div class="card flex flex-col gap-4">
				<label for="server-subdomain" class="flex flex-col gap-2">
					<span class="text-lg font-bold text-contrast">Custom URL</span>
					<span>Your friends can connect to your server using this URL.</span>
				</label>
				<div class="flex w-full items-center gap-2 md:w-[60%]">
					<input
						id="server-subdomain"
						v-model="serverSubdomain"
						class="h-[50%] w-[63%]"
						maxlength="32"
						@keyup.enter="saveGeneral"
					/>
					.modrinth.gg
				</div>
				<div v-if="subdomainError" class="text-sm text-red">
					{{ subdomainError }}
				</div>
			</div>

			<div v-if="!server.is_medal" class="card flex flex-col gap-4">
				<label class="flex flex-col gap-2">
					<span class="text-lg font-bold text-contrast">Server icon</span>
					<span>This icon will be visible on the Minecraft server list and on Modrinth.</span>
				</label>
				<div class="flex gap-4">
					<div
						v-tooltip="'Upload a custom icon'"
						class="group relative flex w-fit cursor-pointer items-center gap-2 rounded-xl bg-table-alternateRow"
						@dragover.prevent
						@dragleave.prevent
						@drop.prevent="handleFileDrop"
						@click="triggerFileInput"
					>
						<div
							class="absolute top-0 hidden size-24 flex-col items-center justify-center rounded-xl bg-button-bg p-2 opacity-80 group-hover:flex"
						>
							<EditIcon class="h-8 w-8 text-contrast" />
						</div>
						<ServerIcon class="size-24" :image="icon" />
					</div>
					<ButtonStyled>
						<button
							v-tooltip="'Synchronize icon with installed modpack'"
							class="my-auto"
							@click="resetIcon"
						>
							<TransferIcon /> Sync icon
						</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<UnsavedChangesPopup
			:original="originalValues"
			:modified="modifiedValues"
			:saving="isSaving"
			@save="saveGeneral"
			@reset="resetGeneral"
		/>
	</div>
</template>

<script setup lang="ts">
import { EditIcon, TransferIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	ServerIcon,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const queryClient = useQueryClient()
const { server, serverId } = injectModrinthServerContext()

const serverName = ref(server.value.name)
const serverSubdomain = ref(server.value.net?.domain ?? '')
const icon = ref<string | undefined>(undefined)
const isSaving = ref(false)

const subdomainError = computed(() => {
	if (serverSubdomain.value.length === 0) return null
	if (serverSubdomain.value.length < 5) return 'Subdomain must be at least 5 characters long.'
	if (!/^[a-zA-Z0-9-]+$/.test(serverSubdomain.value))
		return 'Subdomain can only contain alphanumeric characters and dashes.'
	return null
})

const isValid = computed(() => serverName.value.length > 0 && !subdomainError.value)

const originalValues = computed(() => ({
	name: server.value.name,
	subdomain: server.value.net?.domain ?? '',
}))

const modifiedValues = computed(() => ({
	name: serverName.value,
	subdomain: serverSubdomain.value,
}))

const updateNameMutation = useMutation({
	mutationFn: (name: string) => client.archon.settings_v0.updateName(serverId, name),
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to update name',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

const updateSubdomainMutation = useMutation({
	mutationFn: async (subdomain: string) => {
		const available = await client.archon.settings_v0.checkSubdomainAvailability(subdomain)
		if (!available) throw new Error('Subdomain not available')
		await client.archon.settings_v0.updateSubdomain(serverId, subdomain)
	},
	onSuccess: () => {
		queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
	},
	onError: (err) => {
		addNotification({
			type: 'error',
			title: 'Failed to update subdomain',
			text: err instanceof Error ? err.message : 'Unknown error',
		})
	},
})

async function saveGeneral() {
	if (!isValid.value) return

	try {
		isSaving.value = true
		if (serverName.value !== server.value.name) {
			await updateNameMutation.mutateAsync(serverName.value)
		}
		if (serverSubdomain.value !== server.value.net?.domain) {
			await updateSubdomainMutation.mutateAsync(serverSubdomain.value)
		}
		addNotification({
			type: 'success',
			title: 'Server settings updated',
			text: 'Your server settings were successfully changed.',
		})
	} catch (error) {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update server settings',
			text: 'An error occurred while attempting to update your server settings.',
		})
	} finally {
		isSaving.value = false
	}
}

function resetGeneral() {
	serverName.value = server.value.name
	serverSubdomain.value = server.value.net?.domain ?? ''
}

async function handleFileUpload(file: File) {
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

	try {
		await client.kyros.files_v0.deleteFileOrFolder('/server-icon.png', false).catch(() => {})
		await client.kyros.files_v0.deleteFileOrFolder('/server-icon-original.png', false).catch(() => {})

		await client.kyros.files_v0.uploadFile('/server-icon.png', scaledFile).promise
		await client.kyros.files_v0.uploadFile('/server-icon-original.png', file).promise

		const canvas = document.createElement('canvas')
		const ctx = canvas.getContext('2d')
		const img = new Image()
		await new Promise<void>((resolve) => {
			img.onload = () => {
				canvas.width = 512
				canvas.height = 512
				ctx?.drawImage(img, 0, 0, 512, 512)
				const dataURL = canvas.toDataURL('image/png')
				useState(`server-icon-${serverId}`).value = dataURL
				icon.value = dataURL
				resolve()
				URL.revokeObjectURL(img.src)
			}
			img.src = URL.createObjectURL(file)
		})

		addNotification({
			type: 'success',
			title: 'Server icon updated',
			text: 'Your server icon was successfully changed.',
		})
	} catch (error) {
		console.error('Error uploading icon:', error)
		addNotification({
			type: 'error',
			title: 'Upload failed',
			text: 'Failed to upload server icon.',
		})
	}
}

async function resetIcon() {
	try {
		await client.kyros.files_v0.deleteFileOrFolder('/server-icon.png', false).catch(() => {})
		await client.kyros.files_v0.deleteFileOrFolder('/server-icon-original.png', false).catch(() => {})

		useState(`server-icon-${serverId}`).value = undefined
		icon.value = undefined

		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })

		addNotification({
			type: 'success',
			title: 'Server icon reset',
			text: 'Your server icon was successfully reset.',
		})
	} catch (error) {
		console.error('Error resetting icon:', error)
		addNotification({
			type: 'error',
			title: 'Reset failed',
			text: 'Failed to reset server icon.',
		})
	}
}

function handleFileDrop(e: DragEvent) {
	const file = e.dataTransfer?.files?.[0]
	if (file) handleFileUpload(file)
}

function triggerFileInput() {
	const input = document.createElement('input')
	input.type = 'file'
	input.accept = 'image/png,image/jpeg,image/gif,image/webp'
	input.onchange = (e) => {
		const file = (e.target as HTMLInputElement).files?.[0]
		if (file) handleFileUpload(file)
	}
	input.click()
}
</script>
