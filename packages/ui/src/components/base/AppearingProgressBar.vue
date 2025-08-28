<template>
	<Transition
		enter-active-class="transition-all duration-300 ease-out"
		enter-from-class="opacity-0 max-h-0"
		enter-to-class="opacity-100 max-h-20"
		leave-active-class="transition-all duration-200 ease-in"
		leave-from-class="opacity-100 max-h-20"
		leave-to-class="opacity-0 max-h-0"
	>
		<div v-if="isVisible" class="w-full">
			<div class="mb-2 flex justify-between text-sm">
				<Transition name="phrase-fade" mode="out-in">
					<span :key="currentPhrase" class="text-md font-semibold">{{ currentPhrase }}</span>
				</Transition>
				<div class="flex flex-col items-end">
					<span class="text-secondary">{{ Math.round(progress) }}%</span>
					<span class="text-xs text-secondary"
						>{{ formatBytes(currentValue) }} / {{ formatBytes(maxValue) }}</span
					>
				</div>
			</div>
			<div class="h-2 w-full rounded-full bg-divider">
				<div
					class="h-2 animate-pulse bg-brand rounded-full transition-all duration-300 ease-out"
					:style="{ width: `${progress}%` }"
				></div>
			</div>
		</div>
	</Transition>
</template>

<script setup lang="ts">
import { formatBytes } from '@modrinth/utils'
import { computed, onUnmounted, ref, watch } from 'vue'

interface Props {
	maxValue: number
	currentValue: number
	tips?: string[]
}

const props = withDefaults(defineProps<Props>(), {
	tips: () => [
		'Removing Herobrine...',
		'Feeding parrots...',
		'Teaching villagers new trades...',
		'Convincing creepers to be friendly...',
		'Polishing diamonds...',
		'Training wolves to fetch...',
		'Building pixel art...',
		'Explaining redstone to beginners...',
		'Collecting all the cats...',
		'Negotiating with endermen...',
		'Planting suspicious stew ingredients...',
		'Calibrating TNT blast radius...',
		'Teaching chickens to fly...',
		'Sorting inventory alphabetically...',
		'Convincing iron golems to smile...',
	],
})

const currentPhrase = ref('')
const usedPhrases = ref(new Set<number>())
let phraseInterval: NodeJS.Timeout | null = null

const progress = computed(() => {
	if (props.maxValue === 0) return 0
	return Math.min((props.currentValue / props.maxValue) * 100, 100)
})

const isVisible = computed(() => props.maxValue > 0 && props.currentValue >= 0)

function getNextPhrase() {
	if (usedPhrases.value.size >= props.tips.length) {
		const currentPhraseIndex = props.tips.indexOf(currentPhrase.value)
		usedPhrases.value.clear()
		if (currentPhraseIndex !== -1) {
			usedPhrases.value.add(currentPhraseIndex)
		}
	}
	const availableIndices = props.tips
		.map((_, index) => index)
		.filter((index) => !usedPhrases.value.has(index))

	const randomIndex = availableIndices[Math.floor(Math.random() * availableIndices.length)]
	usedPhrases.value.add(randomIndex)

	return props.tips[randomIndex]
}

function startPhraseRotation() {
	if (phraseInterval) {
		clearInterval(phraseInterval)
	}

	currentPhrase.value = getNextPhrase()
	phraseInterval = setInterval(() => {
		currentPhrase.value = getNextPhrase()
	}, 4500)
}

function stopPhraseRotation() {
	if (phraseInterval) {
		clearInterval(phraseInterval)
		phraseInterval = null
	}
}

watch(isVisible, (newVisible) => {
	if (newVisible) {
		startPhraseRotation()
	} else {
		stopPhraseRotation()
		usedPhrases.value.clear()
	}
})

watch(progress, (newProgress) => {
	if (newProgress >= 100) {
		stopPhraseRotation()
		currentPhrase.value = 'Installing modpack...'
	}
})

onUnmounted(() => {
	stopPhraseRotation()
})
</script>

<style scoped>
.phrase-fade-enter-active,
.phrase-fade-leave-active {
	transition: opacity 0.3s ease;
}

.phrase-fade-enter-from,
.phrase-fade-leave-to {
	opacity: 0;
}
</style>
