import { provideAppBackup } from '@modrinth/ui'
import { type MaybeRefOrGetter, toValue } from 'vue'

import { install_duplicate_instance, installJobInstanceId } from '@/helpers/install'
import { edit, list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

export function provideInstanceBackup(instance: MaybeRefOrGetter<GameInstance>) {
	provideAppBackup({
		async createBackup() {
			const source = toValue(instance)
			const prefix = `${source.name} - Backup #`
			const existingNumbers = (await list())
				.filter((candidate) => candidate.name.startsWith(prefix))
				.map((candidate) => Number.parseInt(candidate.name.slice(prefix.length), 10))
				.filter(Number.isFinite)
			const nextNumber = existingNumbers.length ? Math.max(...existingNumbers) + 1 : 1
			const job = await install_duplicate_instance(source.id)
			const backupInstanceId = installJobInstanceId(job)
			if (backupInstanceId) {
				await edit(backupInstanceId, { name: `${prefix}${nextNumber}` })
			}
		},
	})
}
