<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.header)"
		max-width="830px"
		width="830px"
		no-padding
		actions-divider
		:animate-resize="false"
		:disable-close="saving"
	>
		<div class="flex flex-col">
			<div class="p-5">
				<Admonition type="info">
					<IntlFormatted :message-id="messages.body" :values="{ count: addons.length }">
						<template #bold="{ children }">
							<span class="font-semibold text-contrast"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</Admonition>
			</div>
			<div class="max-h-[50vh] overflow-y-auto border-0 border-t border-solid border-surface-5">
				<ContentCardTable
					v-model:selected-ids="selectedIds"
					:items="tableItems"
					:virtualized="false"
					show-selection
					show-enabled-for-column
					hide-delete
					flat
					@update:enabled-for="onEnabledForChange"
				/>
			</div>
		</div>

		<template #actions>
			<div class="flex justify-end gap-2">
				<Button type="outlined" :disabled="saving" @click="hide">
					<XIcon />
					{{ formatMessage(commonMessages.cancelButton) }}
				</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="addons.length === 0"
					:loading="saving"
					@click="save"
				>
					<CheckIcon />
					{{ formatMessage(commonMessages.confirmButton) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>

<script setup lang="ts">
import type { Archon } from '@modrinth/api-client'
import { CheckIcon, XIcon } from '@modrinth/assets'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, nextTick, ref } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import { Button } from '#ui/components/base/buttons'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { addonToggleKey } from '#ui/composables/server-content-toggle-state'
import ContentCardTable from '#ui/layouts/shared/content-tab/components/ContentCardTable.vue'
import type { ContentCardTableItem, ContentSide } from '#ui/layouts/shared/content-tab/types'
import {
	injectModrinthClient,
	injectModrinthServerContext,
	injectNotificationManager,
} from '#ui/providers'
import { commonMessages } from '#ui/utils/common-messages'

type Sides = Record<ContentSide, boolean>

const emit = defineEmits<{
	reviewed: []
}>()

const { formatMessage } = useVIntl()
const client = injectModrinthClient()
const ctx = injectModrinthServerContext()
const queryClient = useQueryClient()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	header: {
		id: 'servers.curseforge-files-review.header',
		defaultMessage: 'Review modpack files',
	},
	body: {
		id: 'servers.curseforge-files-review.body',
		defaultMessage:
			'{count, plural, one {# file} other {# files}} in this modpack {count, plural, one {isn’t} other {aren’t}} published on Modrinth, or we can’t determine whether {count, plural, one {it} other {they}} should be installed on the server, available to players, or both. Review and complete {count, plural, one {it} other {these}} in the <bold>Enabled for</bold> column.',
	},
})

const modal = ref<InstanceType<typeof NewModal>>()
const addons = ref<Archon.Content.v1.Addon[]>([])
const draft = ref(new Map<string, Sides>())
const selectedIds = ref<string[]>([])
const loading = ref(false)
const saving = ref(false)

function isPlayerOnlyContent(addon: Archon.Content.v1.Addon) {
	return addon.kind === 'resourcepack' || addon.kind === 'shader'
}

function currentSides(addon: Archon.Content.v1.Addon): Sides {
	return {
		server: !isPlayerOnlyContent(addon) && !addon.disabled_server,
		player: !addon.disabled_player,
	}
}

function addonTitle(addon: Archon.Content.v1.Addon) {
	if (addon.name) return addon.name
	if (addon.manifest?.name) return addon.manifest.name
	const extension = addon.filename.lastIndexOf('.')
	return extension === -1 ? addon.filename : addon.filename.substring(0, extension)
}

const tableItems = computed<ContentCardTableItem[]>(() =>
	addons.value.map((addon) => {
		const id = addonToggleKey(addon)
		const sides = draft.value.get(id) ?? currentSides(addon)
		const playerOnly = isPlayerOnlyContent(addon)
		const worldId = ctx.worldId.value
		return {
			id,
			project: {
				id: addon.project_id ?? addon.filename,
				slug: addon.project_id ?? addon.filename,
				title: addonTitle(addon),
				icon_url: addon.icon_url ?? undefined,
			},
			projectType: addon.kind,
			version: {
				id: addon.version?.id ?? addon.filename,
				version_number:
					addon.version?.name ??
					addon.manifest?.version ??
					formatMessage(commonMessages.unknownLabel),
				file_name: addon.filename,
			},
			external: !addon.project_id,
			embeddedIcon:
				!addon.icon_url &&
				addon.manifest?.icon_embedded &&
				worldId &&
				(addon.kind === 'mod' || addon.kind === 'plugin')
					? {
							queryKey: [
								'kyros',
								'content',
								'embedded-icon',
								worldId,
								addon.kind,
								addon.filename,
							] as const,
							queryFn: () =>
								client.kyros.content_v1.getEmbeddedAddonIcon(
									worldId,
									addon.kind === 'mod' ? 'mods' : 'plugins',
									addon.filename,
								),
						}
					: undefined,
			enabledFor: {
				...sides,
				locked: playerOnly,
				disabledSides: playerOnly ? ['server'] : [],
			},
		}
	}),
)

function onEnabledForChange(id: string, side: ContentSide, value: boolean) {
	const targets = selectedIds.value.includes(id) ? selectedIds.value : [id]
	const next = new Map(draft.value)
	for (const addon of addons.value) {
		const key = addonToggleKey(addon)
		if (!targets.includes(key)) continue
		if (side === 'server' && isPlayerOnlyContent(addon)) continue
		next.set(key, { ...(next.get(key) ?? currentSides(addon)), [side]: value })
	}
	draft.value = next
}

async function save() {
	const worldId = ctx.worldId.value
	if (!worldId || saving.value) return
	saving.value = true
	let reviewed = false
	try {
		for (const addon of addons.value) {
			const current = currentSides(addon)
			const next = draft.value.get(addonToggleKey(addon)) ?? current
			const changedSides = (['server', 'player'] as const).filter(
				(side) => next[side] !== current[side],
			)
			if (changedSides.length === 0) continue

			const request = { kind: addon.kind, filename: addon.filename }
			if (!isPlayerOnlyContent(addon) && !addon.side_toggle_unlocked) {
				await client.archon.content_v1.setAddonSideToggleLocked(ctx.serverId, worldId, {
					...request,
					locked: false,
				})
			}
			for (const side of changedSides) {
				const sideRequest = { ...request, enabled: next[side] }
				if (side === 'server') {
					await client.archon.content_v1.setAddonEnabledServer(ctx.serverId, worldId, sideRequest)
				} else {
					await client.archon.content_v1.setAddonEnabledPlayer(ctx.serverId, worldId, sideRequest)
				}
			}
		}

		await client.archon.content_v1.completeSideTypeReview(ctx.serverId, worldId, {
			addons: addons.value.map((addon) => ({ kind: addon.kind, filename: addon.filename })),
		})
		reviewed = true
	} catch (error) {
		handleError(error as Error)
	} finally {
		saving.value = false
		void queryClient.invalidateQueries({ queryKey: ['content', 'list', 'v1', ctx.serverId] })
	}

	if (!reviewed) return
	emit('reviewed')
	await nextTick()
	modal.value?.hide()
}

async function show() {
	const worldId = ctx.worldId.value
	if (!worldId || loading.value) return
	loading.value = true
	try {
		const content = await client.archon.content_v1.getAddons(ctx.serverId, worldId, {
			needs_side_type_review: true,
			updates: false,
		})
		addons.value = content.addons ?? []
		draft.value = new Map()
		selectedIds.value = []
		modal.value?.show()
	} catch (error) {
		handleError(error as Error)
	} finally {
		loading.value = false
	}
}

function hide() {
	modal.value?.hide()
}

defineExpose({ show, hide, loading })
</script>
