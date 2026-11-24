<script setup lang="ts">
import { injectContentManager } from '../../providers/content-manager'
import ConfirmModpackUpdateModal from '../modals/ConfirmModpackUpdateModal.vue'
import ContentUpdaterModal from '../modals/content-updater-modal/index.vue'
import { useUpdateFlow } from './use-update-flow'

const ctx = injectContentManager()

const {
	updaterModal,
	confirmModpackUpdateModal,
	updatingItem,
	updatingModpack,
	versions,
	loadingVersions,
	loadingChangelog,
	isDowngrade,
	currentVersionId,
	projectType,
	projectName,
	projectIconUrl,
	actionDisabled,
	actionDisabledTooltip,
	backupTip,
	openUpdate,
	openSwitch,
	openModpack,
	reset,
	loadChangelog,
	selectVersion,
	installModpackVersion,
	cancelModpackUpdate,
} = useUpdateFlow(ctx)

defineExpose({ openUpdate, openSwitch, openModpack })
</script>

<template>
	<ContentUpdaterModal
		v-if="updatingItem || updatingModpack"
		ref="updaterModal"
		:versions="versions"
		:current-game-version="ctx.target.value.gameVersion"
		:current-loader="ctx.target.value.loader"
		:current-version-id="currentVersionId"
		:is-app="ctx.target.value.kind === 'instance'"
		:project-type="projectType"
		:project-icon-url="projectIconUrl"
		:project-name="projectName"
		:loading="loadingVersions"
		:loading-changelog="loadingChangelog"
		:action-disabled="actionDisabled"
		:action-disabled-tooltip="actionDisabledTooltip"
		@update="selectVersion"
		@cancel="reset"
		@version-select="(version) => loadChangelog(version, true)"
		@version-hover="(version) => loadChangelog(version, false)"
	/>
	<ConfirmModpackUpdateModal
		ref="confirmModpackUpdateModal"
		:downgrade="isDowngrade"
		:backup-tip="backupTip"
		:action-disabled="!!ctx.managed.value?.disabledReason"
		:action-disabled-tooltip="ctx.managed.value?.disabledReason"
		@confirm="installModpackVersion"
		@cancel="cancelModpackUpdate"
	/>
</template>
