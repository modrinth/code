<template>
	<div class="flex flex-col gap-6">
		<template v-if="modpack">
			<div class="flex items-center gap-3 rounded-[20px] bg-surface-2 p-3">
				<div
					class="size-12 shrink-0 overflow-hidden rounded-2xl border border-solid border-surface-5"
				>
					<Avatar
						v-if="modpack.icon_url"
						:src="modpack.icon_url"
						:alt="modpack.name"
						size="100%"
						class="!rounded-2xl"
						no-shadow
					/>
				</div>
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<span class="truncate font-semibold text-contrast">{{ modpack.name }}</span>
					<span v-if="modpack.authors.length" class="truncate text-sm font-medium text-secondary">
						{{ modpack.authors.join(', ') }}
					</span>
				</div>
				<Button type="outlined" @click="changeModpack">
					{{ formatMessage(messages.changeButton) }}
				</Button>
			</div>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{ formatMessage(commonMessages.versionLabel) }}
				</span>
				<Combobox
					v-model="selectedFileId"
					:options="versionOptions"
					:no-options-message="formatMessage(commonMessages.loadingLabel)"
					trigger-class="!h-auto !py-2.5"
				>
					<template #selected>
						<span class="flex min-w-0 flex-col gap-1">
							<span class="truncate">{{ selectedOption?.label }}</span>
							<span
								v-if="selectedOption?.subLabel"
								class="truncate text-sm font-medium text-secondary"
							>
								{{ selectedOption.subLabel }}
							</span>
						</span>
					</template>
				</Combobox>
			</div>

			<div v-if="versionFailed" class="m-0 leading-6 text-red">
				{{ formatMessage(messages.lookupFailed) }}
			</div>
			<div v-else-if="!serverPackStatus" class="flex items-center gap-2 text-secondary">
				<SpinnerIcon class="size-5 animate-spin" aria-hidden="true" />
				{{ formatMessage(messages.checkingServerPack) }}
			</div>
			<Admonition
				v-else-if="serverPackStatus.kind === 'available'"
				type="success"
				:body="formatMessage(messages.serverPackAvailable, { filename: serverPackStatus.filename })"
			/>
			<Admonition v-else-if="serverPackStatus.kind === 'ftb'" type="info">
				{{ formatMessage(messages.ftbInstaller) }}
			</Admonition>
			<Admonition v-else-if="closestFile" type="warning">
				<IntlFormatted
					:message-id="messages.serverPackClosest"
					:values="{ version: closestFile.display_name }"
				>
					<template #link="{ children }">
						<button
							type="button"
							class="m-0 cursor-pointer border-none bg-transparent p-0 text-link hover:underline"
							@click="selectClosestVersion"
						>
							<component :is="() => children" />
						</button>
					</template>
				</IntlFormatted>
			</Admonition>
			<template v-else>
				<Admonition type="warning" :body="formatMessage(messages.serverPackNone)" />
				<div class="flex flex-col gap-2">
					<span class="font-semibold text-contrast">
						<IntlFormatted :message-id="messages.serverPackLabel">
							<template #optional="{ children }">
								<span class="text-primary"><component :is="() => children" /></span>
							</template>
						</IntlFormatted>
					</span>
					<ArchiveInput
						v-model="selection.curseforgeServerPackFile.value"
						v-model:processing="selection.curseforgeProcessing.value"
					/>
				</div>
			</template>
		</template>

		<template v-else>
			<template v-if="!selection.curseforgeModpackFile.value">
				<div class="flex flex-col gap-2">
					<label :for="linkId" class="font-semibold text-contrast">
						{{ formatMessage(messages.linkLabel) }}
					</label>
					<Input
						:id="linkId"
						v-model="selection.curseforgeUrl.value"
						:icon="LinkIcon"
						type="url"
						size="medium"
						input-class="text-sm"
						placeholder="https://curseforge.com/minecraft/modpacks/all-the-mods-10"
						:error="!!linkError"
						:aria-describedby="`${linkId}-description`"
					>
						<template v-if="lookupState === 'loading'" #trailing>
							<SpinnerIcon class="size-5 animate-spin text-secondary" aria-hidden="true" />
						</template>
					</Input>
					<p
						:id="`${linkId}-description`"
						class="m-0 leading-6"
						:class="linkError ? 'text-red' : 'text-primary'"
					>
						{{ formatMessage(linkError ?? messages.linkDescription) }}
					</p>
				</div>
				<div class="flex items-center gap-4">
					<div class="h-px flex-1 bg-surface-5" />
					<span class="font-medium text-primary">{{ formatMessage(messages.orUpload) }}</span>
					<div class="h-px flex-1 bg-surface-5" />
				</div>
			</template>

			<div class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					{{
						formatMessage(
							selection.curseforgeModpackFile.value
								? messages.selectedFileLabel
								: messages.fileLabel,
						)
					}}
				</span>
				<ArchiveInput
					v-model="selection.curseforgeModpackFile.value"
					v-model:processing="selection.curseforgeProcessing.value"
				/>
				<p v-if="!selection.curseforgeModpackFile.value" class="m-0 leading-6 text-primary">
					{{ formatMessage(messages.fileDescription) }}
				</p>
			</div>

			<div v-if="selection.curseforgeModpackFile.value" class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">
					<IntlFormatted :message-id="messages.serverPackLabel">
						<template #optional="{ children }">
							<span class="text-primary"><component :is="() => children" /></span>
						</template>
					</IntlFormatted>
				</span>
				<ArchiveInput
					v-model="selection.curseforgeServerPackFile.value"
					v-model:processing="selection.curseforgeProcessing.value"
				/>
				<p class="m-0 leading-6 text-primary">
					{{ formatMessage(messages.serverPackDescription) }}
				</p>
			</div>
		</template>
	</div>
</template>

<script setup lang="ts">
import type { Kyros, ModrinthApiError } from '@modrinth/api-client'
import { LinkIcon, SpinnerIcon } from '@modrinth/assets'
import { watchDebounced } from '@vueuse/core'
import { computed, onMounted, ref, useId, watch } from 'vue'

import Admonition from '#ui/components/base/Admonition.vue'
import Avatar from '#ui/components/base/Avatar.vue'
import { Button } from '#ui/components/base/buttons'
import Combobox, { type ComboboxOption } from '#ui/components/base/Combobox.vue'
import Input from '#ui/components/base/inputs/Input.vue'
import IntlFormatted from '#ui/components/base/IntlFormatted.vue'
import { useVIntl } from '#ui/composables/i18n'
import { commonMessages } from '#ui/utils/common-messages'
import { formatLoaderLabel } from '#ui/utils/loaders'

import {
	curseforgeMessages as messages,
	injectCurseForgeSelection,
	isCurseForgeModpackUrl,
	normalizeCurseForgeModpackUrl,
} from '../../curseforge'
import ArchiveInput from './archive-input.vue'

type ServerPackStatus =
	| { kind: 'available'; filename: string }
	| { kind: 'ftb' }
	| { kind: 'closest'; file: Kyros.FlameAnvil.v1.ProjectFile }
	| { kind: 'none' }

const selection = injectCurseForgeSelection()
const { formatMessage } = useVIntl()
const linkId = useId()

const lookupState = ref<'idle' | 'loading' | 'not-found' | 'failed'>('idle')
const versionFailed = ref(false)
let lookupRun = 0
let versionRun = 0

const modpack = computed(() => selection.curseforgeModpack.value)

const invalidUrl = computed(
	() =>
		!!selection.curseforgeUrl.value.trim() &&
		!isCurseForgeModpackUrl(selection.curseforgeUrl.value),
)
const linkError = computed(() => {
	if (invalidUrl.value) return messages.invalidLink
	if (lookupState.value === 'not-found') return messages.notFound
	if (lookupState.value === 'failed') return messages.lookupFailed
	return null
})

const versionOptions = computed<ComboboxOption<number>[]>(() =>
	[...selection.curseforgeFiles.value]
		.sort((a, b) => b.file_date.localeCompare(a.file_date))
		.map((file) => ({
			value: file.id,
			label: file.display_name,
			subLabel: formatVersionDetails(file),
		})),
)
const selectedOption = computed(() =>
	versionOptions.value.find((option) => option.value === selection.curseforgeFileId.value),
)
const selectedFileId = computed({
	get: () => selection.curseforgeFileId.value ?? undefined,
	set: (fileId) => {
		if (fileId !== undefined) void selectVersion(fileId)
	},
})

/** The server pack situation of the selected file, `null` until the file is resolved. */
const serverPackStatus = computed<ServerPackStatus | null>(() => {
	const resolved = modpack.value
	if (!resolved || resolved.file.id !== selection.curseforgeFileId.value) return null
	if (resolved.server_pack) return { kind: 'available', filename: resolved.server_pack.file_name }
	if (resolved.ftb_installer_hint) return { kind: 'ftb' }
	if (resolved.closest_server_pack_file) {
		return { kind: 'closest', file: resolved.closest_server_pack_file }
	}
	return { kind: 'none' }
})

const closestFile = computed(() =>
	serverPackStatus.value?.kind === 'closest' ? serverPackStatus.value.file : null,
)

function formatVersionDetails(file: Kyros.FlameAnvil.v1.LinkQueryFile): string {
	return [
		...file.minecraft_versions
			.slice(0, 1)
			.map((version) => formatMessage(messages.minecraftVersion, { version })),
		...file.loaders.map(formatLoaderLabel),
	].join(' • ')
}

/** Adds files of the resolved modpack missing from the listed files, such as older ones. */
function rememberFiles(...files: (Kyros.FlameAnvil.v1.LinkQueryFile | null | undefined)[]) {
	const known = new Set(selection.curseforgeFiles.value.map((file) => file.id))
	const missing = files.filter(
		(file): file is Kyros.FlameAnvil.v1.LinkQueryFile => !!file && !known.has(file.id),
	)
	if (missing.length)
		selection.curseforgeFiles.value = [...selection.curseforgeFiles.value, ...missing]
}

function isNotFound(error: unknown): boolean {
	const status = (error as ModrinthApiError).statusCode
	return status === 400 || status === 404
}

function clearModpack() {
	lookupRun++
	versionRun++
	selection.curseforgeModpack.value = null
	selection.curseforgeFileId.value = null
	selection.curseforgeFiles.value = []
	versionFailed.value = false
}

async function lookUp(value: string) {
	if (modpack.value) return
	const run = ++lookupRun
	const link = normalizeCurseForgeModpackUrl(value)
	if (!link || selection.curseforgeModpackFile.value) {
		lookupState.value = 'idle'
		return
	}

	lookupState.value = 'loading'
	try {
		const resolved = await selection.queryCurseForgeLink(link)
		if (run !== lookupRun) return
		selection.curseforgeModpack.value = resolved
		selection.curseforgeFileId.value = resolved.file.id
		selection.curseforgeFiles.value = []
		rememberFiles(resolved.file, resolved.closest_server_pack_file)
		lookupState.value = 'idle'
		void loadFiles(resolved.project_id, run)
	} catch (error) {
		if (run === lookupRun) lookupState.value = isNotFound(error) ? 'not-found' : 'failed'
	}
}

async function loadFiles(projectId: number, run: number) {
	try {
		const { files } = await selection.listCurseForgeFiles(projectId)
		if (run !== lookupRun) return
		const listed = files.filter((file) => file.available)
		const listedIds = new Set(listed.map((file) => file.id))
		selection.curseforgeFiles.value = [
			...listed,
			...selection.curseforgeFiles.value.filter((file) => !listedIds.has(file.id)),
		]
	} catch {
		// The resolved files stay selectable without the full list.
	}
}

async function selectVersion(fileId: number) {
	const link = normalizeCurseForgeModpackUrl(selection.curseforgeUrl.value)
	const resolvedFileId = modpack.value?.file.id
	if (!link || !resolvedFileId) return
	if (fileId === resolvedFileId && fileId === selection.curseforgeFileId.value) return

	const run = ++versionRun
	selection.curseforgeFileId.value = fileId
	versionFailed.value = false
	try {
		const resolved = await selection.queryCurseForgeLink(link, fileId)
		if (run !== versionRun) return
		selection.curseforgeModpack.value = resolved
		selection.curseforgeFileId.value = resolved.file.id
		rememberFiles(resolved.file, resolved.closest_server_pack_file)
	} catch {
		if (run === versionRun) versionFailed.value = true
	}
}

function selectClosestVersion() {
	if (closestFile.value) void selectVersion(closestFile.value.id)
}

function changeModpack() {
	clearModpack()
	selection.curseforgeUrl.value = ''
	selection.curseforgeServerPackFile.value = null
	lookupState.value = 'idle'
}

watchDebounced(() => selection.curseforgeUrl.value, lookUp, { debounce: 400 })

onMounted(() => void lookUp(selection.curseforgeUrl.value))

watch(selection.curseforgeModpackFile, (file) => {
	selection.curseforgeServerPackFile.value = null
	if (file) {
		clearModpack()
		selection.curseforgeUrl.value = ''
	}
})

watch(serverPackStatus, (status) => {
	if (status && status.kind !== 'none') selection.curseforgeServerPackFile.value = null
})
</script>
