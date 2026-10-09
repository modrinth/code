<template>
	<Admonition
		v-if="isRestricted"
		type="critical"
		:header="formatMessage(messages.header)"
		class="mb-4"
	>
		<div class="flex w-full flex-col gap-4">
			{{ formatMessage(messages.description) }}
			<div class="w-min">
				<ButtonLink
					type="colored"
					color="red"
					to="/settings/account#account-standing"
					@click="emit('navigate')"
				>
					<ShieldAlertIcon />{{ formatMessage(messages.viewStanding) }}
				</ButtonLink>
			</div>
		</div>
	</Admonition>
</template>

<script setup lang="ts">
import { ShieldAlertIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonLink,
	defineMessages,
	injectAuth,
	isScopeRemovedForUser,
	useVIntl,
} from '@modrinth/ui'
import { computed, watch } from 'vue'

const { formatMessage } = useVIntl()
const auth = injectAuth()

const messages = defineMessages({
	header: {
		id: 'create.restricted-alert.header',
		defaultMessage: 'Action restricted',
	},
	description: {
		id: 'create.restricted-alert.description',
		defaultMessage: 'A moderator has removed this permission from your account.',
	},
	viewStanding: {
		id: 'create.restricted-alert.view-standing',
		defaultMessage: 'View account standing',
	},
})

const props = defineProps<{
	scope: string
}>()

const emit = defineEmits<{
	navigate: []
}>()

const model = defineModel<boolean>()

const isRestricted = computed(() => isScopeRemovedForUser(auth.user.value, props.scope))

watch(
	isRestricted,
	(newValue) => {
		model.value = newValue
	},
	{ immediate: true },
)
</script>
