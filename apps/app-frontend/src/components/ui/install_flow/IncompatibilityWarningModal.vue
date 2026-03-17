<template>
	<ModalWrapper ref="incompatibleModal" header="Incompatibility warning" :on-hide="onInstall">
		<div class="modal-body">
			<p>
				This {{ versions?.length > 0 ? 'project' : 'version' }} is not compatible with the instance
				you're trying to install it on. Are you sure you want to continue? Dependencies will not be
				installed.
			</p>
			<table>
				<thead>
					<tr class="header">
						<th>{{ instance?.name }}</th>
						<th>{{ project.title }}</th>
					</tr>
				</thead>
				<tbody>
					<tr class="content">
						<td class="data">{{ instance?.loader }} {{ instance?.game_version }}</td>
						<td>
							<Combobox
								v-if="versions?.length > 1"
								v-model="selectedVersionId"
								:options="versionOptions"
								:searchable="true"
								placeholder="Select version"
								force-direction="up"
								:max-height="150"
							/>
							<span v-else>
								<span>{{ selectedVersionLabel }}</span>
							</span>
						</td>
					</tr>
				</tbody>
			</table>
			<div class="button-group">
				<Button @click="() => incompatibleModal.hide()"><XIcon />Cancel</Button>
				<Button color="primary" :disabled="installing" @click="install()">
					<DownloadIcon /> {{ installing ? 'Installing' : 'Install' }}
				</Button>
			</div>
		</div>
	</ModalWrapper>
</template>

<script setup>
import { DownloadIcon, XIcon } from '@modrinth/assets'
import { Button, Combobox, formatLoader, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { add_project_from_version as installMod } from '@/helpers/profile'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const instance = ref(null)
const project = ref(null)
const versions = ref(null)
const selectedVersion = ref(null)
const incompatibleModal = ref(null)
const installing = ref(false)

const onInstall = ref(() => {})

const selectedVersionLabel = computed(() => {
	if (!selectedVersion.value) return ''
	return `${selectedVersion.value.name} (${selectedVersion.value.loaders
		.map((name) => formatLoader(formatMessage, name))
		.join(', ')} - ${selectedVersion.value.game_versions.join(', ')})`
})

const versionOptions = computed(() =>
	(versions.value ?? []).map((version) => ({
		value: version.id,
		label: `${version.name} (${version.loaders
			.map((name) => formatLoader(formatMessage, name))
			.join(', ')} - ${version.game_versions.join(', ')})`,
	})),
)

const selectedVersionId = computed({
	get: () => selectedVersion.value?.id ?? null,
	set: (value) => {
		if (!value) return
		selectedVersion.value = (versions.value ?? []).find((version) => version.id === value) ?? null
	},
})

defineExpose({
	show: (instanceVal, projectVal, projectVersions, selected, callback) => {
		instance.value = instanceVal
		versions.value = projectVersions ?? []
		selectedVersion.value = selected ?? projectVersions?.[0] ?? null

		project.value = projectVal

		onInstall.value = callback
		installing.value = false

		incompatibleModal.value.show()

		trackEvent('ProjectInstallStart', { source: 'ProjectIncompatibilityWarningModal' })
	},
})

const install = async () => {
	installing.value = true
	await installMod(instance.value.path, selectedVersion.value.id).catch(handleError)
	installing.value = false
	onInstall.value(selectedVersion.value.id)
	incompatibleModal.value.hide()

	trackEvent('ProjectInstall', {
		loader: instance.value.loader,
		game_version: instance.value.game_version,
		id: project.value,
		version_id: selectedVersion.value.id,
		project_type: project.value.project_type,
		title: project.value.title,
		source: 'ProjectIncompatibilityWarningModal',
	})
}
</script>

<style lang="scss" scoped>
.data {
	text-transform: capitalize;
}

table {
	width: 100%;
	border-radius: var(--radius-lg);
	border-collapse: collapse;
	box-shadow: 0 0 0 1px var(--color-button-bg);
}

th {
	text-align: left;
	padding: 1rem;
	background-color: var(--color-bg);
	overflow: hidden;
	border-bottom: 1px solid var(--color-button-bg);
}

th:first-child {
	border-top-left-radius: var(--radius-lg);
	border-right: 1px solid var(--color-button-bg);
}

th:last-child {
	border-top-right-radius: var(--radius-lg);
}

td {
	padding: 1rem;
}

td:first-child {
	border-right: 1px solid var(--color-button-bg);
}

.button-group {
	display: flex;
	justify-content: flex-end;
	gap: 1rem;
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: 1rem;
}
</style>
