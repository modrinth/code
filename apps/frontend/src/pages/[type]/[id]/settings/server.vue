<template>
	<div>
		<section class="universal-card">
			<div class="flex flex-col gap-6">
				<div class="text-2xl font-semibold text-contrast">Server details</div>

				<!-- Region -->
				<div class="max-w-[600px]">
					<label for="server-region">
						<span class="label__title">Region</span>
					</label>
					<Combobox
						id="server-region"
						v-model="region"
						:options="regionOptions"
						searchable
						placeholder="Select region"
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
					<MultiSelect
						id="server-language"
						v-model="languages"
						:options="languageOptions"
						searchable
						:max-tag-rows="2"
						placeholder="Select languages"
						:disabled="!hasPermission"
					/>
				</div>

				<!-- Java Address -->
				<div class="max-w-[600px]">
					<div class="flex items-center justify-between">
						<label for="java-address">
							<span class="label__title !m-0 !text-contrast">Java address</span>
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
						<StyledInput
							id="java-address"
							v-model="javaAddress"
							placeholder="Enter address"
							:disabled="!hasPermission"
							wrapper-class="flex-grow"
							autocomplete="off"
						/>
					</div>
					<div
						v-if="javaAddress"
						class="mt-2 flex gap-1.5"
						:class="{
							'items-center': javaPingResult?.online,
							'items-start': javaPingResult && !javaPingResult.online,
						}"
					>
						<ButtonStyled
							v-if="(javaAddress && javaPingResult) || javaPingLoading"
							circular
							type="transparent"
							size="small"
							color="oranges"
						>
							<button
								v-tooltip="'Refresh ping'"
								:disabled="javaPingLoading"
								@click="pingJavaServer"
							>
								<SpinnerIcon v-if="javaPingLoading" class="animate-spin" />
								<RefreshCwIcon v-else />
							</button>
						</ButtonStyled>
						<div
							v-if="javaPingResult !== null && !javaPingLoading && javaPingResult.online"
							class="mt-0.5 flex items-center gap-1.5 text-green"
						>
							Server is online!
							<template v-if="javaPingResult.latency">
								Latency: {{ javaPingResult.latency }}ms
							</template>
						</div>
						<div v-else-if="javaPingResult !== null && !javaPingLoading" class="mt-0.5 text-orange">
							We couldn’t ping this server. It may be blocked by your host so try refreshing a few
							times. If it still doesn’t respond please
							<a
								class="inline underline"
								href="https://support.modrinth.com"
								target="_blank"
								rel="noopener noreferrer"
							>
								contact support</a
							>.
						</div>
					</div>
					<div v-else class="mt-2 text-sm">
						If you have [SRV records]
						<InfoIcon
							v-tooltip="{
								content:
									'The address you enter here may have DNS SRV records _minecraft._tcp.{your domain} which point to your Minecraft server address and port.',
								popperClass: 'max-w-xs',
							}"
						/>, you do not need to add a port. Otherwise if you have a port which isn't 25565, you
						can include it as :12345
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
import { InfoIcon, RefreshCwIcon, SpinnerIcon } from '@modrinth/assets'
import {
	ButtonStyled,
	Combobox,
	injectModrinthClient,
	injectNotificationManager,
	injectProjectPageContext,
	MultiSelect,
	SERVER_LANGUAGES,
	SERVER_REGIONS,
	StyledInput,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'

import CompatibilityCard from '~/components/ui/project-settings/CompatibilityCard.vue'

const PING_TIMEOUT_MS = 5000

const { formatMessage, locale } = useVIntl()

const client = injectModrinthClient()
const { addNotification } = injectNotificationManager()
const { projectV3, currentMember, patchProjectV3 } = injectProjectPageContext()

const javaAddress = ref('')
const bedrockAddress = ref('')
const bedrockPort = ref(19132)
const region = ref('')
const languages = ref([])

const javaPingLoading = ref(false)
const javaPingResult = ref(null)

const lastPingedAddress = ref('')

const lastPingAddressChanged = computed(() => {
	return javaAddress.value.trim() !== lastPingedAddress.value
})

let pingDebounceTimer = null

watch(javaAddress, () => {
	clearTimeout(pingDebounceTimer)
	pingDebounceTimer = setTimeout(() => {
		pingJavaServer()
	}, 500)
})

const hasPermission = computed(() => {
	const EDIT_DETAILS = 1 << 2
	return ((currentMember.value?.permissions ?? 0) & EDIT_DETAILS) === EDIT_DETAILS
})

async function pingJavaServer() {
	const address = javaAddress.value?.trim()
	if (!address) {
		javaPingResult.value = null
		return
	}

	javaPingLoading.value = true
	javaPingResult.value = null

	try {
		await Promise.race([
			client.labrinth.server_ping_internal.pingMinecraftJava({
				address,
				timeout_ms: PING_TIMEOUT_MS,
			}),
			new Promise((_, reject) =>
				setTimeout(() => reject(new Error('Ping timed out')), PING_TIMEOUT_MS),
			),
		])
		javaPingResult.value = { online: true, latency: null }
	} catch {
		javaPingResult.value = { online: false, latency: null }
	} finally {
		javaPingLoading.value = false
		lastPingedAddress.value = address
	}
}

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
		javaAddress.value.trim() !== (projectV3.value?.minecraft_java_server?.address ?? '')
	if (addressChanged) {
		return {
			address: javaAddress.value.trim(),
		}
	}

	return {}
})

const bedrockServerPatchData = computed(() => {
	const origBedrock = projectV3.value?.minecraft_bedrock_server
	if (bedrockAddress.value !== (origBedrock?.address ?? '')) {
		return {
			address: bedrockAddress.value.trim(),
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

function resetChanges() {
	javaAddress.value = projectV3.value?.minecraft_java_server?.address ?? ''
	bedrockAddress.value = projectV3.value?.minecraft_bedrock_server?.address ?? ''
	bedrockPort.value = projectV3.value?.minecraft_bedrock_server?.port ?? 19132
	region.value = projectV3.value?.minecraft_server?.region ?? ''
	languages.value = projectV3.value?.minecraft_server?.languages ?? []
}

async function handleSave() {
	if (javaAddress.value.trim() && !javaPingResult.value?.online) {
		addNotification({
			title: 'Cannot save',
			text: 'The Java server must be reachable before saving. Please ensure the ping succeeds.',
			type: 'error',
		})
		return
	}

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
