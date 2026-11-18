<script setup lang="ts">
import { ScaleIcon } from '@modrinth/assets'
import { Button, injectProjectPageContext, NewModal } from '@modrinth/ui'
import { isAdmin, isStaff, TeamMemberPermission } from '@modrinth/utils'
import { useMutation } from '@tanstack/vue-query'
import { computed, ref, useTemplateRef } from 'vue'

const { projectV3: project, currentMember, patchProjectV3 } = injectProjectPageContext()
const flags = useFeatureFlags()
const modal = useTemplateRef('modal')
const mode = ref<'member' | 'moderator'>('member')
const monetized = computed(() => project.value.monetization_status === 'monetized')
const forceDisabled = computed(() => project.value.monetization_status === 'force-demonetized')
const moderatorUi = computed(
	() => isStaff(currentMember.value?.user) && !flags.value.showModeratorProjectMemberUi,
)
const canEdit = computed(
	() =>
		isAdmin(currentMember.value?.user) ||
		Boolean((currentMember.value?.permissions ?? 0) & TeamMemberPermission.EDIT_DETAILS),
)
const memberAction = computed(() =>
	monetized.value ? 'Disable monetization' : 'Enable monetization',
)
const moderatorAction = computed(() =>
	forceDisabled.value ? 'Allow monetization' : 'Force disable',
)
const action = computed(() =>
	mode.value === 'moderator' ? moderatorAction.value : memberAction.value,
)
const canConfirm = computed(() =>
	mode.value === 'moderator' ? moderatorUi.value : canEdit.value && !forceDisabled.value,
)
const mutation = useMutation({
	mutationFn: async () => {
		if (!canConfirm.value) throw new Error('You do not have permission to change monetization.')
		const monetization_status =
			mode.value === 'moderator'
				? forceDisabled.value
					? 'monetized'
					: 'force-demonetized'
				: monetized.value
					? 'demonetized'
					: 'monetized'
		await patchProjectV3({ monetization_status })
	},
	onSuccess: () => modal.value?.hide(),
})
function show(nextMode: 'member' | 'moderator', event: MouseEvent) {
	mode.value = nextMode
	mutation.reset()
	modal.value?.show(event)
}
</script>

<template>
	<div class="danger-zone-row">
		<div>
			<h3 class="mb-1 mt-0 text-base font-semibold text-contrast">{{ memberAction }}</h3>
			<p class="m-0 text-sm leading-relaxed text-secondary">
				Projects on Modrinth are automatically enrolled in the
				<NuxtLink to="/legal/cmp-info" target="_blank" class="text-link">Rewards Program</NuxtLink>.
				If you don’t want to (or can’t for legal reasons) earn revenue from this project, you can
				disable rewards here.
			</p>
			<p v-if="forceDisabled" class="mb-0 mt-2 text-sm text-secondary">
				Monetization has been disabled by a moderator.
				<a
					href="https://support.modrinth.com"
					target="_blank"
					rel="noopener noreferrer"
					class="text-link"
					>Contact support</a
				>
				if you believe this is a mistake.
			</p>
		</div>
		<div class="flex shrink-0 flex-wrap items-center gap-2">
			<Button
				v-if="moderatorUi"
				size="sm"
				type="outlined"
				color="orange"
				:disabled="mutation.isPending.value"
				@click="show('moderator', $event)"
				><ScaleIcon aria-hidden="true" />{{ moderatorAction }}</Button
			>
			<Button
				size="sm"
				type="colored"
				:color="monetized ? 'red' : 'brand'"
				:disabled="!canEdit || forceDisabled || mutation.isPending.value"
				@click="show('member', $event)"
				>{{ memberAction }}</Button
			>
		</div>
		<NewModal ref="modal" :header="`${action}?`" width="500px">
			<div class="flex flex-col gap-3 leading-normal">
				<template v-if="mode === 'moderator' && !forceDisabled">
					<p class="m-0">
						<strong>{{ project.name }}</strong> will stop earning revenue from the Rewards Program.
						Earnings already made are not affected.
					</p>
					<p class="m-0 text-secondary">
						Project members will not be able to re-enable monetization. Only a moderator can allow
						it again.
					</p>
				</template>
				<template v-else-if="mode === 'moderator'">
					<p class="m-0">
						Allow <strong>{{ project.name }}</strong> to earn revenue from the Rewards Program
						again.
					</p>
					<p class="m-0 text-secondary">
						This removes the moderation restriction. Project members will be able to manage
						monetization.
					</p>
				</template>
				<template v-else-if="monetized">
					<p class="m-0">
						<strong>{{ project.name }}</strong> will stop earning revenue from the Rewards Program.
						Earnings you’ve already made are not affected.
					</p>
					<p class="m-0 text-secondary">You can turn monetization back on at any time.</p>
				</template>
				<p v-else class="m-0">
					<strong>{{ project.name }}</strong> will start earning revenue through the Rewards
					Program.
				</p>
				<p v-if="mutation.isError.value" role="alert" class="m-0 text-red">
					{{ mutation.error.value?.message }}
				</p>
			</div>
			<template #actions>
				<div class="flex justify-end gap-2">
					<Button :disabled="mutation.isPending.value" @click="modal?.hide()">Cancel</Button
					><Button
						type="colored"
						:color="monetized || (mode === 'moderator' && !forceDisabled) ? 'red' : 'brand'"
						:disabled="!canConfirm"
						:loading="mutation.isPending.value"
						@click="mutation.mutate()"
						>{{ action }}</Button
					>
				</div>
			</template>
		</NewModal>
	</div>
</template>
