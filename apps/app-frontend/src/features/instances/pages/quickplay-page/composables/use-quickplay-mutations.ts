import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import type { ComputedRef } from 'vue'

import { worldKeys } from '@/features/worlds/queries'
import { sortWorlds } from '@/features/worlds/world-utils'
import { set_synced_option } from '@/helpers/instance'
import { instanceKeys } from '@/pages/instance/query-options'
import {
	addServerToInstance,
	deleteWorld,
	desyncServer,
	editServerInInstance,
	removeServerFromInstance,
	renameWorld,
	resetWorldIcon,
	setWorldDisplayStatus,
} from '@/platform/app-lib/worlds/commands'
import type { DesyncServerMode, ServerWorld, World } from '@/platform/app-lib/worlds/types'

import type {
	AddServerValues,
	EditServerValues,
	EditWorldValues,
	RemoveWorldValues,
} from '../types'

type WorldsKey = ReturnType<typeof worldKeys.instance>

interface WorldsMutationContext {
	key: WorldsKey
	previous: World[] | undefined
}

function isServerAt(world: World, index: number): world is ServerWorld {
	return world.type === 'server' && world.index === index
}

function isWorldAt(world: World, path: string) {
	return world.type === 'singleplayer' && world.path === path
}

function nextServerIndex(worlds: World[]) {
	return worlds.reduce(
		(max, world) => (world.type === 'server' ? Math.max(max, world.index + 1) : max),
		0,
	)
}

export function useQuickplayMutations(instanceId: ComputedRef<string>) {
	const queryClient = useQueryClient()
	const { handleError } = injectNotificationManager()

	async function applyOptimistic(
		update?: (worlds: World[]) => World[],
	): Promise<WorldsMutationContext> {
		const key = worldKeys.instance(instanceId.value)
		await queryClient.cancelQueries({ queryKey: key })
		const previous = queryClient.getQueryData<World[]>(key)
		if (previous && update) {
			const next = update(previous)
			sortWorlds(next)
			queryClient.setQueryData(key, next)
		}
		return { key, previous }
	}

	function rollback(error: Error, context: WorldsMutationContext | undefined) {
		if (context?.previous) queryClient.setQueryData(context.key, context.previous)
		handleError(error)
	}

	function invalidateWorlds(context: WorldsMutationContext | undefined) {
		return queryClient.invalidateQueries({
			queryKey: context?.key ?? worldKeys.instance(instanceId.value),
		})
	}

	const addServerMutation = useMutation({
		networkMode: 'always',
		mutationFn: ({ name, address, packStatus }: AddServerValues) =>
			addServerToInstance(instanceId.value, name, address, packStatus),
		onMutate: (values) =>
			applyOptimistic((worlds) => [
				...worlds,
				{
					type: 'server',
					index: nextServerIndex(worlds),
					name: values.name,
					address: values.address,
					pack_status: values.packStatus,
					display_status: 'normal',
				},
			]),
		onError: (error, _values, context) => rollback(error, context),
		onSettled: (_data, _error, _values, context) => invalidateWorlds(context),
	})

	const editServerMutation = useMutation({
		networkMode: 'always',
		mutationFn: async ({ server, name, address, packStatus, displayStatus }: EditServerValues) => {
			await editServerInInstance(instanceId.value, server.index, name, address, packStatus)
			if (displayStatus !== server.display_status) {
				await setWorldDisplayStatus(instanceId.value, 'server', address, displayStatus)
			}
		},
		onMutate: ({ server, name, address, packStatus, displayStatus }) =>
			applyOptimistic((worlds) =>
				worlds.map((world) =>
					isServerAt(world, server.index)
						? {
								...world,
								name,
								address,
								pack_status: packStatus,
								display_status: displayStatus,
							}
						: world,
				),
			),
		onError: (error, _values, context) => rollback(error, context),
		onSettled: (_data, _error, _values, context) => invalidateWorlds(context),
	})

	const editWorldMutation = useMutation({
		networkMode: 'always',
		mutationFn: async ({
			path,
			name,
			removeIcon,
			displayStatus,
			previousDisplayStatus,
		}: EditWorldValues) => {
			await renameWorld(instanceId.value, path, name)
			if (removeIcon) await resetWorldIcon(instanceId.value, path)
			if (displayStatus !== previousDisplayStatus) {
				await setWorldDisplayStatus(instanceId.value, 'singleplayer', path, displayStatus)
			}
		},
		onMutate: ({ path, name, removeIcon, displayStatus }) =>
			applyOptimistic((worlds) =>
				worlds.map((world) =>
					isWorldAt(world, path)
						? {
								...world,
								name,
								icon: removeIcon ? undefined : world.icon,
								display_status: displayStatus,
							}
						: world,
				),
			),
		onError: (error, _values, context) => rollback(error, context),
		onSettled: (_data, _error, _values, context) => invalidateWorlds(context),
	})

	const removeWorldMutation = useMutation({
		networkMode: 'always',
		mutationFn: async ({ world, scope }: RemoveWorldValues) => {
			const id = instanceId.value
			if (world.type === 'singleplayer') {
				await deleteWorld(id, world.path)
				return
			}
			if (scope === 'here' && world.source === 'user_synced') {
				const updated = await set_synced_option(id, 'multiplayer_servers', false)
				queryClient.setQueryData(instanceKeys.detail(id), updated)
			}
			await removeServerFromInstance(id, world.index)
		},
		onMutate: ({ world }) =>
			applyOptimistic((worlds) =>
				worlds.filter((existing) =>
					world.type === 'server'
						? !isServerAt(existing, world.index)
						: !isWorldAt(existing, world.path),
				),
			),
		onError: (error, _values, context) => rollback(error, context),
		onSettled: async (_data, _error, { world }, context) => {
			if (world.type === 'server') {
				await Promise.all([
					queryClient.invalidateQueries({ queryKey: instanceKeys.all }),
					queryClient.invalidateQueries({ queryKey: ['instance-synced-options'] }),
				])
			}
			await invalidateWorlds(context)
		},
	})

	const desyncServerMutation = useMutation({
		networkMode: 'always',
		mutationFn: async ({ server, mode }: { server: ServerWorld; mode: DesyncServerMode }) => {
			if (!server.server_id) throw new Error(`Server ${server.address} is not synced`)
			await desyncServer(instanceId.value, server.server_id, mode)
		},
		onMutate: () => applyOptimistic(),
		onError: (error, _values, context) => rollback(error, context),
		onSettled: (_data, _error, _values, context) => invalidateWorlds(context),
	})

	return {
		addServerMutation,
		editServerMutation,
		editWorldMutation,
		removeWorldMutation,
		desyncServerMutation,
	}
}
