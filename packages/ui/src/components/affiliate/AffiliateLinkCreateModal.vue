<template>
	<NewModal ref="modal" :header="formatMessage(messages.createHeader)">
		<div class="flex flex-col">
			<label v-if="showUserField" class="contents" for="create-affiliate-user-input">
				<span class="text-lg font-semibold text-contrast mb-1">
					{{ formatMessage(messages.createUserLabel) }}
				</span>
				<span class="text-secondary mb-2">{{ formatMessage(messages.createUserDescription) }}</span>
			</label>
			<div v-if="showUserField" class="mb-4">
				<div class="iconified-input">
					<UserIcon aria-hidden="true" />
					<input
						id="create-affiliate-user-input"
						v-model="affiliateUsername"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.createUserPlaceholder)"
					/>
					<Button v-if="affiliateUsername" class="r-btn" @click="() => (affiliateUsername = '')">
						<XIcon />
					</Button>
				</div>
			</div>
			<label class="contents" for="create-affiliate-title-input">
				<span class="text-lg font-semibold text-contrast mb-1">
					{{ formatMessage(messages.createTitleLabel) }}
				</span>
				<span class="text-secondary mb-2">{{
					formatMessage(messages.createTitleDescription)
				}}</span>
			</label>
			<div class="flex items-center gap-2">
				<div class="iconified-input">
					<AutoBrandIcon :keyword="affiliateLinkTitle" aria-hidden="true">
						<AffiliateIcon />
					</AutoBrandIcon>
					<input
						id="create-affiliate-title-input"
						v-model="affiliateLinkTitle"
						class="card-shadow"
						autocomplete="off"
						spellcheck="false"
						type="text"
						:placeholder="formatMessage(messages.createTitlePlaceholder)"
					/>
					<Button v-if="affiliateLinkTitle" class="r-btn" @click="() => (affiliateLinkTitle = '')">
						<XIcon />
					</Button>
				</div>
				<ButtonStyled color="brand">
					<button :disabled="creatingLink || !canCreate" @click="createAffiliateLink">
						<SpinnerIcon v-if="creatingLink" class="animate-spin" />
						<PlusIcon v-else />
						{{ formatMessage(creatingLink ? messages.creatingButton : messages.createButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
<script lang="ts"></script>
<script setup lang="ts">
import { AffiliateIcon, PlusIcon, SpinnerIcon, UserIcon, XIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, useTemplateRef } from 'vue'

import { AutoBrandIcon, Button, ButtonStyled, NewModal } from '../index.ts'
export type CreateAffiliateProps = { sourceName: string; username?: string }

const props = withDefaults(
	defineProps<{
		showUserField?: boolean
		creatingLink?: boolean
	}>(),
	{
		showUserField: false,
		creatingLink: false,
	},
)

const emit = defineEmits<{
	(e: 'create', data: CreateAffiliateProps): void
}>()

const modal = useTemplateRef<typeof NewModal>('modal')
const { formatMessage } = useVIntl()

const affiliateLinkTitle = ref('')
const affiliateUsername = ref('')

const canCreate = computed(() => {
	if (props.showUserField) {
		return affiliateLinkTitle.value.trim() && affiliateUsername.value.trim()
	}
	return affiliateLinkTitle.value.trim()
})

function createAffiliateLink() {
	if (!canCreate.value) {
		return
	}

	emit('create', {
		sourceName: affiliateLinkTitle.value,
		username: props.showUserField ? affiliateUsername.value : undefined,
	})
}

function close() {
	modal.value?.hide()
	affiliateLinkTitle.value = ''
	affiliateUsername.value = ''
}

function show() {
	modal.value?.show()
}

defineExpose({
	show,
	close,
})

const messages = defineMessages({
	createHeader: {
		id: 'affiliate.create.header',
		defaultMessage: 'Creating new affiliate code',
	},
	createTitleLabel: {
		id: 'affiliate.create.title.label',
		defaultMessage: 'Title of affiliate link',
	},
	createTitleDescription: {
		id: 'affiliate.create.title.description',
		defaultMessage: 'Give your affiliate link a name so you know where people are coming from!',
	},
	createTitlePlaceholder: {
		id: 'affiliate.create.title.placeholder',
		defaultMessage: 'e.g. YouTube',
	},
	createUserLabel: {
		id: 'affiliate.create.user.label',
		defaultMessage: 'Username',
	},
	createUserDescription: {
		id: 'affiliate.create.user.description',
		defaultMessage: 'The username of the user to create the affiliate code for',
	},
	createUserPlaceholder: {
		id: 'affiliate.create.user.placeholder',
		defaultMessage: 'Enter username...',
	},
	createButton: {
		id: 'affiliate.create.button',
		defaultMessage: 'Create affiliate link',
	},
	creatingButton: {
		id: 'affiliate.creating.button',
		defaultMessage: 'Creating affiliate link...',
	},
})
</script>
