<template>
	<div class="flex flex-col gap-2 border-0 border-b border-solid border-divider pb-4">
		<div class="grid grid-cols-[1fr_auto] gap-y-6">
			<div class="flex gap-4 w-full">
				<slot name="icon" />
				<div class="flex flex-col gap-2 justify-center w-full">
					<div class="flex justify-between items-start gap-2">
						<div class="flex flex-col gap-1.5 justify-center">
							<div class="flex flex-wrap items-center gap-2">
								<h1 class="m-0 text-2xl font-semibold leading-none text-contrast">
									<slot name="title" />
								</h1>
								<slot name="title-suffix" />
							</div>
							<p
								v-if="$slots.summary"
								class="m-0 max-w-[40rem] empty:hidden"
								:class="[disableLineClamp ? '' : 'line-clamp-2']"
							>
								<slot name="summary" />
							</p>
						</div>
						<div v-if="$slots.summary" class="flex gap-2 items-start max-md:hidden">
							<slot name="actions" />
						</div>
					</div>
					<div v-if="$slots.stats" class="flex flex-wrap gap-3 empty:hidden max-md:hidden">
						<slot name="stats" />
					</div>
				</div>
			</div>
			<div v-if="!$slots.summary" class="flex gap-2 items-start max-md:hidden">
				<slot name="actions" />
			</div>
		</div>
		<div class="flex justify-between">
			<div v-if="$slots.stats" class="flex flex-wrap gap-3 empty:hidden md:hidden">
				<slot name="stats" />
			</div>
			<div class="flex gap-2 items-start self-end md:hidden">
				<slot name="actions" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
interface Props {
	disableLineClamp?: boolean
}

const { disableLineClamp } = defineProps<Props>()
</script>
