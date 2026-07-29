<script setup lang="ts">
import { PlusIcon, SearchIcon } from '@modrinth/assets'
import { ButtonStyled, StyledInput } from '@modrinth/ui'
import { inject } from 'vue'

import FilterMenu from '@/components/ui/library/library-toolbar/filter-menu.vue'
import NewGroupModal from '@/components/ui/library/library-toolbar/new-group-modal.vue'
import SortMenu from '@/components/ui/library/library-toolbar/sort-menu.vue'
import { useLibrary } from '@/components/ui/library/use-library'

const { search, openNewGroupModal } = useLibrary()
const showCreationModal = inject<() => void>('showCreationModal')
</script>

<template>
	<div class="flex flex-col gap-2">
		<div class="flex flex-wrap gap-2">
			<StyledInput
				v-model="search"
				:icon="SearchIcon"
				type="text"
				placeholder="Search"
				clearable
				wrapper-class="min-w-[16rem] flex-1"
			/>
			<ButtonStyled>
				<button type="button" @click="openNewGroupModal()">
					<PlusIcon />
					New group
				</button>
			</ButtonStyled>
			<ButtonStyled color="brand">
				<button type="button" @click="showCreationModal?.()">
					<PlusIcon />
					New instance
				</button>
			</ButtonStyled>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<SortMenu />
			<div class="mx-2 h-6 w-px bg-surface-5" />
			<FilterMenu />
		</div>
	</div>
	<NewGroupModal />
</template>
