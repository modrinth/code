<template>
	<div class="space-y-6 sm:w-[512px]">
		<LoaderPicker
			v-model="draftVersion.loaders"
			:loaders="generatedState.loaders"
			:toggle-loader="toggleLoader"
		/>

		<div v-if="draftVersion.loaders.length" class="space-y-1">
			<div class="flex items-center justify-between">
				<span class="font-semibold text-contrast"> Added loaders </span>
				<ButtonStyled type="transparent" size="standard">
					<button @click="onClearAll()">Clear all</button>
				</ButtonStyled>
			</div>
			<div
				class="flex flex-col gap-1.5 gap-y-4 rounded-xl border border-solid border-surface-5 p-3 py-4"
			>
				<div class="flex flex-wrap gap-2">
					<template
						v-for="loader in draftVersion.loaders.map((loaderName) =>
							loaders.find((loader) => loaderName === loader.name),
						)"
					>
						<TagItem
							v-if="loader"
							:key="`loader-${loader.name}`"
							:action="() => toggleLoader(loader.name)"
							class="border !border-solid border-surface-5 !transition-all hover:bg-button-bgHover hover:no-underline"
							:style="`--_color: var(--color-platform-${loader.name})`"
						>
							<div v-html="loader.icon"></div>
							{{ formatCategory(loader.name) }}
							<XIcon class="text-secondary" />
						</TagItem>
					</template>
				</div>
			</div>
		</div>
	</div>
</template>

<script lang="ts" setup>
import { XIcon } from '@modrinth/assets'
import { ButtonStyled, TagItem } from '@modrinth/ui'
import { formatCategory } from '@modrinth/utils'

import { injectManageVersionContext } from '~/providers/version/manage-version-modal'

import LoaderPicker from '../components/LoaderPicker.vue'

const generatedState = useGeneratedState()

const loaders = computed(() => generatedState.value.loaders)

const { draftVersion } = injectManageVersionContext()

const toggleLoader = (loader: string) => {
	if (draftVersion.value.loaders.includes(loader)) {
		draftVersion.value.loaders = draftVersion.value.loaders.filter((l) => l !== loader)
	} else {
		draftVersion.value.loaders = [...draftVersion.value.loaders, loader]
	}
}

const onClearAll = () => {
	draftVersion.value.loaders = []
}
</script>
