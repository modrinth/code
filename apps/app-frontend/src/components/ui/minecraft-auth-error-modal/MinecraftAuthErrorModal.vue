<script setup lang="ts">
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { handleSevereError } from '@/store/error.js'
import {
	CheckIcon,
	CopyIcon,
	DropdownIcon,
	LogInIcon,
	MessagesSquareIcon,
	WrenchIcon,
} from '@modrinth/assets'
import { Admonition, ButtonStyled, Collapsible, NewModal } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { type MinecraftAuthError, minecraftAuthErrors } from './minecraft-auth-errors'

const modal = ref<InstanceType<typeof NewModal>>()
const rawError = ref<string>('')
const matchedError = ref<MinecraftAuthError | null>(null)
const debugCollapsed = ref(true)
const copied = ref(false)
const loadingSignIn = ref(false)

function show(errorVal: { message?: string }) {
	rawError.value = errorVal?.message ?? String(errorVal) ?? 'Unknown error'

	matchedError.value = minecraftAuthErrors.find((e) => rawError.value.includes(e.errorCode)) ?? null

	debugCollapsed.value = true
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

defineExpose({
	show,
	hide,
})

async function signInAgain() {
	try {
		loadingSignIn.value = true
		const loggedIn = await login_flow()
		if (loggedIn) {
			await set_default_user(loggedIn.profile.id)
		}
		loadingSignIn.value = false
		modal.value?.hide()
	} catch (err) {
		loadingSignIn.value = false
		handleSevereError(err)
	}
}

const debugInfo = computed(() => rawError.value || 'No error message.')

async function copyToClipboard(text: string) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}

import { onMounted } from 'vue'

onMounted(() => {
	show({
		message: 'Minecraft authentication error: XErr 214891dwad6222 - Account is region locked',
	})
})
</script>

<template>
	<NewModal ref="modal" header="Sign in Failed" :max-width="'548px'">
		<div class="flex flex-col gap-6">
			<Admonition
				type="warning"
				body="	We couldn't sign you into your Microsoft account. This may be due to account restrictions or
				regional limitations."
			>
			</Admonition>

			<!-- Matched error details -->
			<div class="bg-surface-2 rounded-2xl p-4 px-5 flex flex-col gap-3">
				<template v-if="matchedError">
					<div class="flex flex-col gap-1.5">
						<h3 class="text-base font-bold m-0">What we think happened</h3>
						<p class="text-sm text-secondary m-0">
							{{ matchedError.whatHappened }}
						</p>
					</div>

					<div class="flex flex-col gap-1.5">
						<h3 class="text-base font-bold m-0">How to fix it</h3>
						<ol class="list-none flex flex-col gap-2 m-0 pl-0">
							<li
								v-for="(step, index) in matchedError.stepsToFix"
								:key="index"
								class="flex items-baseline gap-2"
							>
								<span
									class="inline-flex items-center justify-center shrink-0 w-5 h-5 rounded-full bg-surface-4 border border-solid border-surface-5 text-xs font-medium"
								>
									{{ index + 1 }}
								</span>
								<!-- eslint-disable-next-line vue/no-v-html -->
								<span
									class="text-sm [&_a]:text-info [&_a]:font-medium [&_a]:underline"
									v-html="step"
								/>
							</li>
						</ol>
					</div>
				</template>
				<template v-else>
					<div class="flex flex-col gap-1.5">
						<h3 class="text-base font-bold m-0">Unknown error</h3>
						<p class="text-sm text-secondary m-0">
							We don’t recognize this error and can’t recommend specific steps to resolve it.
						</p>
						<p class="text-sm text-secondary m-0">
							Try visiting
							<a
								class="text-info font-medium underline hover:underline"
								href="https://www.minecraft.net/en-us/login"
								>Minecraft Login</a
							>
							and signing in, as it may prompt you with the necessary steps. You can also contact
							support and we can look into it further.
						</p>
					</div>
				</template>
			</div>

			<!-- Action buttons -->
			<div class="flex items-center gap-2">
				<ButtonStyled>
					<a href="https://support.modrinth.com" @click="modal?.hide()" class="!w-full">
						<MessagesSquareIcon /> Contact support
					</a>
				</ButtonStyled>
				<ButtonStyled color="brand">
					<button :disabled="loadingSignIn" @click="signInAgain" class="!w-full">
						<LogInIcon /> Sign in again
					</button>
				</ButtonStyled>
			</div>

			<div class="flex flex-col gap-2">
				<div class="w-full h-[1px] bg-surface-5"></div>

				<!-- Debug info -->
				<div class="overflow-clip">
					<button
						class="flex items-center justify-between w-full bg-transparent border-0 py-4 cursor-pointer"
						@click="debugCollapsed = !debugCollapsed"
					>
						<span class="flex items-center gap-2 text-contrast font-extrabold m-0">
							<WrenchIcon class="h-4 w-4" />
							Debug information
						</span>
						<DropdownIcon
							class="h-5 w-5 text-secondary transition-transform"
							:class="{ 'rotate-180': !debugCollapsed }"
						/>
					</button>
					<Collapsible :collapsed="debugCollapsed">
						<div class="p-3 bg-surface-2 rounded-2xl text-xs flex items-start">
							<div class="m-0 p-0 rounded-none bg-transparent text-sm font-mono">
								{{ debugInfo }}
							</div>
							<ButtonStyled circular>
								<button
									:disabled="copied"
									@click="copyToClipboard(debugInfo)"
									v-tooltip="'Copy debug info'"
								>
									<template v-if="copied"> <CheckIcon class="text-green" /> </template>
									<template v-else> <CopyIcon /> </template>
								</button>
							</ButtonStyled>
						</div>
					</Collapsible>
				</div>
			</div>
		</div>
	</NewModal>
</template>
