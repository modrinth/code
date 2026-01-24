<template>
	<div v-if="showEnvironmentMigration" class="card experimental-styles-within">
		<h2 class="m-0 mb-2 block text-lg font-extrabold text-contrast">Project environment</h2>
		<EnvironmentMigration />
	</div>
	<div v-else class="grid place-content-center py-32">
		<div class="flex flex-col items-center gap-5 text-center">
			<div class="flex flex-col gap-2">
				<div class="text-xl font-semibold text-contrast">
					Environments are now managed per version.
				</div>
				<div>Visit Project Settings to manage environments for each version.</div>
			</div>
			<ButtonStyled color="green">
				<nuxt-link
					:to="`/${projectV2.project_type}/${projectV2.id}/settings/versions`"
					class="items flex"
				>
					<SettingsIcon /> Edit versions
				</nuxt-link>
			</ButtonStyled>
		</div>
	</div>
</template>

<script setup lang="ts">
import { SettingsIcon } from '@modrinth/assets'
import { ButtonStyled, EnvironmentMigration, injectProjectPageContext } from '@modrinth/ui'
import { isStaff } from '@modrinth/utils'

const { currentMember, projectV2 } = injectProjectPageContext()

const showEnvironmentMigration = computed(() => {
	return isStaff(currentMember.value?.user)
})
</script>
