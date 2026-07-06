<template>
	<PageHeader
		:title="project.title"
		:summary="project.description"
		@contextmenu.prevent.stop="emit('contextmenu', $event)"
	>
		<template #leading>
			<PageHeaderObjectAvatarLeading :src="project.icon_url" :alt="project.title" size="96px" />
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
						:tooltip="downloadsTooltip"
						class="cursor-help"
					/>
					<PageHeaderMetadataNumberItem
						:icon="HeartIcon"
						:value="project.followers"
						label="followers"
						:tooltip="followersTooltip"
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
							{{ serverPlaying ? stopLabel : serverPlayLabel }}
						</button>
					</ButtonStyled>
					<ButtonStyled circular size="large">
						<button
							v-tooltip="addServerToInstanceLabel"
							type="button"
							:aria-label="addServerToInstanceLabel"
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
							v-tooltip="installButtonTooltip"
							type="button"
							:disabled="installButtonDisabled"
							@click="emit('install')"
						>
							<component :is="installButtonIcon" :class="installButtonIconClass" />
							{{ installButtonLabel }}
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

<script setup>
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
	ButtonStyled,
	commonMessages,
	FormattedTag,
	PageHeader,
	PageHeaderActions,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	PageHeaderMetadataTagsItem,
	PageHeaderObjectAvatarLeading,
	ProjectStatusBadge,
	ServerDetails,
	TagItem,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { capitalizeString } from '@modrinth/utils'
import { computed } from 'vue'

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	projectV3: {
		type: Object,
		default: null,
	},
	isServerProject: {
		type: Boolean,
		default: false,
	},
	installButtonLabel: {
		type: String,
		default: '',
	},
	installButtonTooltip: {
		type: String,
		default: null,
	},
	installButtonDisabled: {
		type: Boolean,
		default: false,
	},
	installButtonLoading: {
		type: Boolean,
		default: false,
	},
	installButtonInstalled: {
		type: Boolean,
		default: false,
	},
	serverProjectSelected: {
		type: Boolean,
		default: false,
	},
	serverPlaying: {
		type: Boolean,
		default: false,
	},
	serverInstallLoading: {
		type: Boolean,
		default: false,
	},
	stopLabel: {
		type: String,
		required: true,
	},
	playLabel: {
		type: String,
		required: true,
	},
	installingLabel: {
		type: String,
		required: true,
	},
	addServerToInstanceLabel: {
		type: String,
		required: true,
	},
})

const emit = defineEmits([
	'contextmenu',
	'category',
	'install',
	'playServer',
	'stopServer',
	'addServerToInstance',
	'openBrowser',
	'report',
])

const { formatMessage } = useVIntl()

const downloadsTooltip = computed(() =>
	capitalizeString(
		formatMessage(commonMessages.projectDownloads, {
			count: props.project.downloads,
		}),
	),
)
const followersTooltip = computed(() =>
	capitalizeString(
		formatMessage(commonMessages.projectFollowers, {
			count: props.project.followers,
		}),
	),
)
const serverPlayLabel = computed(() =>
	props.serverInstallLoading ? props.installingLabel : props.playLabel,
)
const installButtonIcon = computed(() => {
	if (props.installButtonLoading && !props.installButtonInstalled) return SpinnerIcon
	if (!props.installButtonInstalled && !props.serverProjectSelected) return DownloadIcon
	return CheckIcon
})
const installButtonIconClass = computed(() =>
	props.installButtonLoading && !props.installButtonInstalled ? 'animate-spin' : undefined,
)
const serverMoreActions = computed(() => [
	{
		id: 'open-in-browser',
		label: 'Open in browser',
		icon: ExternalIcon,
		action: () => emit('openBrowser'),
	},
	{
		divider: true,
	},
	{
		id: 'report',
		label: 'Report',
		icon: ReportIcon,
		color: 'red',
		action: () => emit('report'),
	},
])
const projectMoreActions = computed(() => [
	{
		id: 'follow',
		label: 'Follow',
		icon: HeartIcon,
		disabled: true,
		tooltip: 'Coming soon',
		action: () => {},
	},
	{
		id: 'save',
		label: 'Save',
		icon: BookmarkIcon,
		disabled: true,
		tooltip: 'Coming soon',
		action: () => {},
	},
	{
		id: 'open-in-browser',
		label: 'Open in browser',
		icon: ExternalIcon,
		action: () => emit('openBrowser'),
	},
	{
		divider: true,
	},
	{
		id: 'report',
		label: 'Report',
		icon: ReportIcon,
		color: 'red',
		action: () => emit('report'),
	},
])
</script>
