<template>
	<NewModal ref="modal" :scrollable="true" max-content-height="82vh" :closable="true">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">{{
				formatMessage(bulkEnvironment ? messages.applyToAllVersions : messages.title)
			}}</span>
		</template>
		<div class="max-w-[600px]">
			<EnvironmentMigration
				:key="bulkEnvironment ?? 'project'"
				ref="environmentMigration"
				:show-floating-save="false"
				:bulk-environment="bulkEnvironment"
			/>
		</div>
		<template #actions>
			<div v-if="canSave" class="flex justify-end gap-2 mt-2">
				<Button
					v-if="canReset"
					type="quiet"
					:disabled="saving || !hasChanges"
					@click="resetEnvironment"
				>
					<HistoryIcon /> {{ formatMessage(commonMessages.resetButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="saving || !hasChanges"
					@click="saveEnvironment"
				>
					<SpinnerIcon v-if="saving" class="animate-spin" />
					<CheckIcon v-else-if="needsToVerify" />
					<SaveIcon v-else />
					{{ saveButtonLabel }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, HistoryIcon, SaveIcon, SpinnerIcon } from '@modrinth/assets'
import { computed, onMounted, ref, unref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import { Button } from '#ui/components/base/buttons'
import { defineMessages, useVIntl } from '#ui/composables/i18n'

import { commonMessages } from '../../../../utils/common-messages'
import { NewModal } from '../../../modal'
import EnvironmentMigration from './EnvironmentMigration.vue'
import { ENVIRONMENTS_COPY } from './environments'

const { formatMessage } = useVIntl()
const route = useRoute()
const router = useRouter()
const bulkEnvironment = ref<Labrinth.Projects.v3.Environment>()

const messages = defineMessages({
	title: {
		id: 'project.settings.environment.modal.title',
		defaultMessage: 'Edit project environment',
	},
	applyToAllVersions: {
		id: 'project.settings.environment.bulk.apply-button',
		defaultMessage: 'Apply to all versions',
	},
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
	if (bulkEnvironment.value) {
		return formatMessage(messages.applyToAllVersions)
	}
	if (needsToVerify.value) {
		return formatMessage(messages.verifyButton)
	}
	return formatMessage(commonMessages.saveButton)
})

function show(environment?: Labrinth.Projects.v3.Environment) {
	bulkEnvironment.value = environment
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function resetEnvironment() {
	environmentMigration.value?.reset()
}

async function saveEnvironment() {
	const shouldClose = !!bulkEnvironment.value || needsToVerify.value
	const saved = await environmentMigration.value?.save()
	if (saved && shouldClose) {
		hide()
	}
}

onMounted(() => {
	watch(
		() => route.query.applyEnvironmentOnAllVersions,
		async (environment) => {
			if (
				typeof environment !== 'string' ||
				environment === 'unknown' ||
				!Object.hasOwn(ENVIRONMENTS_COPY, environment)
			) {
				return
			}
			show(environment as Labrinth.Projects.v3.Environment)
			const query = { ...route.query }
			delete query.applyEnvironmentOnAllVersions
			await router.replace({ query, hash: route.hash })
		},
		{ immediate: true, flush: 'post' },
	)
	if (!bulkEnvironment.value && route.query.showEnvironmentMigrationWarning === 'true') {
		show()
	}
})

defineExpose({
	show,
	hide,
})
</script>
