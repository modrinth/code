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
				<Button
					v-if="isSelf && typeof editProfileLink === 'function'"
					size="xl"
					@click="editProfileLink"
				>
					<EditIcon />
					{{ formatMessage(commonMessages.editButton) }}
				</Button>
				<ButtonLink v-else-if="isSelf" size="xl" :to="editProfileLink">
					<EditIcon />
					{{ formatMessage(commonMessages.editButton) }}
				</ButtonLink>
				<TeleportOverflowMenu
					type="quiet"
					size="xl"
					:label="formatMessage(commonMessages.moreOptionsButton)"
					:tooltip="formatMessage(commonMessages.moreOptionsButton)"
					:options="moreActions"
				>
					<MoreVerticalIcon />
				</TeleportOverflowMenu>
			</PageHeaderActions>
		</template>
	</PageHeader>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AffiliateIcon,
	BadgeCheckIcon,
	BanIcon,
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
import { computed } from 'vue'

import Avatar from '#ui/components/base/Avatar.vue'
import type { OverflowMenuOption } from '#ui/components/base/buttons'
import { Button, ButtonLink, TeleportOverflowMenu } from '#ui/components/base/buttons'
import PageHeader from '#ui/components/base/page-header/index.vue'
import PageHeaderMetadata from '#ui/components/base/page-header/metadata/index.vue'
import PageHeaderMetadataNumberItem from '#ui/components/base/page-header/metadata/page-header-metadata-number-item.vue'
import PageHeaderMetadataTimeItem from '#ui/components/base/page-header/metadata/page-header-metadata-time-item.vue'
import PageHeaderActions from '#ui/components/base/page-header/page-header-actions.vue'
import PageHeaderBadgeItem from '#ui/components/base/page-header/page-header-badge-item.vue'
import { defineMessages, useFormatDateTime, useFormatNumber, useVIntl } from '#ui/composables'
import type { AuthUser } from '#ui/providers/auth'
import { commonMessages } from '#ui/utils'

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
	blockButton: {
		id: 'profile.button.block',
		defaultMessage: 'Block',
	},
	unblockButton: {
		id: 'profile.button.unblock',
		defaultMessage: 'Unblock',
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
		authUser?: AuthUser | null
		editProfileLink?: string | (() => void)
		isModrinthUser?: boolean
		isOfficialAccount?: boolean
		showAffiliateBadge?: boolean
		isAffiliate?: boolean
		isSelf?: boolean
		isAdmin?: boolean
		isStaff?: boolean
		showStaffActions?: boolean
		isBlocked?: boolean
		projectsCount?: number
		downloads?: number
	}>(),
	{
		summary: null,
		authUser: null,
		editProfileLink: '/settings/profile',
		isModrinthUser: false,
		isOfficialAccount: false,
		showAffiliateBadge: false,
		isAffiliate: false,
		isSelf: false,
		isAdmin: false,
		isStaff: false,
		showStaffActions: false,
		isBlocked: false,
		projectsCount: 0,
		downloads: 0,
	},
)

const emit = defineEmits<{
	manageProjects: []
	report: []
	block: []
	copyId: []
	copyPermalink: []
	openBilling: []
	toggleAffiliate: []
	openInfo: []
	openAnalytics: []
	editUser: []
}>()

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()
const formatDateTime = useFormatDateTime({
	timeStyle: 'short',
	dateStyle: 'long',
})
const downloadsTooltip = computed(() => formatNumber(props.downloads))
const joinedTooltip = computed(() => formatDateTime(props.user.created))

const moreActions = computed<OverflowMenuOption[]>(() => [
	{
		id: 'manage-projects',
		label: formatMessage(messages.profileManageProjectsButton),
		icon: BoxIcon,
		action: () => emit('manageProjects'),
		shown: props.isSelf,
	},
	{ type: 'divider', shown: props.isSelf },
	{
		id: 'report',
		label: formatMessage(commonMessages.reportButton),
		icon: ReportIcon,
		action: () => emit('report'),
		tone: 'red',
		shown: props.authUser?.id !== props.user.id,
	},
	{
		id: 'block',
		label: formatMessage(props.isBlocked ? messages.unblockButton : messages.blockButton),
		icon: BanIcon,
		action: () => emit('block'),
		tone: 'red',
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
	{ type: 'divider', shown: props.showStaffActions && (props.isAdmin || props.isStaff) },
	{
		id: 'open-billing',
		label: formatMessage(messages.billingButton),
		icon: CurrencyIcon,
		action: () => emit('openBilling'),
		tone: 'orange',
		shown: props.showStaffActions && props.isStaff,
	},
	{
		id: 'toggle-affiliate',
		label: props.isAffiliate
			? formatMessage(messages.removeAffiliateButton)
			: formatMessage(messages.setAffiliateButton),
		icon: AffiliateIcon,
		action: () => emit('toggleAffiliate'),
		shown: props.showStaffActions && props.isAdmin,
		remainOpen: true,
		tone: props.isAffiliate ? 'red' : 'orange',
	},
	{
		id: 'open-info',
		label: formatMessage(messages.infoButton),
		icon: InfoIcon,
		action: () => emit('openInfo'),
		tone: 'orange',
		shown: props.showStaffActions && props.isStaff,
	},
	{
		id: 'open-analytics',
		label: formatMessage(messages.analyticsButton),
		icon: ChartIcon,
		action: () => emit('openAnalytics'),
		tone: 'orange',
		shown: props.showStaffActions && props.isAdmin,
	},
	{
		id: 'edit-user',
		label: 'Edit user',
		icon: EditIcon,
		action: () => emit('editUser'),
		tone: 'orange',
		shown: props.showStaffActions && props.isAdmin,
	},
])
</script>
