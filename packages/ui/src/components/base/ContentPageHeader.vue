<template>
	<div class="flex flex-col gap-2 border-0 border-b border-solid border-divider pb-4">
		<div class="flex flex-wrap items-start gap-4 max-md:flex-col">
			<div class="flex min-w-0 flex-1 gap-4">
				<slot name="icon" />
				<div class="flex min-w-0 flex-col gap-2 justify-center">
					<div class="flex flex-col gap-1.5 justify-center">
						<div class="flex flex-wrap items-center gap-2">
							<h1 class="m-0 text-2xl font-semibold leading-none text-contrast">
								<slot name="title" />
							</h1>
							<slot name="title-suffix" />
						</div>
						<p
							v-if="$slots.summary"
							ref="summary"
							class="m-0 max-w-[44rem] empty:hidden"
							:class="[expandBio || disableLineClamp ? '' : 'line-clamp-2']"
						>
							<slot name="summary" />
						</p>
						<button
							v-if="showToggle"
							@click="expandBio=!expandBio"
							class="text-left underline text-gray-900"
						>
							<b>
								<span v-if="expandBio"><slot name="bio-collapse" /></span>
								<span v-else><slot name="bio-expand" /></span>
							</b>
						</button>
					</div>
					<div v-if="$slots.stats" class="flex flex-wrap gap-3 empty:hidden max-md:hidden">
						<slot name="stats" />
					</div>
				</div>
			</div>
			<div class="flex flex-wrap gap-2 items-center">
				<slot name="actions" />
			</div>
		</div>
		<div v-if="$slots.stats" class="flex justify-between md:hidden">
			<div class="flex flex-wrap gap-3 empty:hidden">
				<slot name="stats" />
			</div>
		</div>
	</div>
</template>

<script setup lang="ts">
interface Props {
	disableLineClamp?: boolean
}

const props = defineProps<Props>()
const expandBio = ref(false)
const summary = ref<HTMLElement | null>(null)
const isCutOff = ref(false)

const showToggle = computed(() => {
	if (props.disableLineClamp) return false
	return isCutOff.value
})

function measureCutOff() {
	const el = summary.value
	if (!el) return
	isCutOff.value = el.scrollHeight > el.clientHeight + 1
}

let ro: ResizeObserver | undefined

onMounted(async () => {
	await nextTick()
	measureCutOff()

	window.addEventListener("resize", function (e) {
		expandBio.value = false
		console.log("meow")
		measureCutOff()
	})
})

onBeforeUnmount(() => {
	expandBio.value = false
	ro?.disconnect()
})

</script>
