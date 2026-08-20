<script setup lang="ts">
import { XIcon } from '@modrinth/assets'
import { Button, defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { ref, useTemplateRef } from 'vue'

import ApplyNewIconsModal from './apply-new-icons-modal.vue'
import icon01 from './assets/01.png'
import icon02 from './assets/02.png'
import icon03 from './assets/03.png'
import icon04 from './assets/04.png'
import icon05 from './assets/05.png'
import icon06 from './assets/06.png'
import icon07 from './assets/07.png'
import icon08 from './assets/08.png'
import icon09 from './assets/09.png'

const emit = defineEmits<{
	dismiss: []
}>()

const { formatMessage } = useVIntl()
const applyNewIconsModal =
	useTemplateRef<InstanceType<typeof ApplyNewIconsModal>>('applyNewIconsModal')
const modalOpen = ref(false)
const icons = [icon01, icon02, icon03, icon04, icon05, icon06, icon07, icon08, icon09]

const messages = defineMessages({
	title: {
		id: 'app.icon-editor.notification.title',
		defaultMessage: 'New icon editor',
	},
	description: {
		id: 'app.icon-editor.notification.description',
		defaultMessage: 'Personalize your instances!',
	},
	viewUpdate: {
		id: 'app.icon-editor.notification.view-update',
		defaultMessage: 'View update',
	},
	dismiss: {
		id: 'app.icon-editor.notification.dismiss',
		defaultMessage: 'Dismiss',
	},
	close: {
		id: 'app.icon-editor.notification.close',
		defaultMessage: 'Close',
	},
})

function viewUpdate() {
	modalOpen.value = true
	applyNewIconsModal.value?.show()
}

function handleModalClose() {
	modalOpen.value = false
	emit('dismiss')
}
</script>

<template>
	<section
		v-show="!modalOpen"
		class="relative ml-auto w-full max-w-full overflow-hidden rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 shadow-xl"
		:aria-label="formatMessage(messages.title)"
	>
		<div class="icon-grid" aria-hidden="true">
			<div v-for="(icon, index) in icons" :key="index" class="icon-tile">
				<img :src="icon" alt="" />
			</div>
		</div>
		<div class="icon-fade" />

		<div class="relative z-[1] flex w-full flex-col items-start gap-3">
			<div class="flex w-full flex-col gap-2">
				<div class="flex w-full items-center justify-between gap-2.5">
					<div class="min-w-0 flex-1 font-semibold text-contrast">
						{{ formatMessage(messages.title) }}
					</div>
					<IconButton
						class="-m-1.5 -mx-2"
						type="quiet"
						size="sm"
						:label="formatMessage(messages.close)"
						@click="emit('dismiss')"
					>
						<XIcon />
					</IconButton>
				</div>
				<div class="text-primary">
					{{ formatMessage(messages.description) }}
				</div>
			</div>

			<div class="flex items-center gap-1.5">
				<Button type="colored" color="brand" @click="viewUpdate">
					{{ formatMessage(messages.viewUpdate) }}
				</Button>
				<Button @click="emit('dismiss')">
					{{ formatMessage(messages.dismiss) }}
				</Button>
			</div>
		</div>
	</section>
	<ApplyNewIconsModal ref="applyNewIconsModal" @close="handleModalClose" />
</template>

<style scoped>
.icon-grid {
	position: absolute;
	top: -4px;
	left: 232px;
	display: grid;
	width: 160px;
	height: 160px;
	grid-template-columns: repeat(3, 1fr);
	grid-template-rows: repeat(3, 1fr);
	gap: 8px;
	transform: rotate(23.89deg);
}

.icon-tile {
	min-width: 0;
	min-height: 0;
	overflow: hidden;
	border: 1px solid rgba(255, 255, 255, 0.15);
	border-radius: 16px;
	box-shadow: 0 1px 1px rgba(0, 0, 0, 0.12);
}

.icon-tile img {
	display: block;
	width: 100%;
	height: 100%;
	object-fit: cover;
}

.icon-fade {
	position: absolute;
	inset: 0 auto 0 210px;
	width: 110px;
	background: linear-gradient(90deg, var(--surface-3) 0%, transparent 100%);
}
</style>
