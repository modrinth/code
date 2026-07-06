<template>
	<PageHeader :title="project.title" :summary="project.description">
		<template #leading>
			<PageHeaderObjectAvatarLeading :src="project.icon_url" :alt="project.title" size="96px" />
		</template>

		<template v-if="showStatusBadge" #badges>
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
						:label="downloadsLabel"
						:tooltip="downloadsTooltip"
						class="cursor-help"
					/>
					<PageHeaderMetadataNumberItem
						:icon="HeartIcon"
						:value="project.followers"
						:label="followersLabel"
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
				<ButtonStyled v-if="showEditProject" color="brand" size="large">
					<nuxt-link :to="`${projectPath}/settings`">
						<SettingsIcon />
						{{ editProjectLabel }}
					</nuxt-link>
				</ButtonStyled>

				<ButtonStyled :color="primaryColor" :circular="primaryLabelHidden" size="large">
					<button
						v-tooltip="primaryTooltip"
						type="button"
						:aria-label="primaryLabel"
						@click="emit('primary', $event)"
					>
						<PlayIcon v-if="isServerProject" />
						<DownloadIcon v-else />
						<span v-if="!primaryLabelHidden">{{ primaryLabel }}</span>
					</button>
				</ButtonStyled>

				<Tooltip
					v-if="showCreateServerAction && showCreateServerPrompt"
					theme="dismissable-prompt"
					class="inline-flex"
					:triggers="[]"
					:shown="showCreateServerPrompt"
					:auto-hide="false"
					placement="bottom-start"
				>
					<ButtonStyled circular size="large">
						<nuxt-link
							v-tooltip="createServerTooltip"
							:to="createServerTo"
							:aria-label="createServerLabel"
							@click="emit('createServer')"
						>
							<ServerPlusIcon />
						</nuxt-link>
					</ButtonStyled>
					<template #popper>
						<div class="grid max-w-[18rem] gap-2">
							<div class="flex items-center justify-between gap-4">
								<div class="flex items-center gap-2">
									<h3 class="m-0 text-base font-bold text-contrast">{{ createServerPromptTitle }}</h3>
									<span
										v-if="createServerPromptBadge"
										class="rounded-full bg-brand-highlight px-2 py-0.5 text-xs font-bold text-brand"
									>
										{{ createServerPromptBadge }}
									</span>
								</div>
								<ButtonStyled size="small" circular>
									<button v-tooltip="createServerDismissLabel" @click="emit('dismissCreateServer')">
										<XIcon aria-hidden="true" />
									</button>
								</ButtonStyled>
							</div>
							<p class="m-0 text-sm font-medium leading-tight text-secondary">
								{{ createServerPromptDescription }}
							</p>
							<p
								v-if="createServerPromptFooter"
								class="m-0 text-sm font-semibold text-contrast"
								v-html="createServerPromptFooter"
							/>
						</div>
					</template>
				</Tooltip>
				<ButtonStyled v-else-if="showCreateServerAction" circular size="large">
					<nuxt-link
						v-tooltip="createServerTooltip"
						:to="createServerTo"
						:aria-label="createServerLabel"
						@click="emit('createServer')"
					>
						<ServerPlusIcon />
					</nuxt-link>
				</ButtonStyled>

				<ButtonStyled circular size="large">
					<button
						v-if="authUser"
						v-tooltip="followTooltip"
						type="button"
						:aria-label="followLabel"
						@click="emit('follow')"
					>
						<HeartIcon :fill="following ? 'currentColor' : 'none'" />
					</button>
					<nuxt-link
						v-else
						v-tooltip="followTooltip"
						:to="signInRoute"
						:aria-label="followLabel"
					>
						<HeartIcon :fill="following ? 'currentColor' : 'none'" />
					</nuxt-link>
				</ButtonStyled>

				<ProjectCollectionSaveButton
					:auth-user="authUser"
					:sign-in-route="signInRoute"
					:project-id="project.id"
					:collections="collections"
					:saved="saved"
					:base-id="baseId"
					:no-collections-label="noCollectionsLabel"
					:create-new-collection-label="createNewCollectionLabel"
					:collect-project="collectProject"
					:create-collection="createCollection"
				/>

				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="moreOptionsLabel"
						:aria-label="moreOptionsLabel"
					>
						<MoreVerticalIcon />
					</TeleportOverflowMenu>
				</ButtonStyled>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup>
import {
	ChartIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	HeartIcon,
	MoreVerticalIcon,
	PlayIcon,
	ReportIcon,
	ScaleIcon,
	ScanEyeIcon,
	ServerPlusIcon,
	SettingsIcon,
	XIcon,
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
import { Tooltip } from 'floating-vue'
import { computed } from 'vue'

import ProjectCollectionSaveButton from '~/components/ui/ProjectCollectionSaveButton.vue'

const props = defineProps({
	project: {
		type: Object,
		required: true,
	},
	projectV3: {
		type: Object,
		default: null,
	},
	authUser: {
		type: Object,
		default: null,
	},
	signInRoute: {
		type: [Object, String],
		required: true,
	},
	collections: {
		type: Array,
		default: () => [],
	},
	baseId: {
		type: String,
		required: true,
	},
	noCollectionsLabel: {
		type: String,
		required: true,
	},
	createNewCollectionLabel: {
		type: String,
		required: true,
	},
	collectProject: {
		type: Function,
		required: true,
	},
	createCollection: {
		type: Function,
		required: true,
	},
	isServerProject: {
		type: Boolean,
		default: false,
	},
	showStatusBadge: {
		type: Boolean,
		default: false,
	},
	showEditProject: {
		type: Boolean,
		default: false,
	},
	primaryMuted: {
		type: Boolean,
		default: false,
	},
	primaryLabelHidden: {
		type: Boolean,
		default: false,
	},
	canCreateServer: {
		type: Boolean,
		default: false,
	},
	showQuickServerButton: {
		type: Boolean,
		default: false,
	},
	showCreateServerPrompt: {
		type: Boolean,
		default: false,
	},
	following: {
		type: Boolean,
		default: false,
	},
	saved: {
		type: Boolean,
		default: false,
	},
	isMember: {
		type: Boolean,
		default: false,
	},
	isStaff: {
		type: Boolean,
		default: false,
	},
	showModerationChecklist: {
		type: Boolean,
		default: false,
	},
	createServerPromptFooter: {
		type: String,
		default: '',
	},
	labels: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits([
	'category',
	'primary',
	'createServer',
	'dismissCreateServer',
	'follow',
	'analytics',
	'moderationChecklist',
	'techReview',
	'report',
	'copyId',
	'copyPermalink',
])

const { formatMessage } = useVIntl()

const downloadsLabel = computed(() =>
	formatMessage(props.labels.downloadsStat, {
		count: props.project.downloads,
	}),
)
const followersLabel = computed(() =>
	formatMessage(props.labels.followersStat, {
		count: props.project.followers,
	}),
)
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
const projectPath = computed(
	() => `/${props.project.project_type}/${props.project.slug ? props.project.slug : props.project.id}`,
)
const primaryLabel = computed(() =>
	props.isServerProject ? props.labels.playButton : props.labels.downloadButton,
)
const primaryColor = computed(() => (props.primaryMuted ? 'standard' : 'brand'))
const primaryTooltip = computed(() => (props.primaryLabelHidden ? primaryLabel.value : undefined))
const showCreateServerAction = computed(() => props.canCreateServer && props.showQuickServerButton)
const createServerTo = computed(() => `/hosting?project=${props.project.id}#plan`)
const createServerLabel = computed(() => props.labels.createServer)
const createServerTooltip = computed(() => props.labels.createServerTooltip)
const createServerPromptTitle = computed(() => props.labels.serversPromoTitle)
const createServerPromptDescription = computed(() => props.labels.serversPromoDescription)
const createServerPromptBadge = computed(() => props.labels.newBadge)
const createServerDismissLabel = computed(() => props.labels.dontShowAgain)
const followLabel = computed(() =>
	props.following ? props.labels.unfollowButton : props.labels.followButton,
)
const followTooltip = computed(() => followLabel.value)
const editProjectLabel = computed(() => props.labels.editProject)
const moreOptionsLabel = computed(() => props.labels.moreOptions)

const moreActions = computed(() => [
	{
		id: 'analytics',
		label: props.labels.analyticsButton,
		icon: ChartIcon,
		link: `${projectPath.value}/settings/analytics`,
		shown: !!props.authUser && props.showEditProject,
	},
	{
		divider: true,
		shown: !!props.authUser && props.showEditProject,
	},
	{
		id: 'moderation-checklist',
		label: props.labels.reviewProject,
		icon: ScaleIcon,
		action: () => emit('moderationChecklist'),
		color: 'orange',
		shown: !!props.authUser && props.isStaff && !props.showModerationChecklist,
	},
	{
		divider: true,
		shown: !!props.authUser && props.isStaff && !props.showModerationChecklist,
	},
	{
		id: 'tech-review',
		label: props.labels.techReview,
		icon: ScanEyeIcon,
		link: `/moderation/technical-review/${props.project.id}`,
		color: 'orange',
		shown: !!props.authUser && props.isStaff,
	},
	{
		divider: true,
		shown: !!props.authUser && props.isStaff,
	},
	{
		id: 'report',
		label: props.labels.reportButton,
		icon: ReportIcon,
		action: () => emit('report'),
		color: 'red',
		shown: !props.isMember,
	},
	{
		id: 'copy-id',
		label: props.labels.copyIdButton,
		icon: ClipboardCopyIcon,
		action: () => emit('copyId'),
	},
	{
		id: 'copy-permalink',
		label: props.labels.copyPermalinkButton,
		icon: ClipboardCopyIcon,
		action: () => emit('copyPermalink'),
	},
])
</script>
