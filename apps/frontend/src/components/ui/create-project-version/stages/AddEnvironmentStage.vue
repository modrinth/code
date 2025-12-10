<template>
	<ProjectSettingsEnvSelector v-model="draftVersion.environment" />
</template>

<script setup lang="ts">
import { ProjectSettingsEnvSelector } from '@modrinth/ui'

import { useManageVersion } from '~/composables/versions/manage-version'

const { draftVersion } = useManageVersion()
</script>
