import type { AbstractModrinthClient, Archon, Labrinth } from '@modrinth/api-client'

import { serverPlayMigrationMessages } from '#ui/components/servers/play-migration/messages'
import { defineMessages, type useVIntl } from '#ui/composables/i18n'
import { addonToggleKey } from '#ui/composables/server-content-toggle-state'
import type {
	ContentItem,
	ContentSide,
	ContentType,
	ContentWarningType,
} from '#ui/layouts/shared/content-tab/types'
import {
	friendlyAddonName,
	getAddonEnvironment,
	hasDetectedEnvironment,
	isIncompatibleEnvironment,
	isPlayerOnlyContent,
} from '#ui/layouts/shared/content-tab/utils/addon-environment'
import { commonMessages } from '#ui/utils/common-messages'

type FormatMessage = ReturnType<typeof useVIntl>['formatMessage']

export type AddonWithUiState = Archon.Content.v1.Addon & { installing?: boolean }

export type FileInstallProgressItem = Archon.Websocket.v0.InstallProgressItem & {
	key: Archon.Websocket.v0.InstallProgressFileKey
}

const messages = defineMessages({
	resourcePackLock: {
		id: 'hosting.content.enabled-for.resource-pack-lock',
		defaultMessage: 'Resource packs can only be enabled for players.',
	},
	shaderLock: {
		id: 'hosting.content.enabled-for.shader-lock',
		defaultMessage: 'Shaders can only be enabled for players.',
	},
	unknownEnvironment: {
		id: 'hosting.content.enabled-for.compatibility-unknown-warning',
		defaultMessage:
			"We couldn't determine whether this content works on the server, for players, or both.",
	},
	clientOnlyEnabledForServer: {
		id: 'hosting.content.enabled-for.client-only-on-server-warning',
		defaultMessage:
			'This client-only content is enabled on the server and may prevent the server from starting.',
	},
	singleplayerOnlyEnabledForServer: {
		id: 'hosting.content.enabled-for.singleplayer-only-on-server-warning',
		defaultMessage:
			'This singleplayer-only content is enabled on the server and may prevent the server from starting.',
	},
	serverOnlyEnabledForPlayers: {
		id: 'hosting.content.enabled-for.server-only-for-players-warning',
		defaultMessage:
			'This server-only content is included for players and may cause issues when they launch the game.',
	},
})

export function primaryContentType(loader: string | undefined): ContentType {
	const normalized = loader?.toLowerCase()
	if (normalized === 'paper' || normalized === 'purpur') return 'plugin'
	if (normalized === 'vanilla') return 'datapack'
	return 'mod'
}

export function contentTypesForLoader(loader: string | undefined): ContentType[] {
	const normalized = loader?.toLowerCase()
	if (normalized === 'paper' || normalized === 'purpur') return ['plugin', 'datapack']
	if (normalized === 'vanilla') return ['datapack']
	return ['mod', 'resourcepack', 'datapack', 'shader']
}

function getEnabledForWarning(
	addon: Archon.Content.v1.Addon,
	formatMessage: FormatMessage,
): { kind: ContentWarningType; tooltip: string } | null {
	if (isPlayerOnlyContent(addon)) return null
	if (!addon.disabled_server) {
		if (addon.pack_client_retained)
			return { kind: 'retained', tooltip: formatMessage(commonMessages.clientRetainedWarning) }
		if (addon.pack_client_depends)
			return { kind: 'depends', tooltip: formatMessage(commonMessages.clientDependsWarning) }
		if (isIncompatibleEnvironment(addon, 'server')) {
			return {
				kind: 'environment',
				tooltip: formatMessage(
					getAddonEnvironment(addon) === 'singleplayer_only'
						? messages.singleplayerOnlyEnabledForServer
						: messages.clientOnlyEnabledForServer,
				),
			}
		}
	}
	if (!addon.disabled_player && isIncompatibleEnvironment(addon, 'player')) {
		return { kind: 'server-only', tooltip: formatMessage(messages.serverOnlyEnabledForPlayers) }
	}
	const sidesUnknown = addon.from_modpack
		? addon.needs_side_type_review
		: !hasDetectedEnvironment(addon)
	if (sidesUnknown)
		return { kind: 'unknown-environment', tooltip: formatMessage(messages.unknownEnvironment) }
	return null
}

export interface AddonToContentItemOptions {
	client: AbstractModrinthClient
	worldId: string | null
	project: Labrinth.Projects.v2.Project | undefined
	migrationRequired: boolean
	formatMessage: FormatMessage
}

export function addonToContentItem(
	addon: AddonWithUiState,
	{ client, worldId, project, migrationRequired, formatMessage }: AddonToContentItemOptions,
): ContentItem {
	const environment = getAddonEnvironment(addon)
	const embeddedIcon =
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
					fallbackUrl: project?.icon_url,
				}
			: undefined
	const serverEnabled = !isPlayerOnlyContent(addon) && !addon.disabled_server
	const playerEnabled = !addon.disabled_player
	const lockedSides: ContentSide[] = migrationRequired
		? ['server', 'player']
		: isPlayerOnlyContent(addon)
			? ['server']
			: []
	const warning = migrationRequired ? undefined : getEnabledForWarning(addon, formatMessage)
	return {
		project: {
			...(project ?? {}),
			id: addon.project_id ?? addon.filename,
			slug: project?.slug ?? addon.project_id ?? addon.filename,
			title: project?.title ?? friendlyAddonName(addon),
			icon_url: addon.icon_url ?? (embeddedIcon ? undefined : project?.icon_url) ?? undefined,
		},
		version: {
			id: addon.version?.id ?? addon.filename,
			version_number:
				addon.version?.name ??
				addon.manifest?.version ??
				formatMessage(commonMessages.unknownLabel),
			file_name: addon.filename,
		},
		owner: addon.owner
			? {
					id: addon.owner.id,
					name: addon.owner.name,
					type: addon.owner.type,
					avatar_url: addon.owner.icon_url,
					link: `/${addon.owner.type}/${addon.owner.id}`,
				}
			: undefined,
		id: addonToggleKey(addon),
		external: !addon.project_id,
		source_kind: addon.from_modpack ? 'modrinth_modpack' : undefined,
		enabled: !addon.disabled && (serverEnabled || playerEnabled),
		enabledFor: {
			server: serverEnabled,
			player: playerEnabled,
			locked: lockedSides.length > 0,
			lockedTooltip: migrationRequired
				? formatMessage(serverPlayMigrationMessages.contentLocked)
				: isPlayerOnlyContent(addon)
					? formatMessage(
							addon.kind === 'resourcepack' ? messages.resourcePackLock : messages.shaderLock,
						)
					: undefined,
			disabledSides: lockedSides,
			warningTooltip: warning?.tooltip,
			warningKind: warning?.kind,
		},
		embeddedIcon,
		file_name: addon.filename,
		date_added: addon.btime,
		project_type: addon.kind,
		has_update: !!addon.has_update,
		update_version_id: addon.has_update,
		environment,
		pack_client_retained: addon.pack_client_retained,
		pack_client_depends: addon.pack_client_depends,
		installing: addon.installing ?? addon.status === 'pending',
	}
}

function normalizeInstallFilename(filename: string) {
	const normalized = filename.endsWith('.disabled')
		? filename.slice(0, -'.disabled'.length)
		: filename
	return normalized.toLowerCase()
}

function getFileInstallFilenames(key: Archon.Websocket.v0.InstallProgressFileKey) {
	return [key.source_filename, key.target_filename]
		.filter((filename): filename is string => !!filename)
		.map(normalizeInstallFilename)
}

export function withInstallProgress(
	item: ContentItem,
	progressItems: FileInstallProgressItem[],
): ContentItem {
	if (!item.installing) return item
	const filename = normalizeInstallFilename(item.version?.file_name || item.file_name)
	const progress = progressItems.find(({ key }) => {
		if (key.project_id === item.project?.id) return true
		if (key.version_id === item.version?.id) return true
		return getFileInstallFilenames(key).includes(filename)
	})
	if (!progress) return item
	return {
		...item,
		installProgress:
			progress.error == null && progress.progress !== 100 ? progress.progress : undefined,
	}
}
