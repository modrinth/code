<template>
	<div>
		<div class="normal-page no-sidebar">
			<h1>{{ formatMessage(commonMessages.settingsLabel) }}</h1>
		</div>
		<div class="normal-page">
			<div class="normal-page__sidebar">
				<NavStack
					:items="
						[
							{ type: 'heading', label: 'Display' },
							{
								link: '/settings',
								label: formatMessage(commonSettingsMessages.appearance),
								icon: PaintbrushIcon,
							},
							isStaging
								? {
										link: '/settings/language',
										label: formatMessage(commonSettingsMessages.language),
										icon: LanguagesIcon,
										badge: `${formatMessage(commonMessages.beta)}`,
									}
								: null,
							auth.user ? { type: 'heading', label: 'Account' } : null,
							auth.user
								? {
										link: '/settings/profile',
										label: formatMessage(commonSettingsMessages.profile),
										icon: UserIcon,
									}
								: null,
							auth.user
								? {
										link: '/settings/account',
										label: formatMessage(commonSettingsMessages.account),
										icon: ShieldIcon,
									}
								: null,
							auth.user
								? {
										link: '/settings/authorizations',
										label: formatMessage(commonSettingsMessages.authorizedApps),
										icon: GridIcon,
									}
								: null,
							auth.user
								? {
										link: '/settings/sessions',
										label: formatMessage(commonSettingsMessages.sessions),
										icon: MonitorSmartphoneIcon,
									}
								: null,
							auth.user
								? {
										link: '/settings/billing',
										label: formatMessage(commonSettingsMessages.billing),
										icon: CardIcon,
									}
								: null,
							auth.user ? { type: 'heading', label: 'Developer' } : null,
							auth.user
								? {
										link: '/settings/pats',
										label: formatMessage(commonSettingsMessages.pats),
										icon: KeyIcon,
									}
								: null,
							auth.user
								? {
										link: '/settings/applications',
										label: formatMessage(commonSettingsMessages.applications),
										icon: ServerIcon,
									}
								: null,
						].filter(Boolean)
					"
				/>
			</div>
			<div class="normal-page__content mt-3 lg:mt-0">
				<NuxtPage :route="route" />
			</div>
		</div>
	</div>
</template>
<script setup>
import {
	CardIcon,
	GridIcon,
	KeyIcon,
	LanguagesIcon,
	MonitorSmartphoneIcon,
	PaintbrushIcon,
	ServerIcon,
	ShieldIcon,
	UserIcon,
} from '@modrinth/assets'
import { commonMessages, commonSettingsMessages } from '@modrinth/ui'

import NavStack from '~/components/ui/NavStack.vue'

const { formatMessage } = useVIntl()

const route = useNativeRoute()
const auth = await useAuth()
const isStaging = useRuntimeConfig().public.siteUrl !== 'https://modrinth.com'
</script>
