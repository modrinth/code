<script setup lang="ts">
import { BlueskyIcon, DiscordIcon, GithubIcon, MastodonIcon, TwitterIcon } from '@modrinth/assets'
import {
	AutoLink,
	ButtonStyled,
	defineMessage,
	defineMessages,
	injectNotificationManager,
	IntlFormatted,
	type MessageDescriptor,
	useVIntl,
} from '@modrinth/ui'

import TextLogo from '~/components/brand/TextLogo.vue'

const flags = useFeatureFlags()
const { formatMessage } = useVIntl()
const { addNotification } = injectNotificationManager()
const config = useRuntimeConfig()

const messages = defineMessages({
	modrinthInformation: {
		id: 'layout.footer.modrinth-information',
		defaultMessage: 'Modrinth information',
	},
	openSource: {
		id: 'layout.footer.open-source',
		defaultMessage: 'Modrinth is <github-link>open source</github-link>.',
	},
	legalDisclaimer: {
		id: 'layout.footer.legal-disclaimer',
		defaultMessage:
			'NOT AN OFFICIAL MINECRAFT SERVICE. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.',
	},
})

const socialLinks: {
	label: MessageDescriptor
	href: string
	icon: Component
	rel?: string
}[] = [
	{
		label: defineMessage({ id: 'layout.footer.social.discord', defaultMessage: 'Discord' }),
		href: 'https://discord.modrinth.com',
		icon: DiscordIcon,
	},
	{
		label: defineMessage({ id: 'layout.footer.social.bluesky', defaultMessage: 'Bluesky' }),
		href: 'https://bsky.app/profile/modrinth.com',
		icon: BlueskyIcon,
	},
	{
		label: defineMessage({ id: 'layout.footer.social.mastodon', defaultMessage: 'Mastodon' }),
		href: 'https://floss.social/@modrinth',
		icon: MastodonIcon,
		rel: 'me',
	},
	{
		label: defineMessage({ id: 'layout.footer.social.x', defaultMessage: 'X' }),
		href: 'https://x.com/modrinth',
		icon: TwitterIcon,
	},
	{
		label: defineMessage({ id: 'layout.footer.social.github', defaultMessage: 'GitHub' }),
		href: 'https://github.com/modrinth',
		icon: GithubIcon,
	},
]

const footerLinks: {
	label: MessageDescriptor
	links: {
		href: string
		label: MessageDescriptor
	}[]
}[] = [
	{
		label: defineMessage({ id: 'layout.footer.about', defaultMessage: 'About' }),
		links: [
			{
				href: '/news',
				label: defineMessage({ id: 'layout.footer.about.news', defaultMessage: 'News' }),
			},
			{
				href: '/news/changelog',
				label: defineMessage({ id: 'layout.footer.about.changelog', defaultMessage: 'Changelog' }),
			},
			{
				href: 'https://status.modrinth.com',
				label: defineMessage({ id: 'layout.footer.about.status', defaultMessage: 'Status' }),
			},
			{
				href: 'https://careers.modrinth.com',
				label: defineMessage({ id: 'layout.footer.about.careers', defaultMessage: 'Careers' }),
			},
			{
				href: '/legal/cmp-info',
				label: defineMessage({
					id: 'layout.footer.about.rewards-program',
					defaultMessage: 'Rewards Program',
				}),
			},
		],
	},
	{
		label: defineMessage({ id: 'layout.footer.products', defaultMessage: 'Products' }),
		links: [
			{
				href: '/plus',
				label: defineMessage({ id: 'layout.footer.products.plus', defaultMessage: 'Modrinth+' }),
			},
			{
				href: '/app',
				label: defineMessage({ id: 'layout.footer.products.app', defaultMessage: 'Modrinth App' }),
			},
			{
				href: '/hosting',
				label: defineMessage({
					id: 'layout.footer.products.servers',
					defaultMessage: 'Modrinth Hosting',
				}),
			},
		],
	},
	{
		label: defineMessage({ id: 'layout.footer.resources', defaultMessage: 'Resources' }),
		links: [
			{
				href: 'https://support.modrinth.com',
				label: defineMessage({
					id: 'layout.footer.resources.help-center',
					defaultMessage: 'Help Center',
				}),
			},
			{
				href: 'https://translate.modrinth.com',
				label: defineMessage({
					id: 'layout.footer.resources.translate',
					defaultMessage: 'Translate',
				}),
			},
			{
				href: 'https://github.com/modrinth/code/issues',
				label: defineMessage({
					id: 'layout.footer.resources.report-issues',
					defaultMessage: 'Report issues',
				}),
			},
			{
				href: 'https://docs.modrinth.com/api/',
				label: defineMessage({
					id: 'layout.footer.resources.api-docs',
					defaultMessage: 'API documentation',
				}),
			},
		],
	},
	{
		label: defineMessage({ id: 'layout.footer.legal', defaultMessage: 'Legal' }),
		links: [
			{
				href: '/legal/rules',
				label: defineMessage({ id: 'layout.footer.legal.rules', defaultMessage: 'Content Rules' }),
			},
			{
				href: '/legal/terms',
				label: defineMessage({
					id: 'layout.footer.legal.terms-of-use',
					defaultMessage: 'Terms of Use',
				}),
			},
			{
				href: '/legal/privacy',
				label: defineMessage({
					id: 'layout.footer.legal.privacy-policy',
					defaultMessage: 'Privacy Policy',
				}),
			},
			{
				href: '/legal/security',
				label: defineMessage({
					id: 'layout.footer.legal.security-notice',
					defaultMessage: 'Security Notice',
				}),
			},
			{
				href: '/legal/copyright',
				label: defineMessage({
					id: 'layout.footer.legal.copyright-policy',
					defaultMessage: 'Copyright Policy and DMCA',
				}),
			},
		],
	},
]

const developerModeCounter = ref(0)

const state = useGeneratedState()

function developerModeIncrement() {
	if (developerModeCounter.value >= 5) {
		flags.value.developerMode = !flags.value.developerMode
		developerModeCounter.value = 0
		saveFeatureFlags()
		if (flags.value.developerMode) {
			addNotification({
				title: 'Developer mode activated',
				text: 'Developer mode has been enabled',
				type: 'success',
			})
		} else {
			addNotification({
				title: 'Developer mode deactivated',
				text: 'Developer mode has been disabled',
				type: 'success',
			})
		}
	} else {
		developerModeCounter.value++
	}
}
</script>

<template>
	<footer
		class="footer-brand-background experimental-styles-within border-0 border-t-[1px] border-solid"
	>
		<div class="mx-auto flex max-w-screen-xl flex-col gap-6 p-6 pb-20 sm:px-12 md:py-12">
			<div
				class="grid grid-cols-1 gap-4 text-primary md:grid-cols-[1fr_2fr] lg:grid-cols-[auto_auto_auto_auto_auto]"
			>
				<div
					class="flex flex-col items-center gap-3 md:items-start"
					role="region"
					:aria-label="formatMessage(messages.modrinthInformation)"
				>
					<TextLogo
						aria-hidden="true"
						class="text-logo button-base h-6 w-auto text-contrast lg:h-8"
						@click="developerModeIncrement()"
					/>
					<div class="flex flex-wrap justify-center gap-px sm:-mx-2">
						<ButtonStyled
							v-for="(social, index) in socialLinks"
							:key="`footer-social-${index}`"
							circular
							type="transparent"
						>
							<a
								v-tooltip="formatMessage(social.label)"
								:href="social.href"
								target="_blank"
								:rel="`noopener${social.rel ? ` ${social.rel}` : ''}`"
							>
								<component :is="social.icon" class="h-5 w-5" />
							</a>
						</ButtonStyled>
					</div>
					<div class="mt-auto flex flex-wrap justify-center gap-3 md:flex-col">
						<p class="m-0">
							<IntlFormatted :message-id="messages.openSource">
								<template #github-link="{ children }">
									<a
										href="https://github.com/modrinth/code"
										class="text-brand hover:underline"
										target="_blank"
										rel="noopener"
									>
										<component :is="() => children" />
									</a>
								</template>
							</IntlFormatted>
						</p>
						<p class="m-0">© {{ state.buildYear ?? '2025' }} Rinth, Inc.</p>
					</div>
				</div>
				<div class="mt-4 grid grid-cols-1 gap-6 sm:grid-cols-2 lg:contents">
					<div
						v-for="group in footerLinks"
						:key="group.label.id"
						class="flex flex-col items-center gap-3 sm:items-start"
					>
						<h3 class="m-0 text-base text-contrast">{{ formatMessage(group.label) }}</h3>
						<template v-for="item in group.links" :key="item.label">
							<nuxt-link
								v-if="item.href.startsWith('/')"
								:to="item.href"
								class="w-fit hover:underline"
							>
								{{ formatMessage(item.label) }}
							</nuxt-link>
							<a
								v-else
								:href="item.href"
								class="w-fit hover:underline"
								target="_blank"
								rel="noopener"
							>
								{{ formatMessage(item.label) }}
							</a>
						</template>
					</div>
				</div>
			</div>
			<p v-if="flags.developerMode" class="m-0 text-sm text-secondary">
				Based on
				<a
					v-if="config.public.owner && config.public.branch"
					class="hover:underline"
					target="_blank"
					:href="`https://github.com/${config.public.owner}/code/tree/${config.public.branch}`"
				>
					{{ config.public.owner }}/{{ config.public.branch }}
				</a>
				@
				<span v-if="config.public.hash === 'unknown'">unknown</span>
				<AutoLink
					v-else
					class="text-link"
					target="_blank"
					:to="`https://github.com/${config.public.owner}/code/commit/${config.public.hash}`"
				>
					{{ config.public.hash }}
				</AutoLink>
			</p>
			<div class="flex justify-center text-center text-xs font-medium text-secondary opacity-50">
				{{ formatMessage(messages.legalDisclaimer) }}
			</div>
		</div>
	</footer>
</template>
<style scoped lang="scss">
.footer-brand-background {
	background: var(--brand-gradient-strong-bg);
	border-color: var(--brand-gradient-border);
}
</style>
