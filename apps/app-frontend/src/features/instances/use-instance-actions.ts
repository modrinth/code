import { injectNotificationManager } from '@modrinth/ui'
import { computed, onMounted, ref, toValue, type MaybeRefOrGetter } from 'vue'
import { useRouter } from 'vue-router'

import type { InstanceCard } from '@/components/ui/library/use-library'
import { useAppEvent } from '@/composables/use-app-event'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { kill, run } from '@/helpers/instance'
import { get_by_instance_id } from '@/helpers/process'
import type { GameInstance } from '@/helpers/types'
import { showInstanceInFolder } from '@/helpers/utils.js'

type ProcessEvent = 'installing' | 'launched' | 'finished'

export function useInstanceActions(
	instanceSource: MaybeRefOrGetter<GameInstance>,
	options: { beforeNavigate?: () => boolean } = {},
) {
	const { handleError } = injectNotificationManager()
	const router = useRouter()
	const instance = computed(() => toValue(instanceSource))

	const playing = ref(false)
	const loading = ref(false)
	const currentEvent = ref<ProcessEvent | null>(null)
	const modLoading = computed(
		() =>
			loading.value ||
			currentEvent.value === 'installing' ||
			(currentEvent.value === 'launched' && !playing.value),
	)
	const installing = computed(() => instance.value.install_stage.includes('installing'))

	const navigate = async (to: Parameters<typeof router.push>[0]) => {
		if (options.beforeNavigate && !options.beforeNavigate()) return
		await router.push(to)
	}

	const checkProcess = async () => {
		const runningProcesses = (await get_by_instance_id(instance.value.id).catch(handleError)) ?? []
		playing.value = runningProcesses.length > 0
	}

	const play = async (event: MouseEvent | null, context: string) => {
		event?.stopPropagation()
		if (instance.value.quarantined || installing.value || modLoading.value) return
		loading.value = true
		await run(instance.value.id)
			.catch((err) => handleSevereError(err, { instanceId: instance.value.id }))
			.finally(() => {
				trackEvent('InstanceStart', {
					loader: instance.value.loader,
					game_version: instance.value.game_version,
					source: context,
				})
			})
		loading.value = false
	}

	const stop = async (event: MouseEvent | null, context: string) => {
		event?.stopPropagation()
		playing.value = false

		await kill(instance.value.id).catch(handleError)

		trackEvent('InstanceStop', {
			loader: instance.value.loader,
			game_version: instance.value.game_version,
			source: context,
		})
	}

	const seeInstance = async () => {
		if (installing.value) return
		await navigate(`/instance/${encodeURIComponent(instance.value.id)}`)
	}

	const openFolder = async () => {
		await showInstanceInFolder(instance.value.id)
	}

	const addContent = async () => {
		if (instance.value.quarantined || installing.value || modLoading.value) return
		await navigate({
			path: `/browse/${instance.value.loader === 'vanilla' ? 'datapack' : 'mod'}`,
			query: { i: instance.value.id },
		})
	}

	useAppEvent('process', (event) => {
		if (event.instance_id === instance.value.id) {
			currentEvent.value = event.event
			playing.value = event.event === 'launched'
		}
	})

	onMounted(() => {
		void checkProcess()
	})

	const card = computed<InstanceCard>(() => ({
		instance: instance.value,
		playing: playing.value,
		play,
		stop,
		addContent,
		seeInstance,
		openFolder,
	}))

	return {
		playing,
		loading,
		modLoading,
		installing,
		checkProcess,
		play,
		stop,
		seeInstance,
		openFolder,
		addContent,
		card,
	}
}
