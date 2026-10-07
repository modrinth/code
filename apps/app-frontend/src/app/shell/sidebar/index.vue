<script setup lang="ts">
import { defineMessages, useScrollIndicator, useVIntl } from '@modrinth/ui'
import { OverlayScrollbars } from 'overlayscrollbars'
import { computed, onMounted, onScopeDispose, ref } from 'vue'

import { useTheme } from '@/composables/use-theme'
import PrideFundraiserBanner from '@/features/campaigns/pride-fundraiser-banner.vue'
import FriendsList from '@/features/friends/friends-list/index.vue'
import AccountsCard from '@/features/minecraft-accounts/accounts-card.vue'
import type { MinecraftAccountsActions } from '@/features/minecraft-accounts/context'
import NewsSidebar from '@/features/news/news-sidebar.vue'
import OnboardingChecklist from '@/features/onboarding/checklist.vue'
import type { ModrinthCredentials } from '@/helpers/mr_auth'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

import Promotion from './promotion.vue'

defineProps<{
	credentials: ModrinthCredentials | null | undefined
	hasPlus: boolean
	showAd: boolean
	visible: boolean
	requestSignIn: () => void
}>()

const emit = defineEmits<{
	'create-instance': []
	'login-modrinth': []
}>()

const { formatMessage } = useVIntl()
const appTheme = useTheme()
const { hasLoggedIntoMinecraft, hasLoggedIntoModrinth, showChecklist } = injectOnboardingChecklist()
const showFriendsList = computed(() => !showChecklist.value || hasLoggedIntoModrinth.value)
const accounts = ref<InstanceType<typeof AccountsCard> | null>(null)
const friends = ref<InstanceType<typeof FriendsList> | null>(null)
const scrollContainer = ref<HTMLElement | null>(null)
const scrollViewport = ref<HTMLElement | null>(null)
const upgradeHeight = ref(0)
const { showTopFade, showBottomFade, checkScrollState } = useScrollIndicator(scrollViewport)

let removeScrollbarListener: (() => void) | undefined

onMounted(() => {
	if (!scrollContainer.value) return
	const scrollbars = OverlayScrollbars(scrollContainer.value)
	if (!scrollbars) return
	scrollViewport.value = scrollbars.elements().viewport
	removeScrollbarListener = scrollbars.on('updated', checkScrollState)
})

onScopeDispose(() => removeScrollbarListener?.())

const minecraftAccounts = computed<MinecraftAccountsActions | null>(() => {
	const card = accounts.value
	if (!card) return null
	return {
		loginDisabled: card.loginDisabled,
		refreshValues: card.refreshValues,
		setEquippedSkin: card.setEquippedSkin,
		setLoginDisabled: card.setLoginDisabled,
		login: card.login,
	}
})

function showAddFriendModal() {
	friends.value?.showAddFriendModal()
}

defineExpose({ minecraftAccounts, showAddFriendModal })

const scrollbarOptions = Object.freeze({
	overflow: {
		x: 'hidden',
		y: 'scroll',
	},
})

const messages = defineMessages({
	playingAs: {
		id: 'app.sidebar.playing-as',
		defaultMessage: 'Playing as',
	},
})
</script>

<template>
	<div
		class="app-sidebar mt-px relative flex h-[calc(100vh-var(--top-bar-height))] w-[300px] shrink-0 flex-col overflow-visible border-0 border-l-[1px] border-solid border-[--brand-gradient-border] before:pointer-events-none before:absolute before:inset-y-0 before:-left-8 before:w-8 before:content-['']"
		:class="{
			'before:shadow-[-15px_0_15px_-15px_rgba(0,0,0,0.1)_inset]': appTheme.advancedRendering,
		}"
	>
		<div class="relative flex min-h-0 grow shrink flex-col">
			<div
				ref="scrollContainer"
				v-overlay-scrollbars="scrollbarOptions"
				class="app-sidebar-scrollable relative min-h-0 grow shrink"
				data-overlayscrollbars-initialize
			>
				<OnboardingChecklist
					@create-instance="emit('create-instance')"
					@login-minecraft="accounts?.login()"
					@login-modrinth="emit('login-modrinth')"
				/>
				<div id="sidebar-teleport-target" class="sidebar-teleport-content contents"></div>
				<div class="sidebar-default-content hidden" :class="{ 'sidebar-enabled': visible }">
					<div
						v-show="hasLoggedIntoMinecraft"
						class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					>
						<h3 class="text-base text-primary font-medium m-0">
							{{ formatMessage(messages.playingAs) }}
						</h3>
						<Suspense>
							<AccountsCard ref="accounts" />
						</Suspense>
					</div>
					<div
						v-show="showFriendsList"
						class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					>
						<Suspense>
							<FriendsList
								ref="friends"
								:credentials="credentials ?? null"
								:sign-in="requestSignIn"
							/>
						</Suspense>
					</div>
					<PrideFundraiserBanner
						class="p-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					/>
					<NewsSidebar
						:style="showAd ? { paddingBottom: `calc(1rem + ${upgradeHeight}px)` } : undefined"
					/>
				</div>
			</div>
			<div
				class="pointer-events-none absolute inset-x-0 top-0 z-10 h-8 bg-gradient-to-b from-[#19211F] to-transparent transition-opacity duration-200 motion-reduce:transition-none"
				:class="showTopFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
			<div
				class="pointer-events-none absolute inset-x-0 bottom-0 z-10 h-8 bg-gradient-to-t from-[#151A1B] to-transparent transition-opacity duration-200 motion-reduce:transition-none"
				:class="showAd || showBottomFade ? 'opacity-100' : 'opacity-0'"
				aria-hidden="true"
			/>
		</div>
		<Promotion v-if="showAd" @upgrade-height="upgradeHeight = $event" />
	</div>
</template>

<style scoped>
.app-sidebar {
	background: var(--brand-gradient-bg);
	--color-button-bg: var(--brand-gradient-button);
	--surface-4: var(--brand-gradient-button);
	--color-button-bg-hover: var(--brand-gradient-border);
	--surface-5: var(--brand-gradient-border);
	--color-divider: var(--brand-gradient-border);
	--color-divider-dark: var(--brand-gradient-border);
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}
</style>
