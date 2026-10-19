<template>
	<label :for="inputId" hidden>{{ formatMessage(commonMessages.passwordLabel) }}</label>
	<Input
		:id="inputId"
		v-model="passwordModel"
		:icon="KeyIcon"
		type="password"
		autocomplete="current-password"
		:placeholder="formatMessage(commonMessages.passwordLabel)"
		wrapper-class="w-full"
		@keyup.enter="onSubmit"
	/>

	<HCaptcha v-if="captchaEnabled && showCaptcha" :ref="onSetCaptchaRef" v-model="tokenModel" />

	<Button
		type="colored"
		color="brand"
		class="!w-full"
		:disabled="captchaEnabled && !tokenModel"
		@click="onSubmit"
	>
		{{ submitLabel }} <RightArrowIcon />
	</Button>
</template>

<script setup lang="ts">
import { KeyIcon, RightArrowIcon } from '@modrinth/assets'
import { Button, commonMessages, Input, useVIntl } from '@modrinth/ui'

import HCaptcha from '@/components/ui/auth/HCaptcha.vue'

interface Props {
	inputId: string
	submitLabel: string
	captchaEnabled?: boolean
	showCaptcha?: boolean
	onSubmit?: () => void
	onSetCaptchaRef?: ((captchaRef: unknown) => void) | undefined
}

const {
	captchaEnabled = false,
	showCaptcha = false,
	onSubmit = () => {},
	onSetCaptchaRef = undefined,
} = defineProps<Props>()

const passwordModel = defineModel<string>('password', { default: '' })
const tokenModel = defineModel<string>('token', { default: '' })
const { formatMessage } = useVIntl()
</script>
