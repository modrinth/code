<template>
	<div class="w-[496px] max-w-[496px]">
		<div class="grid gap-6">
			<div class="flex flex-col gap-4">
				<span class="font-semibold text-contrast">Add dependency</span>
				<div class="grid gap-2.5 rounded-2xl border border-solid border-surface-5 p-4">
					<div class="grid gap-2.5">
						<span class="font-semibold text-contrast">Project <span class="text-red">*</span></span>
						<ModSelect v-model="newDependencyId" />
					</div>

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
				</div>
				<ButtonStyled>
					<button class="self-start" @click="addDependency(newDependencyId, newDependencyType)">
						Add Dependency
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import Combobox from '@modrinth/ui/src/components/base/Combobox.vue'

import ModSelect from '~/components/ui/create-project-version/components/ModSelect.vue'

const newDependencyId = ref('')
const newDependencyType = ref<'required' | 'optional' | 'incompatible' | 'embedded'>('required')
const version = ref('')

const addDependency = (
	_id: string,
	_type: 'required' | 'optional' | 'incompatible' | 'embedded',
) => {
	// todo
}
</script>
