<template>
	<div class="sm:w-[512px]">
		<EnvironmentSelector v-model="draftVersion.environment" />
	</div>
</template>

<script lang="ts" setup>
import { EnvironmentSelector } from '@modrinth/ui'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const { draftVersion } = injectManageVersionContext()
</script>
