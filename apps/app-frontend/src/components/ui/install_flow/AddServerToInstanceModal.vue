<script setup>
import {
	CheckIcon,
	PlusIcon,
	SearchIcon,
} from '@modrinth/assets'
import { Admonition, Avatar, ButtonStyled, injectNotificationManager, StyledInput } from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { computed, ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { list } from '@/helpers/profile'
import { add_server_to_profile, get_profile_worlds } from '@/helpers/worlds.ts'

const { handleError } = injectNotificationManager()

const modal = ref()
const searchFilter = ref('')
const profiles = ref([])

const serverName = ref('')
const serverAddress = ref('')

const shownProfiles = computed(() =>
	profiles.value.filter((profile) => {
		return profile.name.toLowerCase().includes(searchFilter.value.toLowerCase())
	}),
)

defineExpose({
	show: async (name, address) => {
		serverName.value = name
		serverAddress.value = address
		searchFilter.value = ''

		const profilesVal = await list().catch(handleError)
		for (const profile of profilesVal) {
			profile.adding = false
			profile.added = false

			try {
				const worlds = await get_profile_worlds(profile.path)
				profile.added = worlds.some(
					(w) => w.type === 'server' && w.address === serverAddress.value,
				)
			} catch {
				// Ignore - will show as not added
			}
		}

		profiles.value = profilesVal
		modal.value.show()

		trackEvent('AddServerToInstanceStart', { source: 'AddServerToInstanceModal' })
	},
})

async function addServer(profile) {
	profile.adding = true
	try {
		await add_server_to_profile(
			profile.path,
			serverName.value,
			serverAddress.value,
			'prompt',
		)
		profile.added = true

		trackEvent('AddServerToInstance', {
			server_name: serverName.value,
			instance_name: profile.name,
			source: 'AddServerToInstanceModal',
		})
	} catch (err) {
		handleError(err)
	}
	profile.adding = false
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
					v-for="profile in shownProfiles"
					:key="profile.path"
					class="flex w-full items-center justify-between gap-2 bg-bg-raised text-icon shadow-none"
				>
					<router-link
						class="btn btn-transparent p-2 text-left"
						:to="`/instance/${encodeURIComponent(profile.path)}`"
						@click="modal.hide()"
					>
						<Avatar
							:src="profile.icon_path ? convertFileSrc(profile.icon_path) : null"
							class="mr-2 [--size:2rem]"
						/>
						{{ profile.name }}
					</router-link>
					<ButtonStyled>
						<button
							:disabled="profile.added || profile.adding"
							@click="addServer(profile)"
						>
							<PlusIcon v-if="!profile.added && !profile.adding" />
							<CheckIcon v-else-if="profile.added" />
							{{
								profile.adding
									? 'Adding...'
									: profile.added
										? 'Added'
										: 'Add'
							}}
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

