import {
	defineMessages,
	injectModrinthClient,
	isDisclosureCompatibleWithProjectTypes,
	type MessageDescriptor,
	useVIntl,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, type Ref, ref, watch } from 'vue'

import { useAuthState } from '~/composables/auth'

import type { DisclosureFormIssue } from './form'
import {
	disclosuresToForm,
	findDisclosureData,
	getDisclosureFormIssues,
	getDisclosureFormSnapshot,
	toModifyRequests,
} from './form'
import type {
	DisclosureFormState,
	DisclosureLockStatus,
	DisclosureType,
	DisclosureUpdatedByUser,
} from './types'

const DISCLOSURE_QUERY_STALE_TIME = 1000 * 60 * 5

type DisclosureEditorOptions = {
	projectId: Ref<string>
	projectTypes: Ref<string[]>
	canEditDisclosures: Ref<boolean>
	hasPermission: Ref<boolean>
	isActingAsModerator: Ref<boolean>
	isAdminUser: Ref<boolean>
	onSaved?: () => Promise<unknown>
}

export function useDisclosureEditor({
	projectId,
	projectTypes,
	canEditDisclosures,
	hasPermission,
	isActingAsModerator,
	isAdminUser,
	onSaved,
}: DisclosureEditorOptions) {
	const { formatMessage } = useVIntl()
	const { labrinth } = injectModrinthClient()
	const queryClient = useQueryClient()
	const auth = useAuthState()
	function isDisclosureVisible(type: DisclosureType) {
		return isDisclosureCompatibleWithProjectTypes(type, projectTypes.value)
	}
	const messages = defineMessages({
		title: {
			id: 'project.settings.disclosures.content-disclosures',
			defaultMessage: 'Content disclosures',
		},
		description: {
			id: 'project.settings.disclosures.description',
			defaultMessage: `You must add any applicable content disclosures to your project in compliance with <rules>Modrinth's Content Rules</rules>.`,
		},
		description2: {
			id: 'project.settings.disclosures.description.2',
			defaultMessage: `Unsure how to apply content disclosure to your project? Check out our <faq-link>Content Disclosures FAQ</faq-link>.`,
		},
		noPermission: {
			id: 'project.settings.disclosures.save-blocked.no-permission',
			defaultMessage: `You don't have permission to edit this project's disclosures.`,
		},
		uploadVersionFirstHeading: {
			id: 'project.settings.disclosures.upload-version-first.heading',
			defaultMessage: 'Upload versions before adding disclosures',
		},
	})

	const issueMessages = defineMessages({
		'advertising-note': {
			id: 'project.settings.disclosures.save-blocked.advertising-note',
			defaultMessage: 'Advertising disclosure requires explanation.',
		},
		'paid-features-empty': {
			id: 'project.settings.disclosures.save-blocked.paid-features-empty',
			defaultMessage: 'Paid features disclosure must list at least one paid feature.',
		},
		'telemetry-empty': {
			id: 'project.settings.disclosures.save-blocked.telemetry-empty',
			defaultMessage: 'Telemetry disclosure must list at least one type of data collected.',
		},
		'derivative-empty': {
			id: 'project.settings.disclosures.save-blocked.derivative-empty',
			defaultMessage: 'Derivative works disclosure must list at least one source work.',
		},
		'derivative-source-label': {
			id: 'project.settings.disclosures.save-blocked.derivative-source-label',
			defaultMessage: 'Derivative work sources must have a name.',
		},
		'photosensitivity-note': {
			id: 'project.settings.disclosures.save-blocked.photosensitivity-note',
			defaultMessage: 'Photosensitivity warning disclosure must include a description.',
		},
		'system-interactions-note': {
			id: 'project.settings.disclosures.save-blocked.system-interactions-note',
			defaultMessage: 'External system interactions disclosure must include a description.',
		},
	}) satisfies Record<DisclosureFormIssue, MessageDescriptor>

	const disclosuresQueryKey = computed(
		() => ['project', 'disclosures', 'v3', projectId.value] as const,
	)

	const disclosuresQuery = useQuery({
		queryKey: disclosuresQueryKey,
		queryFn: () => labrinth.projects_v3.getDisclosures(projectId.value),
		staleTime: DISCLOSURE_QUERY_STALE_TIME,
		enabled: computed(() => !!projectId.value),
	})

	const disclosuresResponse = disclosuresQuery.data

	const updaterUserIds = computed(() => {
		const ids = new Set<string>()
		for (const disclosure of disclosuresResponse.value?.disclosures ?? []) {
			if (disclosure.updated_by) {
				ids.add(disclosure.updated_by)
			}
		}
		return [...ids]
	})

	const { data: updaterUsers } = useQuery({
		queryKey: computed(() => ['users', 'disclosures', updaterUserIds.value] as const),
		queryFn: () => labrinth.users_v2.getMultiple(updaterUserIds.value),
		enabled: computed(() => updaterUserIds.value.length > 0),
		staleTime: DISCLOSURE_QUERY_STALE_TIME,
	})

	const updaterUsersById = computed(() => {
		const map = new Map<string, NonNullable<typeof updaterUsers.value>[number]>()
		for (const user of updaterUsers.value ?? []) {
			map.set(user.id, user)
		}
		return map
	})

	const currentUser = computed((): DisclosureUpdatedByUser | null => {
		const user = auth.value?.user
		if (!user?.id || !user.username) {
			return null
		}
		return {
			id: user.id,
			username: user.username,
			avatar_url: user.avatar_url,
		}
	})

	function resolveUpdatedBy(userId: string | null | undefined): DisclosureUpdatedByUser | null {
		if (!userId) return null
		return (
			updaterUsersById.value.get(userId) ??
			(currentUser.value?.id === userId ? currentUser.value : null)
		)
	}

	const current = ref(disclosuresToForm([]))
	const saved = computed(() => disclosuresToForm(disclosuresResponse.value?.disclosures ?? []))
	const mutation = useMutation({
		mutationFn: async ({ id, form }: { id: string; form: DisclosureFormState }) => {
			const latest = await labrinth.projects_v3.getDisclosures(id)
			const requests = toModifyRequests(form, disclosuresToForm(latest.disclosures))
			try {
				for (const request of requests) {
					await labrinth.projects_v3.modifyDisclosures(id, request)
				}
			} finally {
				await Promise.all([
					queryClient.invalidateQueries({ queryKey: ['project', 'disclosures', 'v3', id] }),
					queryClient.invalidateQueries({ queryKey: ['project', 'v3', id] }),
					queryClient.invalidateQueries({ queryKey: ['project', 'v2', id] }),
				])
			}
			const response = await labrinth.projects_v3.getDisclosures(id)
			queryClient.setQueryData(['project', 'disclosures', 'v3', id], response)
			if (id === projectId.value) await onSaved?.()
		},
	})
	const saving = mutation.isPending

	function reset() {
		current.value = disclosuresToForm(disclosuresResponse.value?.disclosures ?? [])
		mutation.reset()
	}

	const savedSnapshot = computed(() => getDisclosureFormSnapshot(saved.value))
	const currentSnapshot = computed(() => getDisclosureFormSnapshot(current.value))
	const hasChanges = computed(
		() => JSON.stringify(savedSnapshot.value) !== JSON.stringify(currentSnapshot.value),
	)

	async function save() {
		if (!hasChanges.value || !canSave.value || saving.value) return
		const id = projectId.value
		try {
			await mutation.mutateAsync({
				id,
				form: JSON.parse(JSON.stringify(current.value)) as DisclosureFormState,
			})
			if (id === projectId.value) {
				reset()
			}
		} catch {
			// The mutation error is rendered alongside the retained draft.
		}
	}

	function disclosureUpdateProps(type: DisclosureType) {
		const disclosure = findDisclosureData(disclosuresResponse.value?.disclosures, type)
		const savedLockStatus = disclosure?.lock_status ?? 'unlocked'
		const lockStatus = current.value.lockStatuses[type] ?? savedLockStatus
		const lockedForAuthor = !isActingAsModerator.value

		return {
			disabled:
				saving.value ||
				!hasPermission.value ||
				(lockedForAuthor && savedLockStatus === 'fully_locked'),
			toggleDisabled:
				saving.value || !hasPermission.value || (lockedForAuthor && savedLockStatus !== 'unlocked'),
			updatedAt: disclosure?.updated_at,
			updatedBy: resolveUpdatedBy(disclosure?.updated_by),
			setByModerator: !!disclosure?.set_by_moderator,
			lockStatus,
			showLockControls: isActingAsModerator.value,
		}
	}

	function setDisclosureLockStatus(type: DisclosureType, lockStatus: DisclosureLockStatus) {
		current.value.lockStatuses = {
			...current.value.lockStatuses,
			[type]: lockStatus,
		}
	}

	watch(
		[projectId, disclosuresResponse],
		([id, value], [previousId, previous]) => {
			const changedProject = id !== previousId
			if (changedProject) {
				current.value = disclosuresToForm([])
				mutation.reset()
			}
			if (
				value &&
				!saving.value &&
				(changedProject ||
					!previous ||
					JSON.stringify(getDisclosureFormSnapshot(current.value)) ===
						JSON.stringify(getDisclosureFormSnapshot(disclosuresToForm(previous.disclosures))))
			) {
				current.value = disclosuresToForm(disclosuresResponse.value?.disclosures ?? [])
			}
		},
		{ immediate: true },
	)

	const issues = computed(() => getDisclosureFormIssues(current.value, projectTypes.value))

	const canSave = computed(
		() =>
			disclosuresQuery.isSuccess.value &&
			canEditDisclosures.value &&
			hasPermission.value &&
			(isAdminUser.value || issues.value.length === 0),
	)

	const saveDisabledReason = computed(() => {
		if (!hasPermission.value) {
			return formatMessage(messages.noPermission)
		}
		return [...issues.value.map((issue) => formatMessage(issueMessages[issue]))]
	})

	return {
		messages,
		current,
		savedSnapshot,
		currentSnapshot,
		hasChanges,
		saving,
		reset,
		save,
		canSave,
		saveDisabledReason,
		disclosureUpdateProps,
		setDisclosureLockStatus,
		isDisclosureVisible,
		canEditDisclosures,
		disclosuresQuery,
		saveError: mutation.error,
	}
}
