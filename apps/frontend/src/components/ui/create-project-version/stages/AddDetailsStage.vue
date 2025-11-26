<template>
	<div class="flex w-full max-w-[496px] flex-col gap-6">
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version type <span class="text-red">*</span>
			</span>
			<div class="flex gap-2.5">
				<ButtonStyled
					:color="versionType === 'Release' ? 'green' : 'standard'"
					:highlighted="versionType === 'Release'"
					type="chip"
				>
					<button @click="versionType = 'Release'">Release</button>
				</ButtonStyled>
				<ButtonStyled
					:color="versionType === 'Alpha' ? 'green' : 'standard'"
					:highlighted="versionType === 'Alpha'"
					type="chip"
				>
					<button @click="versionType = 'Alpha'">Alpha</button>
				</ButtonStyled>
				<ButtonStyled
					:color="versionType === 'Beta' ? 'green' : 'standard'"
					:highlighted="versionType === 'Beta'"
					type="chip"
				>
					<button @click="versionType = 'Beta'">Beta</button>
				</ButtonStyled>
			</div>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version number <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="versionNumber"
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
			<span> The version number appears as the version name and used by the API. </span>
		</div>
		<div class="flex flex-col gap-2">
			<span class="font-semibold text-contrast">
				Version subtitle <span class="text-red">*</span>
			</span>
			<input
				id="version-number"
				v-model="versionSubtitle"
				type="text"
				autocomplete="off"
				maxlength="32"
			/>
		</div>
		<div>
			<LoaderPicker v-model="selectedLoaders" :loaders="generatedState.loaders" />
		</div>
	</div>
</template>

<script lang="ts" setup>
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'

import LoaderPicker from '../components/LoaderPicker.vue'

const generatedState = useGeneratedState()

type VersionType = 'Release' | 'Alpha' | 'Beta'
const versionType = ref<VersionType>('Release')
const versionNumber = ref('')
const versionSubtitle = ref('')

const selectedLoaders = ref<string[]>([])
</script>
