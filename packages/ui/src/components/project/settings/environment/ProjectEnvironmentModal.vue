<template>
	<NewModal ref="modal" :scrollable="true" max-content-height="82vh" :closable="true">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Edit project Environment</span>
		</template>
		<div class="max-w-[600px]">
			<EnvironmentMigration ref="environmentMigration" :show-floating-save="false" />
		</div>
		<template #actions>
			<div v-if="canSave" class="flex justify-end gap-2 mt-2">
				<ButtonStyled v-if="canReset" type="transparent">
					<button :disabled="saving || !hasChanges" @click="resetEnvironment">
						<HistoryIcon /> {{ formatMessage(commonMessages.resetButton) }}
					</button>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="saving || !hasChanges" @click="saveEnvironment">
						<SpinnerIcon v-if="saving" class="animate-spin" />
						<CheckIcon v-else-if="needsToVerify" />
						<SaveIcon v-else />
						{{ saveButtonLabel }}
					</button>
				</ButtonStyled>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import { CheckIcon, HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, onMounted, unref, useTemplateRef } from 'vue'
import { useRoute } from 'vue-router'

import { defineMessages, useVIntl } from '../../../../composables/i18n'
import { commonMessages } from '../../../../utils/common-messages'
import ButtonStyled from '../../../base/ButtonStyled.vue'
import { NewModal } from '../../../modal'
import EnvironmentMigration from './EnvironmentMigration.vue'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	verifyButton: {
		id: 'project.settings.environment.verification.verify-button',
		defaultMessage: 'Verify',
	},
})

const modal = useTemplateRef<InstanceType<typeof NewModal>>('modal')
const environmentMigration =
	useTemplateRef<InstanceType<typeof EnvironmentMigration>>('environmentMigration')

const hasChanges = computed(() => unref(environmentMigration.value?.hasChanges) ?? false)
const saving = computed(() => unref(environmentMigration.value?.saving) ?? false)
const canReset = computed(() => unref(environmentMigration.value?.canReset) ?? false)
const canSave = computed(() => unref(environmentMigration.value?.canSave) ?? false)
const needsToVerify = computed(() => unref(environmentMigration.value?.needsToVerify) ?? false)
const saveButtonLabel = computed(() => {
	if (saving.value) {
		return formatMessage(commonMessages.savingButton)
	}
	if (needsToVerify.value) {
		return formatMessage(messages.verifyButton)
	}
	return formatMessage(commonMessages.saveButton)
})

function show() {
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function resetEnvironment() {
	environmentMigration.value?.reset()
}

function saveEnvironment() {
	const shouldVerify = !!needsToVerify.value
	environmentMigration.value?.save()
	if (shouldVerify) {
		hide()
	}
}

onMounted(() => {
	const route = useRoute()
	if (route.query.showEnvironmentMigrationWarning === 'true') {
		show()
	}
})

defineExpose({
	show,
	hide,
})
</script>
