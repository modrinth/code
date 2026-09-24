import {
	useIsFetching,
	useIsMutating,
	useMutation,
	useQuery,
	useQueryClient,
} from '@tanstack/vue-query'
import { computed, onScopeDispose, type Ref, ref, watch } from 'vue'

import type ServerConfigFilePicker from '#ui/components/servers/ServerConfigFilePicker.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { useServerPermissions } from '#ui/composables/server-permissions'
import type ContentDiffModal from '#ui/layouts/shared/installation-settings/components/ContentDiffModal.vue'
import {
	injectAuth,
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'

import { serverShareDiffQueryOptions } from './query-options'

export type ServerShareActionTarget = {
	worldId: string
	userId: string | undefined
	configPaths: string[]
	isCurrent: () => boolean
}

export function useServerShareReview<Action extends string = 'push'>(options?: {
	execute: (action: Action, target: ServerShareActionTarget) => Promise<void>
	disabled?: Ref<boolean>
}) {
	const client = injectModrinthClient()
	const auth = injectAuth()
	const { serverId, worldId, serverFull, busyReasons } = injectModrinthServerContext()
	const { canSetup } = useServerPermissions()
	const { handleError } = injectNotificationManager()
	const { formatMessage } = useVIntl()
	const queryClient = useQueryClient()
	const diffModal = ref<InstanceType<typeof ContentDiffModal>>()
	const configPicker = ref<InstanceType<typeof ServerConfigFilePicker>>()
	const previewOpen = ref(false)
	const sharedInstanceId = computed(
		() =>
			serverFull.value?.worlds.find((world) => world.id === worldId.value)?.content
				?.shared_instance_id,
	)
	let disposed = false
	onScopeDispose(() => {
		disposed = true
	})
	const previewQuery = useQuery(
		computed(() => ({
			...serverShareDiffQueryOptions(client, serverId, worldId.value ?? '', auth.user.value?.id),
			enabled: false,
		})),
	)
	const shareActions = useIsMutating({ mutationKey: ['servers', 'share-action', serverId] })
	const sharePreviews = useIsFetching({ queryKey: ['servers', 'share-diff', serverId] })
	const actionMutation = useMutation({
		mutationKey: ['servers', 'share-action', serverId],
		mutationFn: async ({
			action,
			reviewed,
			target,
		}: {
			action: Action
			reviewed: boolean
			target: ServerShareActionTarget
		}) => {
			if (reviewed) target.configPaths = (await configPicker.value?.resolvePaths()) ?? []
			if (
				!target.isCurrent() ||
				busyReasons.value.length ||
				(reviewed && (!previewOpen.value || !canSetup.value))
			)
				return
			if (options) await options.execute(action, target)
			else await publish(target)
		},
	})
	const pending = computed(
		() =>
			actionMutation.isPending.value ||
			shareActions.value > 0 ||
			sharePreviews.value > 0 ||
			busyReasons.value.length > 0 ||
			!!options?.disabled?.value,
	)

	function currentTarget(): ServerShareActionTarget | undefined {
		const targetWorldId = worldId.value
		const userId = auth.user.value?.id
		if (!targetWorldId || disposed) return
		return {
			worldId: targetWorldId,
			userId,
			configPaths: [],
			isCurrent: () =>
				!disposed && worldId.value === targetWorldId && auth.user.value?.id === userId,
		}
	}

	async function showPreview() {
		if (!canSetup.value || pending.value) return
		const target = currentTarget()
		if (!target) return
		previewOpen.value = true
		const result = await previewQuery.refetch()
		if (!previewOpen.value || !target.isCurrent()) return
		if (result.error) {
			previewOpen.value = false
			handleError(result.error)
		} else diffModal.value?.show()
	}

	async function publish(target: ServerShareActionTarget) {
		if (!canSetup.value) throw new Error(formatMessage(messages.permission))
		const shared = await client.archon.content_v1.share(
			serverId,
			target.worldId,
			target.configPaths,
		)
		await Promise.all([
			queryClient.invalidateQueries({ queryKey: ['servers', 'v1', 'detail', serverId] }),
			queryClient.invalidateQueries({
				queryKey: ['servers', 'share-diff', serverId, target.worldId],
			}),
		])
		return shared
	}

	async function runAction(action: Action, reviewed = false) {
		if (pending.value) return
		const target = currentTarget()
		if (!target) return
		try {
			await actionMutation.mutateAsync({ action, reviewed, target })
			if (target.isCurrent()) previewOpen.value = false
		} catch (error) {
			handleError(error)
			if (reviewed && previewOpen.value && target.isCurrent()) diffModal.value?.show()
		}
	}

	watch([worldId, sharedInstanceId, () => auth.user.value?.id], () => {
		previewOpen.value = false
		diffModal.value?.hide()
	})

	return {
		diffModal,
		configPicker,
		previewOpen,
		previewQuery,
		shareActions,
		sharePreviews,
		actionMutation,
		pending,
		showPreview,
		publish,
		runAction,
	}
}

const messages = defineMessages({
	permission: {
		id: 'servers.play.permission',
		defaultMessage: 'You do not have permission to share this world.',
	},
})
