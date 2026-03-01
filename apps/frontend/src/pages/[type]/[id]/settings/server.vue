<template>
	<div>
		<section class="universal-card">
			<div class="flex flex-col gap-6">
				<div class="text-2xl font-semibold text-contrast">Server details</div>

				<!-- Country -->
				<div class="max-w-[600px]">
					<label for="server-country">
						<span class="label__title">Country</span>
					</label>
					<Combobox
						id="server-country"
						v-model="country"
						:options="countryOptions"
						searchable
						placeholder="Select country"
						:disabled="!hasPermission"
					/>
				</div>

				<!-- Language -->
				<div class="max-w-[600px]">
					<label for="server-language">
						<span class="label__title"
							>Languages <span class="font-normal text-secondary">(optional)</span></span
						>
					</label>
					<Multiselect
						id="server-language"
						v-model="languages"
						:options="languageOptions.map((l) => l.value)"
						:custom-label="(code) => languageOptions.find((l) => l.value === code)?.label ?? code"
						:multiple="true"
						:searchable="true"
						:show-labels="false"
						:close-on-select="false"
						placeholder="Select languages"
						:disabled="!hasPermission"
					/>
				</div>

				<!-- Java Address -->
				<div class="max-w-[600px]">
					<label for="java-address">
						<span class="label__title !text-contrast">Java address</span>
					</label>
					<div class="mt-2 flex items-center gap-2" @focusout="pingJavaServer">
						<StyledInput
							id="java-address"
							v-model="javaAddress"
							placeholder="Enter address"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
						<StyledInput
							v-model="javaPort"
							type="number"
							:min="1"
							:max="65535"
							:disabled="!hasPermission"
							wrapper-class="w-24"
							input-class="text-center"
							autocomplete="off"
						/>
					</div>
					<div
						v-if="javaPingResult !== null"
						class="mt-2 flex items-center gap-1.5"
						:class="javaPingResult.online ? 'text-green' : 'text-orange'"
					>
						<CheckIcon v-if="javaPingResult.online" class="h-4 w-4" />
						<TriangleAlertIcon v-else class="h-4 w-4" />
						{{
							javaPingResult.online
								? `Server is online! ${javaPingResult.latency ? `Latency: ${javaPingResult.latency}ms` : ``}`
								: 'Cannot ping server'
						}}
					</div>
				</div>

				<!-- Bedrock Address -->
				<div class="max-w-[600px]">
					<label for="bedrock-address">
						<span class="label__title !text-contrast"
							>Bedrock address
							<span class="font-normal text-secondary">(optional)</span>
						</span>
					</label>
					<div class="mt-2 flex items-center gap-2">
						<StyledInput
							id="bedrock-address"
							v-model="bedrockAddress"
							placeholder="Enter address"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
						<StyledInput
							v-model="bedrockPort"
							type="number"
							:min="1"
							:max="65535"
							:disabled="!hasPermission"
							wrapper-class="w-24"
							input-class="text-center"
							autocomplete="off"
						/>
					</div>
				</div>

				<CompatibilityCard />
			</div>
		</section>

		<UnsavedChangesPopup
			:original="original"
			:modified="modified"
			:saving="saving"
			@reset="resetChanges"
			@save="handleSave"
		/>
	</div>
</template>

<script setup>
import { CheckIcon, TriangleAlertIcon } from '@modrinth/assets'
import {
	Combobox,
	injectNotificationManager,
	injectProjectPageContext,
	StyledInput,
	UnsavedChangesPopup,
} from '@modrinth/ui'
import { Multiselect } from 'vue-multiselect'

import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'

const { addNotification } = injectNotificationManager()
const { projectV2: project, projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()

const javaAddress = ref('')
const javaPort = ref(25565)
const bedrockAddress = ref('')
const bedrockPort = ref(19132)
const country = ref('')
const languages = ref([])

function initFromProjectV3(v3) {
	if (!v3) return
	javaAddress.value = v3.minecraft_java_server?.address ?? ''
	javaPort.value = v3.minecraft_java_server?.port ?? 25565
	bedrockAddress.value = v3.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = v3.minecraft_bedrock_server?.port ?? 19132
	country.value = v3.minecraft_server?.country ?? ''
	languages.value = v3.minecraft_server?.languages ?? []
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

const javaPingLoading = ref(false)
const javaPingResult = ref(null)

const pingJavaServer = async () => {
	const address = javaAddress.value?.trim()
	if (!address) {
		javaPingResult.value = null
		return
	}

	javaPingLoading.value = true
	javaPingResult.value = null

	const port = javaPort.value || 25565
	const query = port !== 25565 ? `${address}:${port}` : address

	try {
		// TODO replace with api-client labrinth server ping route
	} catch {
		javaPingResult.value = { online: false, latency: null }
	} finally {
		javaPingLoading.value = false
	}
}

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
})

const countryOptions = [
	{ value: 'US', label: 'United States' },
	{ value: 'CA', label: 'Canada' },
	{ value: 'GB', label: 'United Kingdom' },
	{ value: 'DE', label: 'Germany' },
	{ value: 'FR', label: 'France' },
	{ value: 'NL', label: 'Netherlands' },
	{ value: 'FI', label: 'Finland' },
	{ value: 'SE', label: 'Sweden' },
	{ value: 'NO', label: 'Norway' },
	{ value: 'DK', label: 'Denmark' },
	{ value: 'PL', label: 'Poland' },
	{ value: 'CZ', label: 'Czech Republic' },
	{ value: 'RO', label: 'Romania' },
	{ value: 'CH', label: 'Switzerland' },
	{ value: 'AT', label: 'Austria' },
	{ value: 'BE', label: 'Belgium' },
	{ value: 'IE', label: 'Ireland' },
	{ value: 'ES', label: 'Spain' },
	{ value: 'IT', label: 'Italy' },
	{ value: 'PT', label: 'Portugal' },
	{ value: 'RU', label: 'Russia' },
	{ value: 'UA', label: 'Ukraine' },
	{ value: 'LT', label: 'Lithuania' },
	{ value: 'LV', label: 'Latvia' },
	{ value: 'EE', label: 'Estonia' },
	{ value: 'BG', label: 'Bulgaria' },
	{ value: 'HR', label: 'Croatia' },
	{ value: 'HU', label: 'Hungary' },
	{ value: 'SK', label: 'Slovakia' },
	{ value: 'RS', label: 'Serbia' },
	{ value: 'GR', label: 'Greece' },
	{ value: 'TR', label: 'Turkey' },
	{ value: 'IL', label: 'Israel' },
	{ value: 'AE', label: 'United Arab Emirates' },
	{ value: 'SA', label: 'Saudi Arabia' },
	{ value: 'IN', label: 'India' },
	{ value: 'SG', label: 'Singapore' },
	{ value: 'JP', label: 'Japan' },
	{ value: 'KR', label: 'South Korea' },
	{ value: 'CN', label: 'China' },
	{ value: 'HK', label: 'Hong Kong' },
	{ value: 'TW', label: 'Taiwan' },
	{ value: 'AU', label: 'Australia' },
	{ value: 'NZ', label: 'New Zealand' },
	{ value: 'BR', label: 'Brazil' },
	{ value: 'AR', label: 'Argentina' },
	{ value: 'CL', label: 'Chile' },
	{ value: 'CO', label: 'Colombia' },
	{ value: 'MX', label: 'Mexico' },
	{ value: 'ZA', label: 'South Africa' },
	{ value: 'NG', label: 'Nigeria' },
	{ value: 'KE', label: 'Kenya' },
	{ value: 'EG', label: 'Egypt' },
	{ value: 'MY', label: 'Malaysia' },
	{ value: 'TH', label: 'Thailand' },
	{ value: 'VN', label: 'Vietnam' },
	{ value: 'PH', label: 'Philippines' },
	{ value: 'ID', label: 'Indonesia' },
	{ value: 'PK', label: 'Pakistan' },
	{ value: 'BD', label: 'Bangladesh' },
]

const languageOptions = [
	{ value: 'en', label: 'English' },
	{ value: 'es', label: 'Spanish' },
	{ value: 'pt', label: 'Portuguese' },
	{ value: 'fr', label: 'French' },
	{ value: 'de', label: 'German' },
	{ value: 'it', label: 'Italian' },
	{ value: 'nl', label: 'Dutch' },
	{ value: 'ru', label: 'Russian' },
	{ value: 'uk', label: 'Ukrainian' },
	{ value: 'pl', label: 'Polish' },
	{ value: 'cs', label: 'Czech' },
	{ value: 'sk', label: 'Slovak' },
	{ value: 'hu', label: 'Hungarian' },
	{ value: 'ro', label: 'Romanian' },
	{ value: 'bg', label: 'Bulgarian' },
	{ value: 'hr', label: 'Croatian' },
	{ value: 'sr', label: 'Serbian' },
	{ value: 'el', label: 'Greek' },
	{ value: 'tr', label: 'Turkish' },
	{ value: 'ar', label: 'Arabic' },
	{ value: 'he', label: 'Hebrew' },
	{ value: 'hi', label: 'Hindi' },
	{ value: 'bn', label: 'Bengali' },
	{ value: 'ur', label: 'Urdu' },
	{ value: 'zh', label: 'Chinese' },
	{ value: 'ja', label: 'Japanese' },
	{ value: 'ko', label: 'Korean' },
	{ value: 'th', label: 'Thai' },
	{ value: 'vi', label: 'Vietnamese' },
	{ value: 'id', label: 'Indonesian' },
	{ value: 'ms', label: 'Malay' },
	{ value: 'tl', label: 'Filipino' },
	{ value: 'sv', label: 'Swedish' },
	{ value: 'no', label: 'Norwegian' },
	{ value: 'da', label: 'Danish' },
	{ value: 'fi', label: 'Finnish' },
	{ value: 'lt', label: 'Lithuanian' },
	{ value: 'lv', label: 'Latvian' },
	{ value: 'et', label: 'Estonian' },
]

const javaServerPatchData = computed(() => {
	const origJava = projectV3.value?.minecraft_java_server

	const addressChanged =
		javaAddress.value !== (origJava?.address ?? '') || javaPort.value !== (origJava?.port ?? 25565)

	if (addressChanged) {
		return {
			address: javaAddress.value.trim(),
			port: javaPort.value,
		}
	}

	return {}
})

const bedrockServerPatchData = computed(() => {
	const origBedrock = projectV3.value?.minecraft_bedrock_server
	if (
		bedrockAddress.value !== (origBedrock?.address ?? '') ||
		bedrockPort.value !== (origBedrock?.port ?? 19132)
	) {
		return {
			address: bedrockAddress.value.trim(),
			port: bedrockPort.value,
		}
	}

	return {}
})

const serverPatchData = computed(() => {
	const origServer = projectV3.value?.minecraft_server
	const countryChanged = country.value && country.value !== origServer?.country
	const languagesChanged =
		JSON.stringify([...languages.value].sort()) !==
		JSON.stringify([...(origServer?.languages ?? [])].sort())

	if (countryChanged || languagesChanged) {
		return {
			...origServer,
			...(countryChanged ? { country: country.value } : {}),
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
	javaPort: projectV3.value?.minecraft_java_server?.port ?? 25565,
	bedrockAddress: projectV3.value?.minecraft_bedrock_server?.address ?? '',
	bedrockPort: projectV3.value?.minecraft_bedrock_server?.port ?? 19132,
	country: projectV3.value?.minecraft_server?.country ?? '',
	languages: projectV3.value?.minecraft_server?.languages ?? [],
}))

const modified = computed(() => ({
	javaAddress: javaAddress.value,
	javaPort: javaPort.value,
	bedrockAddress: bedrockAddress.value,
	bedrockPort: bedrockPort.value,
	country: country.value,
	languages: languages.value,
}))

function resetChanges() {
	javaAddress.value = projectV3.value?.minecraft_java_server?.address ?? ''
	javaPort.value = projectV3.value?.minecraft_java_server?.port ?? 25565
	bedrockAddress.value = projectV3.value?.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = projectV3.value?.minecraft_bedrock_server?.port ?? 19132
	country.value = projectV3.value?.minecraft_server?.country ?? ''
	languages.value = projectV3.value?.minecraft_server?.languages ?? []
}

async function handleSave() {
	saving.value = true
	try {
		const hasV3Changes = Object.keys(v3PatchData.value).length > 0
		if (hasV3Changes) {
			await patchProjectV3(v3PatchData.value)
		}
	} finally {
		saving.value = false
	}
}
</script>
