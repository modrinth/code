import type { Labrinth } from '@modrinth/api-client'
import { computed, type Ref } from 'vue'

import type { useProjectSaveValidation } from './project-save-validation'

export interface ProjectIssueFieldAction {
	draft: Partial<Labrinth.Projects.v3.Project>
	canSave: boolean
	save: () => Promise<boolean>
}

/** Supplies an inline issue card with a save operation scoped to its editable field. */
export function useProjectIssueFieldAction(options: {
	draft: () => Partial<Labrinth.Projects.v3.Project>
	canSave: () => boolean
	saving: Ref<boolean>
	validation: ReturnType<typeof useProjectSaveValidation>
	save: () => Promise<boolean | void>
}) {
	return computed<ProjectIssueFieldAction>(() => ({
		draft: options.draft(),
		canSave: options.canSave() && !options.saving.value,
		save: async () => {
			if (!options.canSave() || options.saving.value) return false
			const submittedState = options.validation.snapshot()
			options.saving.value = true
			try {
				const success = await options.save()
				if (success === false) return false
				options.validation.clear()
				return true
			} catch (error) {
				if (!options.validation.capture(error, submittedState)) throw error
				return false
			} finally {
				options.saving.value = false
			}
		},
	}))
}
