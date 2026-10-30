import type { Archon } from '@modrinth/api-client'
import { type ComputedRef, nextTick, type Ref, watch } from 'vue'

import type ServerPlayMigrationModal from '#ui/components/servers/play-migration/ServerPlayMigrationModal.vue'
import type { ServerPlayMigrationSource } from '#ui/components/servers/play-migration/types'
import type ServerInviteLinkModal from '#ui/components/sharing/server-invite-link/ServerInviteLinkModal.vue'
import { injectServerInviteHandoff } from '#ui/components/sharing/server-sharing-update/handoff'
import type { useServerBackupsQueue } from '#ui/composables/server-backups-queue'

import {
	provideServerPlayMigration,
	type ServerPlayMigrationNext,
} from '../providers/server-play-migration'
import { type TrackUpload, useServerPlayMigration } from './use-server-play-migration'

type ReadableRef<T> = Ref<T> | ComputedRef<T>

type BackupsQueue = Pick<
	ReturnType<typeof useServerBackupsQueue>,
	'backups' | 'invalidate' | 'trackMigration' | 'migrationStateFor' | 'hasActiveMigration'
>

/**
 * Owns the server play migration for the panel: provides the migration context to child pages,
 * drives the confirmation and invite link modals, and answers a pending invite handoff once the
 * server has loaded.
 */
export function useServerPlayMigrationFlow(options: {
	serverId: string
	worldId: ReadableRef<string | null>
	serverFull: ReadableRef<Archon.Servers.v1.ServerFull | null | undefined>
	content: ReadableRef<Archon.Content.v1.Addons | null | undefined>
	forced: () => boolean
	backupsQueue: BackupsQueue
	canSetup: ReadableRef<boolean>
	serverLoaded: ReadableRef<boolean>
	migrationModal: Ref<InstanceType<typeof ServerPlayMigrationModal> | undefined>
	inviteLinkModal: Ref<InstanceType<typeof ServerInviteLinkModal> | undefined>
	trackUpload: TrackUpload
	onTagged: () => void
}) {
	let pendingNext: ServerPlayMigrationNext | undefined

	const migration = useServerPlayMigration({
		serverId: options.serverId,
		worldId: options.worldId,
		serverFull: options.serverFull,
		forced: options.forced,
		content: options.content,
		backups: options.backupsQueue.backups,
		invalidateBackups: options.backupsQueue.invalidate,
		trackMigration: options.backupsQueue.trackMigration,
		migrationStateFor: options.backupsQueue.migrationStateFor,
		migrationActive: options.backupsQueue.hasActiveMigration,
		trackUpload: options.trackUpload,
		prompt: (next) => {
			pendingNext = next
			options.migrationModal.value?.show()
		},
		onMigrated: (next) => {
			if (next === 'invite') options.inviteLinkModal.value?.show()
			else next()
		},
	})
	provideServerPlayMigration(migration)

	watch(
		migration.tagged,
		(tagged) => {
			if (tagged) options.onTagged()
		},
		{ immediate: true },
	)

	function confirmMigration(source: ServerPlayMigrationSource) {
		const next = pendingNext
		pendingNext = undefined
		options.migrationModal.value?.hide()
		void migration.start(source, next)
	}

	const inviteHandoff = injectServerInviteHandoff(null)
	watch(
		() =>
			!!inviteHandoff?.awaitingServer.value && options.serverLoaded.value && migration.ready.value,
		(ready) => {
			if (!ready || !inviteHandoff) return
			inviteHandoff.awaitingServer.value = false
			if (!options.canSetup.value) return
			void nextTick(() => {
				if (migration.required.value) migration.prompt('invite')
				else options.inviteLinkModal.value?.show()
			})
		},
		{ immediate: true },
	)

	return { migration, confirmMigration }
}
