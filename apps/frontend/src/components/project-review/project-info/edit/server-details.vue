<template>
	<EditModal
		ref="modal"
		:section="formatMessage(reviewMessages.serverDetails)"
		:saving="saving"
		:can-save="canSave"
		width="52rem"
		@cancel="reset"
		@save="save"
	>
		<fieldset :disabled="saving" class="m-0 flex min-w-0 flex-col gap-6 border-0 p-0">
			<div class="flex flex-col gap-2">
				<label for="project-review-server-region" class="font-semibold text-contrast">
					{{ formatMessage(reviewMessages.serverRegion) }}
				</label>
				<Combobox
					id="project-review-server-region"
					v-model="region"
					:options="regionOptions"
					:disabled="saving"
					searchable
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="project-review-server-languages" class="font-semibold text-contrast">
					{{ formatMessage(reviewMessages.serverLanguages) }}
				</label>
				<MultiSelect
					id="project-review-server-languages"
					v-model="languages"
					:options="languageOptions"
					:disabled="saving"
					:max-tag-rows="2"
					searchable
					include-select-all-option
				/>
			</div>
			<div class="flex flex-col gap-2">
				<label for="project-review-java-address" class="font-semibold text-contrast">
					{{ formatMessage(messages.javaAddress) }}
				</label>
				<Input
					id="project-review-java-address"
					v-model="javaAddress"
					:disabled="saving"
					autocomplete="off"
					wrapper-class="w-full"
				/>
				<div v-if="javaAddress.trim()" class="flex items-start gap-1.5 text-sm" role="status">
					<IconButton
						v-tooltip="formatMessage(messages.refreshPing)"
						:label="formatMessage(messages.refreshPing)"
						:disabled="javaPingLoading || saving"
						type="quiet"
						color="orange"
						size="xs"
						class="!size-6 shrink-0"
						@click="pingJavaServer"
					>
						<SpinnerIcon v-if="javaPingLoading" class="animate-spin" />
						<RefreshCwIcon v-else />
					</IconButton>
					<span v-if="javaPingOnline === true" class="mt-0.5 text-green">
						{{ formatMessage(messages.serverOnline) }}
					</span>
					<div v-else-if="javaPingOnline === false" class="mt-0.5 text-orange">
						<IntlFormatted :message-id="messages.pingFailed">
							<template #support-link="{ children }">
								<a
									href="https://support.modrinth.com"
									target="_blank"
									rel="noopener noreferrer"
									class="inline underline"
								>
									<component :is="() => normalizeChildren(children)" />
								</a>
							</template>
						</IntlFormatted>
					</div>
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<label for="project-review-bedrock-address" class="font-semibold text-contrast">
					{{ formatMessage(messages.bedrockAddress) }}
				</label>
				<Input
					id="project-review-bedrock-address"
					v-model="bedrockAddress"
					:disabled="saving"
					autocomplete="off"
					wrapper-class="w-full"
				/>
			</div>
			<CompatibilityCard
				v-if="opened"
				:project="project"
				:save-project="saveCompatibility"
				can-edit
				@busy="compatibilityBusy = $event"
			/>
		</fieldset>
	</EditModal>
</template>

<script setup lang="ts">
import type { Labrinth } from '@modrinth/api-client'
import { RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Combobox,
	defineMessages,
	IconButton,
	Input,
	IntlFormatted,
	MultiSelect,
	normalizeChildren,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref, useTemplateRef } from 'vue'

import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'
import { useJavaServerPing } from '~/providers/manage-server-compatibility-modal'

import { projectReviewMessages as reviewMessages } from '../../messages'
import EditModal from './edit-modal.vue'
import { useProjectInfoEdit } from './use-project-edit'

const messages = defineMessages({
	javaAddress: {
		id: 'project.settings.server.java-address-label',
		defaultMessage: 'Java address',
	},
	bedrockAddress: {
		id: 'project.settings.server.bedrock-address-label',
		defaultMessage: 'Bedrock address',
	},
	refreshPing: {
		id: 'project.settings.server.refresh-ping-tooltip',
		defaultMessage: 'Refresh ping',
	},
	serverOnline: {
		id: 'project.settings.server.server-online',
		defaultMessage: 'Server is online!',
	},
	pingFailed: {
		id: 'project.settings.server.ping-failed-message',
		defaultMessage:
			"We couldn't ping this server. It may be blocked by your host so try refreshing a few times. If it still doesn't respond please <support-link>contact support</support-link>.",
	},
})

const { formatMessage, locale } = useVIntl()
const { project, saving: projectSaving, beginEditing, saveProject } = useProjectInfoEdit()
const modal = useTemplateRef<InstanceType<typeof EditModal>>('modal')
const compatibilityBusy = ref(false)
const saving = computed(() => projectSaving.value || compatibilityBusy.value)
const opened = ref(false)
const region = ref('')
const languages = ref<string[]>([])
const javaAddress = ref('')
const bedrockAddress = ref('')
const {
	loading: javaPingLoading,
	result: javaPingResult,
	ping: pingJavaServer,
} = useJavaServerPing(javaAddress, opened)
const javaPingOnline = computed(() => javaPingResult.value?.online ?? null)

async function saveCompatibility(patch: Labrinth.Projects.v3.EditProjectRequest) {
	if (!(await saveProject(patch))) throw new Error('Failed to save server compatibility')
	return true
}

const regionOptions = computed(() =>
	Object.entries(SERVER_REGIONS)
		.map(([value, message]) => ({ value, label: formatMessage(message) }))
		.sort((a, b) => a.label.localeCompare(b.label, locale.value)),
)
const languageOptions = computed(() =>
	Object.entries(SERVER_LANGUAGES)
		.map(([value, message]) => ({ value, label: formatMessage(message) }))
		.sort((a, b) => a.label.localeCompare(b.label, locale.value)),
)

function reset() {
	opened.value = false
	compatibilityBusy.value = false
	region.value = project.value?.minecraft_server?.region ?? ''
	languages.value = [...(project.value?.minecraft_server?.languages ?? [])]
	javaAddress.value = project.value?.minecraft_java_server?.address ?? ''
	bedrockAddress.value = project.value?.minecraft_bedrock_server?.address ?? ''
}

function show() {
	beginEditing()
	reset()
	opened.value = true
	modal.value?.show()
	void pingJavaServer()
}

const patchData = computed<Labrinth.Projects.v3.EditProjectRequest>(() => {
	const patch: Labrinth.Projects.v3.EditProjectRequest = {}
	const original = project.value
	if (!original) return patch
	const languagesChanged =
		JSON.stringify([...languages.value].sort()) !==
		JSON.stringify([...(original.minecraft_server?.languages ?? [])].sort())
	if (region.value !== (original.minecraft_server?.region ?? '') || languagesChanged) {
		patch.minecraft_server = {
			...(region.value ? { region: region.value } : {}),
			languages: [...languages.value],
		}
	}
	if (javaAddress.value.trim() !== (original.minecraft_java_server?.address ?? '')) {
		patch.minecraft_java_server = { address: javaAddress.value.trim() }
	}
	if (bedrockAddress.value.trim() !== (original.minecraft_bedrock_server?.address ?? '')) {
		patch.minecraft_bedrock_server = { address: bedrockAddress.value.trim() }
	}
	return patch
})
const canSave = computed(() => !saving.value && Object.keys(patchData.value).length > 0)

async function save() {
	if (!canSave.value) return
	if (await saveProject(patchData.value)) modal.value?.hide()
}

defineExpose({ show })
</script>
