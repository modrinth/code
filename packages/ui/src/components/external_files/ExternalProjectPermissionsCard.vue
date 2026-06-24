<script setup lang="ts">
import { ChevronDownIcon, ListBulletedIcon, SaveIcon, VersionIcon, XIcon } from '@modrinth/assets'
import { ref } from 'vue'

import { Admonition, ButtonStyled, Chips, Collapsible, Combobox, StyledInput } from '#ui/components'

defineProps<{
	title: string
}>()

const collapsed = ref(true)

const selectedPermissionsType = ref('My project')
</script>
<template>
	<div
		class="bg-surface-2 p-0 rounded-2xl flex flex-col border-[1px] border-solid border-surface-5 overflow-hidden"
	>
		<div class="flex items-center bg-surface-3">
			<button
				class="flex grow m-0 appearance-none p-4 bg-transparent group transition-all"
				@click="collapsed = !collapsed"
			>
				<span class="flex items-center gap-3 group-active:scale-[0.98]">
					<ChevronDownIcon
						class="size-6 text-primary transition-transform duration-300"
						:class="{ 'rotate-180': !collapsed }"
					/>
					<span class="text-contrast font-semibold">{{ title }}</span>
				</span>
			</button>
			<div class="flex items-center gap-2 m-4 ml-0">
				<ButtonStyled type="outlined">
					<button>
						<ListBulletedIcon />
						Versions
					</button>
				</ButtonStyled>
			</div>
		</div>
		<Collapsible
			:collapsed="collapsed"
			class="border-0 border-solid border-t border-surface-5 rounded-b-2xl"
		>
			<div class="flex flex-col gap-2 p-4">
				<span class="text-contrast font-semibold">Included in versions:</span>
				<div class="flex flex-wrap gap-2">
					<template v-for="version in ['4.0.0', '3.5.15', '3.5.14']" :key="version">
						<div
							class="px-3 py-2 rounded-xl flex items-center gap-2 border-[1px] border-solid border-surface-5"
						>
							<VersionIcon />
							{{ version }}
						</div>
					</template>
				</div>
				<div
					class="rounded-2xl p-4 mt-2 border-[1px] border-solid border-surface-5 flex flex-col gap-3"
				>
					<span class="text-contrast font-semibold">Type</span>
					<Chips
						v-model="selectedPermissionsType"
						:items="['License', 'My project', 'Special permission', 'No permission']"
					/>
					<template v-if="selectedPermissionsType === 'License'">
						<span>The license of this work permits you to redistribute it in your modpack.</span>
						<span class="text-contrast font-semibold mt-1">License</span>
						<Combobox
							class="max-w-80"
							:options="[{ label: 'MIT', value: 'MIT' }]"
							:model-value="'MIT'"
						/>
						<span class="text-contrast font-semibold mt-1"> Link to work </span>
						<StyledInput
							type="text"
							class="max-w-[30rem]"
							placeholder="https://example.com/link-to-work"
						/>
						<span class="text-contrast font-semibold mt-1">
							Notes
							<span class="font-normal text-primary">(optional)</span>
						</span>
						<StyledInput
							type="text"
							resize="both"
							multiline
							class="max-w-[40rem]"
							placeholder="Write something here..."
						/>
					</template>
					<template v-else-if="selectedPermissionsType === 'My project'">
						<span>Original work created by you.</span>
						<span class="text-contrast font-semibold mt-1">License</span>
						<Combobox
							class="max-w-80"
							:options="[{ label: 'MIT', value: 'MIT' }]"
							:model-value="'MIT'"
						/>
						<span class="text-contrast font-semibold mt-1">
							Notes
							<span class="font-normal text-primary">(optional)</span>
						</span>
						<StyledInput
							type="text"
							resize="both"
							multiline
							class="max-w-[40rem]"
							placeholder="Write something here..."
						/>
					</template>
					<template v-else-if="selectedPermissionsType === 'Special permission'">
						<span>
							You have obtained special permission to redistribute this work in your modpack.
						</span>
						<span class="text-contrast font-semibold mt-1"> Link to work </span>
						<StyledInput
							type="text"
							class="max-w-[30rem]"
							placeholder="https://example.com/link-to-work"
						/>
						<div class="flex flex-col gap-1 mt-1">
							<span class="text-contrast font-semibold"> Proof and explanation </span>
							<span>
								Include screenshots of messages, emails, or replies from the copyright owner showing
								that they granted you permission to redistribute their work in your modpack.
							</span>
						</div>
						<StyledInput
							type="text"
							resize="both"
							multiline
							class="max-w-[40rem]"
							placeholder="Write something here..."
						/>
						<Admonition
							type="warning"
							header="Modrinth staff may attempt to verify submitted proof"
						>
							If you are found to have lied or manipulated the images uploaded, your project and
							account may be terminated.
						</Admonition>
					</template>
					<template v-else-if="selectedPermissionsType === 'No permission'">
						<span>You don't have permission to use this work.</span>
						<span class="text-contrast font-semibold mt-1">
							Notes
							<span class="font-normal text-primary">(optional)</span>
						</span>
						<StyledInput
							type="text"
							resize="both"
							multiline
							class="max-w-[40rem]"
							placeholder="Write something here..."
						/>
					</template>
					<hr class="mt-1 bg-surface-5 border-none h-[1px] w-full" />
					<div class="flex items-center gap-2 justify-end">
						<ButtonStyled type="outlined">
							<button>
								<XIcon />
								Cancel
							</button>
						</ButtonStyled>
						<ButtonStyled color="brand">
							<button>
								<SaveIcon />
								Save
							</button>
						</ButtonStyled>
					</div>
				</div>
			</div>
		</Collapsible>
	</div>
</template>
