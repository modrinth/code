<template>
	<div>
		<CreateProjectVersionModal ref="modal"></CreateProjectVersionModal>
		<div class="universal-card">
			<div class="markdown-disclaimer">
				<h2>Versions</h2>
			</div>

			Table placeholder

			<ButtonStyled>
				<button @click="openModal">open</button>
			</ButtonStyled>
		</div>
	</div>
</template>

<script lang="ts" setup>
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import type { Project, Version } from '@modrinth/utils'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { provideVersionsContext } from '~/providers/versions'

const modal = ref<InstanceType<typeof CreateProjectVersionModal>>()

function openModal() {
	modal.value?.show?.()
}

const props = defineProps<{
	project: Project
	versions: Version[]
}>()

provideVersionsContext(props)
</script>
