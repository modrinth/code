<template>
	<PageHeader
		:title="project.title"
		:summary="project.description"
		@contextmenu.prevent.stop="emit('contextmenu', $event)"
	>
		<template #leading>
			<Avatar :src="project.icon_url" :alt="project.title" :tint-by="project.id" size="96px" />
		</template>

		<template v-if="project.status !== 'approved'" #badges>
			<ProjectStatusBadge :status="project.status" />
		</template>

		<template #metadata>
			<PageHeaderMetadata>
				<template v-if="isServerProject">
					<ServerDetails
						v-if="projectV3?.status !== 'draft'"
						:online-players="projectV3?.minecraft_java_server?.ping?.data?.players_online ?? 0"
						:status-online="!!projectV3?.minecraft_java_server?.ping?.data"
						:recent-plays="projectV3?.minecraft_java_server?.verified_plays_2w ?? 0"
					/>
				</template>
				<template v-else>
					<PageHeaderMetadataNumberItem
						:icon="DownloadIcon"
						:value="project.downloads"
						label="downloads"
						:tooltip="formatNumber(project.downloads)"
						class="cursor-help"
					/>
					<PageHeaderMetadataNumberItem
						:icon="HeartIcon"
						:value="project.followers"
						label="followers"
						:tooltip="formatNumber(project.followers)"
						class="cursor-help"
					/>
				</template>
				<PageHeaderMetadataTagsItem v-if="project.categories.length > 0" class="hidden md:flex">
					<TagItem
						v-for="category in project.categories"
						:key="category"
						:action="() => emit('category', category)"
					>
						<FormattedTag :tag="category" />
					</TagItem>
				</PageHeaderMetadataTagsItem>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<template v-if="isServerProject">
					<ButtonStyled :color="serverPlaying ? 'red' : 'brand'" size="large">
						<button
							type="button"
							:disabled="serverInstallLoading"
							@click="serverPlaying ? emit('stopServer') : emit('playServer')"
						>
							<StopCircleIcon v-if="serverPlaying" />
							<PlayIcon v-else />
							{{
								serverPlaying
									? formatMessage(commonMessages.stopButton)
									: serverInstallLoading
										? formatMessage(commonMessages.installingLabel)
										: formatMessage(commonMessages.playButton)
							}}
						</button>
					</ButtonStyled>
					<ButtonStyled circular size="large">
						<button
							v-tooltip="formatMessage(commonMessages.addServerToInstanceButton)"
							type="button"
							:aria-label="formatMessage(commonMessages.addServerToInstanceButton)"
							@click="emit('addServerToInstance')"
						>
							<PlusIcon />
						</button>
					</ButtonStyled>
					<ButtonStyled circular size="large" type="transparent">
						<TeleportOverflowMenu
							:options="serverMoreActions"
							tooltip="More options"
							aria-label="More options"
						>
							<MoreVerticalIcon />
						</TeleportOverflowMenu>
					</ButtonStyled>
				</template>
				<template v-else>
					<ButtonStyled color="brand" size="large">
						<button
							v-tooltip="
								installButtonInstalled ? formatMessage(messages.alreadyInstalled) : undefined
							"
							type="button"
							:disabled="installButtonDisabled"
							@click="emit('install')"
						>
							<component :is="installButtonIcon" :class="installButtonIconClass" />
							{{
								installButtonInstalled
									? formatMessage(commonMessages.installedLabel)
									: installButtonValidating
										? formatMessage(commonMessages.validatingLabel)
										: installButtonLoading
											? formatMessage(commonMessages.installingLabel)
											: serverProjectSelected
												? formatMessage(commonMessages.selectedLabel)
												: formatMessage(commonMessages.installButton)
							}}
						</button>
					</ButtonStyled>
					<ButtonStyled circular size="large" type="transparent">
						<TeleportOverflowMenu
							:options="projectMoreActions"
							tooltip="More options"
							aria-label="More options"
						>
							<MoreVerticalIcon />
						</TeleportOverflowMenu>
					</ButtonStyled>
				</template>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	BookmarkIcon,
	CheckIcon,
	DownloadIcon,
	ExternalIcon,
	HeartIcon,
	MoreVerticalIcon,
	PlayIcon,
	PlusIcon,
	ReportIcon,
	SpinnerIcon,
	StopCircleIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	FormattedTag,
	PageHeader,
	PageHeaderActions,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	PageHeaderMetadataTagsItem,
	ProjectStatusBadge,
	ServerDetails,
	TagItem,
	TeleportOverflowMenu,
	type TeleportOverflowMenuItem,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

type HeaderProject = Pick<
	Labrinth.Projects.v2.Project,
	'id' | 'title' | 'description' | 'status' | 'downloads' | 'followers' | 'categories'
> & {
	icon_url?: string | null
}

type HeaderProjectV3 = Pick<
	Labrinth.Projects.v3.Project,
	'status' | 'minecraft_java_server'
>

const props = withDefaults(defineProps<{
	project: HeaderProject
	projectV3?: HeaderProjectV3 | null
	isServerProject?: boolean
	installButtonDisabled?: boolean
	installButtonValidating?: boolean
	installButtonLoading?: boolean
	installButtonInstalled?: boolean
	serverProjectSelected?: boolean
	serverPlaying?: boolean
	serverInstallLoading?: boolean
}>(), {
	projectV3: null,
	isServerProject: false,
	installButtonDisabled: false,
	installButtonValidating: false,
	installButtonLoading: false,
	installButtonInstalled: false,
	serverProjectSelected: false,
	serverPlaying: false,
	serverInstallLoading: false,
})

const emit = defineEmits<{
	contextmenu: [event: MouseEvent]
	category: [category: string]
	install: []
	playServer: []
	stopServer: []
	addServerToInstance: []
	openBrowser: []
	report: []
}>()

const messages = defineMessages({
	alreadyInstalled: {
		id: 'app.project.install-button.already-installed',
		defaultMessage: 'This project is already installed',
	},
})

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()

const installButtonIcon = computed(() => {
	if (props.installButtonLoading && !props.installButtonInstalled) return SpinnerIcon
	if (!props.installButtonInstalled && !props.serverProjectSelected) return DownloadIcon
	return CheckIcon
})
const installButtonIconClass = computed(() =>
	props.installButtonLoading && !props.installButtonInstalled ? 'animate-spin' : undefined,
)
const serverMoreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'open-in-browser',
		label: formatMessage(commonMessages.openInModrinthButton),
		icon: ExternalIcon,
		action: () => emit('openBrowser'),
	},
	{
		divider: true,
	},
	{
		id: 'report',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		color: 'red',
		action: () => emit('report'),
	},
])
const projectMoreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'follow',
		label: formatMessage(commonMessages.followButton),
		icon: HeartIcon,
		disabled: true,
		tooltip: 'Coming soon',
		action: () => {},
	},
	{
		id: 'save',
		label: formatMessage(commonMessages.saveButton),
		icon: BookmarkIcon,
		disabled: true,
		tooltip: 'Coming soon',
		action: () => {},
	},
	{
		id: 'open-in-browser',
		label: formatMessage(commonMessages.openInModrinthButton),
		icon: ExternalIcon,
		action: () => emit('openBrowser'),
	},
	{
		divider: true,
	},
	{
		id: 'report',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		color: 'red',
		action: () => emit('report'),
	},
])
</script>
