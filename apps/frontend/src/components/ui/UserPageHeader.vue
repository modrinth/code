<template>
	<PageHeader :title="user.username" :summary="summary">
		<template #leading>
			<Avatar
				:src="user.avatar_url"
				:alt="user.username"
				:size="isModrinthUser ? '64px' : '96px'"
				:tint-by="user.username"
				circle
			/>
		</template>

		<template v-if="isOfficialAccount || showAffiliateBadge" #badges>
			<PageHeaderBadgeItem
				v-if="isOfficialAccount"
				:icon="BadgeCheckIcon"
				:icon-props="{ fill: 'var(--color-brand-highlight)' }"
				:tooltip="formatMessage(messages.officialAccount)"
				class="border-brand-highlight bg-brand-highlight text-brand"
			>
				{{ formatMessage(messages.officialAccount) }}
			</PageHeaderBadgeItem>
			<PageHeaderBadgeItem
				v-if="showAffiliateBadge"
				:icon="AffiliateIcon"
				class="border-brand-highlight bg-brand-highlight text-brand"
			>
				{{ formatMessage(messages.affiliateLabel) }}
			</PageHeaderBadgeItem>
		</template>

		<template v-if="$slots.summary" #summary>
			<slot name="summary" />
		</template>

		<template v-if="!isModrinthUser" #metadata>
			<PageHeaderMetadata>
				<PageHeaderMetadataNumberItem
					:icon="BoxIcon"
					:value="projectsCount"
					:label="formatMessage(messages.profileProjectCountLabel, { count: projectsCount })"
				/>
				<PageHeaderMetadataNumberItem
					:icon="DownloadIcon"
					:value="downloads"
					:label="formatMessage(messages.profileDownloadCountLabel, { count: downloads })"
					:tooltip="downloadsTooltip"
				/>
				<PageHeaderMetadataTimeItem
					:icon="CalendarIcon"
					:date="user.created"
					:label="formatMessage(messages.profileJoinedLabel)"
					:tooltip="joinedTooltip"
				/>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="isSelf" size="large">
					<nuxt-link to="/settings/profile">
						<EditIcon />
						{{ formatMessage(commonMessages.editButton) }}
					</nuxt-link>
				</ButtonStyled>
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
	AffiliateIcon,
	BadgeCheckIcon,
	BoxIcon,
	CalendarIcon,
	ChartIcon,
	ClipboardCopyIcon,
	CurrencyIcon,
	DownloadIcon,
	EditIcon,
	InfoIcon,
	MoreVerticalIcon,
	ReportIcon,
} from '@modrinth/assets'
import {
	Avatar,
	ButtonStyled,
	commonMessages,
	defineMessages,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	PageHeaderMetadataTimeItem,
	TeleportOverflowMenu,
	type TeleportOverflowMenuItem,
	useFormatDateTime,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { computed } from 'vue'

const messages = defineMessages({
	affiliateLabel: {
		id: 'profile.label.affiliate',
		defaultMessage: 'Affiliate',
	},
	analyticsButton: {
		id: 'profile.button.analytics',
		defaultMessage: 'View user analytics',
	},
	billingButton: {
		id: 'profile.button.billing',
		defaultMessage: 'Manage user billing',
	},
	editRoleButton: {
		id: 'profile.button.edit-role',
		defaultMessage: 'Edit role',
	},
	infoButton: {
		id: 'profile.button.info',
		defaultMessage: 'View user details',
	},
	officialAccount: {
		id: 'profile.official-account',
		defaultMessage: 'Official Modrinth account',
	},
	profileJoinedLabel: {
		id: 'profile.label.joined',
		defaultMessage: 'Joined',
	},
	profileProjectCountLabel: {
		id: 'profile.label.project-count',
		defaultMessage: '{count, plural, one {project} other {projects}}',
	},
	profileDownloadCountLabel: {
		id: 'profile.label.download-count',
		defaultMessage: '{count, plural, one {download} other {downloads}}',
	},
	profileManageProjectsButton: {
		id: 'profile.button.manage-projects',
		defaultMessage: 'Manage projects',
	},
	removeAffiliateButton: {
		id: 'profile.button.remove-affiliate',
		defaultMessage: 'Remove as affiliate',
	},
	setAffiliateButton: {
		id: 'profile.button.set-affiliate',
		defaultMessage: 'Set as affiliate',
	},
})

const props = withDefaults(
	defineProps<{
		user: Labrinth.Users.v3.User
		summary?: string | null
		authUser?: Labrinth.Users.v3.User | null
		isModrinthUser?: boolean
		isOfficialAccount?: boolean
		showAffiliateBadge?: boolean
		isAffiliate?: boolean
		isSelf?: boolean
		isAdmin?: boolean
		isStaff?: boolean
		projectsCount?: number
		downloads?: number
	}>(),
	{
		summary: null,
		authUser: null,
		isModrinthUser: false,
		isOfficialAccount: false,
		showAffiliateBadge: false,
		isAffiliate: false,
		isSelf: false,
		isAdmin: false,
		isStaff: false,
		projectsCount: 0,
		downloads: 0,
	},
)

const emit = defineEmits<{
	manageProjects: []
	report: []
	copyId: []
	copyPermalink: []
	openBilling: []
	toggleAffiliate: []
	openInfo: []
	openAnalytics: []
	editRole: []
}>()

const { formatMessage } = useVIntl()

const formatNumber = useFormatNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const downloadsTooltip = computed(() => formatNumber(props.downloads))
const joinedTooltip = computed(() => formatDateTime(props.user.created))

const moreActions = computed<TeleportOverflowMenuItem[]>(() => [
	{
		id: 'manage-projects',
		label: formatMessage(messages.profileManageProjectsButton),
		icon: BoxIcon,
		action: () => emit('manageProjects'),
		shown: props.isSelf,
	},
	{
		divider: true,
		shown: props.isSelf,
	},
	{
		id: 'report',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		action: () => emit('report'),
		color: 'red',
		shown: props.authUser?.id !== props.user.id,
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
	{
		divider: true,
		shown: props.isAdmin,
	},
	{
		id: 'open-billing',
		label: formatMessage(messages.billingButton),
		icon: CurrencyIcon,
		action: () => emit('openBilling'),
		shown: props.isStaff,
	},
	{
		id: 'toggle-affiliate',
		label: props.isAffiliate
			? formatMessage(messages.removeAffiliateButton)
			: formatMessage(messages.setAffiliateButton),
		icon: AffiliateIcon,
		action: () => emit('toggleAffiliate'),
		shown: props.isAdmin,
		remainOnClick: true,
		color: props.isAffiliate ? 'red' : 'orange',
	},
	{
		id: 'open-info',
		label: formatMessage(messages.infoButton),
		icon: InfoIcon,
		action: () => emit('openInfo'),
		shown: props.isStaff,
	},
	{
		id: 'open-analytics',
		label: formatMessage(messages.analyticsButton),
		icon: ChartIcon,
		action: () => emit('openAnalytics'),
		shown: props.isAdmin,
	},
	{
		id: 'edit-role',
		label: formatMessage(messages.editRoleButton),
		icon: EditIcon,
		action: () => emit('editRole'),
		shown: props.isAdmin,
	},
])
</script>
