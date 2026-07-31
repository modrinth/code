<template>
	<div class="flex gap-2">
		<ButtonLink
			v-tooltip="formatMessage(messages.shareBluesky)"
			:aria-label="formatMessage(messages.shareBluesky)"
			:href="`https://bsky.app/intent/compose?text=${encodedUrl}`"
			target="_blank"
		>
			<BlueskyIcon aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="formatMessage(messages.shareMastodon)"
			:aria-label="formatMessage(messages.shareMastodon)"
			:href="`https://tootpick.org/#text=${encodedUrl}`"
			target="_blank"
		>
			<MastodonIcon aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="formatMessage(messages.shareX)"
			:aria-label="formatMessage(messages.shareX)"
			:href="`https://www.x.com/intent/post?url=${encodedUrl}`"
			target="_blank"
		>
			<TwitterIcon aria-hidden="true" />
		</ButtonLink>
		<ButtonLink
			v-tooltip="formatMessage(messages.shareEmail)"
			:aria-label="formatMessage(messages.shareEmail)"
			:href="`mailto:${encodedTitle ? `?subject=${encodedTitle}&` : `?`}body=${encodedUrl}`"
			target="_blank"
		>
			<MailIcon aria-hidden="true" />
		</ButtonLink>
		<IconButton
			v-tooltip="formatMessage(copied ? messages.copied : messages.copyLink)"
			:label="formatMessage(copied ? messages.copied : messages.copyLink)"
			:disabled="copied"
			class="relative overflow-hidden"
			@click="copyToClipboard(url)"
		>
			<CheckIcon
				aria-hidden="true"
				class="absolute transition-all ease-in-out"
				:class="copied ? 'translate-y-0' : 'translate-y-7'"
			/>
			<LinkIcon
				aria-hidden="true"
				class="absolute transition-all ease-in-out"
				:class="copied ? '-translate-y-7' : 'translate-y-0'"
			/>
		</IconButton>
	</div>
</template>

<script setup lang="ts">
import {
	BlueskyIcon,
	CheckIcon,
	LinkIcon,
	MailIcon,
	MastodonIcon,
	TwitterIcon,
} from '@modrinth/assets'
import { defineMessages, useVIntl } from '@modrinth/ui'
import ButtonLink from '@modrinth/ui/src/components/base/buttons/ButtonLink.vue'
import IconButton from '@modrinth/ui/src/components/base/buttons/IconButton.vue'

const props = defineProps<{
	title?: string
	url: string
}>()

const { formatMessage } = useVIntl()
const messages = defineMessages({
	shareBluesky: {
		id: 'article.share.bluesky',
		defaultMessage: 'Share on Bluesky',
	},
	shareMastodon: {
		id: 'article.share.mastodon',
		defaultMessage: 'Share on Mastodon',
	},
	shareX: {
		id: 'article.share.x',
		defaultMessage: 'Share on X',
	},
	shareEmail: {
		id: 'article.share.email',
		defaultMessage: 'Share via email',
	},
	copyLink: {
		id: 'article.share.copy-link',
		defaultMessage: 'Copy link',
	},
	copied: {
		id: 'article.share.copied',
		defaultMessage: 'Copied to clipboard',
	},
})
const copied = ref(false)
const encodedUrl = computed(() => encodeURIComponent(props.url))
const encodedTitle = computed(() => (props.title ? encodeURIComponent(props.title) : undefined))

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}
</script>
