<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="600px">
		<div v-if="!auth.user.value" class="flex flex-col gap-3">
			<p class="m-0 text-secondary">{{ formatMessage(messages.signInRequired) }}</p>
		</div>
		<div v-else-if="!selected" class="flex flex-col gap-3">
			<p class="m-0 text-secondary">{{ formatMessage(messages.searchDescription) }}</p>
			<Input
				v-model="query"
				:icon="SearchIcon"
				:placeholder="formatMessage(messages.searchPlaceholder)"
				clearable
			/>
			<div
				v-if="results.length > 0"
				class="flex max-h-80 w-full flex-col overflow-y-auto rounded-2xl border border-solid border-surface-5 bg-surface-4"
			>
				<button
					v-for="result in results"
					:key="result.key"
					v-tooltip="result.key"
					type="button"
					data-button
					class="flex w-full items-center gap-2 border-0 border-t border-solid border-surface-5 bg-surface-4 px-4 py-2 text-left transition-colors first:border-t-0 hover:bg-surface-5"
					@click="select(result)"
				>
					<span class="flex min-w-0 flex-col gap-1">
						<span class="text-contrast">{{ result.translated }}</span>
						<span v-if="result.isTranslated" class="text-sm text-secondary">
							{{ result.source }}
						</span>
						<span v-else class="text-sm text-secondary">
							{{ formatMessage(messages.notTranslatedLabel) }}
						</span>
					</span>
					<RightArrowIcon aria-hidden="true" class="ml-auto h-5 w-5 shrink-0 text-secondary" />
				</button>
			</div>
			<p v-else-if="query.trim().length >= MIN_QUERY_LENGTH" class="m-0 text-secondary">
				{{ formatMessage(messages.noResults) }}
			</p>
		</div>
		<div v-else class="flex flex-col gap-4">
			<div
				v-tooltip="selected.key"
				class="flex flex-col gap-1 rounded-2xl border border-solid border-surface-5 bg-surface-4 px-4 py-2"
			>
				<span class="text-contrast">{{ selected.translated }}</span>
				<span v-if="selected.isTranslated" class="text-sm text-secondary">
					{{ selected.source }}
				</span>
			</div>
			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.kindLabel) }}</span>
				<Chips
					v-model="kind"
					:items="kinds"
					:format-label="(item) => formatMessage(kindMessages[item])"
					:capitalize="false"
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="translation-feedback-suggestion" class="font-semibold text-contrast">
					{{ formatMessage(messages.suggestionLabel) }}
				</label>
				<Textarea
					id="translation-feedback-suggestion"
					v-model="suggestion"
					:maxlength="MAX_TEXT_LENGTH"
					:placeholder="formatMessage(messages.suggestionPlaceholder)"
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="translation-feedback-comment" class="font-semibold text-contrast">
					{{ formatMessage(messages.commentLabel) }}
				</label>
				<Textarea
					id="translation-feedback-comment"
					v-model="comment"
					:maxlength="MAX_TEXT_LENGTH"
					:placeholder="formatMessage(messages.commentPlaceholder)"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button v-if="selected" type="outlined" :disabled="submitting" @click="selected = null">
					<LeftArrowIcon />
					{{ formatMessage(commonMessages.backButton) }}
				</Button>
				<Button v-else type="outlined" @click="modal?.hide()">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button v-if="!auth.user.value" type="colored" color="brand" @click="signIn">
					<LogInIcon />
					{{ formatMessage(commonMessages.signInButton) }}
				</Button>
				<Button
					v-else
					type="colored"
					color="brand"
					:disabled="!selected"
					:loading="submitting"
					@click="submit"
				>
					<SendIcon />
					{{ formatMessage(messages.submitButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import {
	LeftArrowIcon,
	LogInIcon,
	RightArrowIcon,
	SearchIcon,
	SendIcon,
	XIcon,
} from '@modrinth/assets'
import Fuse from 'fuse.js/dist/fuse.basic'
import { computed, ref, shallowRef } from 'vue'

import { Button } from '#ui/components/base/buttons'
import Chips from '#ui/components/base/Chips.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import Textarea from '#ui/components/base/inputs/Textarea.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import {
	injectAuth,
	injectI18n,
	injectModrinthClient,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils'

type Kind = Labrinth.TranslationFeedback.v3.Kind

type MessageEntry = {
	key: string
	source: string
	translated: string
	isTranslated: boolean
}

const SOURCE_LOCALE = 'en-US'
const MIN_QUERY_LENGTH = 2
const MAX_RESULTS = 25
const MAX_TEXT_LENGTH = 2000

const props = defineProps<{
	product: 'app' | 'website'
}>()

const { formatMessage } = useVIntl()
const { locale, getMessages } = injectI18n()
const auth = injectAuth()
const client = injectModrinthClient()
const notificationManager = injectNotificationManager()

const messages = defineMessages({
	header: {
		id: 'settings.language.feedback.header',
		defaultMessage: 'Report a translation issue',
	},
	signInRequired: {
		id: 'settings.language.feedback.sign-in-required',
		defaultMessage: 'Sign in to send feedback about translations to our translators.',
	},
	searchDescription: {
		id: 'settings.language.feedback.search.description',
		defaultMessage: 'Search for the text that looks wrong, then pick it from the list.',
	},
	searchPlaceholder: {
		id: 'settings.language.feedback.search.placeholder',
		defaultMessage: 'Search for text...',
	},
	noResults: {
		id: 'settings.language.feedback.search.no-results',
		defaultMessage: 'No text matches your search.',
	},
	notTranslatedLabel: {
		id: 'settings.language.feedback.search.not-translated',
		defaultMessage: 'Not translated yet',
	},
	kindLabel: {
		id: 'settings.language.feedback.kind.label',
		defaultMessage: "What's wrong?",
	},
	suggestionLabel: {
		id: 'settings.language.feedback.suggestion.label',
		defaultMessage: 'Suggested translation',
	},
	suggestionPlaceholder: {
		id: 'settings.language.feedback.suggestion.placeholder',
		defaultMessage: 'How should this text read?',
	},
	commentLabel: {
		id: 'settings.language.feedback.comment.label',
		defaultMessage: 'Additional details',
	},
	commentPlaceholder: {
		id: 'settings.language.feedback.comment.placeholder',
		defaultMessage: 'Where did you see this text, and what should translators know?',
	},
	submitButton: {
		id: 'settings.language.feedback.submit',
		defaultMessage: 'Send feedback',
	},
	submitSuccessTitle: {
		id: 'settings.language.feedback.success.title',
		defaultMessage: 'Feedback sent',
	},
	submitSuccessText: {
		id: 'settings.language.feedback.success.text',
		defaultMessage: 'Thank you! Our translators will review your feedback.',
	},
	submitErrorTitle: {
		id: 'settings.language.feedback.error.title',
		defaultMessage: 'Could not send feedback',
	},
	submitErrorText: {
		id: 'settings.language.feedback.error.text',
		defaultMessage: 'Something went wrong. Please try again later.',
	},
})

const kindMessages = defineMessages({
	incorrect: {
		id: 'settings.language.feedback.kind.incorrect',
		defaultMessage: 'Wrong meaning',
	},
	typo: {
		id: 'settings.language.feedback.kind.typo',
		defaultMessage: 'Spelling or grammar',
	},
	inconsistent: {
		id: 'settings.language.feedback.kind.inconsistent',
		defaultMessage: 'Inconsistent',
	},
	untranslated: {
		id: 'settings.language.feedback.kind.untranslated',
		defaultMessage: 'Not translated',
	},
	other: {
		id: 'settings.language.feedback.kind.other',
		defaultMessage: 'Other',
	},
})

const kinds: Kind[] = ['incorrect', 'typo', 'inconsistent', 'untranslated', 'other']

const modal = ref<InstanceType<typeof NewModal>>()
const query = ref('')
const selected = ref<MessageEntry | null>(null)
const kind = ref<Kind>('incorrect')
const suggestion = ref('')
const comment = ref('')
const submitting = ref(false)
const reportedLocale = ref(locale.value)
const fuse = shallowRef<Fuse<MessageEntry> | null>(null)

const results = computed(() => {
	const trimmed = query.value.trim()
	if (!fuse.value || trimmed.length < MIN_QUERY_LENGTH) return []
	return fuse.value
		.search(trimmed, { limit: MAX_RESULTS })
		.map(({ item }: { item: MessageEntry }) => item)
})

function buildEntries(): MessageEntry[] {
	const source = getMessages(SOURCE_LOCALE)
	const localized = getMessages(reportedLocale.value)

	return Object.entries(source).map(([key, sourceText]) => ({
		key,
		source: sourceText,
		translated: localized[key] ?? sourceText,
		isTranslated: key in localized,
	}))
}

function show(): void {
	reportedLocale.value = locale.value
	query.value = ''
	selected.value = null
	suggestion.value = ''
	comment.value = ''
	fuse.value = new Fuse(buildEntries(), {
		keys: ['translated', 'source', 'key'],
		threshold: 0.3,
		ignoreLocation: true,
	})
	modal.value?.show()
}

function select(entry: MessageEntry): void {
	selected.value = entry
	kind.value = entry.isTranslated ? 'incorrect' : 'untranslated'
}

function signIn(): void {
	modal.value?.hide()
	void auth.requestSignIn(typeof window === 'undefined' ? '/' : window.location.pathname)
}

async function submit(): Promise<void> {
	if (!selected.value || submitting.value) return

	submitting.value = true
	try {
		await client.labrinth.translation_feedback_v3.create({
			product: props.product,
			locale: reportedLocale.value,
			key: selected.value.key,
			kind: kind.value,
			seen_text: selected.value.translated,
			suggestion: suggestion.value.trim() || undefined,
			comment: comment.value.trim() || undefined,
		})
		notificationManager.addNotification({
			type: 'success',
			title: formatMessage(messages.submitSuccessTitle),
			text: formatMessage(messages.submitSuccessText),
		})
		modal.value?.hide()
	} catch {
		notificationManager.addNotification({
			type: 'error',
			title: formatMessage(messages.submitErrorTitle),
			text: formatMessage(messages.submitErrorText),
		})
	} finally {
		submitting.value = false
	}
}

defineExpose({ show })
</script>
