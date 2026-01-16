<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	commonMessages,
	defineMessages,
	IntlFormatted,
	normalizeChildren,
	PagewideBanner,
	useVIntl,
} from '@modrinth/ui'

const { formatMessage } = useVIntl()
const flags = useFeatureFlags()
const config = useRuntimeConfig()

const messages = defineMessages({
	title: {
		id: 'layout.banner.preview.title',
		defaultMessage: `This is a preview deploy of the Modrinth website.`,
	},
	description: {
		id: 'layout.banner.preview.description',
		defaultMessage: `If you meant to access the official Modrinth website, visit <link>https://modrinth.com</link>. This preview deploy is used by Modrinth staff for testing purposes. It was built using <branch-link>{owner}/{branch}</branch-link> @ {commit}.`,
	},
})

function hidePreviewBanner() {
	flags.value.hidePreviewBanner = true
	saveFeatureFlags()
}
</script>

<template>
	<PagewideBanner v-if="!flags.hidePreviewBanner" variant="info">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			<span>
				<IntlFormatted
					:message-id="messages.description"
					:values="{
						owner: config.public.owner,
						branch: config.public.branch,
					}"
				>
					<template #link="{ children }">
						<a href="https://modrinth.com" target="_blank" rel="noopener" class="text-link">
							<component :is="() => normalizeChildren(children)" />
						</a>
					</template>
					<template #branch-link="{ children }">
						<a
							:href="`https://github.com/${config.public.owner}/code/tree/${config.public.branch}`"
							target="_blank"
							rel="noopener"
							class="hover:underline"
						>
							<component :is="() => normalizeChildren(children)" />
						</a>
					</template>
					<template #commit>
						<span v-if="config.public.hash === 'unknown'">unknown</span>
						<a
							v-else
							:href="`https://github.com/${config.public.owner}/code/commit/${config.public.hash}`"
							target="_blank"
							rel="noopener"
							class="text-link"
						>
							{{ config.public.hash }}
						</a>
					</template>
				</IntlFormatted>
			</span>
		</template>
		<template #actions_right>
			<ButtonStyled type="transparent" circular>
				<button :aria-label="formatMessage(commonMessages.closeButton)" @click="hidePreviewBanner">
					<XIcon aria-hidden="true" />
				</button>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
