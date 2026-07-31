<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		:on-hide="handleHide"
		:on-after-hide="handleAfterHide"
		max-width="544px"
		width="544px"
	>
		<div class="flex flex-col items-end gap-6">
			<Admonition
				type="warning"
				:header="
					formatMessage(
						mode === 'modpack' ? messages.modpackWarningTitle : messages.modWarningTitle,
					)
				"
				class="w-full"
			>
				<span class="font-medium text-contrast">{{ fileName }}</span>
				{{
					formatMessage(mode === 'modpack' ? messages.modpackWarningBody : messages.modWarningBody)
				}}
			</Admonition>

			<p class="m-0 w-full leading-6 text-primary">
				{{ formatMessage(messages.reviewedFiles) }}
			</p>

			<div v-if="mode === 'modpack'" class="relative w-full">
				<div
					ref="externalFileTableBody"
					class="max-h-[242px] overflow-y-auto rounded-2xl"
					@scroll="checkTableScrollState"
				>
					<Table
						:columns="externalFileColumns"
						:data="externalFileRows"
						row-key="id"
						virtualized
						:virtual-row-height="48"
						class="shadow-sm"
					>
						<template #cell-name="{ value }">
							<span class="block truncate" :title="String(value)">{{ value }}</span>
						</template>
					</Table>
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
						v-if="showTableTopFade"
						class="pointer-events-none absolute left-0 right-0 top-0 z-10 h-2 bg-gradient-to-b from-bg-raised to-transparent"
					/>
				</Transition>
				<Transition
					enter-active-class="transition-all duration-200 ease-out"
					enter-from-class="opacity-0 max-h-0"
					enter-to-class="opacity-100 max-h-2"
					leave-active-class="transition-all duration-200 ease-in"
					leave-from-class="opacity-100 max-h-2"
					leave-to-class="opacity-0 max-h-0"
				>
					<div
						v-if="showTableBottomFade"
						class="pointer-events-none absolute bottom-0 left-0 right-0 z-10 h-2 bg-gradient-to-t from-bg-raised to-transparent"
					/>
				</Transition>
			</div>

			<p class="m-0 w-full font-medium leading-6 text-orange">
				{{ formatMessage(messages.malwareWarning) }}
			</p>

			<Checkbox
				v-if="mode === 'mod'"
				v-model="dontShowAgain"
				class="w-full"
				:label="formatMessage(messages.dontShowAgain)"
			/>

			<div class="flex items-center gap-2">
				<ButtonStyled type="transparent" color="orange">
					<button type="button" @click="continueInstallation">
						{{ formatMessage(messages.installAnyway) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button type="button" @click="cancelInstallation">
						<BanIcon aria-hidden="true" />
						{{ formatMessage(messages.dontInstall) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { BanIcon } from '@modrinth/assets'
import { computed, nextTick, ref, useTemplateRef } from 'vue'

import { defineMessages, useVIntl } from '../../composables/i18n'
import { useScrollIndicator } from '../../composables/scroll-indicator'
import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import Checkbox from '../base/Checkbox.vue'
import Table, { type TableColumn } from '../base/Table.vue'
import NewModal from './NewModal.vue'

const props = withDefaults(
	defineProps<{
		mode: 'modpack' | 'mod'
		fileName: string
		externalFilesInModpack?: string[]
	}>(),
	{
		externalFilesInModpack: () => [],
	},
)

const emit = defineEmits<{
	cancel: []
	continue: [dontShowAgain: boolean]
}>()

const { formatMessage } = useVIntl()
const modal = useTemplateRef('modal')
const externalFileTableBody = ref<HTMLElement | null>(null)
const dontShowAgain = ref(false)
let pendingAction: { type: 'cancel' } | { type: 'continue'; dontShowAgain: boolean } | null = null
const {
	showTopFade: showTableTopFade,
	showBottomFade: showTableBottomFade,
	checkScrollState: checkTableScrollState,
	forceCheck: forceCheckTableScroll,
} = useScrollIndicator(externalFileTableBody)

const messages = defineMessages({
	header: {
		id: 'unknown-file-warning-modal.header',
		defaultMessage: 'Confirm installation',
	},
	modpackWarningTitle: {
		id: 'unknown-file-warning-modal.modpack-warning-title',
		defaultMessage: 'Unknown files warning',
	},
	modWarningTitle: {
		id: 'unknown-file-warning-modal.mod-warning-title',
		defaultMessage: 'Unknown file warning',
	},
	modpackWarningBody: {
		id: 'unknown-file-warning-modal.modpack-warning-body',
		defaultMessage:
			' contains files that aren’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	modWarningBody: {
		id: 'unknown-file-warning-modal.mod-warning-body',
		defaultMessage:
			' isn’t published on Modrinth. We strongly recommend only installing files from sources you trust.',
	},
	reviewedFiles: {
		id: 'unknown-file-warning-modal.reviewed-files',
		defaultMessage:
			'A file is only reviewed if it’s published to Modrinth, regardless of its file format (including .mrpack).',
	},
	unrecognizedFiles: {
		id: 'unknown-file-warning-modal.unrecognized-files',
		defaultMessage: 'Unrecognized files',
	},
	malwareWarning: {
		id: 'unknown-file-warning-modal.malware-warning',
		defaultMessage:
			'Malware is often distributed through mod files by sharing them on platforms like Discord.',
	},
	dontShowAgain: {
		id: 'unknown-file-warning-modal.dont-show-again',
		defaultMessage: 'Don’t show this warning again',
	},
	installAnyway: {
		id: 'unknown-file-warning-modal.install-anyway',
		defaultMessage: 'Install anyway',
	},
	dontInstall: {
		id: 'unknown-file-warning-modal.dont-install',
		defaultMessage: 'Dont install',
	},
})

type ExternalFileColumn = 'name'
type ExternalFileRow = {
	id: string
	name: string
}

const externalFileColumns = computed<TableColumn<ExternalFileColumn>[]>(() => [
	{
		key: 'name',
		label: formatMessage(messages.unrecognizedFiles),
		cellClass: '!h-12',
	},
])
const externalFileRows = computed<ExternalFileRow[]>(() =>
	props.externalFilesInModpack.map((name, index) => ({
		id: `${index}-${name}`,
		name,
	})),
)

async function show() {
	dontShowAgain.value = false
	pendingAction = null
	modal.value?.show()
	await nextTick()
	forceCheckTableScroll()
}

function hide() {
	modal.value?.hide()
}

function handleHide() {
	pendingAction ??= { type: 'cancel' }
}

function handleAfterHide() {
	const action = pendingAction
	pendingAction = null
	dontShowAgain.value = false

	if (action?.type === 'continue') {
		emit('continue', action.dontShowAgain)
	} else {
		emit('cancel')
	}
}

function cancelInstallation() {
	pendingAction = { type: 'cancel' }
	modal.value?.hide()
}

function continueInstallation() {
	pendingAction = { type: 'continue', dontShowAgain: dontShowAgain.value }
	modal.value?.hide()
}

defineExpose({ show, hide })
</script>
