<template>
	<div class="relative h-full w-full">
		<div
			v-if="server.moduleErrors.startup"
			class="flex w-full flex-col items-center justify-center gap-4 p-4"
		>
			<div class="flex max-w-lg flex-col items-center rounded-3xl bg-bg-raised p-6 shadow-xl">
				<div class="flex flex-col items-center text-center">
					<div class="flex flex-col items-center gap-4">
						<div class="grid place-content-center rounded-full bg-bg-orange p-4">
							<IssuesIcon class="size-12 text-orange" />
						</div>
						<h1 class="m-0 mb-2 w-fit text-4xl font-bold">Failed to load startup settings</h1>
					</div>
					<p class="text-lg text-secondary">
						We couldn't load your server's startup settings. Here's what we know:
					</p>
					<p>
						<span class="break-all font-mono">{{
							JSON.stringify(server.moduleErrors.startup.error)
						}}</span>
					</p>
					<ButtonStyled size="large" color="brand" @click="() => server.refresh(['startup'])">
						<button class="mt-6 !w-full">Retry</button>
					</ButtonStyled>
				</div>
			</div>
		</div>
		<div v-else-if="data" class="flex h-full w-full flex-col gap-4">
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
								:disabled="invocation === originalInvocation"
								class="!w-full sm:!w-auto"
								@click="resetToDefault"
							>
								<UpdatedIcon class="h-5 w-5" />
								Restore default command
							</button>
						</ButtonStyled>
					</div>
					<textarea
						id="startup-command-field"
						v-model="invocation"
						class="min-h-[270px] w-full resize-y font-[family-name:var(--mono-font)]"
					/>
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
						<div class="flex items-center gap-2">
							<input
								id="show-all-versions"
								v-model="showAllVersions"
								class="switch stylized-toggle flex-none"
								type="checkbox"
							/>
							<label for="show-all-versions" class="text-sm">Show all Java versions</label>
						</div>
						<TeleportDropdownMenu
							:id="'java-version-field'"
							v-model="jdkVersion"
							name="java-version"
							:options="displayedJavaVersions"
							placeholder="Java Version"
						/>
					</div>
					<div class="flex flex-col gap-4">
						<div class="flex flex-col gap-2">
							<span class="text-lg font-bold text-contrast">Runtime</span>
							<span> The Java runtime your server will use. </span>
						</div>
						<TeleportDropdownMenu
							:id="'runtime-field'"
							v-model="jdkBuild"
							name="runtime"
							:options="['Corretto', 'Temurin', 'GraalVM']"
							placeholder="Runtime"
						/>
					</div>
				</div>
			</div>
		</div>
		<SaveBanner
			:is-visible="!!hasUnsavedChanges"
			:server="props.server"
			:is-updating="isUpdating"
			:save="saveStartup"
			:reset="resetStartup"
		/>
	</div>
</template>

<script setup lang="ts">
import { IssuesIcon, UpdatedIcon } from '@modrinth/assets'
import { ButtonStyled, injectNotificationManager, TeleportDropdownMenu } from '@modrinth/ui'

import SaveBanner from '~/components/ui/servers/SaveBanner.vue'
import type { ModrinthServer } from '~/composables/servers/modrinth-servers.ts'

const { addNotification } = injectNotificationManager()
const props = defineProps<{
	server: ModrinthServer
}>()

await props.server.startup.fetch()

const data = computed(() => props.server.general)
const showAllVersions = ref(false)

const jdkVersionMap = [
	{ value: 'lts8', label: 'Java 8' },
	{ value: 'lts11', label: 'Java 11' },
	{ value: 'lts17', label: 'Java 17' },
	{ value: 'lts21', label: 'Java 21' },
]

const jdkBuildMap = [
	{ value: 'corretto', label: 'Corretto' },
	{ value: 'temurin', label: 'Temurin' },
	{ value: 'graal', label: 'GraalVM' },
]

const invocation = ref(props.server.startup.invocation)
const jdkVersion = ref(
	jdkVersionMap.find((v) => v.value === props.server.startup.jdk_version)?.label,
)
const jdkBuild = ref(jdkBuildMap.find((v) => v.value === props.server.startup.jdk_build)?.label)

const originalInvocation = ref(invocation.value)
const originalJdkVersion = ref(jdkVersion.value)
const originalJdkBuild = ref(jdkBuild.value)

const hasUnsavedChanges = computed(
	() =>
		invocation.value !== originalInvocation.value ||
		jdkVersion.value !== originalJdkVersion.value ||
		jdkBuild.value !== originalJdkBuild.value,
)

const isUpdating = ref(false)

const compatibleJavaVersions = computed(() => {
	const mcVersion = data.value?.mc_version ?? ''
	if (!mcVersion) return jdkVersionMap.map((v) => v.label)

	const [major, minor] = mcVersion.split('.').map(Number)

	if (major >= 1) {
		if (minor >= 20) return ['Java 21']
		if (minor >= 18) return ['Java 17', 'Java 21']
		if (minor >= 17) return ['Java 16', 'Java 17', 'Java 21']
		if (minor >= 12) return ['Java 8', 'Java 11', 'Java 17', 'Java 21']
		if (minor >= 6) return ['Java 8', 'Java 11']
	}

	return ['Java 8']
})

const displayedJavaVersions = computed(() => {
	return showAllVersions.value ? jdkVersionMap.map((v) => v.label) : compatibleJavaVersions.value
})

async function saveStartup() {
	try {
		isUpdating.value = true
		const invocationValue = invocation.value ?? ''
		const jdkVersionKey = jdkVersionMap.find((v) => v.label === jdkVersion.value)?.value
		const jdkBuildKey = jdkBuildMap.find((v) => v.label === jdkBuild.value)?.value
		await props.server.startup?.update(invocationValue, jdkVersionKey as any, jdkBuildKey as any)

		await new Promise((resolve) => setTimeout(resolve, 10))

		await props.server.refresh(['startup'])

		if (props.server.startup) {
			invocation.value = props.server.startup.invocation
			jdkVersion.value =
				jdkVersionMap.find((v) => v.value === props.server.startup?.jdk_version)?.label || ''
			jdkBuild.value =
				jdkBuildMap.find((v) => v.value === props.server.startup?.jdk_build)?.label || ''
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
			title: 'Failed to update server arguments',
			text: 'Please try again later.',
		})
	} finally {
		isUpdating.value = false
	}
}

function resetStartup() {
	invocation.value = originalInvocation.value
	jdkVersion.value = originalJdkVersion.value
	jdkBuild.value = originalJdkBuild.value
}

function resetToDefault() {
	invocation.value = originalInvocation.value ?? ''
}
</script>

<style scoped>
.stylized-toggle:checked::after {
	background: var(--color-accent-contrast) !important;
}
</style>
