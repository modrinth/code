<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	AlphaBadge,
	BetaBadge,
	Downloads1mBadge,
	Downloads10mBadge,
	Downloads25mBadge,
	Downloads50mBadge,
	Downloads100mBadge,
	Downloads250mBadge,
	Downloads500mBadge,
	EarlyDatapackBadge,
	EarlyHostingBadge,
	EarlyModpackBadge,
	EarlyPluginBadge,
	EarlyResourcepackBadge,
	EarlyServersBadge,
	EarlyShadersBadge,
	ModeratorBadge,
	PlusBadge,
	PrideBadge,
	StaffBadge,
} from '@modrinth/assets'
import {
	defineMessage,
	defineMessages,
	type MessageDescriptor,
	useFormatNumber,
	useVIntl,
} from '@modrinth/ui'
import { UserBadge as BadgeBitflag } from '@modrinth/utils'
import { type Component, computed } from 'vue'

import UserBadge from './UserBadge.vue'

const { formatMessage } = useVIntl()
const formatNumber = useFormatNumber()

type EarlyAdopterProjectTypes =
	| 'modpack'
	| 'resourcepack'
	| 'plugin'
	| 'datapack'
	| 'shader'
	| 'server'

type BadgeCriterion =
	| {
			type: 'earliest_project_date'
			project_type: EarlyAdopterProjectTypes
			cutoff: Date
	  }
	| {
			type: 'join_date'
			cutoff: Date
	  }
	| {
			type: 'badge'
			bitflag: number
	  }
	| {
			type: 'role'
			role: Labrinth.Users.v3.Role
	  }

type Badge = {
	icon: Component
	name: MessageDescriptor
	about: MessageDescriptor[]
	criteria: BadgeCriterion[] // if any criterion matches, the badge will apply (OR logic)
	link?: {
		href: string
		message: MessageDescriptor
	}
}

const BADGES = [
	{
		icon: StaffBadge,
		name: defineMessage({
			id: 'user.profile.badge.staff.name',
			defaultMessage: 'Modrinth Team',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.staff.about.1',
				defaultMessage: `This user works for Modrinth.`,
			}),
		],
		criteria: [
			{
				type: 'role',
				role: 'admin',
			},
			{
				type: 'role',
				role: 'moderator',
			},
		],
	},
	{
		icon: ModeratorBadge,
		name: defineMessage({
			id: 'user.profile.badge.moderator.name',
			defaultMessage: 'Content Moderator',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.moderator.about.1',
				defaultMessage: `This user works for Modrinth as a Content Moderator.`,
			}),
			defineMessage({
				id: 'user.profile.badge.moderator.about.2',
				defaultMessage: `Content Moderators on Modrinth review projects, handle reports, and help keep Modrinth safe.`,
			}),
		],
		criteria: [
			{
				type: 'role',
				role: 'moderator',
			},
		],
	},
	{
		icon: AlphaBadge,
		name: defineMessage({
			id: 'user.profile.badge.alpha.name',
			defaultMessage: 'Alpha Tester',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.alpha.about.1',
				defaultMessage: `This user has been around since Modrinth Alpha, which ended in November 2020`,
			}),
		],
		criteria: [
			{
				type: 'badge',
				bitflag: BadgeBitflag.ALPHA_TESTER,
			},
			{
				type: 'join_date',
				cutoff: new Date('2020-11-30T08:00:00.000Z'),
			},
		],
	},
	{
		icon: BetaBadge,
		name: defineMessage({
			id: 'user.profile.badge.beta.name',
			defaultMessage: 'Beta Tester',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.beta.about.1',
				defaultMessage: `This user has been around since Modrinth Beta, which ended in February 2022.`,
			}),
		],
		criteria: [
			{
				type: 'join_date',
				cutoff: new Date('2022-02-27T08:00:00.000Z'),
			},
		],
		link: {
			href: 'https://modrinth.com/news/article/modrinth-beta/',
			message: defineMessage({
				id: 'user.profile.badge.beta.link',
				defaultMessage: `Click to read about the launch of Modrinth Beta.`,
			}),
		},
	},
	{
		icon: PlusBadge,
		name: defineMessage({
			id: 'user.profile.badge.plus.name',
			defaultMessage: 'Modrinth+ Member',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.plus.about.1',
				defaultMessage: `This user is going the extra mile to support Modrinth and the creators on the platform.`,
			}),
		],
		criteria: [
			{
				type: 'badge',
				bitflag: BadgeBitflag.MIDAS,
			},
		],
		link: {
			href: 'https://modrinth.com/plus',
			message: defineMessage({
				id: 'user.profile.badge.plus.link',
				defaultMessage: `Click to learn more about how you can become a member.`,
			}),
		},
	},
	{
		icon: PrideBadge,
		name: defineMessage({
			id: 'user.profile.badge.pride.name',
			defaultMessage: 'Pride Fundraiser Supporter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.pride.about.1',
				defaultMessage: `This user participated in at least one of Modrinth's Pride fundraisers for the LGBTQ+ community.`,
			}),
		],
		criteria: [
			{
				type: 'badge',
				bitflag: BadgeBitflag.PRIDE,
			},
		],
		link: {
			href: 'https://modrinth.com/pride',
			message: defineMessage({
				id: 'user.profile.badge.pride.link',
				defaultMessage: `Click to visit our latest Pride fundraiser.`,
			}),
		},
	},
	{
		icon: EarlyModpackBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-modpack-adopter.name',
			defaultMessage: 'Early Modpack Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-modpack-adopter.about.1',
				defaultMessage: `This user helped us test Modpack projects on Modrinth before we launched them in May 2022.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'modpack',
				cutoff: new Date('2022-05-23T00:57:00.000Z'),
			},
			{
				type: 'badge',
				bitflag: BadgeBitflag.EARLY_MODPACK_ADOPTER,
			},
		],
	},
	{
		icon: EarlyResourcepackBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-resourcepack-adopter.name',
			defaultMessage: 'Early Resource Pack Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-resourcepack-adopter.about.1',
				defaultMessage: `This user helped us test Resource Pack projects on Modrinth before we launched them in August 2022.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'resourcepack',
				cutoff: new Date('2022-08-27T23:03:00.000Z'),
			},
			{
				type: 'badge',
				bitflag: BadgeBitflag.EARLY_RESPACK_ADOPTER,
			},
		],
	},
	{
		icon: EarlyPluginBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-plugin-adopter.name',
			defaultMessage: 'Early Plugin Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-plugin-adopter.about.1',
				defaultMessage: `This user helped us test Plugin projects on Modrinth before we launched them in August 2022.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'plugin',
				cutoff: new Date('2022-08-27T23:03:00.000Z'),
			},
			{
				type: 'badge',
				bitflag: BadgeBitflag.EARLY_PLUGIN_ADOPTER,
			},
		],
	},
	{
		icon: EarlyDatapackBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-datapack-adopter.name',
			defaultMessage: 'Early Data Pack Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-datapack-adopter.about.1',
				defaultMessage: `This user helped us test Data Pack projects on Modrinth before we launched them in January 2023.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'datapack',
				cutoff: new Date('2023-01-08T02:00:00.000Z'),
			},
		],
	},
	{
		icon: EarlyShadersBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-shader-adopter.name',
			defaultMessage: 'Early Shader Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-shader-adopter.about.1',
				defaultMessage: `This user helped us test Shader projects on Modrinth before we launched them in January 2023.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'shader',
				cutoff: new Date('2023-01-08T02:00:00.000Z'),
			},
		],
	},
	{
		icon: EarlyServersBadge,
		name: defineMessage({
			id: 'user.profile.badge.early-server-adopter.name',
			defaultMessage: 'Early Server Adopter',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.early-server-adopter.about.1',
				defaultMessage: `This user helped us test Server projects on Modrinth before we launched them in March 2026.`,
			}),
		],
		criteria: [
			{
				type: 'earliest_project_date',
				project_type: 'server',
				cutoff: new Date('2026-03-04T01:33:00.000Z'),
			},
		],
	},
	{
		icon: EarlyHostingBadge,
		name: defineMessage({
			id: 'user.profile.badge.hosting-alpha.name',
			defaultMessage: 'Modrinth Hosting Alpha Tester',
		}),
		about: [
			defineMessage({
				id: 'user.profile.badge.hosting-alpha.about.1',
				defaultMessage: `This user participated in a closed alpha test of Modrinth Hosting before we launched Modrinth Hosting Beta in November 2024`,
			}),
		],
		criteria: [], // TODO: Add badge on backend for Hosting Alpha Tester
	},
] satisfies Badge[]

const DOWNLOAD_BADGES = [
	{
		icon: Downloads1mBadge,
		threshold: 1_000_000,
	},
	{
		icon: Downloads10mBadge,
		threshold: 10_000_000,
	},
	{
		icon: Downloads25mBadge,
		threshold: 25_000_000,
	},
	{
		icon: Downloads50mBadge,
		threshold: 50_000_000,
	},
	{
		icon: Downloads100mBadge,
		threshold: 100_000_000,
	},
	{
		icon: Downloads250mBadge,
		threshold: 250_000_000,
	},
	{
		icon: Downloads500mBadge,
		threshold: 500_000_000,
	},
].sort((a, b) => b.threshold - a.threshold)

const props = defineProps<{
	role: Labrinth.Users.v2.Role
	badges: number
	downloads: number
	joinDate: Date
	earliestProjectByType: Record<EarlyAdopterProjectTypes, Date>
}>()

const downloadsBadge = computed(() => {
	return DOWNLOAD_BADGES.find((badge) => props.downloads >= badge.threshold)
})

const messages = defineMessages({
	title: {
		id: 'profile.label.badges',
		defaultMessage: 'Badges',
	},
	downloadsBadgeName: {
		id: 'user.profile.badge.downloads.name',
		defaultMessage: '{download_sum} Downloads',
	},
	downloadsBadgeAbout1: {
		id: 'user.profile.badge.downloads.about.1',
		defaultMessage: `This user's projects have collectively achieved {download_sum} downloads.`,
	},
})

function passesCriterion(criterion: BadgeCriterion) {
	switch (criterion.type) {
		case 'role': {
			return props.role === criterion.role
		}
		case 'badge': {
			return props.badges & criterion.bitflag
		}
		case 'join_date': {
			return props.joinDate < criterion.cutoff
		}
		case 'earliest_project_date': {
			const date = props.earliestProjectByType[criterion.project_type]
			return date && date < criterion.cutoff
		}
		default: {
			return false
		}
	}
}

const earnedBadges = computed(() => {
	const badges: Badge[] = []

	loopingBadges: for (const badge of BADGES) {
		for (const criterion of badge.criteria) {
			if (passesCriterion(criterion)) {
				badges.push(badge)
				continue loopingBadges
			}
		}
	}
	return badges
})
</script>

<template>
	<div v-if="earnedBadges.length > 0 || !!downloadsBadge" class="flex flex-col">
		<h2 class="text-lg text-contrast m-0 mb-2">
			{{ formatMessage(messages.title) }}
		</h2>
		<div class="grid grid-cols-[repeat(auto-fill,minmax(64px,1fr))] gap-2">
			<UserBadge
				v-for="badge in earnedBadges"
				:key="badge.name.id"
				:name="badge.name"
				:icon="badge.icon"
				:about="badge.about"
				:link="badge.link"
			/>
			<UserBadge
				v-if="downloadsBadge"
				:name="messages.downloadsBadgeName"
				:icon="downloadsBadge.icon"
				:about="[messages.downloadsBadgeAbout1]"
				:values="{ download_sum: formatNumber(downloadsBadge.threshold) }"
			/>
		</div>
	</div>
</template>
