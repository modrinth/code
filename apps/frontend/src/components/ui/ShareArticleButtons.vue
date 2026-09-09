<template>
	<div class="flex gap-2">
		<ButtonLink
			v-tooltip="`Share on Bluesky`"
			:href="`https://bsky.app/intent/compose?text=${encodedUrl}`"
			target="_blank"
			class="!w-9 !rounded-full !px-0"
		>
			<BlueskyIcon />
		</ButtonLink>
		<ButtonLink
			v-tooltip="`Share on Mastodon`"
			:href="`https://tootpick.org/#text=${encodedUrl}`"
			target="_blank"
			class="!w-9 !rounded-full !px-0"
		>
			<MastodonIcon />
		</ButtonLink>
		<ButtonLink
			v-tooltip="`Share on X`"
			:href="`https://www.x.com/intent/post?url=${encodedUrl}`"
			target="_blank"
			class="!w-9 !rounded-full !px-0"
		>
			<TwitterIcon />
		</ButtonLink>
		<ButtonLink
			v-tooltip="`Share via email`"
			:href="`mailto:${encodedTitle ? `?subject=${encodedTitle}&` : `?`}body=${encodedUrl}`"
			target="_blank"
			class="!w-9 !rounded-full !px-0"
		>
			<MailIcon />
		</ButtonLink>
		<CopyLinkButton :url="url" />
	</div>
</template>

<script setup lang="ts">
import { BlueskyIcon, MailIcon, MastodonIcon, TwitterIcon } from '@modrinth/assets'
import { ButtonLink, CopyLinkButton } from '@modrinth/ui'

const props = defineProps<{
	title?: string
	url: string
}>()

const encodedUrl = computed(() => encodeURIComponent(props.url))
const encodedTitle = computed(() => (props.title ? encodeURIComponent(props.title) : undefined))
</script>
