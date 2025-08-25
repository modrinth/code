<script setup lang="ts">
import { PlusIcon, XIcon } from '@modrinth/assets'
import { defineMessages, useVIntl } from '@vintl/vintl'

import { commonMessages } from '../../utils'
import { ButtonStyled, NewModal } from '../index'
import type { AddPaymentMethodProps } from './AddPaymentMethod.vue'
import AddPaymentMethod from './AddPaymentMethod.vue'

const { formatMessage } = useVIntl()

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const addPaymentMethod = useTemplateRef<InstanceType<typeof AddPaymentMethod>>('addPaymentMethod')

const props = defineProps<AddPaymentMethodProps>()
const loading = ref(false)

async function open(paymentMethods: Stripe.PaymentMethod[]) {
	modal.value?.show()
	await nextTick()
	await addPaymentMethod.value?.reload(paymentMethods)
}

const messages = defineMessages({
	addingPaymentMethod: {
		id: 'modal.add-payment-method.title',
		defaultMessage: 'Adding a payment method',
	},
	paymentMethodAdd: {
		id: 'modal.add-payment-method.action',
		defaultMessage: 'Add payment method',
	},
})

defineExpose({
	show: open,
})
</script>

<template>
	<NewModal ref="modal">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">
				{{ formatMessage(messages.addingPaymentMethod) }}
			</span>
		</template>
		<div class="w-[40rem] max-w-full">
			<AddPaymentMethod
				ref="addPaymentMethod"
				:publishable-key="props.publishableKey"
				:return-url="props.returnUrl"
				:create-setup-intent="props.createSetupIntent"
				:on-error="props.onError"
				@start-loading="loading = true"
				@stop-loading="loading = false"
			/>
			<div class="input-group mt-auto pt-4">
				<ButtonStyled color="brand">
					<button :disabled="loading" @click="addPaymentMethod.submit()">
						<PlusIcon />
						{{ formatMessage(messages.paymentMethodAdd) }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="modal.hide()">
						<XIcon />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>
