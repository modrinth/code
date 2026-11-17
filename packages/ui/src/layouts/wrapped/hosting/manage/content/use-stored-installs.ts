import type { Archon, Labrinth } from '@modrinth/api-client'
import { type Ref, ref, watch } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { waitForServerContextRuntimeReady } from '#ui/composables/server-context-runtime'
import {
	type BrowseInstallPlan,
	flushStoredServerAddonInstallQueue,
	getStoredServerAddonInstallQueue,
	getTargetInstallPreferences,
	resolveServerAddonInstallPlans,
} from '#ui/layouts/shared/browse-tab/composables/install-logic'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { resolveServerPlatform } from '#ui/utils/loaders'

const messages = defineMessages({
	failedToInstallContent: {
		id: 'hosting.content.failed-to-install',
		defaultMessage: 'Failed to install content',
	},
})

function toResolvePreferences(
	preferences?: BrowseInstallPlan['preferences'],
): Labrinth.Content.v3.ResolutionPreferences {
	return {
		game_versions: preferences?.gameVersions,
		loaders: preferences?.loaders,
	}
}

export function useStoredInstalls(
	serverId: string,
	content: Readonly<Ref<Archon.Content.v1.Addons | undefined>>,
) {
	const { formatMessage } = useVIntl()
	const client = injectModrinthClient()
	const { addNotification } = injectNotificationManager()
	const { server, serverFull, worldId } = injectModrinthServerContext()
	const flushing = ref(false)

	function notifyFailure(error: unknown) {
		addNotification({
			type: 'error',
			title: formatMessage(messages.failedToInstallContent),
			text: error instanceof Error ? error.message : undefined,
		})
	}

	async function resolvePlans(plans: BrowseInstallPlan[]) {
		return await resolveServerAddonInstallPlans({
			plans,
			existingProjectIds: new Set(
				(content.value?.addons ?? [])
					.map((addon) => addon.project_id)
					.filter((projectId): projectId is string => !!projectId),
			),
			resolvePlan: async (plan, existingProjectIds) => {
				const target = getTargetInstallPreferences(
					resolveServerPlatform(server.value, content.value),
					plan.contentType,
				)
				const resolved = await client.labrinth.content_v3.resolve({
					project_id: plan.projectId,
					version_id: plan.versionId,
					content_type: plan.contentType as Labrinth.Content.v3.ContentType,
					selected: toResolvePreferences(plan.preferences),
					target: toResolvePreferences(target),
					existing_project_ids: existingProjectIds,
					update_channel: serverFull.value?.worlds.find((world) => world.id === worldId.value)
						?.update_channel,
				})

				return [resolved.primary, ...resolved.dependencies].map((item) => ({
					projectId: item.project_id,
					versionId: item.version_id,
				}))
			},
		})
	}

	async function flush() {
		const wid = worldId.value
		if (!wid || flushing.value) return
		if (getStoredServerAddonInstallQueue(serverId, wid).size === 0) return

		try {
			await waitForServerContextRuntimeReady(client, serverId)
		} catch (error) {
			notifyFailure(error)
			return
		}

		flushing.value = true
		try {
			const result = await flushStoredServerAddonInstallQueue({
				serverId,
				worldId: wid,
				install: async (plans) => {
					const addons = await resolvePlans(plans)
					if (addons.length > 0) {
						await client.archon.content_v1.addAddons(serverId, wid, addons)
					}
				},
			})
			if (!result.ok) notifyFailure(result.error)
		} finally {
			flushing.value = false
		}
	}

	watch(worldId, () => void flush(), { immediate: true })
}
