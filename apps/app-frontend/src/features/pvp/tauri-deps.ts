import { resolveResource } from '@tauri-apps/api/path'

import { get_project } from '@/helpers/cache.js'
import {
	install_create_instance,
	installJobInstanceId,
	wait_for_install_job,
} from '@/helpers/install'
import {
	add_project_from_path,
	edit,
	install_project_with_dependencies,
	remove,
} from '@/helpers/instance'
import type { AppEvents } from '@/providers/app-events'

import type { InstallerDeps } from './installer'

const MOD_INSTALL_TIMEOUT_MS = 3 * 60 * 1000

/**
 * Real implementation of the installer's side effects on top of the existing theseus backend.
 * No Rust changes are needed: instance creation, dependency-aware Modrinth installs and
 * "add local file" are all existing commands.
 */
export function createTauriInstallerDeps(appEvents: AppEvents): InstallerDeps {
	/** Wait for the background content install of `projectId` in `instanceId` to finish. */
	function waitForContentInstall(instanceId: string, projectId: string) {
		let settle: (err?: Error) => void = () => {}
		const done = new Promise<void>((resolve, reject) => {
			settle = (err) => (err ? reject(err) : resolve())
		})
		const unlisten = appEvents.on('instance', (payload) => {
			if (payload.instance_id !== instanceId) return
			if (payload.event === 'content_install_finished' && payload.project_ids.includes(projectId)) {
				settle()
			} else if (
				payload.event === 'content_install_failed' &&
				payload.project_ids.includes(projectId)
			) {
				settle(new Error(payload.message))
			}
		})
		const timer = setTimeout(
			() => settle(new Error('Timed out waiting for the download to finish')),
			MOD_INSTALL_TIMEOUT_MS,
		)
		return {
			done: done.finally(() => {
				clearTimeout(timer)
				unlisten()
			}),
			cancel: () => {
				clearTimeout(timer)
				unlisten()
			},
		}
	}

	return {
		async createInstance({ name, gameVersion, loader }) {
			const job = await install_create_instance({
				name,
				gameVersion,
				loader,
				loaderVersion: 'latest',
				iconPath: null,
			})
			const id = installJobInstanceId(job)
			if (!id) throw new Error('The launcher did not return an instance id')
			await wait_for_install_job(appEvents, job.job_id)
			return id
		},

		async resolveProjectId(slug) {
			const project = await get_project(slug, 'must_revalidate')
			if (!project?.id) throw new Error(`"${slug}" was not found on Modrinth`)
			return project.id as string
		},

		async installMod({ instanceId, projectId, gameVersion, loader }) {
			// Register the waiter BEFORE starting the install so a fast finish is never missed.
			const waiter = waitForContentInstall(instanceId, projectId)
			try {
				await install_project_with_dependencies(instanceId, {
					project_id: projectId,
					content_type: 'mod',
					selected: { game_versions: [gameVersion], loaders: [loader] },
				})
			} catch (e) {
				waiter.cancel()
				throw e
			}
			await waiter.done
		},

		async tuneInstance({ instanceId, jvmArgs, memoryMb }) {
			await edit(instanceId, {
				extra_launch_args: jvmArgs,
				memory: { maximum: memoryMb },
			})
		},

		async resolveBundledResource(relativePath) {
			return await resolveResource(relativePath)
		},

		async addLocalMod(instanceId, absolutePath) {
			await add_project_from_path(instanceId, absolutePath, 'mod')
		},

		async removeInstance(instanceId) {
			await remove(instanceId)
		},
	}
}
