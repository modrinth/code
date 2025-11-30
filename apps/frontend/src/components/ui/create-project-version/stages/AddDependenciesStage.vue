<template>
	<div class="w-[576px] max-w-[576px]">
		<div class="grid gap-6">
			<div class="flex flex-col gap-4">
				<span class="font-semibold text-contrast">Add dependency</span>
				<div class="grid gap-3 rounded-2xl border border-solid border-surface-5 p-4">
					<div class="grid gap-2.5">
						<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
						<ModSelect v-model="newDependencyId" />
					</div>

					<template v-if="newDependencyId">
						<div class="grid gap-2.5">
							<span class="font-semibold text-contrast"> Version </span>
							<Combobox
								v-model="version"
								placeholder="Select project"
								:options="[
									{ label: '1.0.0', value: '1.0.0' },
									{ label: '2.0.0', value: '2.0.0' },
								]"
								:searchable="true"
							/>
						</div>

						<div class="grid gap-2.5">
							<span class="font-semibold text-contrast"> Dependency relation </span>
							<Combobox
								v-model="newDependencyType"
								placeholder="Select project"
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
				<ButtonStyled>
					<button
						class="self-start"
						:disabled="!newDependencyId"
						@click="
							addDependency({
								project_id: newDependencyId,
								version_id: version,
								dependency_type: newDependencyType,
								project: {} as any,
								version: {} as any,
								link: '',
							})
						"
					>
						Add Dependency
					</button>
				</ButtonStyled>
			</div>

			<div class="flex flex-col gap-4">
				<span class="font-semibold text-contrast">Add dependency</span>
				<div class="5 flex flex-col gap-2">
					<AddedDependencyRow
						v-for="(dependency, index) in addedDependencies"
						:key="index"
						:name="dependency.name"
						:icon="dependency.icon"
						:dependency-type="dependency.dependencyType"
						:version-name="dependency.versionName"
						@remove="() => {} /* TODO: remove dependency */"
					/>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import type { Labrinth } from '@modrinth/api-client'
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'

import ModSelect from '~/components/ui/create-project-version/components/ModSelect.vue'
import { useManageVersion } from '~/composables/versions/manage-version'

import AddedDependencyRow from '../components/AddedDependencyRow.vue'

const newDependencyId = ref('')
const newDependencyType = ref<'required' | 'optional' | 'incompatible' | 'embedded'>('required')
const version = ref('')

const { draftVersion } = useManageVersion()

const addedDependencies = computed(() =>
	(draftVersion.value.dependencies || []).map((_dep) => {
		// todo: fetch project and version data, or pass in fields from search
		return {
			name: '',
			icon: '',
			dependencyType: 'required' as const,
			versionName: '',
		}
	}),
)

export interface ProjectDependency {
	project: Labrinth.Projects.v3.Project
	project_id: string

	version: Labrinth.Versions.v3.Version
	version_id: string

	dependency_type: string
	link: string
}

const addDependency = (_dependency: ProjectDependency) => {
	// todo
}
</script>
