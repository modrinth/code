<template>
	<EnvironmentSelector v-model="draftVersion.environment" />
</template>

<script lang="ts" setup>
import { EnvironmentSelector } from '@modrinth/ui'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion } = injectManageVersionContext()
</script>
