<template>
	<NewModal
		ref="modal"
		:noblur="noblur"
		:danger="danger"
		:on-hide="onHide"
		max-width="800px"
		:header="title"
	>
		<div class="flex flex-col gap-4">
			<template v-if="description">
				<div
					v-if="markdown"
					class="markdown-body max-w-[35rem]"
					v-html="renderString(description)"
				/>
				<p v-else class="max-w-[35rem] m-0">
					{{ description }}
				</p>
			</template>
			<slot />
			<label v-if="hasToType" for="confirmation">
				<span>
					To confirm you want to proceed, type
					<span class="font-semibold text-contrast">{{ confirmationText }}</span> below:
				</span>
			</label>
			<StyledInput
				v-if="hasToType"
				id="confirmation"
				v-model="confirmation_typed"
				placeholder="Type here..."
				wrapper-class="max-w-[20rem]"
			/>
			<div class="flex gap-2 justify-end">
				<Button @click="hide()">
					<XIcon />
					Cancel
				</Button>
				<Button
					type="colored"
					:color="danger ? 'red' : 'brand'"
					:disabled="action_disabled"
					@click="proceed"
				>
					<component :is="proceedIcon" />
					{{ proceedLabel }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<script setup>
import { TrashIcon, XIcon } from '@modrinth/assets'
import { renderString } from '@modrinth/utils'
import { computed, ref } from 'vue'

import { Button } from '#ui/components/base/buttons'

import StyledInput from '../base/StyledInput.vue'
import NewModal from './NewModal.vue'

const props = defineProps({
	confirmationText: {
		type: String,
		default: '',
	},
	hasToType: {
		type: Boolean,
		default: false,
	},
	title: {
		type: String,
		default: 'No title defined',
		required: true,
	},
	description: {
		type: String,
		default: undefined,
		required: false,
	},
	proceedIcon: {
		type: Object,
		default: () => TrashIcon,
	},
	proceedLabel: {
		type: String,
		default: 'Proceed',
	},
	noblur: {
		type: Boolean,
		default: false,
	},
	danger: {
		type: Boolean,
		default: true,
	},
	onHide: {
		type: Function,
		default() {
			return () => {}
		},
	},
	markdown: {
		type: Boolean,
		default: true,
	},
})

const emit = defineEmits(['proceed'])
const modal = ref(null)

const confirmation_typed = ref('')

const action_disabled = computed(
	() =>
		props.hasToType &&
		confirmation_typed.value.toLowerCase() !== props.confirmationText.toLowerCase(),
)

function proceed() {
	modal.value.hide()
	confirmation_typed.value = ''
	emit('proceed')
}

function show() {
	modal.value.show()
}
function hide() {
	modal.value.hide()
}

defineExpose({ show, hide })
</script>
