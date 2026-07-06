<template>
	<div>
		<ProjectPageVersions
			:loaders="loaders"
			:game-versions="gameVersions"
			:versions="versions"
			:project="project"
			:show-environment-column="themeStore.featureFlags.show_version_environment_column"
			:version-link="(version) => buildProjectHref(`/project/${project.id}/version/${version.id}`)"
		>
			<template #actions="{ version }">
				<ButtonStyled
					circular
					type="transparent"
					:color="installed && version.id === installedVersion ? 'standard' : 'green'"
				>
					<button
						v-tooltip="
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
					</button>
				</ButtonStyled>
				<ButtonStyled circular type="transparent">
					<OverflowMenu
						v-if="false"
						:options="[
							{
								id: 'install-elsewhere',
								action: () => {},
								shown: false && !!instance,
								color: 'primary',
								hoverFilled: true,
							},
							{
								id: 'open-in-browser',
								link: `https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`,
							},
						]"
						aria-label="More options"
					>
						<MoreVerticalIcon aria-hidden="true" />
						<template #install-elsewhere>
							<DownloadIcon aria-hidden="true" />
							Add to another instance
						</template>
						<template #open-in-browser>
							<ExternalIcon /> {{ formatMessage(commonMessages.openInBrowserButton) }}
						</template>
					</OverflowMenu>
					<a
						v-else
						v-tooltip="formatMessage(commonMessages.openInBrowserButton)"
						:href="`https://modrinth.com/${project.project_type}/${project.slug}/version/${version.id}`"
						target="_blank"
					>
						<ExternalIcon />
					</a>
				</ButtonStyled>
			</template>
		</ProjectPageVersions>
	</div>
</template>

<script setup>
import { CheckIcon, DownloadIcon, ExternalIcon, MoreVerticalIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	injectNotificationManager,
	OverflowMenu,
	ProjectPageVersions,
	useVIntl,
} from '@modrinth/ui'
import { ref } from 'vue'
import { useRoute } from 'vue-router'

import { SwapIcon } from '@/assets/icons/index.js'
import { get_game_versions, get_loaders } from '@/helpers/tags.js'
import { useTheming } from '@/store/theme.ts'

const { formatMessage } = useVIntl()
const themeStore = useTheming()

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
