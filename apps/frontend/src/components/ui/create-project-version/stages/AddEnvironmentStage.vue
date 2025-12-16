<template>
	<ProjectSettingsEnvSelector v-model="draftVersion.environment" />
</template>

<script lang="ts" setup>
import { ProjectSettingsEnvSelector } from '@modrinth/ui'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion } = injectManageVersionContext()
</script>
