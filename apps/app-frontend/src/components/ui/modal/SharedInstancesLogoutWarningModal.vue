<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" fade="warning" max-width="560px">
		<div class="flex flex-col gap-4">
			<Admonition type="warning" :header="formatMessage(messages.admonitionHeader)">
				{{ formatMessage(messages.admonitionBody) }}
			</Admonition>

			<div v-if="visibleInstances.length > 0" class="relative">
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>
				<div
					ref="instanceListRef"
					class="flex max-h-[224px] flex-col gap-1 overflow-y-auto"
					@scroll="checkScrollState"
				>
					<Instance
						v-for="instance in visibleInstances"
						:key="instance.id"
						:instance="instance"
						list
						@select="hide"
					/>
				</div>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<ButtonStyled type="outlined">
					<button @click="hide">
						<XIcon aria-hidden="true" />
						{{ formatMessage(commonMessages.cancelButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="orange">
					<button @click="confirm">
						<LogOutIcon aria-hidden="true" />
						{{ formatMessage(messages.signOutButton) }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { LogOutIcon, XIcon } from '@modrinth/assets'
import {
	Admonition,
	ButtonStyled,
	commonMessages,
	defineMessages,
	NewModal,
	useScrollIndicator,
	useVIntl,
} from '@modrinth/ui'
import { nextTick, ref } from 'vue'

import Instance from '@/components/ui/Instance.vue'
import type { GameInstance } from '@/helpers/types'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	header: {
		id: 'app.sign-out.shared-instances-warning.header',
		defaultMessage: 'Sign out of Modrinth?',
	},
	admonitionHeader: {
		id: 'app.sign-out.shared-instances-warning.admonition-header',
		defaultMessage: 'Shared instances installed',
	},
	admonitionBody: {
		id: 'app.sign-out.shared-instances-warning.admonition-body',
		defaultMessage:
			'Signing out will prevent you from managing or receiving updates for shared instances linked to this Modrinth account until you sign back in.',
	},
	signOutButton: {
		id: 'app.sign-out.shared-instances-warning.sign-out-button',
		defaultMessage: 'Sign out',
	},
})

const emit = defineEmits<{
	(e: 'sign-out'): void
}>()

const modal = ref<InstanceType<typeof NewModal>>()
const instanceListRef = ref<HTMLElement | null>(null)
const visibleInstances = ref<GameInstance[]>([])
const {
	showTopFade,
	showBottomFade,
	checkScrollState,
	forceCheck: forceCheckScroll,
} = useScrollIndicator(instanceListRef)

async function show(instances: GameInstance[]) {
	visibleInstances.value = instances
	await nextTick()
	modal.value?.show()
	await nextTick()
	forceCheckScroll()
}

function hide() {
	modal.value?.hide()
}

function confirm() {
	modal.value?.hide()
	emit('sign-out')
}

defineExpose({
	show,
	hide,
})
</script>
