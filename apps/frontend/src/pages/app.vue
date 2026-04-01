<script setup lang="ts">
import { ButtonStyled } from '@modrinth/ui'

interface LauncherPlatform {
	install_urls: string[]
}

interface LauncherUpdates {
	platforms: {
		'darwin-aarch64': LauncherPlatform
		'windows-x86_64': LauncherPlatform
		'linux-x86_64': LauncherPlatform
	}
}

type OSType = 'Mac' | 'Windows' | 'Linux' | null

const downloadWindows = ref<HTMLAnchorElement | null>(null)
const downloadMac = ref<HTMLAnchorElement | null>(null)

const linuxLinks = reactive({
	appImage: null as string | null,
	deb: null as string | null,
	rpm: null as string | null,
})

const macLinks = reactive({
	universal: null as string | null,
})

const { data: launcherUpdates } = await useFetch<LauncherUpdates>(
	'https://launcher-files.modrinth.com/updates.json?new',
	{
		server: false,
		getCachedData(key, nuxtApp) {
			const cached = (nuxtApp.ssrContext?.cache as any)?.[key] || nuxtApp.payload.data[key]
			if (!cached) return

			const now = Date.now()
			const cacheTime = cached._cacheTime || 0
			const maxAge = 5 * 60 * 1000

			if (now - cacheTime > maxAge) {
				return null
			}

			return cached
		},
		transform(data) {
			return {
				...data,
				_cacheTime: Date.now(),
			}
		},
	},
)

const windowsLink = ref<string | null>(null)

const platform = computed<string>(() => {
	if (import.meta.server) {
		const headers = useRequestHeaders()
		return headers['user-agent'] || ''
	}
	return navigator.userAgent || ''
})

const os = computed<OSType>(() => {
	if (platform.value.includes('Mac')) return 'Mac'
	if (platform.value.includes('Win')) return 'Windows'
	if (platform.value.includes('Linux')) return 'Linux'
	return null
})

watch(
	launcherUpdates,
	(newData) => {
		if (newData?.platforms) {
			macLinks.universal = newData.platforms['darwin-aarch64']?.install_urls[0] || null
			windowsLink.value = newData.platforms['windows-x86_64']?.install_urls[0] || null
			linuxLinks.appImage = newData.platforms['linux-x86_64']?.install_urls[1] || null
			linuxLinks.deb = newData.platforms['linux-x86_64']?.install_urls[0] || null
			linuxLinks.rpm = newData.platforms['linux-x86_64']?.install_urls[2] || null
		}
	},
	{ immediate: true },
)

function handlePrimaryDownload() {
	if (os.value === 'Windows') {
		downloadWindows.value?.click()
	} else if (os.value === 'Mac') {
		downloadMac.value?.click()
	}
}

const title = 'Download Modrinth App!'
const description =
	'Modrinth App is a unique, open source launcher that allows you to play your favorite mods, and keep them up to date, all in one neat little package.'

useSeoMeta({
	title,
	description,
	ogTitle: title,
	ogDescription: description,
})
</script>

<template>
	<div class="p-4">
		<p class="m-0 border border-solid border-yellow-600 bg-yellow-100 p-3">
			Modrinth's desktop application is currently in beta, if you encounter issues please let us
			know!
		</p>

		<h1 class="mb-2 mt-4 text-3xl font-normal">
			<template v-if="os">Download Modrinth for {{ os }}</template>
			<template v-else>Download Modrinth</template>
		</h1>
		<p class="m-0 text-secondary">
			Modrinth's desktop application is a unique, open source Minecraft launcher that allows you to
			play your favorite mods, and keep them up to date, all in one neat little package.
		</p>
		<p class="mb-0 mt-3 font-bold italic text-secondary opacity-60">
			Your favorite launcher likely supports Modrinth too, such as MultiMC, Prism, or ATLauncher!
		</p>

		<div class="mt-4">
			<template v-if="os === 'Windows' || os === 'Mac'">
				<ButtonStyled color="brand" size="large">
					<button
						type="button"
						class="m-0 flex items-center gap-2 p-0"
						@click="handlePrimaryDownload"
					>
						Download now!
					</button>
				</ButtonStyled>
			</template>
			<template v-else>
				<ButtonStyled color="brand" size="large">
					<a
						class="flex items-center gap-2"
						href="https://flathub.org/en/apps/com.modrinth.ModrinthApp"
						rel="noopener"
						target="_blank"
					>
						Get it on Flathub*
					</a>
				</ButtonStyled>
			</template>
		</div>

		<div class="mt-6 w-fit border border-solid border-[#c3c3c3] bg-[#EFEFEF] p-4">
			<p class="m-0 font-bold">Installation requirements:</p>
			<ul class="mb-0 mt-2 pl-6">
				<li>A PC or Mac with a recent 64-bit processor</li>
				<li>An internet connection to sign in and download content</li>
				<li>A purchased premium copy of Minecraft</li>
			</ul>
		</div>

		<div
			id="all-downloads"
			class="mt-4 flex w-fit flex-col gap-2 border border-solid border-[#96CEE0] bg-[#E6F1F5] p-4"
		>
			<p class="m-0 mb-3 font-bold">All installers</p>
			<a :href="windowsLink || undefined" class="w-fit text-link" download=""
				>Download for Windows (64-bit)</a
			>
			<a :href="windowsLink || undefined" class="w-fit text-link" download="">Download for OS X</a>

			<a
				class="w-fit text-link"
				href="https://flathub.org/en/apps/com.modrinth.ModrinthApp"
				rel="noopener"
				target="_blank"
			>
				Download for Linux on Flathub*
			</a>
			<details class="mt-1">
				<summary class="cursor-pointer text-link">Other Linux package formats (advanced)</summary>
				<div class="mt-2 flex flex-col gap-2 border border-dashed border-black p-4">
					<a
						:href="linuxLinks.appImage || undefined"
						class="inline-flex w-fit items-center gap-1 text-link"
						download=""
					>
						Download the AppImage
					</a>
					<a
						:href="linuxLinks.deb || undefined"
						class="inline-flex w-fit items-center gap-1 text-link"
						download=""
					>
						Download the DEB
					</a>
					<a
						:href="linuxLinks.rpm || undefined"
						class="inline-flex w-fit items-center gap-1 text-link"
						download=""
					>
						Download the RPM
					</a>
				</div>
			</details>
		</div>

		<p class="mt-4">
			By downloading Modrinth App, you agree to our
			<nuxt-link class="text-link" to="/legal/terms">Terms</nuxt-link>
			and
			<nuxt-link class="text-link" to="/legal/privacy">Privacy Policy</nuxt-link>.
		</p>
		<p class="mt-2 max-w-xl text-secondary">
			*The Linux versions of Modrinth App are
			<a
				class="text-link"
				href="https://github.com/modrinth/code/issues/3057"
				rel="noopener"
				target="_blank"
			>
				known to have issues
			</a>
			on certain systems and configurations. If Modrinth App is unstable on your system, we
			encourage you to try other apps like
			<a class="text-link" href="https://prismlauncher.org/" rel="noopener" target="_blank"
				>Prism Launcher</a
			>
			to easily install Modrinth content.
		</p>
	</div>
</template>
