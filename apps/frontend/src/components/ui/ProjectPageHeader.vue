<template>
	<PageHeader :title="project.title" :summary="project.description">
		<template #leading>
			<Avatar :src="project.icon_url" :alt="project.title" :tint-by="project.id" size="96px" />
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
						:label="formatMessage(messages.downloadsStat, { count: project.downloads })"
						:tooltip="formatNumber(project.downloads)"
						class="cursor-help"
					/>
					<PageHeaderMetadataNumberItem
						:icon="HeartIcon"
						:value="project.followers"
						:label="formatMessage(messages.followersStat, { count: project.followers })"
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
				<ButtonStyled v-if="showEditProject" color="brand" size="large">
					<nuxt-link :to="`${projectPath}/settings`">
						<SettingsIcon />
						{{ formatMessage(messages.editProject) }}
					</nuxt-link>
				</ButtonStyled>

				<ButtonStyled :color="primaryColor" :circular="primaryLabelHidden" size="large">
					<button
						v-tooltip="
							primaryLabelHidden
								? isServerProject
									? formatMessage(commonMessages.playButton)
									: formatMessage(commonMessages.downloadButton)
								: undefined
						"
						type="button"
						:aria-label="
							isServerProject
								? formatMessage(commonMessages.playButton)
								: formatMessage(commonMessages.downloadButton)
						"
						@click="emit('primary', $event)"
					>
						<PlayIcon v-if="isServerProject" />
						<DownloadIcon v-else />
						<span v-if="!primaryLabelHidden">
							{{
								isServerProject
									? formatMessage(commonMessages.playButton)
									: formatMessage(commonMessages.downloadButton)
							}}
						</span>
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
							v-tooltip="formatMessage(messages.createServerTooltip)"
							:to="createServerTo"
							:aria-label="formatMessage(messages.serversPromoTitle)"
							@click="emit('createServer')"
						>
							<ServerPlusIcon />
						</nuxt-link>
					</ButtonStyled>
					<template #popper>
						<div class="grid max-w-[18rem] gap-2">
							<div class="flex items-center justify-between gap-4">
								<div class="flex items-center gap-2">
									<h3 class="m-0 text-base font-bold text-contrast">
										{{ formatMessage(messages.serversPromoTitle) }}
									</h3>
									<span
										class="rounded-full bg-brand-highlight px-2 py-0.5 text-xs font-bold text-brand"
									>
										{{ formatMessage(commonMessages.newBadge) }}
									</span>
								</div>
								<ButtonStyled size="small" circular>
									<button
										v-tooltip="formatMessage(messages.dontShowAgain)"
										@click="emit('dismissCreateServer')"
									>
										<XIcon aria-hidden="true" />
									</button>
								</ButtonStyled>
							</div>
							<p class="m-0 text-sm font-medium leading-tight text-secondary">
								{{ formatMessage(messages.serversPromoDescription) }}
							</p>
							<p class="m-0 text-sm font-semibold text-contrast">
								<IntlFormatted
									:message-id="messages.serversPromoPricing"
									:values="{ price: formatPrice(500, 'USD', true) }"
								>
									<template #small="{ children }">
										<small><component :is="() => children" /></small>
									</template>
								</IntlFormatted>
							</p>
						</div>
					</template>
				</Tooltip>
				<ButtonStyled v-else-if="showCreateServerAction" circular size="large">
					<nuxt-link
						v-tooltip="formatMessage(messages.createServerTooltip)"
						:to="createServerTo"
						:aria-label="formatMessage(messages.serversPromoTitle)"
						@click="emit('createServer')"
					>
						<ServerPlusIcon />
					</nuxt-link>
				</ButtonStyled>

				<ButtonStyled circular size="large">
					<button
						v-if="authUser"
						v-tooltip="
							following
								? formatMessage(commonMessages.unfollowButton)
								: formatMessage(commonMessages.followButton)
						"
						type="button"
						:aria-label="
							following
								? formatMessage(commonMessages.unfollowButton)
								: formatMessage(commonMessages.followButton)
						"
						@click="emit('follow')"
					>
						<HeartIcon :fill="following ? 'currentColor' : 'none'" />
					</button>
					<nuxt-link
						v-else
						v-tooltip="
							following
								? formatMessage(commonMessages.unfollowButton)
								: formatMessage(commonMessages.followButton)
						"
						:to="signInRoute"
						:aria-label="
							following
								? formatMessage(commonMessages.unfollowButton)
								: formatMessage(commonMessages.followButton)
						"
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
					:no-collections-label="formatMessage(messages.noCollectionsFound)"
					:create-new-collection-label="formatMessage(messages.createNewCollection)"
					:collect-project="collectProject"
					:create-collection="createCollection"
				/>

				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="formatMessage(commonMessages.moreOptionsButton)"
						:aria-label="formatMessage(commonMessages.moreOptionsButton)"
					>
						<MoreVerticalIcon />
					</TeleportOverflowMenu>
				</ButtonStyled>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	ChartIcon,
	ClipboardCopyIcon,
	DownloadIcon,
	FolderSearchIcon,
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
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	FormattedTag,
	IntlFormatted,
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
	useFormatPrice,
	useVIntl,
} from '@modrinth/ui'
import { Tooltip } from 'floating-vue'
import { computed } from 'vue'

import ProjectCollectionSaveButton from '~/components/ui/ProjectCollectionSaveButton.vue'

type HeaderProject = Pick<
	Labrinth.Projects.v2.Project,
	| 'id'
	| 'slug'
	| 'project_type'
	| 'title'
	| 'description'
	| 'status'
	| 'downloads'
	| 'followers'
	| 'categories'
> & {
	icon_url?: string | null
}

type HeaderProjectV3 = Pick<Labrinth.Projects.v3.Project, 'status' | 'minecraft_java_server'>

type HeaderCollection = {
	projects: string[]
	[key: string]: unknown
}

type SignInRoute = string | Record<string, unknown>

const props = withDefaults(
	defineProps<{
		project: HeaderProject
		projectV3?: HeaderProjectV3 | null
		authUser?: Labrinth.Users.v3.User | null
		signInRoute: SignInRoute
		collections?: HeaderCollection[]
		baseId: string
		collectProject: (...args: unknown[]) => unknown
		createCollection: (...args: unknown[]) => unknown
		isServerProject?: boolean
		showStatusBadge?: boolean
		showEditProject?: boolean
		primaryMuted?: boolean
		primaryLabelHidden?: boolean
		canCreateServer?: boolean
		showQuickServerButton?: boolean
		showCreateServerPrompt?: boolean
		following?: boolean
		saved?: boolean
		isMember?: boolean
		isStaff?: boolean
		showModerationChecklist?: boolean
		showModerationModpackRescan?: boolean
	}>(),
	{
		projectV3: null,
		authUser: null,
		collections: () => [],
		isServerProject: false,
		showStatusBadge: false,
		showEditProject: false,
		primaryMuted: false,
		primaryLabelHidden: false,
		canCreateServer: false,
		showQuickServerButton: false,
		showCreateServerPrompt: false,
		following: false,
		saved: false,
		isMember: false,
		isStaff: false,
		showModerationChecklist: false,
		showModerationModpackRescan: false,
	},
)

const emit = defineEmits<{
	category: [category: string]
	primary: [event: MouseEvent]
	createServer: []
	dismissCreateServer: []
	follow: []
	analytics: []
	moderationChecklist: []
	moderationModpackRescan: []
	techReview: []
	report: []
	copyId: []
	copyPermalink: []
}>()

const messages = defineMessages({
	createServerTooltip: {
		id: 'project.actions.create-server-tooltip',
		defaultMessage: 'Create a server',
	},
	createNewCollection: {
		id: 'project.collections.create-new',
		defaultMessage: 'Create new collection',
	},
	dontShowAgain: {
		id: 'project.actions.dont-show-again',
		defaultMessage: "Don't show again",
	},
	downloadsStat: {
		id: 'project.stats.downloads-label',
		defaultMessage: '{count, plural, one {download} other {downloads}}',
	},
	editProject: {
		id: 'project.actions.edit-project',
		defaultMessage: 'Edit project',
	},
	followersStat: {
		id: 'project.stats.followers-label',
		defaultMessage: '{count, plural, one {follower} other {followers}}',
	},
	noCollectionsFound: {
		id: 'project.collections.none-found',
		defaultMessage: 'No collections found.',
	},
	reviewProject: {
		id: 'project.actions.review-project',
		defaultMessage: 'Review project',
	},
	rescanModpack: {
		id: 'project.actions.rescan-modpack',
		defaultMessage: 'Rescan modpack',
	},
	serversPromoDescription: {
		id: 'project.actions.servers-promo.description',
		defaultMessage: 'Modrinth Hosting is the easiest way to play with your friends without hassle!',
	},
	serversPromoPricing: {
		id: 'project.actions.servers-promo.pricing',
		defaultMessage: 'Starting at {price}<small> / month</small>',
	},
	serversPromoTitle: {
		id: 'project.actions.servers-promo.title',
		defaultMessage: 'Create a server',
	},
})

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const formatPrice = useFormatPrice()

const projectPath = computed(
	() =>
		`/${props.project.project_type}/${props.project.slug ? props.project.slug : props.project.id}`,
)
const primaryColor = computed(() => (props.primaryMuted ? 'standard' : 'brand'))
const showCreateServerAction = computed(() => props.canCreateServer && props.showQuickServerButton)
const createServerTo = computed(() => `/hosting?project=${props.project.id}#plan`)

const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'analytics',
		label: formatMessage(commonMessages.analyticsButton),
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
		label: formatMessage(messages.reviewProject),
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
		label: 'Tech review',
		icon: ScanEyeIcon,
		link: `/moderation/technical-review/${props.project.id}`,
		color: 'orange',
		shown: !!props.authUser && props.isStaff,
	},
	{
		id: 'moderation-modpack-rescan',
		label: formatMessage(messages.rescanModpack),
		icon: FolderSearchIcon,
		action: () => emit('moderationModpackRescan'),
		color: 'orange',
		shown: !!props.authUser && props.isStaff && props.showModerationModpackRescan,
	},
	{
		divider: true,
		shown: !!props.authUser && props.isStaff,
	},
	{
		id: 'report',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		action: () => emit('report'),
		color: 'red',
		shown: !props.isMember,
	},
	{
		id: 'copy-id',
		label: formatMessage(commonMessages.copyIdButton),
		icon: ClipboardCopyIcon,
		action: () => emit('copyId'),
	},
	{
		id: 'copy-permalink',
		label: formatMessage(commonMessages.copyPermalinkButton),
		icon: ClipboardCopyIcon,
		action: () => emit('copyPermalink'),
	},
])
</script>
