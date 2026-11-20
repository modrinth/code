import type { Ref } from 'vue'

import { defineMessages, useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'

import type { ContentManagerContext } from '../providers/content-manager'
import type { ContentCardTableItem, ContentItem } from '../types'
import { getClientWarningType } from './content-filtering'

const messages = defineMessages({
	pleaseWait: {
		id: 'content.page-layout.please-wait',
		defaultMessage: 'Please wait',
	},
})

export interface ContentRowsOptions {
	ctx: ContentManagerContext
	isServer: Readonly<Ref<boolean>>
	isPending: (item: ContentItem) => boolean
}

export function useContentRows({ ctx, isServer, isPending }: ContentRowsOptions) {
	const { formatMessage } = useVIntl()

	function toDisplayRow(item: ContentItem): ContentCardTableItem {
		const projectLink = ctx.links.project(item)
		const versionLink = ctx.links.version(item)
		return {
			id: item.id,
			project: item.project ?? {
				id: item.id,
				slug: null,
				title: item.embedded_metadata?.name ?? item.file_name.replace('.disabled', ''),
				icon_url: item.embedded_metadata?.icon_url ?? null,
			},
			projectType: item.project_type,
			projectLink,
			version: item.version ?? {
				id: item.id,
				version_number:
					item.embedded_metadata?.version ?? formatMessage(commonMessages.unknownLabel),
				file_name: item.file_name,
			},
			versionLink,
			owner: item.owner ? { ...item.owner, link: ctx.links.owner(item.owner) } : undefined,
			source: item.source,
			external: item.external ?? !item.project,
			embeddedIcon: item.embeddedIcon,
		}
	}

	function toRow(item: ContentItem): ContentCardTableItem {
		const base = toDisplayRow(item)
		const permissions = ctx.permissions(item)
		const busyMessage = ctx.busy.value ? (ctx.busy.value.message ?? null) : null
		const pending = isPending(item)
		const clientWarning = item.enabledFor ? null : getClientWarningType(item, isServer.value)
		return {
			...base,
			enabled: permissions.toggle ? item.enabled : undefined,
			enabledFor: item.enabledFor,
			synced: !!item.synced_pack,
			syncUpdatePending: item.synced_pack?.update_pending,
			locked: item.locked ?? false,
			installing: item.installing === true,
			installProgress: item.installProgress,
			hasUpdate: item.has_update && !item.locked,
			clientWarning,
			hideDelete: !permissions.remove,
			hideSwitchVersion:
				!permissions.changeVersion || !item.project?.id || !item.version?.id || !base.versionLink,
			disabled: pending || !!ctx.busy.value || item.installing === true,
			disabledTooltip: ctx.busy.value
				? busyMessage
				: pending
					? formatMessage(messages.pleaseWait)
					: item.installing
						? formatMessage(commonMessages.installingLabel)
						: null,
			toggleDisabled: !!ctx.busy.value,
			toggleDisabledTooltip: busyMessage,
			overflowOptions: ctx.menu?.(item),
		}
	}

	return { toRow, toDisplayRow }
}
