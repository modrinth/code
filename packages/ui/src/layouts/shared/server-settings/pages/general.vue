<template>
	<div class="relative h-full w-full">
		<div v-if="data" class="flex h-full w-full flex-col">
			<div class="flex flex-col gap-6">
				<div class="flex justify-start gap-16">
					<div class="flex max-w-[500px] grow flex-col gap-6">
						<!-- Server name -->
						<div class="flex flex-col gap-2.5">
							<label for="server-name-field" class="flex flex-col gap-2">
								<span class="text-lg font-semibold text-contrast">Server name</span>
							</label>
							<div class="flex flex-col gap-2.5">
								<StyledInput
									id="server-name-field"
									v-model="serverName"
									wrapper-class="w-full"
									:maxlength="48"
									@keyup.enter="!serverName && saveGeneral"
								/>
								<span>This name is only visible on Modrinth.</span>
								<div class="text-red font-medium">
									<span v-if="!isValidServerName"> Server name cannot be empty. </span>
								</div>
							</div>
						</div>

						<!-- Hostname -->
						<div class="flex flex-col gap-2.5">
							<label for="server-subdomain" class="flex flex-col gap-2.5">
								<span class="text-lg font-semibold text-contrast">Hostname</span>
								<div
									class="flex w-full overflow-hidden rounded-xl bg-button-bg px-3 [box-shadow:var(--shadow-inset-sm)] transition-[box-shadow] duration-100 ease-in-out focus-within:[box-shadow:0_0_0_0.25rem_var(--color-brand-shadow)]"
								>
									<div class="relative inline-flex min-h-9 items-center">
										<span
											class="pointer-events-none invisible whitespace-pre px-px text-base font-medium"
											aria-hidden="true"
											>{{ serverSubdomain || 'Enter subdomain...' }}</span
										>
										<input
											id="server-subdomain"
											:value="serverSubdomain"
											placeholder="Enter subdomain..."
											:maxlength="32"
											class="absolute left-px inset-0 bg-transparent !p-0 text-base font-medium text-primary !shadow-none transition-colors placeholder:text-secondary focus:text-contrast"
											autocomplete="off"
											@input="serverSubdomain = ($event.target as HTMLInputElement).value"
											@keyup.enter="saveGeneral"
										/>
									</div>
									<div
										class="flex min-h-9 shrink-0 select-none items-center py-2 pr-4 font-medium opacity-50 [filter:grayscale(50%)]"
										:class="!serverSubdomain ? '!ml-auto' : ''"
									>
										.modrinth.gg
									</div>
								</div>
							</label>
							<span>Your friends can connect to your server using this address.</span>
							<div v-if="!isValidSubdomain" class="text-red font-medium">
								<span v-if="!isValidLengthSubdomain">
									Subdomain must be at least 5 characters long.
								</span>
								<span v-if="!isValidCharsSubdomain">
									Subdomain can only contain alphanumeric characters and dashes.
								</span>
							</div>
						</div>
					</div>

					<EditServerIcon v-if="!data.is_medal" />
				</div>

				<!-- preferences -->
				<div
					v-for="(prefConfig, key) in preferences"
					:key="key"
					class="flex items-center justify-between gap-2"
				>
					<label :for="`pref-${key}`" class="flex flex-col gap-1">
						<div class="flex flex-row items-center gap-2">
							<span class="text-lg font-semibold text-contrast">{{ prefConfig.displayName }}</span>
							<div
								v-if="!prefConfig.implemented"
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
						:disabled="!prefConfig.implemented"
					/>
				</div>

				<!-- Info -->
				<div class="flex flex-col gap-2.5">
					<div class="text-lg m-0 font-semibold text-contrast">Info</div>
					<div class="flex flex-col gap-2.5 rounded-xl bg-surface-2 p-4">
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
			:is-visible="(!!hasUnsavedChanges && !!isValidServerName) || isUpdating"
			:server-id="serverId"
			:is-updating="isUpdating || busyReasons.length > 0"
			:save="saveGeneral"
			:reset="resetGeneral"
		/>
	</div>
</template>

<script setup lang="ts">
import { useQueryClient } from '@tanstack/vue-query'
import { useStorage } from '@vueuse/core'
import { computed, ref, watch } from 'vue'

import { CopyCode, StyledInput, Toggle } from '#ui/components'
import EditServerIcon from '#ui/components/servers/edit-server-icon/EditServerIcon.vue'
import SaveBanner from '#ui/components/servers/SaveBanner.vue'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

const { addNotification } = injectNotificationManager()
const client = injectModrinthClient()
const { server, serverId, busyReasons } = injectModrinthServerContext()
const queryClient = useQueryClient()

const data = server
const serverName = ref(data.value?.name)
const serverSubdomain = ref(data.value?.net?.domain ?? '')
const isValidLengthSubdomain = computed(() => serverSubdomain.value.length >= 5)
const isValidCharsSubdomain = computed(
	() => !serverSubdomain.value || /^[a-zA-Z0-9-]+$/.test(serverSubdomain.value),
)
const isValidSubdomain = computed(() => isValidLengthSubdomain.value && isValidCharsSubdomain.value)

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
	// autoRestart: {
	// 	displayName: 'Auto restarts',
	// 	description: 'Automatically restart the server if it crashes.',
	// 	implemented: false,
	// },
	ramAsNumber: {
		displayName: 'RAM as bytes',
		description: 'Show RAM usage in bytes instead of a percentage.',
		implemented: true,
	},
	powerDontAskAgain: {
		displayName: 'Power action confirmation',
		description: 'Ask for confirmation before stopping or restarting the server.',
		implemented: true,
	},
} as const

type PreferenceKeys = keyof typeof preferences

type UserPreferences = {
	[K in PreferenceKeys]: boolean
}

const defaultPreferences: UserPreferences = {
	hideSubdomainLabel: false,
	// autoRestart: false,
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
</script>
