<template>
	<div class="flex w-full max-w-full flex-col gap-3">
		<div class="grid gap-2.5">
			<span class="font-semibold text-contrast">Project</span>
			<DependencySelect v-model="newDependencyProjectId" />
		</div>

		<template v-if="newDependencyProjectId">
			<div class="grid gap-2.5">
				<span class="font-semibold text-contrast"> Version </span>
				<Combobox
					v-model="newDependencyVersionId"
					placeholder="Select version"
					:options="[{ label: 'Any version', value: null }, ...newDependencyVersions]"
					:searchable="true"
				/>
			</div>

			<div class="grid gap-2.5">
				<span class="font-semibold text-contrast"> Dependency relation </span>
				<Combobox
					v-model="newDependencyType"
					placeholder="Select dependency type"
					:options="[
						{ label: 'Required', value: 'required' },
						{ label: 'Optional', value: 'optional' },
						{ label: 'Incompatible', value: 'incompatible' },
						{ label: 'Embedded', value: 'embedded' },
					]"
				/>
			</div>
		</template>
	</div>
</template>

<script lang="ts" setup>
import { Combobox } from '@modrinth/ui'

import DependencySelect from '~/components/ui/create-project-version/components/DependencySelect.vue'
import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

const {
	newDependencyProjectId,
	newDependencyType,
	newDependencyVersionId,
	newDependencyVersions,
} = injectManageVersionContext()
</script>
