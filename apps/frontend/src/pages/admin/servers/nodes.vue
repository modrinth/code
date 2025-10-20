<template>
	<div class="page experimental-styles-within">
		<div
			class="mb-6 flex items-end justify-between border-0 border-b border-solid border-divider pb-4"
		>
			<h1 class="m-0 text-2xl">Server nodes</h1>
			<ButtonStyled color="brand">
				<button @click="openBatchModal"><PlusIcon /> Batch credit</button>
			</ButtonStyled>
		</div>

		<NewModal ref="batchModal">
			<template #title>
				<span class="text-lg font-extrabold text-contrast">Batch credit</span>
			</template>
			<div class="flex w-[720px] max-w-[90vw] flex-col gap-6">
				<div class="flex flex-col gap-2">
					<label class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast"> Type </span>
						<span>Select target to credit.</span>
					</label>
					<TeleportDropdownMenu
						v-model="mode"
						:options="modeOptions"
						:display-name="(x) => x.name"
						name="Type"
						class="max-w-[8rem]"
					/>
				</div>
				<div class="flex flex-col gap-2">
					<label for="days" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast"> Days to credit </span>
					</label>
					<input
						id="days"
						v-model.number="days"
						class="w-32"
						type="number"
						min="1"
						autocomplete="off"
					/>
				</div>

				<div v-if="mode.id === 'nodes'" class="flex flex-col gap-3">
					<div class="flex flex-col gap-2">
						<label for="node-input" class="flex flex-col gap-1">
							<span class="text-lg font-semibold text-contrast"> Node hostnames </span>
						</label>
						<div class="flex items-center gap-2">
							<input
								id="node-input"
								v-model="nodeInput"
								class="w-32"
								type="text"
								autocomplete="off"
							/>
							<ButtonStyled color="blue" color-fill="text">
								<button class="shrink-0" @click="addNode">
									<PlusIcon />
									Add
								</button>
							</ButtonStyled>
						</div>
						<div v-if="selectedNodes.length" class="mt-1 flex flex-wrap gap-2">
							<TagItem v-for="h in selectedNodes" :key="`node-${h}`" :action="() => removeNode(h)">
								<XIcon />
								{{ h }}
							</TagItem>
						</div>
					</div>
				</div>

				<div v-else class="flex flex-col gap-3">
					<div class="flex flex-col gap-2">
						<label for="region-select" class="flex flex-col gap-1">
							<span class="text-lg font-semibold text-contrast"> Region </span>
							<span>This will credit all active servers in the region.</span>
						</label>
						<TeleportDropdownMenu
							id="region-select"
							v-model="selectedRegion"
							:options="regions"
							:display-name="(x) => x.display"
							name="Region"
							class="max-w-[24rem]"
						/>
					</div>
				</div>

				<div class="between flex items-center gap-4">
					<label for="send-email-nodes" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast"> Send email </span>
					</label>
					<Toggle id="send-email-nodes" v-model="sendEmail" />
				</div>

				<div v-if="sendEmail" class="flex flex-col gap-2">
					<label for="message-region" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast"> Customize Email </span>
						<span>
							Unless a particularly bad or out of the ordinary event happened, keep this to the
							default
						</span>
					</label>
					<div class="text-muted rounded-lg border border-divider bg-button-bg p-4">
						<p>Hi {user.name},</p>
						<div class="textarea-wrapper">
							<textarea
								id="message-region"
								v-model="message"
								rows="3"
								class="w-full overflow-hidden"
							/>
						</div>
						<p>
							To make up for it, we've added {{ days }} day{{ pluralize(days) }} to your Modrinth
							Servers subscription.
						</p>
						<p>
							Your next charge was scheduled for {credit.previous_due} and will now be on
							{credit.next_due}.
						</p>
					</div>
				</div>

				<div class="flex gap-2">
					<ButtonStyled color="brand">
						<button :disabled="applyDisabled" @click="apply">
							<CheckIcon aria-hidden="true" />
							Apply credits
						</button>
					</ButtonStyled>
					<ButtonStyled>
						<button @click="batchModal?.hide?.()">
							<XIcon aria-hidden="true" />
							Cancel
						</button>
					</ButtonStyled>
				</div>
			</div>
		</NewModal>
	</div>
</template>

<script setup lang="ts">
import { CheckIcon, PlusIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	injectNotificationManager,
	NewModal,
	TagItem,
	TeleportDropdownMenu,
	Toggle,
} from '@modrinth/ui'
import { DEFAULT_CREDIT_EMAIL_MESSAGE } from '@modrinth/utils/utils.ts'
import { computed, ref } from 'vue'

import { useBaseFetch } from '#imports'
import { useServersFetch } from '~/composables/servers/servers-fetch.ts'

const { addNotification } = injectNotificationManager()

const batchModal = ref<InstanceType<typeof NewModal>>()

const days = ref(1)
const sendEmail = ref(true)
const message = ref('')

const modeOptions = [
	{ id: 'nodes', name: 'Nodes' },
	{ id: 'region', name: 'Region' },
]
const mode = ref(modeOptions[0])

const nodeInput = ref('')
const selectedNodes = ref<string[]>([])

type RegionOpt = { key: string; display: string }
const regions = ref<RegionOpt[]>([])
const selectedRegion = ref<RegionOpt | null>(null)
const nodeHostnames = ref<string[]>([])

function openBatchModal() {
	void ensureOverview()

	message.value = DEFAULT_CREDIT_EMAIL_MESSAGE
	batchModal.value?.show()
}

function addNode() {
	const v = nodeInput.value.trim()
	if (!v) return
	if (!nodeHostnames.value.includes(v)) {
		addNotification({
			title: 'Unknown node',
			text: "This hostname doesn't exist",
			type: 'error',
		})
		return
	}
	if (!selectedNodes.value.includes(v)) selectedNodes.value.push(v)
	nodeInput.value = ''
}

function removeNode(v: string) {
	selectedNodes.value = selectedNodes.value.filter((x) => x !== v)
}

const applyDisabled = computed(() => {
	if (days.value < 1) return true
	if (mode.value.id === 'nodes') return selectedNodes.value.length === 0
	return !selectedRegion.value
})

async function ensureOverview() {
	if (regions.value.length || nodeHostnames.value.length) return
	try {
		const data = await useServersFetch<any>('/nodes/overview', { version: 'internal' })
		regions.value = (data.regions || []).map((r: any) => ({
			key: r.key,
			display: `${r.display_name} (${r.key})`,
		}))
		nodeHostnames.value = data.node_hostnames || []
		if (!selectedRegion.value && regions.value.length) selectedRegion.value = regions.value[0]
	} catch (err) {
		addNotification({ title: 'Failed to load nodes overview', text: String(err), type: 'error' })
	}
}

async function apply() {
	try {
		const body =
			mode.value.id === 'nodes'
				? {
						nodes: selectedNodes.value.slice(),
						days: Math.max(1, Math.floor(days.value)),
						send_email: sendEmail.value,
						message: message.value?.trim() || DEFAULT_CREDIT_EMAIL_MESSAGE,
					}
				: {
						region: selectedRegion.value!.key,
						days: Math.max(1, Math.floor(days.value)),
						send_email: sendEmail.value,
						message: message.value?.trim() || DEFAULT_CREDIT_EMAIL_MESSAGE,
					}
		await useBaseFetch('billing/credit', {
			method: 'POST',
			body: JSON.stringify(body),
			internal: true,
		})
		addNotification({ title: 'Credits applied', type: 'success' })
		batchModal.value?.hide()
		selectedNodes.value = []
		nodeInput.value = ''
		message.value = ''
	} catch (err: any) {
		addNotification({
			title: 'Error applying credits',
			text: err?.data?.description ?? String(err),
			type: 'error',
		})
	}
}

function pluralize(n: number): string {
	return n === 1 ? '' : 's'
}
</script>

<style scoped>
.page {
	padding: 1rem;
	margin-left: auto;
	margin-right: auto;
	max-width: 56rem;
}
</style>
