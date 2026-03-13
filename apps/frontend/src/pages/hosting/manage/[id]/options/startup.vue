<template>
	<div class="relative h-full w-full">
		<div class="flex h-full w-full flex-col gap-4">
			<div
				class="rounded-2xl border-[1px] border-solid border-orange bg-bg-orange p-4 text-contrast"
			>
				These settings are for advanced users. Changing them can break your server.
			</div>

			<div class="gap-2">
				<div class="card flex flex-col gap-4">
					<div class="flex flex-col justify-between gap-4 sm:flex-row">
						<label for="startup-command-field" class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Startup command</span>
							<span> The command that runs when your server is started. </span>
						</label>
						<ButtonStyled>
							<button
								:disabled="isStartupLoading || startupCommand === defaultStartupCommand"
								class="!w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								Restore default command
							</button>
						</ButtonStyled>
					</div>
					<div class="relative">
						<StyledInput
							id="startup-command-field"
							v-model="startupCommand"
							multiline
							resize="vertical"
							input-class="min-h-[270px] font-[family-name:var(--mono-font)]"
							:disabled="isStartupLoading"
						/>
						<div
							v-if="isStartupLoading"
							class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
						>
							<SpinnerIcon class="h-6 w-6 animate-spin text-secondary" />
						</div>
					</div>
				</div>

				<div class="card flex flex-col gap-8">
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Java version</span>
							<span>
								The version of Java that your server will run on. By default, only the Java versions
								compatible with this version of Minecraft are shown. Some mods may require a
								different Java version to work properly.
							</span>
						</div>
						<div class="relative max-w-xs">
							<Combobox
								:id="'java-version-field'"
								v-model="javaVersion"
								name="java-version"
								:options="displayedJavaVersions"
								:display-value="javaVersionLabel ?? 'Java Version'"
								:disabled="isStartupLoading"
							>
								<template #dropdown-footer>
									<button
										class="flex w-full cursor-pointer items-center justify-center gap-1.5 border-0 border-t border-solid border-surface-5 bg-transparent py-3 text-center text-sm font-semibold text-secondary transition-colors hover:text-contrast"
										@mousedown.prevent
										@click="showAllVersions = !showAllVersions"
									>
										<EyeOffIcon v-if="showAllVersions" class="size-4" />
										<EyeIcon v-else class="size-4" />
										{{ showAllVersions ? 'Hide extra versions' : 'Show all versions' }}
									</button>
								</template>
							</Combobox>
							<div
								v-if="isStartupLoading"
								class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
							>
								<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
							</div>
						</div>
					</div>
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Runtime</span>
							<span> The Java runtime your server will use. </span>
						</div>
						<div class="relative max-w-xs">
							<Combobox
								:id="'runtime-field'"
								v-model="jreVendor"
								name="runtime"
								:options="JRE_VENDORS"
								:display-value="jreVendorLabel ?? 'Runtime'"
								:disabled="isStartupLoading"
							/>
							<div
								v-if="isStartupLoading"
								class="bg-bg/50 absolute inset-0 flex items-center justify-center rounded-xl"
							>
								<SpinnerIcon class="h-5 w-5 animate-spin text-secondary" />
							</div>
						</div>
					</div>
				</div>
			</div>
		</div>
		<SaveBanner
			:is-visible="!!hasUnsavedChanges"
			:server-id="serverId"
			:is-updating="isPending"
			:save="() => saveStartup()"
			:reset="resetStartup"
		/>
	</div>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { EyeIcon, EyeOffIcon, SpinnerIcon, UpdatedIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
	StyledInput,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'

const { addNotification } = injectNotificationManager()
const { server, serverId, worldId } = injectModrinthServerContext()
const client = injectModrinthClient()
const queryClient = useQueryClient()

const startupQueryKey = computed(() => ['servers', 'startup', 'v1', serverId, worldId.value])

const { data: startupData, isLoading: isStartupLoading } = useQuery({
	queryKey: startupQueryKey,
	queryFn: () => client.archon.options_v1.getStartup(serverId, worldId.value!),
	enabled: computed(() => worldId.value !== null),
})

const JAVA_VERSIONS = [
	{ value: 8, label: 'Java 8' },
	{ value: 11, label: 'Java 11' },
	{ value: 17, label: 'Java 17' },
	{ value: 21, label: 'Java 21' },
]

const JRE_VENDORS: { value: Archon.Content.v1.JreVendor; label: string }[] = [
	{ value: 'corretto', label: 'Corretto' },
	{ value: 'temurin', label: 'Temurin' },
	{ value: 'graal', label: 'GraalVM' },
]

// Saved state derived directly from query
const savedStartupCommand = computed(() => startupData.value?.startup_command ?? '')
const savedJavaVersion = computed(() => startupData.value?.java_version ?? undefined)
const savedJreVendor = computed(() => startupData.value?.jre_vendor ?? undefined)
const defaultStartupCommand = computed(
	() => startupData.value?.original_invocation ?? savedStartupCommand.value,
)

// Local form state
const startupCommand = ref('')
const javaVersion = ref<number>()
const jreVendor = ref<Archon.Content.v1.JreVendor>()

// Display labels for comboboxes
const javaVersionLabel = computed(
	() => JAVA_VERSIONS.find((v) => v.value === javaVersion.value)?.label,
)
const jreVendorLabel = computed(() => JRE_VENDORS.find((v) => v.value === jreVendor.value)?.label)

function syncFormFromData() {
	startupCommand.value = savedStartupCommand.value
	javaVersion.value = savedJavaVersion.value
	jreVendor.value = savedJreVendor.value
}

watch(
	startupData,
	(newData, oldData) => {
		if (newData && !oldData) {
			syncFormFromData()
		}
	},
	{ immediate: true },
)

const hasUnsavedChanges = computed(
	() =>
		startupCommand.value !== savedStartupCommand.value ||
		javaVersion.value !== savedJavaVersion.value ||
		jreVendor.value !== savedJreVendor.value,
)

// Java version filtering
const showAllVersions = ref(false)

const displayedJavaVersions = computed(() => {
	if (showAllVersions.value) return JAVA_VERSIONS

	const mcVersion = server.value?.mc_version ?? ''
	if (!mcVersion) return JAVA_VERSIONS

	const [, minor] = mcVersion.split('.').map(Number)

	if (minor >= 20) return JAVA_VERSIONS.filter((v) => v.value === 21)
	if (minor >= 17) return JAVA_VERSIONS.filter((v) => [17, 21].includes(v.value))
	if (minor >= 12) return JAVA_VERSIONS
	if (minor >= 6) return JAVA_VERSIONS.filter((v) => [8, 11].includes(v.value))
	return JAVA_VERSIONS.filter((v) => v.value === 8)
})

// Save mutation
const { mutate: saveStartup, isPending } = useMutation({
	mutationFn: () =>
		client.archon.options_v1.patchStartup(serverId, worldId.value!, {
			startup_command: startupCommand.value || null,
			java_version: javaVersion.value ?? null,
			jre_vendor: jreVendor.value ?? null,
		}),
	onSuccess: async () => {
		await queryClient.invalidateQueries({ queryKey: startupQueryKey.value })
		syncFormFromData()
		addNotification({
			type: 'success',
			title: 'Server settings updated',
			text: 'Your server settings were successfully changed.',
		})
	},
	onError: (error) => {
		console.error(error)
		addNotification({
			type: 'error',
			title: 'Failed to update server arguments',
			text: 'Please try again later.',
		})
	},
})

function resetStartup() {
	syncFormFromData()
}

function resetToDefault() {
	startupCommand.value = defaultStartupCommand.value
}
</script>
