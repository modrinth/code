<template>
	<div>
		<ProjectPageVersions
			:loaders="loaders"
			:game-versions="gameVersions"
			:versions="versions"
			:project="project"
			:show-environment-column="appSettings.featureFlags.show_version_environment_column"
			:version-link="(version) => buildProjectHref(`/project/${project.id}/version/${version.id}`)"
		>
			<template #actions="{ version }">
				<IconButton
					v-tooltip="
						!installed
							? formatMessage(commonMessages.installButton)
							: version.id !== installedVersion
								? formatMessage(commonMessages.switchToVersionButton)
								: formatMessage(messages.alreadyInstalled)
					"
					type="quiet"
					:color="installed && version.id === installedVersion ? undefined : 'green'"
					:label="
						!installed
							? formatMessage(commonMessages.installButton)
							: version.id !== installedVersion
								? formatMessage(commonMessages.switchToVersionButton)
								: formatMessage(messages.alreadyInstalled)
					"
					:disabled="installing || (installed && version.id === installedVersion)"
					@click.stop="() => install(version.id)"
				>
					<DownloadIcon v-if="!installed" />
					<SwapIcon v-else-if="installed && version.id !== installedVersion" />
					<CheckIcon v-else />
				</IconButton>
				<ButtonLink
					v-tooltip="formatMessage(commonMessages.openInBrowserButton)"
					type="quiet"
					:href="`https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`"
					target="_blank"
					:aria-label="formatMessage(commonMessages.openInBrowserButton)"
					class="!w-9 !px-0 !rounded-full"
				>
					<ExternalIcon />
				</ButtonLink>
			</template>
		</ProjectPageVersions>
	</div>
</template>

<script setup>
import { CheckIcon, DownloadIcon, ExternalIcon } from '@modrinth/assets'
import {
	ButtonLink,
	commonMessages,
	defineMessages,
	IconButton,
	injectNotificationManager,
	ProjectPageVersions,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'
import { useRoute } from 'vue-router'

import { SwapIcon } from '@/assets/icons/index.js'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get_game_versions, get_loaders } from '@/helpers/tags.js'

const { formatMessage } = useVIntl()
const appSettings = useAppSettings()

const messages = defineMessages({
	alreadyInstalled: {
		id: 'app.project.versions.already-installed',
		defaultMessage: 'Already installed',
	},
})

defineProps({
	project: {
		type: Object,
		default: () => {},
	},
	versions: {
		type: Array,
		required: true,
	},
	install: {
		type: Function,
		required: true,
	},
	installed: {
		type: Boolean,
		default: null,
	},
	installing: {
		type: Boolean,
		default: false,
	},
	instance: {
		type: Object,
		default: null,
	},
	installedVersion: {
		type: String,
		default: null,
	},
})

const { handleError } = injectNotificationManager()
const route = useRoute()

function buildProjectHref(path) {
	const params = new URLSearchParams()
	for (const [key, val] of Object.entries(route.query)) {
		if (Array.isArray(val)) {
			for (const v of val) params.append(key, v)
		} else if (val) {
			params.append(key, String(val))
		}
	}
	const qs = params.toString()
	return qs ? `${path}?${qs}` : path
}

const [loaders, gameVersions] = await Promise.all([
	get_loaders().catch(handleError).then(ref),
	get_game_versions().catch(handleError).then(ref),
])
</script>

<style scoped lang="scss">
.filter-header {
	display: flex;
	flex-wrap: wrap;
	justify-content: space-between;
	align-items: center;
	gap: 0.5rem;
	margin-bottom: 0.5rem;
}

.table-row {
	grid-template-columns: min-content 1fr 1fr 1.5fr;
}

.manage {
	display: flex;
	gap: 0.5rem;
	flex-grow: 1;
}

.card-row {
	display: flex;
	align-items: center;
	justify-content: space-between;
	background-color: var(--color-raised-bg);
}

.mod-card {
	display: flex;
	flex-direction: column;
	gap: 1rem;
	overflow: hidden;
	margin-top: 0.5rem;
}

.text-combo {
	display: flex;
	align-items: center;
	gap: 0.5rem;
}

.select {
	width: 100% !important;
	max-width: 20rem;
}

.version-link {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	text-wrap: wrap;

	.version-badge {
		display: flex;
		flex-wrap: wrap;

		.channel-indicator {
			margin-right: 0.5rem;
		}
	}
}

.stacked-text {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	align-items: flex-start;
}

.download-cell {
	width: 4rem;
	padding: 1rem;
}

.filter-checkbox {
	:deep(.checkbox) {
		border: none;
	}
}
</style>
