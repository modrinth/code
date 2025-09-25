<script setup lang="ts">
import { Column, Container, Img, Link as VLink, Row, Section, Text } from '@vue-email/components'

import StyledTemplate from '../../shared/StyledTemplate.vue'

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
	<StyledTemplate :title="title">
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
				>If you're having trouble with the links above, copy and paste these URLs into your web
				browser:</small
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
	</StyledTemplate>
</template>
