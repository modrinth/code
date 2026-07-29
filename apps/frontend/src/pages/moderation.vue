<template>
	<div class="relative mx-auto mb-6 flex min-h-screen w-full max-w-[1280px] flex-col px-6">
		<h1>Moderation</h1>
		<NavTabs :links="moderationLinks" class="mb-4 hidden sm:flex" />
		<div class="mb-4 sm:hidden">
			<Chips
				v-model="selectedChip"
				:items="mobileNavOptions"
				:never-empty="true"
				@change="navigateToPage"
			/>
		</div>
		<NuxtPage />
	</div>
</template>

<script setup lang="ts">
import { FolderIcon, GlobeIcon, HashIcon, ReportIcon, ShieldCheckIcon } from '@modrinth/assets'
import { Chips, defineMessages, NavTabs, useVIntl } from '@modrinth/ui'

definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})

const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()

const messages = defineMessages({
	projectsTitle: {
		id: 'moderation.page.projects',
		defaultMessage: 'Projects',
	},
	technicalReviewTitle: {
		id: 'moderation.page.technicalReview',
		defaultMessage: 'Tech review',
	},
	reportsTitle: {
		id: 'moderation.page.reports',
		defaultMessage: 'Reports',
	},
	externalFilesTitle: {
		id: 'moderation.page.external-projects',
		defaultMessage: 'External projects',
	},
	globalDetailTracesTitle: {
		id: 'moderation.page.global-detail-traces',
		defaultMessage: 'Global traces',
	},
})

const moderationLinks = [
	{ label: formatMessage(messages.projectsTitle), href: '/moderation', icon: FolderIcon },
	{
		label: formatMessage(messages.technicalReviewTitle),
		href: '/moderation/technical-review',
		icon: ShieldCheckIcon,
	},
	{ label: formatMessage(messages.reportsTitle), href: '/moderation/reports', icon: ReportIcon },
	{
		label: formatMessage(messages.externalFilesTitle),
		href: '/moderation/external-projects',
		icon: GlobeIcon,
	},
	{
		label: formatMessage(messages.globalDetailTracesTitle),
		href: '/moderation/global-traces',
		icon: HashIcon,
	},
]

const mobileNavOptions = [
	formatMessage(messages.projectsTitle),
	formatMessage(messages.technicalReviewTitle),
	formatMessage(messages.reportsTitle),
	formatMessage(messages.externalFilesTitle),
	formatMessage(messages.globalDetailTracesTitle),
]

const selectedChip = computed({
	get() {
		const path = route.path
		if (path.startsWith('/moderation/technical-review')) {
			return formatMessage(messages.technicalReviewTitle)
		} else if (path.startsWith('/moderation/reports')) {
			return formatMessage(messages.reportsTitle)
		} else if (path.startsWith('/moderation/external-projects')) {
			return formatMessage(messages.externalFilesTitle)
		} else if (path.startsWith('/moderation/global-traces')) {
			return formatMessage(messages.globalDetailTracesTitle)
		} else {
			return formatMessage(messages.projectsTitle)
		}
	},
	set(value: string) {
		navigateToPage(value)
	},
})

function navigateToPage(selectedOption: string) {
	if (selectedOption === formatMessage(messages.technicalReviewTitle)) {
		router.push('/moderation/technical-review')
	} else if (selectedOption === formatMessage(messages.reportsTitle)) {
		router.push('/moderation/reports')
	} else if (selectedOption === formatMessage(messages.externalFilesTitle)) {
		router.push('/moderation/external-projects')
	} else if (selectedOption === formatMessage(messages.globalDetailTracesTitle)) {
		router.push('/moderation/global-traces')
	} else {
		router.push('/moderation')
	}
}
</script>
