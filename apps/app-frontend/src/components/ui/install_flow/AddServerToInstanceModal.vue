<script setup>
import { CheckIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	defineMessages,
	injectNotificationManager,
	StyledInput,
	useVIntl,
} from '@modrinth/ui'
import { useQueryClient } from '@tanstack/vue-query'
import { computed, ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { getInstanceIconUrl, list } from '@/helpers/instance'
import { add_server_to_instance, get_instance_worlds } from '@/helpers/worlds.ts'
import { instanceKeys } from '@/pages/instance/query-options'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const messages = defineMessages({
	title: { id: 'app.instance.add-server.title', defaultMessage: 'Add server to instance' },
	compatibilityWarning: {
		id: 'app.instance.add-server.compatibility-warning',
		defaultMessage: 'This server may not be compatible with all instances.',
	},
	searchPlaceholder: {
		id: 'app.instance.add-server.search-placeholder',
		defaultMessage: 'Search for an instance',
	},
	adding: { id: 'app.instance.add-server.adding', defaultMessage: 'Adding...' },
	added: { id: 'app.instance.add-server.added', defaultMessage: 'Added' },
	add: { id: 'app.instance.add-server.add', defaultMessage: 'Add' },
	cancel: { id: 'app.instance.add-server.cancel', defaultMessage: 'Cancel' },
})
const queryClient = useQueryClient()

const modal = ref()
const searchFilter = ref('')
const instances = ref([])

const serverName = ref('')
const serverAddress = ref('')

const shownInstances = computed(() =>
	instances.value.filter((instance) => {
		return instance.name.toLowerCase().includes(searchFilter.value.toLowerCase())
	}),
)

defineExpose({
	show: async (name, address) => {
		serverName.value = name
		serverAddress.value = address
		searchFilter.value = ''

		const instanceValues = await list().catch(handleError)
		await Promise.allSettled(
			instanceValues.map(async (instance) => {
				instance.adding = false
				instance.added = false

				try {
					const worlds = await get_instance_worlds(instance.id)
					instance.added = worlds.some(
						(w) => w.type === 'server' && w.address === serverAddress.value,
					)
				} catch {
					// Ignore - will show as not added
				}
			}),
		)

		instances.value = instanceValues
		modal.value.show()

		trackEvent('AddServerToInstanceStart', { source: 'AddServerToInstanceModal' })
	},
})

async function addServer(instance) {
	instance.adding = true
	try {
		await add_server_to_instance(instance.id, serverName.value, serverAddress.value, 'prompt')
		instance.added = true
		await queryClient.invalidateQueries({ queryKey: instanceKeys.worlds(instance.id) })

		trackEvent('AddServerToInstance', {
			server_name: serverName.value,
			instance_name: instance.name,
			source: 'AddServerToInstanceModal',
		})
	} catch (err) {
		handleError(err)
	}
	instance.adding = false
}
</script>

<template>
	<ModalWrapper ref="modal" :header="formatMessage(messages.title)">
		<div class="flex flex-col gap-4 min-w-[350px]">
			<Admonition type="warning" :body="formatMessage(messages.compatibilityWarning)" />
			<StyledInput
				v-model="searchFilter"
				:icon="SearchIcon"
				type="search"
				:placeholder="formatMessage(messages.searchPlaceholder)"
				autocomplete="off"
			/>
			<div class="max-h-[21rem] overflow-y-auto">
				<div
					v-for="instance in shownInstances"
					:key="instance.id"
					class="flex w-full items-center justify-between gap-2 bg-bg-raised text-icon shadow-none"
				>
					<router-link
						class="btn btn-transparent p-2 text-left"
						:to="`/instance/${encodeURIComponent(instance.id)}`"
						@click="modal.hide()"
					>
						<Avatar :src="getInstanceIconUrl(instance.icon_path)" class="mr-2 [--size:2rem]" />
						{{ instance.name }}
					</router-link>
					<Button :disabled="instance.added || instance.adding" @click="addServer(instance)">
						<PlusIcon v-if="!instance.added && !instance.adding" />
						<CheckIcon v-else-if="instance.added" />
						{{
							formatMessage(
								instance.adding ? messages.adding : instance.added ? messages.added : messages.add,
							)
						}}
					</Button>
				</div>
			</div>
			<div class="input-group push-right">
				<Button @click="modal.hide()">{{ formatMessage(messages.cancel) }}</Button>
			</div>
		</div>
	</ModalWrapper>
</template>
