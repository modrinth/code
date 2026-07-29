<template>
	<Modal ref="detectJavaModal" :header="formatMessage(messages.title)">
		<div class="flex flex-col gap-4">
			<Table :columns="javaInstallColumns" :data="chosenInstallOptions" row-key="path">
				<template #cell-version="{ value }">
					<span class="font-semibold text-primary">{{ value }}</span>
				</template>
				<template #cell-path="{ value }">
					<span v-tooltip="value" class="block truncate font-mono text-xs">{{ value }}</span>
				</template>
				<template #cell-actions="{ row }">
					<div class="flex items-center justify-end">
						<ButtonStyled v-if="currentSelected.path === row.path">
							<button class="!shadow-none" disabled>
								<CheckIcon aria-hidden="true" />
								{{ formatMessage(messages.selected) }}
							</button>
						</ButtonStyled>
						<ButtonStyled v-else>
							<button class="!shadow-none" @click="setJavaInstall(row)">
								<PlusIcon aria-hidden="true" />
								{{ formatMessage(messages.select) }}
							</button>
						</ButtonStyled>
					</div>
				</template>
				<template #empty-state>
					<div class="p-4 text-secondary">
						{{ formatMessage(messages.noInstallationsFound) }}
					</div>
				</template>
			</Table>
			<div class="flex justify-end">
				<ButtonStyled type="outlined">
					<button
						class="!shadow-none !border-surface-4 !border"
						@click="$refs.detectJavaModal.hide()"
					>
						<XIcon aria-hidden="true" />
						{{ formatMessage(messages.cancel) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</Modal>
</template>
<script setup>
import { CheckIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	defineMessages,
	injectNotificationManager,
	Modal,
	Table,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { trackEvent } from '@/helpers/analytics'
import { find_filtered_jres } from '@/helpers/jre.js'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.java-detection.title',
		defaultMessage: 'Select Java installation',
	},
	versionColumn: {
		id: 'app.java-detection.columns.version',
		defaultMessage: 'Version',
	},
	pathColumn: {
		id: 'app.java-detection.columns.path',
		defaultMessage: 'Path',
	},
	actionsColumn: {
		id: 'app.java-detection.columns.actions',
		defaultMessage: 'Actions',
	},
	selected: {
		id: 'app.java-detection.selected',
		defaultMessage: 'Selected',
	},
	select: {
		id: 'app.java-detection.select',
		defaultMessage: 'Select',
	},
	noInstallationsFound: {
		id: 'app.java-detection.no-installations-found',
		defaultMessage: 'No Java installations found.',
	},
	cancel: {
		id: 'app.java-detection.cancel',
		defaultMessage: 'Cancel',
	},
})

const chosenInstallOptions = ref([])
const detectJavaModal = ref(null)
const currentSelected = ref({})
const javaInstallColumns = computed(() => [
	{ key: 'version', label: formatMessage(messages.versionColumn), width: '9rem' },
	{ key: 'path', label: formatMessage(messages.pathColumn) },
	{
		key: 'actions',
		label: formatMessage(messages.actionsColumn),
		align: 'right',
		width: '10rem',
	},
])

defineExpose({
	show: async (version, currentSelectedJava) => {
		chosenInstallOptions.value = await find_filtered_jres(version).catch(handleError)

		currentSelected.value = currentSelectedJava
		if (!currentSelected.value) {
			currentSelected.value = { path: '', version: '' }
		}

		detectJavaModal.value.show()
	},
})

const emit = defineEmits(['submit'])

function setJavaInstall(javaInstall) {
	emit('submit', javaInstall)
	detectJavaModal.value.hide()
	trackEvent('JavaAutoDetect', {
		path: javaInstall.path,
		version: javaInstall.version,
	})
}
</script>
