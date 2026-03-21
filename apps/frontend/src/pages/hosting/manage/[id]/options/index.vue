<template>
	<div class="relative h-full w-full overflow-y-auto">
		<div v-if="data" class="flex h-full w-full flex-col">
			<div class="card flex flex-col gap-6">
				<!-- Server name -->
				<div class="flex flex-col gap-2">
					<label for="server-name-field" class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">Server name</span>
						<span> This name is only visible on Modrinth.</span>
					</label>
					<div class="flex flex-col gap-2">
						<StyledInput
							id="server-name-field"
							v-model="serverName"
							wrapper-class="w-full md:w-[50%]"
							:maxlength="48"
							@keyup.enter="!serverName && saveGeneral"
						/>
						<span v-if="!serverName" class="text-sm text-rose-400">
							Server name must be at least 1 character long.
						</span>
						<span v-if="!isValidServerName" class="text-sm text-rose-400">
							Server name can contain any character.
						</span>
					</div>
				</div>

				<!-- Hostname -->
				<div class="flex flex-col gap-2">
					<label for="server-subdomain" class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">Hostname</span>
						<span> Your friends can connect to your server using this URL. </span>
					</label>
					<div class="flex w-full items-center gap-2 md:w-[60%]">
						<StyledInput
							id="server-subdomain"
							v-model="serverSubdomain"
							wrapper-class="h-[50%] w-[63%]"
							:maxlength="32"
							@keyup.enter="saveGeneral"
						/>
						.modrinth.gg
					</div>
					<div v-if="!isValidSubdomain" class="flex flex-col text-sm text-rose-400">
						<span v-if="!isValidLengthSubdomain">
							Subdomain must be at least 5 characters long.
						</span>
						<span v-if="!isValidCharsSubdomain">
							Subdomain can only contain alphanumeric characters and dashes.
						</span>
					</div>
				</div>

				<!-- Server icon -->
				<div v-if="!data.is_medal" class="flex flex-col gap-2">
					<label for="server-icon-field" class="flex flex-col gap-2">
						<span class="text-lg font-bold text-contrast">Server icon</span>
						<span> This icon will be visible on the Minecraft server list and on Modrinth. </span>
					</label>
					<div class="flex gap-4">
						<div
							v-tooltip="'Upload a custom Icon'"
							class="group relative flex w-fit cursor-pointer items-center gap-2 rounded-xl bg-surface-2"
							@dragover.prevent="onDragOver"
							@dragleave.prevent="onDragLeave"
							@drop.prevent="onDrop"
							@click="triggerFileInput"
						>
							<input
								v-if="icon"
								id="server-icon-field"
								type="file"
								accept="image/png,image/jpeg,image/gif,image/webp"
								hidden
								@change="uploadFile"
							/>
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

				<!-- Preferences -->
				<div class="flex flex-col gap-4">
					<h2 class="m-0 text-lg font-bold text-contrast">Preferences</h2>
					<div
						v-for="(prefConfig, key) in preferences"
						:key="key"
						class="flex items-center justify-between gap-2"
					>
						<label :for="`pref-${key}`" class="flex flex-col gap-2">
							<div class="flex flex-row gap-2">
								<span class="text-lg font-bold text-contrast">{{ prefConfig.displayName }}</span>
								<div
									v-if="prefConfig.implemented === false"
									class="hidden items-center gap-1 rounded-full bg-surface-2 p-1 px-1.5 text-xs font-semibold sm:flex"
								>
									Coming Soon
								</div>
							</div>
							<span>{{ prefConfig.description }}</span>
						</label>
						<Toggle
							:id="`pref-${key}`"
							v-model="newUserPreferences[key]"
							class="flex-none"
							:disabled="prefConfig.implemented === false"
						/>
					</div>
				</div>

				<!-- Info -->
				<div class="flex flex-col gap-2">
					<h2 class="m-0 text-lg font-bold text-contrast">Info</h2>
					<div class="flex flex-col gap-2 rounded-xl bg-surface-2 p-4">
						<div
							v-for="property in infoProperties"
							:key="property.name"
							class="flex items-center justify-between gap-4"
						>
							<template v-if="property.value !== 'Unknown'">
								<span>{{ property.name }}</span>
								<CopyCode :text="property.value" />
							</template>
						</div>
					</div>
				</div>
			</div>
		</div>
		<div v-else />
		<SaveBanner
			:is-visible="!!hasUnsavedChanges && !!isValidServerName"
			:server-id="serverId"
			:is-updating="isUpdating || busyReasons.length > 0"
			:save="saveGeneral"
			:reset="resetGeneral"
		/>
	</div>
</template>

<script setup lang="ts">
import { EditIcon, TransferIcon } from '@modrinth/assets'
import {
	CopyCode,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	ServerIcon,
	StyledInput,
	Toggle,
} from '@modrinth/ui'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { server, serverId, busyReasons } = injectModrinthServerContext()
const queryClient = useQueryClient()

const data = server
const serverName = ref(data.value?.name)
const serverSubdomain = ref(data.value?.net?.domain ?? '')
const isValidLengthSubdomain = computed(() => serverSubdomain.value.length >= 5)
const isValidCharsSubdomain = computed(() => /^[a-zA-Z0-9-]+$/.test(serverSubdomain.value))
const isValidSubdomain = computed(() => isValidLengthSubdomain.value && isValidCharsSubdomain.value)
const icon = useState<string | undefined>(`server-icon-${serverId}`)

const isUpdating = ref(false)
const isValidServerName = computed(() => (serverName.value?.length ?? 0) > 0)

watch(serverName, (oldValue) => {
	if (!isValidServerName.value) {
		serverName.value = oldValue
	}
})

// Preferences
const preferences = {
	hideSubdomainLabel: {
		displayName: 'Hide subdomain label',
		description: 'When enabled, the subdomain label will be hidden from the server header.',
		implemented: true,
	},
	autoRestart: {
		displayName: 'Auto restart',
		description: 'When enabled, your server will automatically restart if it crashes.',
		implemented: false,
	},
	ramAsNumber: {
		displayName: 'RAM as bytes',
		description:
			"When enabled, RAM will be displayed as bytes instead of a percentage in your server's Overview.",
		implemented: true,
	},
	powerDontAskAgain: {
		displayName: 'Power actions confirmation',
		description: 'When enabled, you will be prompted before stopping and restarting your server.',
		implemented: true,
	},
} as const

type PreferenceKeys = keyof typeof preferences

type UserPreferences = {
	[K in PreferenceKeys]: boolean
}

const defaultPreferences: UserPreferences = {
	hideSubdomainLabel: false,
	autoRestart: false,
	ramAsNumber: false,
	powerDontAskAgain: false,
}

const userPreferences = useStorage<UserPreferences>(
	`pyro-server-${serverId}-preferences`,
	defaultPreferences,
)

const newUserPreferences = ref<UserPreferences>(JSON.parse(JSON.stringify(userPreferences.value)))

// Info properties
const infoProperties = [
	{ name: 'Server ID', value: serverId ?? 'Unknown' },
	{ name: 'Node', value: data.value?.node?.instance ?? 'Unknown' },
]

// Unsaved changes tracking (API fields + preferences)
const hasUnsavedChanges = computed(
	() =>
		(serverName.value && serverName.value !== data.value?.name) ||
		serverSubdomain.value !== data.value?.net?.domain ||
		JSON.stringify(newUserPreferences.value) !== JSON.stringify(userPreferences.value),
)

const saveGeneral = async () => {
	if (!isValidServerName.value || !isValidSubdomain.value) return

	try {
		isUpdating.value = true
		if (serverName.value !== data.value?.name) {
			await client.archon.servers_v0.updateName(serverId, serverName.value ?? '')
		}
		if (serverSubdomain.value !== data.value?.net?.domain) {
			try {
				const result = await client.archon.servers_v0.checkSubdomainAvailability(
					serverSubdomain.value,
				)
				const available = result.available

				if (!available) {
					addNotification({
						type: 'error',
						title: 'Subdomain not available',
						text: 'The subdomain you entered is already in use.',
					})
					return
				}

				await client.archon.servers_v0.changeSubdomain(serverId, serverSubdomain.value)
			} catch (error) {
				console.error('Error checking subdomain availability:', error)
				addNotification({
					type: 'error',
					title: 'Error checking availability',
					text: 'Failed to verify if the subdomain is available.',
				})
				return
			}
		}

		// Save preferences to localStorage
		userPreferences.value = { ...newUserPreferences.value }

		await new Promise((resolve) => setTimeout(resolve, 500))
		await queryClient.invalidateQueries({ queryKey: ['servers', 'detail', serverId] })
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
		isUpdating.value = false
	}
}

const resetGeneral = () => {
	serverName.value = data.value?.name || ''
	serverSubdomain.value = data.value?.net?.domain ?? ''
	newUserPreferences.value = { ...userPreferences.value }
}

const uploadFile = async (e: Event) => {
	const file = (e.target as HTMLInputElement).files?.[0]
	if (!file) {
		addNotification({
			type: 'error',
			title: 'No file selected',
			text: 'Please select a file to upload.',
		})
		return
	}

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
		if (icon.value) {
			await client.kyros.files_v0.deleteFileOrFolder('/server-icon.png', false)
			await client.kyros.files_v0.deleteFileOrFolder('/server-icon-original.png', false)
		}

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

const resetIcon = async () => {
	if (icon.value) {
		try {
			await client.kyros.files_v0.deleteFileOrFolder('/server-icon.png', false)
			await client.kyros.files_v0.deleteFileOrFolder('/server-icon-original.png', false)

			useState(`server-icon-${serverId}`).value = undefined

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
}

const onDragOver = (e: DragEvent) => {
	e.preventDefault()
}

const onDragLeave = (e: DragEvent) => {
	e.preventDefault()
}

const onDrop = (e: DragEvent) => {
	e.preventDefault()
	uploadFile(e)
}

const triggerFileInput = () => {
	const input = document.createElement('input')
	input.type = 'file'
	input.id = 'server-icon-field'
	input.accept = 'image/png,image/jpeg,image/gif,image/webp'
	input.onchange = uploadFile
	input.click()
}
</script>
