<script setup>
import {
	ClipboardCopyIcon,
	ExternalIcon,
	GlobeIcon,
	MailIcon,
	MastodonIcon,
	RedditIcon,
	ShareIcon,
	TwitterIcon,
} from '@modrinth/assets'
import QrcodeVue from 'qrcode.vue'
import { computed, nextTick, ref } from 'vue'

import { injectNotificationManager } from '#ui/providers'

import Button from '../base/buttons/Button.vue'
import ButtonLink from '../base/buttons/ButtonLink.vue'
import IconButton from '../base/buttons/IconButton.vue'
import { NewModal, StyledInput } from '../index'

const props = defineProps({
	header: {
		type: String,
		default: 'Share',
	},
	shareTitle: {
		type: String,
		default: 'Modrinth',
	},
	shareText: {
		type: String,
		default: null,
	},
	link: {
		type: Boolean,
		default: false,
	},
	openInNewTab: {
		type: Boolean,
		default: true,
	},
	noblur: {
		type: Boolean,
		default: false,
	},
	socialButtons: {
		type: Boolean,
		default: true,
	},
	onHide: {
		type: Function,
		default() {
			return () => {}
		},
	},
})

const shareModal = ref(null)
const { addNotification } = injectNotificationManager()

const qrCode = ref(null)
const qrImage = ref(null)
const content = ref(null)
const url = ref(null)
const canShare = ref(false)
const share = () => {
	navigator.share(
		props.link
			? {
					title: props.shareTitle.toString(),
					text: props.shareText,
					url: url.value,
				}
			: {
					title: props.shareTitle.toString(),
					text: content.value,
				},
	)
}

const show = async (passedContent) => {
	content.value = props.shareText ? `${props.shareText}\n\n${passedContent}` : passedContent
	shareModal.value.show()
	if (props.link) {
		url.value = passedContent
		nextTick(() => {
			console.log(qrCode.value)
			fetch(qrCode.value.getElementsByTagName('canvas')[0].toDataURL('image/png'))
				.then((res) => res.blob())
				.then((blob) => {
					console.log(blob)
					qrImage.value = blob
				})
		})
	}
	if (navigator.canShare({ title: props.shareTitle.toString(), text: content.value })) {
		canShare.value = true
	}
}

const copyImage = async () => {
	const item = new ClipboardItem({ 'image/png': qrImage.value })
	await navigator.clipboard.write([item])
}

const copyText = async () => {
	try {
		await navigator.clipboard.writeText(url.value ?? content.value)
		addNotification({
			type: 'success',
			title: 'Link copied',
			text: 'The link has been copied to your clipboard.',
		})
	} catch (error) {
		const message = error instanceof Error ? error.message : String(error)
		addNotification({
			type: 'error',
			title: 'Failed to copy text',
			text: message,
		})
	}
}

const sendEmail = computed(
	() =>
		`mailto:user@test.com
    ?subject=${encodeURIComponent(props.shareTitle)}
    &body=${encodeURIComponent(content.value)}`,
)

const targetParameter = computed(() => (props.openInNewTab ? '_blank' : '_self'))

const sendTweet = computed(
	() => `https://twitter.com/intent/tweet?text=${encodeURIComponent(content.value)}`,
)

const sendToot = computed(() => `https://tootpick.org/#text=${encodeURIComponent(content.value)}`)

const postOnReddit = computed(
	() =>
		`https://www.reddit.com/submit?title=${encodeURIComponent(props.shareTitle)}&text=${encodeURIComponent(
			content.value,
		)}`,
)

defineExpose({
	show,
})
</script>

<template>
	<NewModal ref="shareModal" :header="header" :noblur="noblur" :on-hide="onHide">
		<div class="flex flex-col items-center gap-2">
			<div
				:class="['flex items-center justify-center', link ? 'flex-wrap gap-4' : 'flex-col gap-2']"
			>
				<div v-if="link" class="group relative shrink-0">
					<div ref="qrCode">
						<QrcodeVue :value="url" class="!bg-white rounded-[var(--radius-md)]" margin="3" />
					</div>
					<IconButton
						label="Copy QR code"
						type="quiet"
						class="absolute top-0 right-0 m-2"
						@click="copyImage"
					>
						<ClipboardCopyIcon aria-hidden="true" />
					</IconButton>
				</div>
				<StyledInput
					v-else
					v-model="content"
					multiline
					resize="vertical"
					wrapper-class="h-full w-[30rem]"
				>
					<template #right>
						<IconButton
							label="Copy Text"
							type="quiet"
							class="absolute top-0 right-0 m-2"
							@click="copyText"
						>
							<ClipboardCopyIcon aria-hidden="true" />
						</IconButton>
					</template>
				</StyledInput>
				<div
					v-if="link || socialButtons"
					:class="['flex flex-col justify-center gap-2', link ? 'w-64 max-w-full' : 'flex-grow']"
				>
					<Button
						v-if="link"
						aria-label="Copy Link"
						class="w-full !justify-between !pr-1.5"
						@click="copyText"
					>
						<span class="min-w-0 cursor-pointer truncate text-left font-semibold text-primary">
							{{ url }}
						</span>
						<div class="grid h-10 w-10 place-content-center">
							<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
						</div>
					</Button>
					<ButtonLink v-if="link" :href="url" target="_blank" class="w-full">
						Open in new tab
						<ExternalIcon aria-hidden="true" />
					</ButtonLink>
					<div v-if="socialButtons" class="flex flex-row gap-1">
						<IconButton v-if="canShare" label="Share" @click="share">
							<ShareIcon aria-hidden="true" />
						</IconButton>
						<ButtonLink :href="sendEmail" :target="targetParameter" aria-label="Send as an email">
							<MailIcon aria-hidden="true" />
						</ButtonLink>
						<ButtonLink
							v-if="link"
							:href="url"
							:target="targetParameter"
							aria-label="Open link in browser"
						>
							<GlobeIcon aria-hidden="true" />
						</ButtonLink>
						<ButtonLink :href="sendToot" :target="targetParameter" aria-label="Toot about it">
							<MastodonIcon aria-hidden="true" />
						</ButtonLink>
						<ButtonLink
							:href="sendTweet"
							:target="targetParameter"
							aria-label="Tweet about it"
						>
							<TwitterIcon aria-hidden="true" />
						</ButtonLink>
						<ButtonLink
							:href="postOnReddit"
							:target="targetParameter"
							aria-label="Share on Reddit"
						>
							<RedditIcon aria-hidden="true" />
						</ButtonLink>
					</div>
				</div>
			</div>
		</div>
	</NewModal>
</template>
