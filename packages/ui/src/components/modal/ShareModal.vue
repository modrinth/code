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

import { Button, ButtonStyled, NewModal, StyledInput } from '../index'

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
		<div class="flex flex-row flex-wrap items-center gap-2">
			<div v-if="link" class="group relative mx-auto">
				<div ref="qrCode">
					<QrcodeVue :value="url" class="!bg-white rounded-[var(--radius-md)]" margin="3" />
				</div>
				<ButtonStyled circular>
					<button
						v-tooltip="'Copy QR code'"
						class="absolute top-0 right-0 m-2 opacity-0 transition-all duration-200 ease-in-out group-hover:opacity-100 group-focus-within:opacity-100 motion-reduce:transition-none"
						aria-label="Copy QR code"
						@click="copyImage"
					>
						<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
					</button>
				</ButtonStyled>
			</div>
			<StyledInput v-else v-model="content" multiline resize="vertical" wrapper-class="h-full">
				<template #right>
					<button
						v-tooltip="'Copy Text'"
						type="button"
						aria-label="Copy Text"
						class="absolute top-0 right-0 m-2 grid h-10 w-10 cursor-pointer place-content-center rounded-lg border-none bg-button-bg text-primary transition-all hover:bg-button-bg-hover hover:brightness-125 active:scale-95"
						@click="copyText"
					>
						<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
					</button>
				</template>
			</StyledInput>
			<div class="flex flex-grow flex-col justify-center gap-2">
				<button
					v-if="link"
					v-tooltip="'Copy Link'"
					type="button"
					aria-label="Copy Link"
					class="flex h-10 w-full cursor-pointer items-center justify-between gap-2 rounded-xl border-none bg-button-bg px-3 pr-1.5 text-primary transition-all hover:bg-button-bg-hover hover:brightness-125 active:scale-95"
					@click="copyText"
				>
					<span class="cursor-pointer truncate text-left font-semibold text-primary">
						{{ url }}
					</span>
					<div class="grid h-10 w-10 place-content-center">
						<ClipboardCopyIcon class="h-5 w-5" aria-hidden="true" />
					</div>
				</button>
				<ButtonStyled v-if="link">
					<a :href="url" target="_blank" rel="noopener noreferrer" aria-label="Open in new tab">
						Open in new tab
						<ExternalIcon aria-hidden="true" />
					</a>
				</ButtonStyled>
				<div v-if="socialButtons" class="flex flex-row gap-2">
					<Button v-if="canShare" v-tooltip="'Share'" aria-label="Share" icon-only @click="share">
						<ShareIcon aria-hidden="true" />
					</Button>
					<a
						v-tooltip="'Send as an email'"
						class="btn icon-only fill-contrast text-contrast"
						:href="sendEmail"
						:target="targetParameter"
						aria-label="Send as an email"
					>
						<MailIcon aria-hidden="true" />
					</a>
					<a
						v-if="link"
						v-tooltip="'Open link in browser'"
						class="btn icon-only fill-contrast text-contrast"
						:target="targetParameter"
						:href="url"
						aria-label="Open link in browser"
					>
						<GlobeIcon aria-hidden="true" />
					</a>
					<a
						v-tooltip="'Toot about it'"
						class="btn icon-only fill-contrast text-contrast bg-[#563acc]"
						:target="targetParameter"
						:href="sendToot"
						aria-label="Toot about it"
					>
						<MastodonIcon aria-hidden="true" />
					</a>
					<a
						v-tooltip="'Tweet about it'"
						class="btn icon-only fill-contrast text-contrast bg-[#1da1f2]"
						:target="targetParameter"
						:href="sendTweet"
						aria-label="Tweet about it"
					>
						<TwitterIcon aria-hidden="true" />
					</a>
					<a
						v-tooltip="'Share on Reddit'"
						class="btn icon-only fill-contrast text-contrast bg-[#ff4500]"
						:target="targetParameter"
						:href="postOnReddit"
						aria-label="Share on Reddit"
					>
						<RedditIcon aria-hidden="true" />
					</a>
				</div>
			</div>
		</div>
	</NewModal>
</template>
