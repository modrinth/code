<template>
	<div class="flex flex-col gap-6">
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
		<div class="flex flex-col gap-2">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Loaders <span class="text-red">*</span> </span>
				<ButtonStyled type="transparent" size="standard">
					<button @click="clearAllLoaders()">Clear all</button>
				</ButtonStyled>
			</div>
			<div
				class="flex flex-wrap gap-1.5 gap-y-2 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<ButtonStyled
					v-for="loader in generatedState.loaders.map((x) => x.name)"
					:key="`platform-tag-${loader}`"
					:color="selectedLoaders.includes(loader) ? 'green' : undefined"
					:highlighted="selectedLoaders.includes(loader)"
					type="chip"
					size="standard"
				>
					<button :style="`color: var(--color-platform-${loader})`" @click="toggleLoader(loader)">
						<div>
							<!-- eslint-disable-next-line vue/no-v-html -->
							<div v-html="generatedState.loaders?.find((x) => x.name === loader)?.icon"></div>
						</div>
						{{ formatCategory(loader) }}
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import ButtonStyled from '@modrinth/ui/src/components/base/ButtonStyled.vue'
import { formatCategory } from '@modrinth/utils'

const generatedState = useGeneratedState()

type VersionType = 'Release' | 'Alpha' | 'Beta'
const versionType = ref<VersionType>('Release')
const versionNumber = ref('')
const versionSubtitle = ref('')

const selectedLoaders = ref<string[]>([])

const toggleLoader = (loader: string) => {
	if (selectedLoaders.value.includes(loader)) {
		selectedLoaders.value = selectedLoaders.value.filter((l) => l !== loader)
	} else {
		selectedLoaders.value.push(loader)
	}
}

const clearAllLoaders = () => {
	selectedLoaders.value = []
}
</script>
