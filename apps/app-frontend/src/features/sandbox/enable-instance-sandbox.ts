import { type InstallJobSnapshot, installJobInstanceId } from '@/helpers/install'
import { edit, get } from '@/helpers/instance'

export type InstallSandboxChoice = { sandbox: boolean }

export async function enableInstanceSandbox(instanceId: string) {
	const instance = await get(instanceId)
	if (!instance) return
	await edit(instanceId, { sandbox: { ...instance.sandbox, mode: 'unrecognized' } })
}

export async function enableInstallJobSandbox(job: InstallJobSnapshot) {
	const instanceId = installJobInstanceId(job)
	if (instanceId) await enableInstanceSandbox(instanceId)
}
