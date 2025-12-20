<template>
	<div class="sm:w-[512px]">
		<ProjectSettingsEnvSelector v-model="draftVersion.environment" />
	</div>
</template>

<script lang="ts" setup>
import { ProjectSettingsEnvSelector } from '@modrinth/ui'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion } = injectManageVersionContext()
</script>
