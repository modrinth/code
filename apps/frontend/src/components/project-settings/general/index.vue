<template>
	<div>
		<About ref="about" :page-saving="saving" />

		<License v-if="!isServerProject" ref="licenseSettings" class="mt-8" />

		<DangerZone class="mt-8" />
		<ValidationMessage :check="about?.validationMessages ?? []" class="my-4" />
		<UnsavedChangesPopup
			:original="{ ...about?.original, license: licenseSettings?.saved }"
			:modified="{ ...about?.modified, license: licenseSettings?.current }"
			:saving="saving || Boolean(about?.saving)"
			:can-save="canSave"
			@reset="resetChanges"
			@save="handleSave"
		/>
		<ConfirmLeaveModal ref="confirmLeaveModal" />
	</div>
</template>

<script setup lang="ts">
import {
	ConfirmLeaveModal,
	injectProjectPageContext,
	UnsavedChangesPopup,
	usePageLeaveSafety,
} from '@modrinth/ui'

import ValidationMessage from '~/components/ValidationMessage.vue'

import About from './about.vue'
import DangerZone from './danger-zone/index.vue'
import License from './license.vue'

const { projectV3: project } = injectProjectPageContext()

const about = useTemplateRef('about')
const licenseSettings = useTemplateRef('licenseSettings')
const saving = ref(false)
const isServerProject = computed(() => project.value?.minecraft_server != null)
const hasChanges = computed(() =>
	Boolean(about.value?.hasChanges || licenseSettings.value?.hasChanges),
)
const canSave = computed(
	() =>
		Boolean(about.value?.canSave) &&
		(!licenseSettings.value?.hasChanges || licenseSettings.value.canSave),
)
const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

function resetChanges() {
	licenseSettings.value?.reset()
	about.value?.reset()
}

async function handleSave() {
	if (!canSave.value || saving.value || about.value?.saving) return
	saving.value = true
	try {
		if (!(await about.value?.save())) return
		if (licenseSettings.value?.hasChanges) await licenseSettings.value.save()
	} finally {
		saving.value = false
	}
}
</script>
