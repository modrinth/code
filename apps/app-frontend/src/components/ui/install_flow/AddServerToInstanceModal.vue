<script setup>
import { CheckIcon, PlusIcon, SearchIcon } from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	ButtonStyled,
	injectNotificationManager,
	StyledInput,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { list } from '@/helpers/instance'
import { add_server_to_instance, get_instance_worlds } from '@/helpers/worlds.ts'

const { handleError } = injectNotificationManager()

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
	<ModalWrapper ref="modal" header="Add server to instance">
		<div class="flex flex-col gap-4 min-w-[350px]">
			<Admonition type="warning" body="This server may not be compatible with all instances." />
			<StyledInput
				v-model="searchFilter"
				:icon="SearchIcon"
				type="search"
				placeholder="Search for an instance"
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
						<Avatar
							:src="instance.icon_path ? convertFileSrc(instance.icon_path) : null"
							class="mr-2 [--size:2rem]"
						/>
						{{ instance.name }}
					</router-link>
					<ButtonStyled>
						<button :disabled="instance.added || instance.adding" @click="addServer(instance)">
							<PlusIcon v-if="!instance.added && !instance.adding" />
							<CheckIcon v-else-if="instance.added" />
							{{ instance.adding ? 'Adding...' : instance.added ? 'Added' : 'Add' }}
						</button>
					</ButtonStyled>
				</div>
			</div>
			<div class="input-group push-right">
				<ButtonStyled>
					<button @click="modal.hide()">Cancel</button>
				</ButtonStyled>
			</div>
		</div>
	</ModalWrapper>
</template>
