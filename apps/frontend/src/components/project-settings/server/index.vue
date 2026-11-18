<template>
	<div data-project-field="server" data-project-field-scroll-only>
		<ProjectIssueCard location="server" class="mb-4" />
		<ConfirmLeaveModal ref="confirmLeaveModal" />
		<h2 class="settings-section-title">{{ formatMessage(messages.serverDetailsHeading) }}</h2>
		<section class="universal-card settings-content-card flex flex-col gap-8">
			<div class="flex flex-col gap-6">
				<!-- Region -->
				<div data-project-field="server-region" class="max-w-[600px]">
					<label for="server-region" class="w-fit">
						<span class="label__title">{{ formatMessage(messages.regionLabel) }}</span>
					</label>
					<Combobox
						id="server-region"
						v-model="region"
						:options="regionOptions"
						searchable
						:placeholder="formatMessage(messages.selectRegionPlaceholder)"
						:disabled="!hasPermission"
					/>
					<ValidationMessage
						:check="regionValidation"
						:project-field="projectV3?.minecraft_server?.region ?? ''"
						:current-field="region"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('server-region')" class="mt-2" />
				</div>

				<!-- Language -->
				<div data-project-field="server-languages" class="max-w-[600px]">
					<label for="server-language" class="block w-fit">
						<span class="label__title"
							>{{ formatMessage(messages.languagesLabel) }}
							<span class="font-normal text-secondary"
								>({{ formatMessage(messages.optionalLabel) }})</span
							></span
						>
					</label>
					<MultiSelect
						id="server-language"
						v-model="languages"
						:options="languageOptions"
						searchable
						include-select-all-option
						:max-tag-rows="2"
						:placeholder="formatMessage(messages.selectLanguagesPlaceholder)"
						:disabled="!hasPermission"
					/>
					<ValidationMessage
						:check="languageValidation"
						:project-field="
							JSON.stringify([...(projectV3?.minecraft_server?.languages ?? [])].sort())
						"
						:current-field="JSON.stringify([...languages].sort())"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('server-languages')" class="mt-2" />
					<ProjectIssueCard
						target="modify_server_languages"
						:field-action="languagesIssueAction"
						class="mt-2"
					/>
				</div>

				<!-- Java Address -->
				<div data-project-field="java-address" class="max-w-[600px]">
					<div class="flex items-center justify-between">
						<label for="java-address" class="block w-fit">
							<span class="label__title !m-0 !text-contrast">{{
								formatMessage(messages.javaAddressLabel)
							}}</span>
						</label>
					</div>
					<div
						class="mt-2 flex items-center gap-2 text-sm"
						@focusout="
							() => {
								if (!lastPingAddressChanged && javaPingResult) return
								pingJavaServer()
							}
						"
					>
						<Input
							id="java-address"
							v-model="javaAddress"
							:placeholder="formatMessage(messages.enterAddressPlaceholder)"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
					</div>
					<JavaPingStatus
						:java-address="javaAddress"
						:java-ping-loading="javaPingLoading"
						:java-ping-result="javaPingResult"
						@refresh="pingJavaServer"
					/>
					<ValidationMessage
						:check="javaAddressValidation"
						:project-field="projectV3?.minecraft_java_server?.address ?? ''"
						:current-field="javaAddress.trim()"
						class="mt-2"
					/>
					<ValidationMessage :check="saveValidation.forField('java-address')" class="mt-2" />
					<ProjectIssueCard
						target="modify_server_address"
						platform="minecraft_java"
						:field-action="javaAddressIssueAction"
						class="mt-2"
					/>
				</div>

				<!-- Bedrock Address -->
				<div class="max-w-[600px]">
					<label for="bedrock-address" class="block w-fit">
						<span class="label__title !text-contrast"
							>{{ formatMessage(messages.bedrockAddressLabel) }}
							<span class="font-normal text-secondary"
								>({{ formatMessage(messages.optionalLabel) }})</span
							>
						</span>
					</label>
					<div class="mt-2 flex items-center gap-2">
						<Input
							id="bedrock-address"
							v-model="bedrockAddress"
							:placeholder="formatMessage(messages.enterAddressPlaceholder)"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
					</div>
					<ProjectIssueCard
						target="modify_server_address"
						platform="minecraft_bedrock"
						:field-action="bedrockAddressIssueAction"
						class="mt-2"
					/>
				</div>
			</div>
			<div data-project-field="server-compatibility">
				<CompatibilityCard
					:project="projectV3"
					:save-project="patchProjectV3"
					:can-edit="hasPermission"
				/>
				<ValidationMessage :check="compatibilityValidation" class="mt-2" />
				<ValidationMessage :check="saveValidation.forField('server-compatibility')" class="mt-2" />
			</div>
		</section>

		<ValidationMessage
			:check="
				saveValidation.withoutFields([
					'server-region',
					'server-languages',
					'java-address',
					'server-compatibility',
				])
			"
			class="my-4"
		/>
		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			:can-save="!saveValidation.hasErrors.value"
			@reset="resetChanges"
			@save="handleSave"
		/>
	</div>
</template>

<script setup>
import {
	Combobox,
	ConfirmLeaveModal,
	injectNotificationManager,
	injectProjectPageContext,
	Input,
	MultiSelect,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	UnsavedChangesPopup,
	usePageLeaveSafety,
	useVIntl,
} from '@modrinth/ui'
import { isAdmin } from '@modrinth/utils'

import ProjectIssueCard from '~/components/ui/project-issue-card/index.vue'
import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'
import ValidationMessage from '~/components/ValidationMessage.vue'
import { useProjectIssueFieldAction } from '~/composables/project-issue-field-action'
import { useProjectNagMessages } from '~/composables/project-nag-validation'
import { useProjectSaveValidation } from '~/composables/project-save-validation'
import { useJavaServerPing } from '~/providers/manage-server-compatibility-modal'

import JavaPingStatus from './java-ping-status.vue'
import { messages } from './messages'

const { formatMessage, locale } = useVIntl()

const { addNotification } = injectNotificationManager()
const { projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()

const regionValidation = useProjectNagMessages('server-region')
const languageValidation = useProjectNagMessages('server-languages')
const javaAddressValidation = useProjectNagMessages('java-address')
const compatibilityValidation = useProjectNagMessages('server-compatibility')

const javaAddress = ref('')
const bedrockAddress = ref('')
const bedrockPort = ref(19132)
const region = ref('')
const languages = ref([])

const {
	loading: javaPingLoading,
	result: javaPingResult,
	addressChanged: lastPingAddressChanged,
	ping: pingJavaServer,
} = useJavaServerPing(javaAddress)

const isAdminUser = computed(() => isAdmin(currentMember.value?.user))
const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return (
		isAdminUser.value || ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
	)
})

function initFromProjectV3(v3) {
	if (!v3) return
	javaAddress.value = v3.minecraft_java_server?.address ?? ''
	bedrockAddress.value = v3.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = v3.minecraft_bedrock_server?.port ?? 19132
	region.value = v3.minecraft_server?.region ?? ''
	languages.value = v3.minecraft_server?.languages ?? []

	pingJavaServer()
}

// initialize projectV3 values once
if (projectV3.value) {
	initFromProjectV3(projectV3.value)
} else {
	const stop = watch(
		() => projectV3.value,
		(v3) => {
			if (!v3) return
			initFromProjectV3(v3)
			stop()
		},
	)
}

const regionOptions = computed(() =>
	Object.entries(SERVER_REGIONS)
		.sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})
		.map(([code, name]) => ({
			value: code,
			label: formatMessage(name),
		})),
)

const languageOptions = computed(() =>
	Object.entries(SERVER_LANGUAGES)
		.sort(([_, a], [__, b]) => {
			const aFormatted = formatMessage(a)
			const bFormatted = formatMessage(b)
			return aFormatted.localeCompare(bFormatted, locale.value)
		})
		.map(([code, name]) => ({
			value: code,
			label: formatMessage(name),
		})),
)

const javaServerPatchData = computed(() => {
	const addressChanged =
		javaAddress.value !== (projectV3.value?.minecraft_java_server?.address ?? '')
	if (addressChanged) {
		return {
			address: javaAddress.value,
		}
	}

	return {}
})

const bedrockServerPatchData = computed(() => {
	const origBedrock = projectV3.value?.minecraft_bedrock_server
	if (bedrockAddress.value !== (origBedrock?.address ?? '')) {
		return {
			address: bedrockAddress.value,
		}
	}

	return {}
})

const serverPatchData = computed(() => {
	const origServer = projectV3.value?.minecraft_server
	const regionChanged = region.value && region.value !== origServer?.region
	const languagesChanged =
		JSON.stringify([...languages.value].sort()) !==
		JSON.stringify([...(origServer?.languages ?? [])].sort())

	if (regionChanged || languagesChanged) {
		return {
			...origServer,
			...(regionChanged ? { region: region.value } : {}),
			...(languagesChanged ? { languages: languages.value } : {}),
		}
	}

	return {}
})

const v3PatchData = computed(() => {
	const data = {}
	if (Object.keys(serverPatchData.value).length > 0) {
		data.minecraft_server = serverPatchData.value
	}
	if (Object.keys(javaServerPatchData.value).length > 0) {
		data.minecraft_java_server = javaServerPatchData.value
	}
	if (Object.keys(bedrockServerPatchData.value).length > 0) {
		data.minecraft_bedrock_server = bedrockServerPatchData.value
	}
	return data
})

const saving = ref(false)

const original = computed(() => ({
	javaAddress: projectV3.value?.minecraft_java_server?.address ?? '',
	bedrockAddress: projectV3.value?.minecraft_bedrock_server?.address ?? '',
	bedrockPort: projectV3.value?.minecraft_bedrock_server?.port ?? 19132,
	region: projectV3.value?.minecraft_server?.region ?? '',
	languages: projectV3.value?.minecraft_server?.languages ?? [],
}))

const modified = computed(() => ({
	javaAddress: javaAddress.value,
	bedrockAddress: bedrockAddress.value,
	bedrockPort: bedrockPort.value,
	region: region.value,
	languages: languages.value,
}))

const hasChanges = computed(() =>
	Object.keys(original.value).some((key) => {
		const a = original.value[key]
		const b = modified.value[key]
		if (Array.isArray(a) && Array.isArray(b)) {
			return a.length !== b.length || a.some((v, i) => v !== b[i])
		}
		return a !== b
	}),
)

const { confirmLeaveModal } = usePageLeaveSafety(hasChanges)

const saveValidation = useProjectSaveValidation(() => modified.value)

function useServerIssueAction(field, value, validationField, canSave = () => true) {
	return useProjectIssueFieldAction({
		draft: () => ({ [field]: { ...projectV3.value?.[field], ...value() } }),
		canSave: () =>
			hasPermission.value &&
			canSave() &&
			!saveValidation.forField(validationField).some((message) => message.severity === 'error'),
		saving,
		validation: saveValidation,
		save: () => patchProjectV3({ [field]: value() }, true, true),
	})
}

const languagesIssueAction = useServerIssueAction(
	'minecraft_server',
	() => ({ languages: [...languages.value] }),
	'server-languages',
)
const javaAddressIssueAction = useServerIssueAction(
	'minecraft_java_server',
	() => ({ address: javaAddress.value.trim() }),
	'java-address',
	() =>
		isAdminUser.value ||
		!javaAddress.value.trim() ||
		(!javaPingLoading.value && !lastPingAddressChanged.value && !!javaPingResult.value?.online),
)
const bedrockAddressIssueAction = useServerIssueAction(
	'minecraft_bedrock_server',
	() => ({ address: bedrockAddress.value.trim() }),
	'bedrock-address',
)

function resetChanges() {
	saveValidation.clear()
	javaAddress.value = projectV3.value?.minecraft_java_server?.address ?? ''
	bedrockAddress.value = projectV3.value?.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = projectV3.value?.minecraft_bedrock_server?.port ?? 19132
	region.value = projectV3.value?.minecraft_server?.region ?? ''
	languages.value = projectV3.value?.minecraft_server?.languages ?? []
}

async function handleSave() {
	if (saving.value || saveValidation.hasErrors.value) return
	if (!isAdminUser.value && javaAddress.value.trim() && !javaPingResult.value?.online) {
		addNotification({
			title: formatMessage(messages.cannotSaveTitle),
			text: formatMessage(messages.cannotSaveText),
			type: 'error',
		})
		return
	}

	javaAddress.value = javaAddress.value.trim()
	bedrockAddress.value = bedrockAddress.value.trim()
	const submittedState = saveValidation.snapshot()
	saving.value = true
	try {
		const hasV3Changes = Object.keys(v3PatchData.value).length > 0
		if (hasV3Changes) {
			await patchProjectV3(v3PatchData.value, false, true)
			saveValidation.clear()
		}
	} catch (error) {
		if (!saveValidation.capture(error, submittedState)) throw error
	} finally {
		saving.value = false
	}
}
</script>
