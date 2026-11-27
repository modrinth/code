import { injectNotificationManager } from '@modrinth/ui'
import { useMutation, useQueryClient } from '@tanstack/vue-query'
import { useDebounceFn } from '@vueuse/core'
import { computed, onBeforeUnmount, reactive, type Ref, ref, watch } from 'vue'

import { edit } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { instanceKeys } from '@/pages/instance/query-options'

import { injectInstanceSettings } from './instance-settings-context'

export type SettingsSource = 'app' | 'custom'

const SAVE_DEBOUNCE_MS = 500
const MUTATION_KEY = ['instance-settings', 'edit'] as const

type InstanceEdit = { [K in keyof GameInstance]?: GameInstance[K] | null }

interface EditVariables<T> {
	instanceId: string
	value: T
	request: number
}

function clone<T>(value: T): T {
	return JSON.parse(JSON.stringify(value))
}

function hasCustomValue(value: Record<string, unknown>) {
	return Object.values(value).some((entry) => entry !== null)
}

export function useSettingsSource<T extends Record<string, unknown>>(options: {
	read: (instance: GameInstance) => T
	write: (value: T) => InstanceEdit
	defaults: () => NoInfer<{ [K in keyof T]: NonNullable<T[K]> }>
}) {
	const { instance } = injectInstanceSettings()
	const { handleError } = injectNotificationManager()
	const queryClient = useQueryClient()

	const values = computed(() => options.read(instance.value))
	const pinnedCustom = ref(false)
	const flushers = new Set<() => void>()
	let lastCustom: T | null = null
	let lastSubmitted: T | null = null
	let latestRequest = 0

	function applyTo(target: GameInstance, value: T): GameInstance {
		return { ...target, ...options.write(value) } as GameInstance
	}

	function currentValue(): T {
		if (lastSubmitted) return lastSubmitted
		const cached = queryClient.getQueryData<GameInstance>(instanceKeys.detail(instance.value.id))
		return options.read(cached ?? instance.value)
	}

	const mutation = useMutation({
		mutationKey: MUTATION_KEY,
		scope: { id: MUTATION_KEY.join(':') },
		mutationFn: async ({ instanceId, value, request }: EditVariables<T>) => {
			if (request !== latestRequest) return
			await edit(instanceId, options.write(value) as Partial<GameInstance>)
		},
		onMutate: async ({ instanceId, value }) => {
			const detailKey = instanceKeys.detail(instanceId)
			const listKey = instanceKeys.list()
			await Promise.all([
				queryClient.cancelQueries({ queryKey: detailKey, exact: true }),
				queryClient.cancelQueries({ queryKey: listKey, exact: true }),
			])
			const previousDetail = queryClient.getQueryData<GameInstance>(detailKey)
			const previousList = queryClient.getQueryData<GameInstance[]>(listKey)
			queryClient.setQueryData<GameInstance>(detailKey, (current) =>
				current ? applyTo(current, value) : current,
			)
			queryClient.setQueryData<GameInstance[]>(listKey, (instances) =>
				instances?.map((candidate) =>
					candidate.id === instanceId ? applyTo(candidate, value) : candidate,
				),
			)
			return { previousDetail, previousList }
		},
		onError: (error, { instanceId }, context) => {
			if (context?.previousDetail) {
				queryClient.setQueryData(instanceKeys.detail(instanceId), context.previousDetail)
			}
			if (context?.previousList) {
				queryClient.setQueryData(instanceKeys.list(), context.previousList)
			}
			handleError(error)
		},
		onSettled: async (_data, _error, { instanceId, request }) => {
			if (request === latestRequest) lastSubmitted = null
			if (queryClient.isMutating({ mutationKey: MUTATION_KEY }) > 1) return
			await Promise.all([
				queryClient.invalidateQueries({ queryKey: instanceKeys.detail(instanceId), exact: true }),
				queryClient.invalidateQueries({ queryKey: instanceKeys.list(), exact: true }),
			])
		},
	})

	function save(value: T) {
		lastSubmitted = value
		mutation.mutate({ instanceId: instance.value.id, value, request: ++latestRequest })
	}

	function update(patch: Partial<T>) {
		pinnedCustom.value = true
		save({ ...currentValue(), ...patch })
	}

	function flush() {
		flushers.forEach((commit) => commit())
	}

	const source = computed<SettingsSource>({
		get: () => (pinnedCustom.value || hasCustomValue(values.value) ? 'custom' : 'app'),
		set: (next) => {
			if (next === source.value) return
			flush()
			if (next === 'app') {
				const current = currentValue()
				lastCustom = hasCustomValue(current) ? clone(current) : null
				pinnedCustom.value = false
				save(Object.fromEntries(Object.keys(current).map((key) => [key, null])) as T)
			} else {
				pinnedCustom.value = true
				save(lastCustom ?? (clone(options.defaults()) as T))
			}
		},
	})

	function debouncedField<V>(display: () => V, commitValue: (value: V) => void) {
		const local = ref(display()) as Ref<V>
		let pending = false
		let focused = false

		function commit() {
			if (!pending) return
			pending = false
			commitValue(local.value)
		}

		const scheduleCommit = useDebounceFn(commit, SAVE_DEBOUNCE_MS)
		flushers.add(commit)

		watch(display, (value) => {
			if (!pending && !focused) local.value = value
		})

		const model = computed({
			get: () => local.value,
			set: (value: V) => {
				local.value = value
				pending = true
				void scheduleCommit()
			},
		})

		function onFocus() {
			focused = true
		}

		function onBlur() {
			focused = false
			if (pending) {
				commit()
			} else {
				local.value = display()
			}
		}

		return reactive({ model, onFocus, onBlur })
	}

	onBeforeUnmount(flush)

	return { source, values, update, debouncedField }
}
