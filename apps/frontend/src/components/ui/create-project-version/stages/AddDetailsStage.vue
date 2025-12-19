<template>
	<div class="flex flex-col gap-6 sm:w-[512px]">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version type <span class="text-red">*</span>
			</span>
			<Chips
				v-model="draftVersion.version_type"
				:items="['release', 'beta', 'alpha']"
				:never-empty="true"
				:capitalize="true"
			/>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version number <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="draftVersion.version_number"
				placeholder="Enter version number, e.g. 1.2.3-alpha.1"
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
			<span> The version number differentiates this specific version from others. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast"> Version subtitle </span>
			<input
				id="version-number"
				v-model="draftVersion.name"
				placeholder="Enter subtitle..."
				type="text"
				autocomplete="off"
				maxlength="256"
			/>
		</div>

		<template v-if="!noLoadersProject && (inferredVersionData?.loaders?.length || editingVersion)">
			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast">
						{{ usingDetectedLoaders ? 'Detected loaders' : 'Loaders' }}
					</span>

					<ButtonStyled type="transparent" size="standard">
						<button
							v-tooltip="isModpack ? 'Modpack versions cannot be edited' : undefined"
							:disabled="isModpack"
							@click="editLoaders"
						>
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div
					class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
				>
					<div class="flex flex-wrap gap-2">
						<template
							v-for="loader in draftVersionLoaders.map((selectedLoader) =>
								loaders.find((loader) => selectedLoader === loader.name),
							)"
						>
							<TagItem
								v-if="loader"
								:key="`loader-${loader.name}`"
								class="border !border-solid border-surface-5 hover:no-underline"
								:style="`--_color: var(--color-platform-${loader.name})`"
							>
								<div v-html="loader.icon"></div>
								{{ formatCategory(loader.name) }}
							</TagItem>
						</template>

						<span v-if="!draftVersion.loaders.length">No loaders selected.</span>
					</div>
				</div>
			</div>
		</template>

		<template v-if="inferredVersionData?.game_versions?.length || editingVersion">
			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast">
						{{ usingDetectedVersions ? 'Detected versions' : 'Versions' }}
					</span>

					<ButtonStyled type="transparent" size="standard">
						<button
							v-tooltip="isModpack ? 'Modpack versions cannot be edited' : undefined"
							:disabled="isModpack"
							@click="editVersions"
						>
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div
					class="flex max-h-56 flex-col gap-1.5 gap-y-4 overflow-y-auto rounded-xl border border-solid border-surface-5 p-3 py-4"
				>
					<div class="flex flex-wrap gap-2">
						<TagItem
							v-for="version in draftVersion.game_versions"
							:key="version"
							class="border !border-solid border-surface-5 hover:no-underline"
						>
							{{ version }}
						</TagItem>

						<span v-if="!draftVersion.game_versions.length">No versions selected.</span>
					</div>
				</div>
			</div>
		</template>

		<template
			v-if="
				!noEnvironmentProject &&
				((!editingVersion && inferredVersionData?.environment) ||
					(editingVersion && draftVersion.environment))
			"
		>
			<div class="flex flex-col gap-1">
				<div class="flex items-center justify-between">
					<span class="font-semibold text-contrast"> Environment </span>

					<ButtonStyled type="transparent" size="standard">
						<button @click="editEnvironment">
							<EditIcon />
							Edit
						</button>
					</ButtonStyled>
				</div>

				<div class="flex flex-col gap-1.5 gap-y-4 rounded-xl bg-surface-2 p-3 py-4">
					<div v-if="draftVersion.environment" class="flex flex-col gap-1">
						<div class="font-semibold text-contrast">
							{{ environmentCopy.title }}
						</div>
						<div class="text-sm font-medium">{{ environmentCopy.description }}</div>
					</div>

					<span v-else class="text-sm font-medium">No environment has been set.</span>
				</div>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import { EditIcon } from '@modrinth/assets'
import { ButtonStyled, Chips, TagItem } from '@modrinth/ui'
import { formatCategory } from '@modrinth/utils'

import { useGeneratedState } from '~/composables/generated'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const {
	draftVersion,
	inferredVersionData,
	projectType,
	editingVersion,
	noLoadersProject,
	noEnvironmentProject,
	modal,
} = injectManageVersionContext()

const generatedState = useGeneratedState()
const loaders = computed(() => generatedState.value.loaders)
const isModpack = computed(() => projectType.value === 'modpack')

const draftVersionLoaders = computed(() =>
	[
		...new Set([...draftVersion.value.loaders, ...(draftVersion.value.mrpack_loaders ?? [])]),
	].filter((loader) => loader !== 'mrpack'),
)

const editLoaders = () => {
	modal.value?.setStage('from-details-loaders')
}
const editVersions = () => {
	modal.value?.setStage('from-details-mc-versions')
}
const editEnvironment = () => {
	modal.value?.setStage('from-details-environment')
}

const usingDetectedVersions = computed(() => {
	if (!inferredVersionData.value?.game_versions) return false

	const versionsMatch =
		draftVersion.value.game_versions.length === inferredVersionData.value.game_versions.length &&
		draftVersion.value.game_versions.every((version) =>
			inferredVersionData.value?.game_versions?.includes(version),
		)

	return versionsMatch
})

const usingDetectedLoaders = computed(() => {
	if (!inferredVersionData.value?.loaders) return false

	const loadersMatch =
		draftVersion.value.loaders.length === inferredVersionData.value.loaders.length &&
		draftVersion.value.loaders.every((loader) =>
			inferredVersionData.value?.loaders?.includes(loader),
		)

	return loadersMatch
})

const environmentCopy = computed(() => {
	const emptyMessage = {
		title: 'No environment set',
		description: 'The environment for this version has not been specified.',
	}
	if (!draftVersion.value.environment) return emptyMessage

	const envCopy: Record<string, { title: string; description: string }> = {
		client_only: {
			title: 'Client-side only',
			description: 'All functionality is done client-side and is compatible with vanilla servers.',
		},
		server_only: {
			title: 'Server-side only',
			description: 'All functionality is done server-side and is compatible with vanilla clients.',
		},
		singleplayer_only: {
			title: 'Singleplayer only',
			description: 'Only functions in Singleplayer or when not connected to a Multiplayer server.',
		},
		dedicated_server_only: {
			title: 'Server-side only',
			description: 'All functionality is done server-side and is compatible with vanilla clients.',
		},
		client_and_server: {
			title: 'Client and server',
			description: 'Has some functionality on both the client and server, even if only partially.',
		},
		client_only_server_optional: {
			title: 'Client and server',
			description: 'Has some functionality on both the client and server, even if only partially.',
		},
		server_only_client_optional: {
			title: 'Client and server',
			description: 'Has some functionality on both the client and server, even if only partially.',
		},
		client_or_server: {
			title: 'Client and server',
			description: 'Has some functionality on both the client and server, even if only partially.',
		},
		client_or_server_prefers_both: {
			title: 'Client and server',
			description: 'Has some functionality on both the client and server, even if only partially.',
		},
		unknown: {
			title: 'Unknown environment',
			description: 'The environment for this version could not be determined.',
		},
	}

	return (
		envCopy[draftVersion.value.environment] || {
			title: 'Unknown environment',
			description: `The environment: "${draftVersion.value.environment}" is not recognized.`,
		}
	)
})
</script>
