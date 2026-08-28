<template>
	<div>
		<NormalPage sidebar="left">
			<template #header>
				<h1 class="m-0 text-3xl font-semibold">Admin</h1>
			</template>
			<template #sidebar>
				<NavStack
					aria-label="Admin navigation"
					:items="[
						{ type: 'heading', label: 'Lookup' },
						{
							link: '/admin/file_lookup',
							label: 'File lookup',
							icon: FileSearchCornerIcon,
						},
						{
							link: '/admin/user_email',
							label: 'User lookup',
							icon: UserSearchIcon,
							shown: admin,
						},
						{
							type: 'heading',
							label: 'Hosting',
							shown: admin,
						},
						{
							link: '/admin/servers/lookup',
							label: 'Server lookup',
							icon: ServerSearchIcon,
							shown: admin,
						},
						{
							link: '/admin/servers/notices',
							label: 'Server notices',
							icon: IssuesIcon,
							shown: admin,
						},
						{
							link: '/admin/servers/transfers',
							label: 'Server transfers',
							icon: TransferIcon,
							shown: admin,
						},
						{
							type: 'heading',
							label: 'Management',
							shown: admin,
						},
						{
							link: '/admin/affiliates',
							label: 'Affiliate links',
							icon: AffiliateIcon,
							shown: admin,
						},
						{
							link: '/admin/analytics/events',
							label: 'Analytics events',
							icon: ChartIcon,
							shown: admin,
						},
						{ type: 'heading', label: 'Templates' },
						{
							link: '/admin/emails',
							label: 'Email templates',
							icon: MailIcon,
						},
						{
							link: '/admin/docs',
							label: 'Document templates',
							icon: BookOpenIcon,
						},
					]"
				/>
			</template>
			<NuxtPage :route="route" />
		</NormalPage>
	</div>
</template>
<script setup lang="ts">
import {
	AffiliateIcon,
	BookOpenIcon,
	ChartIcon,
	FileSearchCornerIcon,
	IssuesIcon,
	MailIcon,
	ServerSearchIcon,
	TransferIcon,
	UserSearchIcon,
} from '@modrinth/assets'
import { NormalPage } from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'

import NavStack from '~/components/ui/NavStack.vue'

const auth = await useAuth()
const route = useNativeRoute()
const admin = computed(() => isAdmin(auth.value.user))

definePageMeta({
	middleware: ['auth', 'staff'],
})

useSeoMeta({
	robots: 'noindex',
})
</script>
