import {
	checkLink,
	extractProjectLinks,
	getLinkCheckState,
	type LinkCheckContext,
	type LinkCheckResult,
	type ProjectTextValidationResult,
	type ProjectTitleMetadata,
	validateProjectDescription,
	validateProjectSummary,
	validateProjectTitle,
} from '@modrinth/moderation'
import { defineMessages } from '@modrinth/ui'
import { computed, type MaybeRefOrGetter, onScopeDispose, ref, toValue, watch } from 'vue'

export const projectTextValidationMessages = defineMessages({
	resolveIssuesToSave: {
		id: 'project.text-validation.resolve-issues-to-save',
		defaultMessage: 'Resolve the issues from your edits to save.',
	},
})

function useProjectTitleMetadata() {
	const generatedState = useGeneratedState()

	return computed<ProjectTitleMetadata>(() => ({
		gameVersions: generatedState.value.gameVersions.map(({ version }) => version),
		loaders: generatedState.value.loaders.map(({ name }) => name),
	}))
}

export function useProjectTitleValidation(text: MaybeRefOrGetter<string | null | undefined>) {
	const metadata = useProjectTitleMetadata()
	return computed(() => validateProjectTitle(toValue(text), metadata.value))
}

export function useProjectSummaryValidation(
	summary: MaybeRefOrGetter<string | null | undefined>,
	title: MaybeRefOrGetter<string | null | undefined>,
) {
	return computed(() => validateProjectSummary(toValue(summary), toValue(title)))
}

export function useProjectDescriptionValidation(
	description: MaybeRefOrGetter<string | null | undefined>,
) {
	const linkValidation = ref<LinkCheckResult | null>(null)
	const pending = ref(false)
	let debounceTimer: ReturnType<typeof setTimeout> | undefined
	let requestId = 0

	watch(
		() => toValue(description),
		(text) => {
			clearTimeout(debounceTimer)
			const currentRequestId = ++requestId
			linkValidation.value = null

			if (import.meta.server) return

			const links = extractProjectLinks(text ?? '')
			if (links.length === 0) {
				pending.value = false
				return
			}

			pending.value = true
			debounceTimer = setTimeout(async () => {
				const contexts: LinkCheckContext[] = links.map((url) => ({
					field: 'description',
					generalContent: true,
					url,
				}))

				try {
					await Promise.all(contexts.map((context) => checkLink(context)))
					if (currentRequestId !== requestId) return

					const checks = contexts
						.map((context) => getLinkCheckState(context))
						.filter((check): check is LinkCheckResult => check !== undefined)
					linkValidation.value =
						checks.find((check) => check.severity === 'error') ??
						checks.find((check) => check.severity === 'warn') ??
						null
				} catch {
					if (currentRequestId === requestId) linkValidation.value = null
				} finally {
					if (currentRequestId === requestId) pending.value = false
				}
			}, 500)
		},
		{ immediate: true },
	)

	onScopeDispose(() => {
		clearTimeout(debounceTimer)
		requestId++
	})

	const validation = computed<ProjectTextValidationResult | LinkCheckResult | null>(
		() => validateProjectDescription(toValue(description)) ?? linkValidation.value,
	)

	return {
		pending,
		validation,
	}
}
