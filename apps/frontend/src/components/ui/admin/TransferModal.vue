<template>
	<NewModal ref="modal">
		<template #title>
			<span class="text-lg font-extrabold text-contrast">Schedule transfer</span>
		</template>
		<div class="flex w-[550px] max-w-[90vw] flex-col gap-6">
			<div class="flex flex-col gap-2">
				<label class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast"> Type </span>
					<span>Select transfer type.</span>
				</label>
				<Combobox
					v-model="mode"
					:options="modeOptions"
					placeholder="Select type"
					class="max-w-[10rem]"
				/>
			</div>

			<div v-if="mode === 'servers'" class="flex flex-col gap-2">
				<label for="server-ids" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Server IDs
						<span class="text-brand-red">*</span>
					</span>
					<span>Server IDs (one per line or comma-separated.)</span>
				</label>
				<div class="textarea-wrapper">
					<textarea
						id="server-ids"
						v-model="serverIdsInput"
						rows="4"
						class="w-full bg-surface-3"
						placeholder="123e4569-e89b-12d3-a456-426614174005&#10;123e9569-e89b-12d3-a456-413678919876"
					/>
				</div>
				<span v-if="parsedServerIds.length" class="text-sm text-secondary">
					{{ parsedServerIds.length }} server{{ parsedServerIds.length === 1 ? '' : 's' }} selected
				</span>
			</div>

			<div v-else class="flex flex-col gap-4">
				<div class="flex flex-col gap-2">
					<label for="node-input" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast">
							Node hostnames
							<span class="text-brand-red">*</span>
						</span>
						<span>Add nodes to transfer (comma or space-separated).</span>
					</label>
					<div class="flex items-center gap-2">
						<input
							id="node-input"
							v-model="nodeInput"
							class="w-64"
							type="text"
							autocomplete="off"
							placeholder="us-vin200, us-vin201"
							@keydown.enter.prevent="addNodes"
						/>
						<ButtonStyled color="blue" color-fill="text">
							<button class="shrink-0" @click="addNodes">
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
				<div class="flex flex-col gap-3">
					<label for="cordon-nodes" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast">Cordon nodes now</span>
						<span>
							Prevent new servers from being provisioned on the transferred nodes from now on.<br /><br />
							Note that if this option isn't chosen, new servers provisioned onto transferred nodes
							between now and the scheduled time will still be transferred.
						</span>
					</label>
					<Toggle id="cordon-nodes" v-model="cordonNodes" />
				</div>
				<div class="flex flex-col gap-2">
					<label for="tag-nodes" class="flex flex-col gap-1">
						<span class="text-lg font-semibold text-contrast">Tag transferred nodes</span>
						<span>Optional tag to add to the transferred nodes.</span>
					</label>
					<input
						id="tag-nodes"
						v-model="tagNodes"
						class="max-w-[12rem]"
						type="text"
						autocomplete="off"
					/>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<label for="region-select" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast"> Target region </span>
					<span>Select the destination region for transferred servers.</span>
				</label>
				<Combobox
					v-model="selectedRegion"
					:options="regions"
					placeholder="Select region"
					class="max-w-[24rem]"
				/>
			</div>

			<div class="flex flex-col gap-2">
				<label for="tag-input" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast"> Node tags </span>
					<span>Optional preferred node tags for node selection.</span>
				</label>
				<div class="flex items-center gap-2">
					<input
						id="tag-input"
						v-model="tagInput"
						class="w-40"
						type="text"
						autocomplete="off"
						placeholder="ovh-gen4"
						@keydown.enter.prevent="addTag"
					/>
					<ButtonStyled color="blue" color-fill="text">
						<button class="shrink-0" @click="addTag">
							<PlusIcon />
							Add
						</button>
					</ButtonStyled>
				</div>
				<div v-if="selectedTags.length" class="mt-1 flex flex-wrap gap-2">
					<TagItem v-for="t in selectedTags" :key="`tag-${t}`" :action="() => removeTag(t)">
						<XIcon />
						{{ t }}
					</TagItem>
				</div>
			</div>

			<div class="flex flex-col gap-2">
				<label class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast"> Schedule </span>
				</label>
				<Chips
					v-model="scheduleOption"
					:items="scheduleOptions"
					:format-label="(item) => scheduleOptionLabels[item]"
					:capitalize="false"
				/>
				<input
					v-if="scheduleOption === 'later'"
					v-model="scheduledDate"
					type="datetime-local"
					class="mt-2 max-w-[16rem]"
					autocomplete="off"
				/>
			</div>

			<div class="flex flex-col gap-2">
				<label for="reason" class="flex flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						Reason
						<span class="text-brand-red">*</span>
					</span>
					<span>Provide a reason for this transfer batch.</span>
				</label>
				<div class="textarea-wrapper">
					<textarea
						id="reason"
						v-model="reason"
						rows="2"
						class="w-full bg-surface-3"
						placeholder="Node maintenance scheduled"
					/>
				</div>
			</div>

			<div class="flex gap-2">
				<ButtonStyled color="brand">
					<button :disabled="submitDisabled || submitting" @click="submit">
						<SendIcon aria-hidden="true" />
						{{ submitting ? 'Scheduling...' : 'Schedule transfer' }}
					</button>
				</ButtonStyled>
				<ButtonStyled>
					<button @click="modal?.hide?.()">
						<XIcon aria-hidden="true" />
						Cancel
					</button>
				</ButtonStyled>
			</div>
		</div>
	</NewModal>
</template>

<script setup lang="ts">
import { PlusIcon, SendIcon, XIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Chips,
	Combobox,
	injectNotificationManager,
	NewModal,
	TagItem,
	Toggle,
} from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, ref } from 'vue'

import { useServersFetch } from '~/composables/servers/servers-fetch.ts'

const emit = defineEmits<{
	success: []
}>()

const { addNotification } = injectNotificationManager()

const modal = ref<InstanceType<typeof NewModal>>()

const modeOptions = [
	{ value: 'servers', label: 'Servers' },
	{ value: 'nodes', label: 'Nodes' },
]
const mode = ref<string>('servers')

const serverIdsInput = ref('')
const parsedServerIds = computed(() => {
	const input = serverIdsInput.value.trim()
	if (!input) return []
	return input
		.split(/[\n,\s]+/)
		.map((s) => s.trim())
		.filter((s) => s.length > 0)
})

const nodeInput = ref('')
const selectedNodes = ref<string[]>([])
const cordonNodes = ref(true)
const tagNodes = ref('')

type RegionOpt = { value: string; label: string }
const regions = ref<RegionOpt[]>([])
const selectedRegion = ref<string | null>(null)
const nodeHostnames = ref<string[]>([])

const tagInput = ref('')
const selectedTags = ref<string[]>([])

const scheduleOptions: ('now' | 'later')[] = ['now', 'later']
const scheduleOptionLabels: Record<string, string> = {
	now: 'Now',
	later: 'Schedule for later',
}
const scheduleOption = ref<'now' | 'later'>('now')
const scheduledDate = ref<string>('')

const reason = ref('')

const submitting = ref(false)

function show(event?: MouseEvent) {
	void ensureOverview()
	mode.value = 'servers'
	serverIdsInput.value = ''
	selectedNodes.value = []
	cordonNodes.value = true
	tagNodes.value = `migration${dayjs().format('YYYYMMDD')}`
	selectedTags.value = []
	tagInput.value = ''
	nodeInput.value = ''
	scheduleOption.value = 'now'
	scheduledDate.value = ''
	reason.value = ''
	modal.value?.show(event)
}

function hide() {
	modal.value?.hide()
}

function addNodes() {
	const input = nodeInput.value.trim()
	if (!input) return

	const nodes = input
		.split(/[,\s]+/)
		.map((s) => s.trim())
		.filter((s) => s.length > 0)

	const unknownNodes: string[] = []
	const addedNodes: string[] = []

	for (const v of nodes) {
		if (!nodeHostnames.value.includes(v)) {
			unknownNodes.push(v)
			continue
		}
		if (!selectedNodes.value.includes(v)) {
			selectedNodes.value.push(v)
			addedNodes.push(v)
		}
	}

	if (unknownNodes.length > 0) {
		addNotification({
			title: `Unknown node${unknownNodes.length > 1 ? 's' : ''}`,
			text: unknownNodes.join(', '),
			type: 'error',
		})
	}

	nodeInput.value = ''
}

function removeNode(v: string) {
	selectedNodes.value = selectedNodes.value.filter((x) => x !== v)
}

function addTag() {
	const v = tagInput.value.trim()
	if (!v) return
	if (!selectedTags.value.includes(v)) selectedTags.value.push(v)
	tagInput.value = ''
}

function removeTag(v: string) {
	selectedTags.value = selectedTags.value.filter((x) => x !== v)
}

const submitDisabled = computed(() => {
	if (!reason.value.trim()) return true
	if (mode.value === 'servers') {
		if (parsedServerIds.value.length === 0) return true
	} else {
		if (selectedNodes.value.length === 0) return true
	}
	if (scheduleOption.value === 'later' && !scheduledDate.value) return true
	return false
})

async function ensureOverview() {
	if (regions.value.length || nodeHostnames.value.length) return
	try {
		const data = await useServersFetch<any>('/nodes/overview', { version: 'internal' })
		regions.value = (data.regions || []).map((r: any) => ({
			value: r.key,
			label: `${r.display_name} (${r.key})`,
		}))
		nodeHostnames.value = data.node_hostnames || []
		if (!selectedRegion.value && regions.value.length) {
			selectedRegion.value = regions.value[0].value
		}
	} catch (err) {
		addNotification({ title: 'Failed to load nodes overview', text: String(err), type: 'error' })
	}
}

async function submit() {
	if (submitDisabled.value || submitting.value) return

	submitting.value = true
	try {
		const scheduledAt =
			scheduleOption.value === 'now' ? undefined : dayjs(scheduledDate.value).toISOString()

		if (mode.value === 'servers') {
			await useServersFetch('/transfers/schedule/servers', {
				version: 'internal',
				method: 'POST',
				body: {
					server_ids: parsedServerIds.value,
					scheduled_at: scheduledAt,
					target_region: selectedRegion.value || undefined,
					node_tags: selectedTags.value.length > 0 ? selectedTags.value : undefined,
					reason: reason.value.trim(),
				},
			})
		} else {
			await useServersFetch('/transfers/schedule/nodes', {
				version: 'internal',
				method: 'POST',
				body: {
					node_hostnames: selectedNodes.value.slice(),
					scheduled_at: scheduledAt,
					target_region: selectedRegion.value || undefined,
					node_tags: selectedTags.value.length > 0 ? selectedTags.value : undefined,
					reason: reason.value.trim(),
					cordon_nodes: cordonNodes.value,
					tag_nodes: tagNodes.value.trim() || undefined,
				},
			})
		}

		addNotification({ title: 'Transfer scheduled', type: 'success' })
		emit('success')
		modal.value?.hide()
	} catch (err: any) {
		addNotification({
			title: 'Error scheduling transfer',
			text: err?.data?.description ?? err?.message ?? String(err),
			type: 'error',
		})
	} finally {
		submitting.value = false
	}
}

defineExpose({
	show,
	hide,
})
</script>
