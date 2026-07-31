<template>
	<NewModal ref="modal" :header="formatMessage(messages.createHeader)">
		<div class="flex flex-col">
			<label v-if="showUserField" class="contents" for="create-affiliate-user-input">
				<span class="text-lg font-semibold text-contrast mb-1">
					{{ formatMessage(commonMessages.usernameLabel) }}
				</span>
				<span class="text-secondary mb-2">{{ formatMessage(messages.createUserDescription) }}</span>
			</label>
			<StyledInput
				v-if="showUserField"
				id="create-affiliate-user-input"
				v-model="affiliateUsername"
				:icon="UserIcon"
				autocomplete="off"
				type="text"
				:placeholder="formatMessage(messages.createUserPlaceholder)"
				clearable
				wrapper-class="mb-4"
			/>
			<label class="contents" for="create-affiliate-title-input">
				<span class="text-lg font-semibold text-contrast mb-1">
					{{ formatMessage(messages.createTitleLabel) }}
				</span>
				<span class="text-secondary mb-2">{{
					formatMessage(messages.createTitleDescription)
				}}</span>
			</label>
			<div class="flex items-center gap-2">
				<div class="relative inline-flex items-center flex-1">
					<AutoBrandIcon
						:keyword="affiliateLinkTitle"
						aria-hidden="true"
						class="absolute left-3 h-5 w-5 z-[1] pointer-events-none text-secondary"
					>
						<AffiliateIcon />
					</AutoBrandIcon>
					<StyledInput
						id="create-affiliate-title-input"
						v-model="affiliateLinkTitle"
						autocomplete="off"
						type="text"
						:placeholder="formatMessage(messages.createTitlePlaceholder)"
						clearable
						wrapper-class="w-full"
						input-class="pl-10"
					/>
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
import { AffiliateIcon, PlusIcon, SpinnerIcon, UserIcon } from '@modrinth/assets'
import { computed, ref, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { commonMessages } from '../../utils/common-messages'
import { AutoBrandIcon, ButtonStyled, NewModal, StyledInput } from '../index'
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
