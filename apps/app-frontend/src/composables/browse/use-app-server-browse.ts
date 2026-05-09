import type { Labrinth } from '@modrinth/api-client'
import { CheckIcon, PlayIcon, PlusIcon, StopCircleIcon } from '@modrinth/assets'
import type { CardAction } from '@modrinth/ui'
import { commonMessages, defineMessages, useDebugLogger, useVIntl } from '@modrinth/ui'
import { openUrl } from '@tauri-apps/plugin-opener'
import type { ComputedRef, Ref } from 'vue'
import { onUnmounted, ref, shallowRef } from 'vue'
import type { Router } from 'vue-router'

import { process_listener } from '@/helpers/events'
import { get_by_profile_path } from '@/helpers/process'
import { kill, list as listInstances } from '@/helpers/profile.js'
import type { GameInstance } from '@/helpers/types'
import { add_server_to_profile, getServerLatency } from '@/helpers/worlds'
import { getServerAddress } from '@/store/install.js'

interface BrowseServerInstance {
	name: string
	path: string
}

interface ContextMenuHandle {
	showMenu: (
		event: MouseEvent,
		result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
		options: { name: string }[],
	) => void
}

interface ContextMenuOptionClick {
	option: 'open_link' | 'copy_link'
	item: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject
}

export interface UseAppServerBrowseOptions {
	instance: Ref<BrowseServerInstance | null>
	isFromWorlds: ComputedRef<boolean>
	allInstalledIds: ComputedRef<Set<string>>
	newlyInstalled: Ref<string[]>
	installingServerProjects: Ref<string[]>
	playServerProject: (projectId: string) => Promise<void>
	showAddServerToInstanceModal: (serverName: string, serverAddress: string) => void
	handleError: (error: unknown) => void
	router: Router
}

const messages = defineMessages({
	addToInstance: {
		id: 'app.browse.add-to-instance',
		defaultMessage: 'Add to instance',
	},
	addToInstanceName: {
		id: 'app.browse.add-to-instance-name',
		defaultMessage: 'Add to {instanceName}',
	},
	added: {
		id: 'app.browse.added',
		defaultMessage: 'Added',
	},
	alreadyAdded: {
		id: 'app.browse.already-added',
		defaultMessage: 'Already added',
	},
})

export function useAppServerBrowse(options: UseAppServerBrowseOptions) {
	const { formatMessage } = useVIntl()
	const debugLog = useDebugLogger('BrowseServer')
	const serverPings = shallowRef<Record<string, number | undefined>>({})
	const serverPingCache = new Map<string, number | undefined>()
	const pendingServerPings = new Map<string, Promise<number | undefined>>()
	const runningServerProjects = ref<Record<string, string>>({})
	const lastServerHits = shallowRef<Labrinth.Search.v3.ResultSearchProject[]>([])
	const contextMenuRef = ref<ContextMenuHandle | null>(null)
	let serverPingCacheActive = true
	let unlistenProcesses: (() => void) | null = null

	async function checkServerRunningStates(hits: Labrinth.Search.v3.ResultSearchProject[]) {
		debugLog('checkServerRunningStates', { hitCount: hits.length })
		const packs = await listInstances().catch((error) => {
			options.handleError(error)
			return []
		})
		const newRunning: Record<string, string> = {}
		for (const hit of hits) {
			const inst = packs.find(
				(pack: GameInstance) => pack.linked_data?.project_id === hit.project_id,
			)
			if (inst) {
				const processes = await get_by_profile_path(inst.path).catch(() => [])
				if (Array.isArray(processes) && processes.length > 0) {
					newRunning[hit.project_id] = inst.path
				}
			}
		}
		debugLog('runningServerProjects updated', newRunning)
		runningServerProjects.value = newRunning
	}

	async function handleStopServerProject(projectId: string) {
		debugLog('handleStopServerProject', projectId)
		const instancePath = runningServerProjects.value[projectId]
		if (!instancePath) return
		await kill(instancePath).catch(() => {})
		const { [projectId]: _, ...rest } = runningServerProjects.value
		runningServerProjects.value = rest
	}

	async function handlePlayServerProject(projectId: string) {
		debugLog('handlePlayServerProject', projectId)
		await options.playServerProject(projectId)
		checkServerRunningStates(lastServerHits.value)
	}

	async function handleAddServerToInstance(project: Labrinth.Search.v3.ResultSearchProject) {
		debugLog('handleAddServerToInstance', { projectId: project.project_id, name: project.name })
		const address = getServerAddress(project.minecraft_java_server)
		if (!address) return

		if (options.instance.value) {
			try {
				await add_server_to_profile(
					options.instance.value.path,
					project.name,
					address,
					'prompt',
					project.project_id,
					project.minecraft_java_server?.content?.kind,
				)
				options.newlyInstalled.value.push(project.project_id)
			} catch (error) {
				options.handleError(error)
			}
		} else {
			options.showAddServerToInstanceModal(project.name, address)
		}
	}

	async function pingServerHits(hits: Labrinth.Search.v3.ResultSearchProject[]) {
		debugLog('pingServerHits', { hitCount: hits.length })
		const pingsToFetch = hits.flatMap((hit) => {
			const address = hit.minecraft_java_server?.address
			if (!address) return []
			return [{ hit, address }]
		})
		const nextPings = { ...serverPings.value }
		for (const { hit, address } of pingsToFetch) {
			if (serverPingCache.has(address)) {
				nextPings[hit.project_id] = serverPingCache.get(address)
			}
		}
		serverPings.value = nextPings

		await Promise.all(
			pingsToFetch.map(async ({ hit, address }) => {
				if (serverPingCache.has(address)) return

				let pending = pendingServerPings.get(address)
				if (!pending) {
					pending = getServerLatency(address)
						.then((latency) => {
							if (serverPingCacheActive) serverPingCache.set(address, latency)
							return latency
						})
						.catch((error) => {
							console.error(`Failed to ping server ${address}:`, error)
							if (serverPingCacheActive) serverPingCache.set(address, undefined)
							return undefined
						})
						.finally(() => {
							pendingServerPings.delete(address)
						})
					pendingServerPings.set(address, pending)
				}

				const latency = await pending
				if (!serverPingCacheActive) return
				serverPings.value = { ...serverPings.value, [hit.project_id]: latency }
			}),
		)
	}

	function updateServerHits(hits: Labrinth.Search.v3.ResultSearchProject[]) {
		lastServerHits.value = hits
		pingServerHits(hits)
		checkServerRunningStates(hits)
	}

	function getServerModpackContent(project: Labrinth.Search.v3.ResultSearchProject) {
		const content = project.minecraft_java_server?.content
		if (content?.kind === 'modpack') {
			const { project_name, project_icon, project_id } = content
			if (!project_name) return undefined
			return {
				name: project_name,
				icon: project_icon ?? undefined,
				onclick:
					project_id !== project.project_id
						? () => {
								options.router.push(`/project/${project_id}`)
							}
						: undefined,
				showCustomModpackTooltip: project_id === project.project_id,
			}
		}
		return undefined
	}

	function getServerCardActions(
		serverResult: Labrinth.Search.v3.ResultSearchProject,
	): CardAction[] {
		const isInstalled = options.allInstalledIds.value.has(serverResult.project_id)

		if (options.isFromWorlds.value && options.instance.value) {
			return [
				{
					key: 'add-to-instance',
					label: formatMessage(isInstalled ? messages.added : messages.addToInstance),
					icon: isInstalled ? CheckIcon : PlusIcon,
					disabled: isInstalled,
					color: 'brand',
					type: 'outlined',
					onClick: () => handleAddServerToInstance(serverResult),
				},
			]
		}

		const actions: CardAction[] = []

		actions.push({
			key: 'add',
			label: '',
			icon: isInstalled ? CheckIcon : PlusIcon,
			disabled: isInstalled,
			circular: true,
			tooltip: isInstalled
				? formatMessage(messages.alreadyAdded)
				: options.instance.value
					? formatMessage(messages.addToInstanceName, {
							instanceName: options.instance.value.name,
						})
					: formatMessage(commonMessages.addServerToInstanceButton),
			onClick: () => handleAddServerToInstance(serverResult),
		})

		if (runningServerProjects.value[serverResult.project_id]) {
			actions.push({
				key: 'stop',
				label: formatMessage(commonMessages.stopButton),
				icon: StopCircleIcon,
				color: 'red',
				type: 'outlined',
				onClick: () => handleStopServerProject(serverResult.project_id),
			})
		} else {
			const isInstalling = options.installingServerProjects.value.includes(serverResult.project_id)
			actions.push({
				key: 'play',
				label: formatMessage(
					isInstalling ? commonMessages.installingLabel : commonMessages.playButton,
				),
				icon: PlayIcon,
				disabled: isInstalling,
				color: 'brand',
				type: 'outlined',
				onClick: () => handlePlayServerProject(serverResult.project_id),
			})
		}

		return actions
	}

	function handleRightClick(
		event: MouseEvent,
		result: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
	) {
		contextMenuRef.value?.showMenu(event, result, [{ name: 'open_link' }, { name: 'copy_link' }])
	}

	function handleOptionsClick(args: ContextMenuOptionClick) {
		const url = getProjectUrl(args.item)
		switch (args.option) {
			case 'open_link':
				openUrl(url)
				break
			case 'copy_link':
				navigator.clipboard.writeText(url)
				break
		}
	}

	process_listener((event: { event: string; profile_path_id: string }) => {
		debugLog('process event', event)
		if (event.event === 'finished') {
			const projectId = Object.entries(runningServerProjects.value).find(
				([, path]) => path === event.profile_path_id,
			)?.[0]
			if (projectId) {
				const { [projectId]: _, ...rest } = runningServerProjects.value
				runningServerProjects.value = rest
			}
		}
	})
		.then((unlisten) => {
			unlistenProcesses = unlisten
		})
		.catch(options.handleError)

	onUnmounted(() => {
		serverPingCacheActive = false
		unlistenProcesses?.()
		serverPingCache.clear()
		pendingServerPings.clear()
	})

	return {
		serverPings,
		contextMenuRef,
		updateServerHits,
		getServerModpackContent,
		getServerCardActions,
		handleRightClick,
		handleOptionsClick,
	}
}

function getProjectUrl(
	item: Labrinth.Search.v2.ResultSearchProject | Labrinth.Search.v3.ResultSearchProject,
) {
	const projectType = 'project_types' in item ? item.project_types?.[0] : item.project_type
	return `https://modrinth.com/${projectType ?? 'project'}/${item.slug ?? item.project_id}`
}
