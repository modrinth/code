<script setup lang="ts">
import {
	Body,
	Column,
	Container,
	Head,
	Html,
	Img,
	Link as VLink,
	Row,
	Section,
	Style,
	Tailwind,
	Text,
} from '@vue-email/components'

defineProps<{
	title?: string
	manualLinks?: { link: string; label?: string }[]
	supportInfo?: string[]
}>()

interface SocialLink {
	href: string
	alt: string
	src: string
}

const socialLinks = Object.freeze<readonly SocialLink[]>([
	{
		href: 'https://discord.modrinth.com',
		alt: 'Discord',
		src: 'https://cdn-raw.modrinth.com/email/discord.png',
	},
	{
		href: 'https://bsky.app/profile/modrinth.com',
		alt: 'Bluesky',
		src: 'https://cdn-raw.modrinth.com/email/bluesky.png',
	},
	{
		href: 'https://floss.social/@modrinth',
		alt: 'Mastodon',
		src: 'https://cdn-raw.modrinth.com/email/mastodon.png',
	},
	{
		href: 'https://x.com/modrinth',
		alt: 'X (Twitter)',
		src: 'https://cdn-raw.modrinth.com/email/x.png',
	},
	{
		href: 'https://www.instagram.com/modrinth/',
		alt: 'Instagram',
		src: 'https://cdn-raw.modrinth.com/email/instagram.png',
	},
	{
		href: 'https://www.youtube.com/@modrinth',
		alt: 'YouTube',
		src: 'https://cdn-raw.modrinth.com/email/youtube.png',
	},
	{
		href: 'https://github.com/modrinth',
		alt: 'GitHub',
		src: 'https://cdn-raw.modrinth.com/email/github.png',
	},
])
</script>

<template>
	<Html lang="en">
		<Head>
			<title>{{ title }}</title>
			<meta http-equiv="X-UA-Compatible" content="IE=edge" />
			<meta http-equiv="Content-Type" content="text/html; charset=UTF-8" />
			<meta name="viewport" content="width=device-width, initial-scale=1" />
			<link
				href="https://fonts.googleapis.com/css?family=Inter:700,400"
				rel="stylesheet"
				type="text/css"
			/>
			<Style>
				/* Outlook.com centering + line-height fixes */ .ExternalClass { width:100%; }
				.ExternalClass p, .ExternalClass span, .ExternalClass font, .ExternalClass td {
				line-height:100%; } table { border-collapse:separate; } a, a:link, a:visited {
				text-decoration:none; color:#1f68c0; } a:hover { text-decoration:underline; }
				h1,h2,h3,h4,h5,h6 { color:#000 !important; margin:0; mso-line-height-rule:exactly; }
			</Style>
		</Head>

		<Body>
			<Tailwind
				:config="{
					theme: {
						extend: {
							colors: {
								bg: { DEFAULT: '#ebebeb', raised: '#ffffff', super: '#e9e9e9' },
								divider: { DEFAULT: '#babfc5', dark: '#c8cdd3' },
								base: '#2c2e31',
								secondary: '#484d54',
								contrast: '#1a202c',
								accentContrast: '#ffffff',
								red: '#cb2245',
								orange: '#e08325',
								green: '#00af5c',
								blue: '#1f68c0',
								purple: '#8e32f3',
								gray: '#595b61',
								brand: {
									DEFAULT: '#00af5c',
									highlight: 'rgba(0, 175, 92, 0.25)',
									shadow: 'rgba(0, 175, 92, 0.7)',
								},
								highlight: {
									red: 'rgba(203, 34, 69, 0.25)',
									orange: 'rgba(224, 131, 37, 0.25)',
									green: 'rgba(0, 175, 92, 0.25)',
									blue: 'rgba(31, 104, 192, 0.25)',
									purple: 'rgba(142, 50, 243, 0.25)',
									gray: 'rgba(89, 91, 97, 0.25)',
								},
								tint: {
									red: 'rgba(203, 34, 69, 0.1)',
									orange: 'rgba(224, 131, 37, 0.1)',
									green: 'rgba(0, 175, 92, 0.1)',
									blue: 'rgba(31, 104, 192, 0.1)',
									purple: 'rgba(142, 50, 243, 0.1)',
								},
								link: { DEFAULT: '#1f68c0', hover: '#1f68c0', active: '#1f68c0' },
								muted: '#777777',
								footerText: '#4d4d4d',
							},
							fontSize: {
								base: ['16px', { lineHeight: '24px' }],
								'2xl': ['28px', { lineHeight: '32px' }],
								xs: ['13px', { lineHeight: '16px' }],
								'2xs': ['11px', { lineHeight: '16px' }],
							},
							fontFamily: {
								sans: ['Inter', 'Arial', 'sans-serif'],
							},
						},
					},
				}"
			>
				<Section class="m-0 pb-0 pl-0 pr-0 pt-0 font-sans">
					<Section class="bg-bg pb-4 pl-4 pr-4 pt-4">
						<Container class="max-w-[600px]">
							<Row>
								<Column>
									<Img
										src="https://cdn.modrinth.com/email/f740e2decee8764a4629bff677a284f9.png"
										width="29"
										alt=""
										class="block h-auto"
									/>
								</Column>
							</Row>
						</Container>
					</Section>

					<Section class="bg-white pb-8 pl-8 pr-8 pt-8">
						<Container class="max-w-[600px]">
							<slot />
						</Container>
					</Section>

					<Section class="mb-4 bg-bg pb-4 pl-4 pr-4 pt-4">
						<Container class="max-w-[600px]">
							<Row>
								<Column class="align-middle">
									<VLink href="https://modrinth.com" aria-label="Modrinth">
										<Img
											src="https://cdn.modrinth.com/email/bd3357dfae4b1d266250372db3a0988f.png"
											width="175"
											alt="modrinth logo"
											class="block h-auto"
										/>
									</VLink>

									<Row class="text-right align-middle">
										<Section class="m-0 inline-block pb-0 pl-0 pr-0 pt-0">
											<template v-for="(item, index) in socialLinks" :key="item.href">
												<VLink
													:href="item.href"
													:class="['inline-block', index !== socialLinks.length - 1 ? 'mr-4' : '']"
												>
													<Img width="20" height="20" :alt="item.alt" :src="item.src" />
												</VLink>
											</template>
										</Section>
									</Row>

									<Text class="mb-0 mt-2 text-xs" :style="{ color: '#4d4d4d' }"> Rinth, Inc. </Text>
									<Section class="m-0 pb-0 pl-0 pr-0 pt-0">
										<Text class="m-0 text-xs text-secondary">800 N King St</Text>
										<Text class="m-0 text-xs text-secondary">Suite 304 #3133</Text>
										<Text class="m-0 text-xs text-secondary">Wilmington, DE 19801</Text>
									</Section>
								</Column>
							</Row>
						</Container>
					</Section>

					<!-- <Text
						class="text-footerText text-2xs mb-4 mt-0 pb-0 pl-4 pr-4 pt-0 text-center font-sans"
					>
						This email was sent to you as a registered user of Modrinth. You can customize the
						emails you recieve in your
						<VLink href="https://modrinth.com/settings/notifications" class="text-green underline"
							>notification settings</VLink
						>. Some emails are required to keep your account secure and cannot be disabled.
					</Text> -->

					<hr />

					<Section v-if="supportInfo && supportInfo.length" class="mb-0 pb-0 pl-4 pr-4 pt-0">
						<Text
							v-for="(line, index) in supportInfo"
							:key="index"
							class="text-footerText text-2xs font-sans"
						>
							{{ line }}
						</Text>
					</Section>

					<Section
						v-if="manualLinks && manualLinks.length"
						class="text-footerText text-2xs mb-4 pb-0 pl-4 pr-4 pt-0 font-sans"
					>
						<small class="text-muted text-2xs"
							>If you're having trouble with the links above, copy and paste these URLs into your
							web browser:</small
						>
						<Text class="text-2xs text-muted mt-0">
							<span v-for="(item, index) in manualLinks" :key="index" class="block break-words">
								<span v-if="item.label">
									<b>{{ item.label }}:</b><br />
								</span>
								{{ item.link }}
							</span>
							<!-- <span class="block break-words">
								<span> <b>Notification settings:</b><br /> </span>
								https://modrinth.com/settings/notifications
							</span> -->
						</Text>
					</Section>
				</Section>
			</Tailwind>
		</Body>
	</Html>
</template>
