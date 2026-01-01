<script setup lang="ts">
import { FileTextIcon } from '@modrinth/assets'
import { ButtonStyled, defineMessages, PagewideBanner, useVIntl } from '@modrinth/ui'

import CreatorTaxFormModal from '~/components/ui/dashboard/CreatorTaxFormModal.vue'

const { formatMessage } = useVIntl()

const modal = useTemplateRef('modal')

const messages = defineMessages({
	title: {
		id: 'layout.banner.tax.title',
		defaultMessage: 'Tax form required',
	},
	description: {
		id: 'layout.banner.tax.description',
		defaultMessage:
			"You've already withdrawn over $600 from Modrinth this year. To comply with tax regulations, you need to complete a tax form. Your withdrawals are paused until this form is submitted.",
	},
	action: {
		id: 'layout.banner.tax.action',
		defaultMessage: 'Complete tax form',
	},
})

function openTaxForm(e: MouseEvent) {
	if (modal.value && modal.value.startTaxForm) {
		modal.value.startTaxForm(e)
	}
}
</script>

<template>
	<CreatorTaxFormModal ref="modal" close-button-text="Close" :emit-success-on-close="false" />
	<PagewideBanner variant="warning">
		<template #title>
			<span>{{ formatMessage(messages.title) }}</span>
		</template>
		<template #description>
			<span>{{ formatMessage(messages.description) }}</span>
		</template>
		<template #actions>
			<ButtonStyled color="orange">
				<button @click="openTaxForm"><FileTextIcon /> {{ formatMessage(messages.action) }}</button>
			</ButtonStyled>
		</template>
	</PagewideBanner>
</template>
