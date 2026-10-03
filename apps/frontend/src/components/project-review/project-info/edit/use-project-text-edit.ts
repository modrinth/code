import { defineMessages, useVIntl } from '@modrinth/ui'
import { computed, nextTick, ref, watch } from 'vue'

import { useProjectInfoEdit } from './use-project-edit'

type TextField = 'name' | 'slug' | 'summary' | 'description'

const limits: Record<TextField, { min: number; max: number }> = {
	name: { min: 3, max: 64 },
	slug: { min: 3, max: 64 },
	summary: { min: 3, max: 256 },
	description: { min: 0, max: 65536 },
}

const messages = defineMessages({
	length: {
		id: 'moderation.project-review.validation.length',
		defaultMessage: 'Enter between {min} and {max} characters.',
	},
	slug: {
		id: 'moderation.project-review.validation.slug',
		defaultMessage: 'Use only letters, numbers, periods, underscores, and hyphens.',
	},
})

export function useProjectTextEdit(field: TextField, focus: () => unknown) {
	const { formatMessage } = useVIntl()
	const { project, saving, beginEditing, saveProject } = useProjectInfoEdit()
	const editing = ref(false)
	const draft = ref('')
	const value = computed(() => (field === 'description' ? draft.value : draft.value.trim()))
	const validationMessage = computed(() => {
		const length = Array.from(value.value).length
		if (length < limits[field].min || length > limits[field].max) {
			return formatMessage(messages.length, limits[field])
		}
		if (field === 'slug' && !/^[a-zA-Z0-9._-]+$/.test(value.value)) {
			return formatMessage(messages.slug)
		}
		return ''
	})
	const valid = computed(() => !validationMessage.value)

	watch(
		() => project.value?.id,
		() => {
			editing.value = false
			draft.value = ''
		},
		{ flush: 'sync' },
	)

	async function startEditing() {
		if (!project.value || editing.value || saving.value) return
		beginEditing()
		draft.value = project.value[field] ?? ''
		editing.value = true
		await nextTick()
		focus()
	}

	function resetEditing() {
		if (saving.value) return
		draft.value = ''
		editing.value = false
	}

	async function save() {
		if (!editing.value || !valid.value || saving.value) return
		if (value.value === project.value?.[field] || (await saveProject({ [field]: value.value }))) {
			editing.value = false
		}
	}

	return {
		project,
		saving,
		editing,
		draft,
		valid,
		validationMessage,
		startEditing,
		resetEditing,
		save,
	}
}
