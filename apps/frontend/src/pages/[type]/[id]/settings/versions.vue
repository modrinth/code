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
import type { Labrinth } from '@modrinth/api-client'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'

import CreateProjectVersionModal from '~/components/ui/create-project-version/CreateProjectVersionModal.vue'
import { provideVersionsContext } from '~/providers/versions'

const modal = ref<InstanceType<typeof CreateProjectVersionModal>>()

function openModal() {
	modal.value?.show?.()
}

const props = defineProps<{
	project: Labrinth.Projects.v3.Project
	versions: Labrinth.Versions.v3.Version[]
}>()

provideVersionsContext(props)
</script>
