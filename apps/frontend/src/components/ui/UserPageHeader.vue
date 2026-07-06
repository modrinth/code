<template>
	<PageHeader :title="user.username" :summary="summary">
		<template #leading>
			<PageHeaderUserAvatarLeading
				:src="user.avatar_url"
				:username="user.username"
				:size="isModrinthUser ? '64px' : '96px'"
			/>
		</template>

		<template v-if="isOfficialAccount || showAffiliateBadge" #badges>
			<PageHeaderBadgeItem
				v-if="isOfficialAccount"
				:icon="BadgeCheckIcon"
				:icon-props="{ fill: 'var(--color-brand-highlight)' }"
				:tooltip="labels.officialAccount"
				class="border-brand-highlight bg-brand-highlight text-brand"
			>
				{{ labels.officialAccount }}
			</PageHeaderBadgeItem>
			<PageHeaderBadgeItem
				v-if="showAffiliateBadge"
				:icon="AffiliateIcon"
				class="border-brand-highlight bg-brand-highlight text-brand"
			>
				{{ labels.affiliate }}
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
					:label="projectCountLabel"
				/>
				<PageHeaderMetadataNumberItem
					:icon="DownloadIcon"
					:value="downloads"
					:label="downloadsLabel"
					:tooltip="downloadsTooltip"
				/>
				<PageHeaderMetadataTimeItem
					:icon="CalendarIcon"
					:date="user.created"
					:label="labels.joined"
					:tooltip="joinedTooltip"
				/>
			</PageHeaderMetadata>
		</template>

		<template #actions>
			<PageHeaderActions>
				<ButtonStyled v-if="isSelf" size="large">
					<nuxt-link to="/settings/profile">
						<EditIcon />
						{{ labels.edit }}
					</nuxt-link>
				</ButtonStyled>
				<ButtonStyled circular size="large" type="transparent">
					<TeleportOverflowMenu
						:options="moreActions"
						:tooltip="labels.moreOptions"
						:aria-label="labels.moreOptions"
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
	ButtonStyled,
	PageHeader,
	PageHeaderActions,
	PageHeaderBadgeItem,
	PageHeaderMetadata,
	PageHeaderMetadataNumberItem,
	PageHeaderMetadataTimeItem,
	PageHeaderUserAvatarLeading,
	TeleportOverflowMenu,
} from '@modrinth/ui'
import { computed } from 'vue'

const props = defineProps({
	user: {
		type: Object,
		required: true,
	},
	summary: {
		type: String,
		default: null,
	},
	authUser: {
		type: Object,
		default: null,
	},
	isModrinthUser: {
		type: Boolean,
		default: false,
	},
	isOfficialAccount: {
		type: Boolean,
		default: false,
	},
	showAffiliateBadge: {
		type: Boolean,
		default: false,
	},
	isAffiliate: {
		type: Boolean,
		default: false,
	},
	isSelf: {
		type: Boolean,
		default: false,
	},
	isAdmin: {
		type: Boolean,
		default: false,
	},
	isStaff: {
		type: Boolean,
		default: false,
	},
	projectsCount: {
		type: Number,
		default: 0,
	},
	downloads: {
		type: Number,
		default: 0,
	},
	projectCountLabel: {
		type: String,
		required: true,
	},
	downloadsLabel: {
		type: String,
		required: true,
	},
	downloadsTooltip: {
		type: String,
		default: undefined,
	},
	joinedTooltip: {
		type: String,
		default: undefined,
	},
	labels: {
		type: Object,
		required: true,
	},
})

const emit = defineEmits([
	'manageProjects',
	'report',
	'copyId',
	'copyPermalink',
	'openBilling',
	'toggleAffiliate',
	'openInfo',
	'openAnalytics',
	'editRole',
])

const moreActions = computed(() => [
	{
		id: 'manage-projects',
		label: props.labels.manageProjects,
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
		label: props.labels.report,
		icon: ReportIcon,
		action: () => emit('report'),
		color: 'red',
		shown: props.authUser?.id !== props.user.id,
	},
	{
		id: 'copy-id',
		label: props.labels.copyId,
		icon: ClipboardCopyIcon,
		action: () => emit('copyId'),
	},
	{
		id: 'copy-permalink',
		label: props.labels.copyPermalink,
		icon: ClipboardCopyIcon,
		action: () => emit('copyPermalink'),
	},
	{
		divider: true,
		shown: props.isAdmin,
	},
	{
		id: 'open-billing',
		label: props.labels.billing,
		icon: CurrencyIcon,
		action: () => emit('openBilling'),
		shown: props.isStaff,
	},
	{
		id: 'toggle-affiliate',
		label: props.isAffiliate ? props.labels.removeAffiliate : props.labels.setAffiliate,
		icon: AffiliateIcon,
		action: () => emit('toggleAffiliate'),
		shown: props.isAdmin,
		remainOnClick: true,
		color: props.isAffiliate ? 'red' : 'orange',
	},
	{
		id: 'open-info',
		label: props.labels.info,
		icon: InfoIcon,
		action: () => emit('openInfo'),
		shown: props.isStaff,
	},
	{
		id: 'open-analytics',
		label: props.labels.analytics,
		icon: ChartIcon,
		action: () => emit('openAnalytics'),
		shown: props.isAdmin,
	},
	{
		id: 'edit-role',
		label: props.labels.editRole,
		icon: EditIcon,
		action: () => emit('editRole'),
		shown: props.isAdmin,
	},
])
</script>
