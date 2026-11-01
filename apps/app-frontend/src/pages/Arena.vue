<script setup lang="ts">
import { CheckIcon, ChevronDownIcon, DownloadIcon, PlayIcon, SpinnerIcon } from '@modrinth/assets'
import { injectNotificationManager } from '@modrinth/ui'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { computed, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'

import { installPvpProfile, type InstallResult, type InstallStage } from '@/features/pvp/installer'
import {
	type PvpGameVersion,
	type PvpProfile,
	profilesForVersion,
} from '@/features/pvp/profiles'
import { createTauriInstallerDeps } from '@/features/pvp/tauri-deps'
import { run } from '@/helpers/instance'
import { instanceKeys, instanceListQueryOptions } from '@/pages/instance/query-options'
import { injectAppEvents } from '@/providers/app-events'
import { useRootBreadcrumb } from '@/shared/breadcrumbs'

defineOptions({ name: 'ArenaPage' })

const router = useRouter()
const queryClient = useQueryClient()
const appEvents = injectAppEvents()
const { addNotification, handleError } = injectNotificationManager()

useRootBreadcrumb({
	slot: 'root',
	id: 'arena',
	label: 'Arena',
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})

const STORE_KEY = 'inferno.pvp.profiles.v1'
const VERSION_KEY = 'inferno.pvp.version.v1'

function readStore(): Record<string, string> {
	try {
		const raw = localStorage.getItem(STORE_KEY)
		const parsed = raw ? JSON.parse(raw) : {}
		return parsed && typeof parsed === 'object' ? parsed : {}
	} catch {
		return {}
	}
}
function writeStore(map: Record<string, string>) {
	try {
		localStorage.setItem(STORE_KEY, JSON.stringify(map))
	} catch {
		/* storage unavailable: the profile still exists, we just cannot re-link it to its card */
	}
}

const links = ref<Record<string, string>>(readStore())

const version = ref<PvpGameVersion>(
	(() => {
		try {
			const v = localStorage.getItem(VERSION_KEY)
			return v === '1.21.10' ? '1.21.10' : '1.21.11'
		} catch {
			return '1.21.11'
		}
	})(),
)
function setVersion(v: PvpGameVersion) {
	version.value = v
	try {
		localStorage.setItem(VERSION_KEY, v)
	} catch {
		/* ignore */
	}
}

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])
const profiles = computed(() => profilesForVersion(version.value))

/** Instance id for a profile, only if that instance still exists. */
function instanceFor(profile: PvpProfile): string | null {
	const id = links.value[profile.id]
	if (!id) return null
	return instances.value.some((i) => i.id === id) ? id : null
}

type Busy = { stage: string; pct: number }
const busy = reactive<Record<string, Busy | undefined>>({})
const results = reactive<Record<string, InstallResult | undefined>>({})
const expanded = reactive<Record<string, boolean>>({})

function stageText(stage: InstallStage): Busy {
	switch (stage.kind) {
		case 'creating':
			return { stage: 'Setting up Fabric…', pct: 4 }
		case 'mod':
			return {
				stage: `Installing ${stage.label} (${stage.index}/${stage.total})`,
				pct: 8 + Math.round(((stage.index - 1) / Math.max(stage.total, 1)) * 82),
			}
		case 'tuning':
			return { stage: 'Tuning Java for low latency…', pct: 92 }
		case 'inferno':
			return { stage: 'Adding Inferno Client…', pct: 96 }
		case 'done':
			return { stage: 'Done', pct: 100 }
	}
}

async function build(profile: PvpProfile) {
	if (busy[profile.id]) return
	busy[profile.id] = { stage: 'Starting…', pct: 2 }
	try {
		const deps = createTauriInstallerDeps(appEvents)
		const result = await installPvpProfile(
			profile,
			instances.value.map((i) => i.name),
			deps,
			(s) => (busy[profile.id] = stageText(s)),
		)
		links.value = { ...links.value, [profile.id]: result.instanceId }
		writeStore(links.value)
		results[profile.id] = result
		await queryClient.invalidateQueries({ queryKey: instanceKeys.list() })

		const warnings = result.outcomes.filter((o) => o.status === 'skipped' && !o.optional)
		const quiet = result.outcomes.filter((o) => o.status === 'skipped' && o.optional)
		if (warnings.length) {
			addNotification({
				type: 'warning',
				title: `${profile.name} is ready, with ${warnings.length} skipped`,
				text: `No compatible build for ${profile.gameVersion}: ${warnings.map((w) => w.label).join(', ')}.`,
			})
		} else {
			addNotification({
				type: 'success',
				title: `${profile.name} is ready`,
				text: quiet.length
					? `${quiet.length} optional mod${quiet.length > 1 ? 's' : ''} had no ${profile.gameVersion} build and ${quiet.length > 1 ? 'were' : 'was'} skipped.`
					: 'All mods installed. Hit Play when you are ready.',
			})
		}
	} catch (e) {
		handleError(e as Error)
	} finally {
		busy[profile.id] = undefined
	}
}

async function play(profile: PvpProfile) {
	const id = instanceFor(profile)
	if (!id) return
	try {
		await run(id)
	} catch (e) {
		handleError(e as Error)
	}
}

function openProfile(profile: PvpProfile) {
	const id = instanceFor(profile)
	if (id) router.push(`/instance/${encodeURIComponent(id)}`)
}

function skippedFor(profile: PvpProfile) {
	return (results[profile.id]?.outcomes ?? []).filter((o) => o.status === 'skipped')
}
</script>

<template>
	<div class="arena">
		<!-- Hero -->
		<header class="hero">
			<div class="hero-copy">
				<p class="eyebrow">Inferno Launcher</p>
				<h1 class="hero-title">Pick a loadout.<br /><span class="ember">Get in the fight.</span></h1>
				<p class="hero-sub">
					One-click PvP profiles with the Inferno Client built in. Every mod here is about
					performance, visibility or quality of life. <strong>No cheats, no hacked clients.</strong>
				</p>
			</div>
			<div class="hero-art" aria-hidden="true">
				<svg viewBox="0 0 120 150" class="flame">
					<defs>
						<linearGradient id="fl" x1="0" y1="1" x2="0" y2="0">
							<stop offset="0" stop-color="#ff3d00" />
							<stop offset="0.55" stop-color="#ff7a1a" />
							<stop offset="1" stop-color="#ffd166" />
						</linearGradient>
					</defs>
					<path
						fill="url(#fl)"
						d="M60 6c6 26 34 38 34 72 0 22-14 38-34 38S26 100 26 80c0-14 8-24 14-32 2 12 8 16 12 16-6-20-2-40 8-58z"
					/>
					<path
						fill="#fff4d6"
						opacity=".85"
						d="M60 70c3 12 16 18 16 34 0 10-7 18-16 18s-16-8-16-18c0-8 4-12 7-16 1 6 4 8 6 8-3-10-1-18 3-26z"
					/>
				</svg>
			</div>
		</header>

		<!-- Version switch -->
		<div class="toolbar">
			<div class="seg" role="tablist" aria-label="Minecraft version">
				<button
					v-for="v in ['1.21.11', '1.21.10'] as PvpGameVersion[]"
					:key="v"
					role="tab"
					:aria-selected="version === v"
					class="seg-btn"
					:class="{ on: version === v }"
					@click="setVersion(v)"
				>
					Minecraft {{ v }}
				</button>
			</div>
			<p class="toolbar-note">
				Inferno Client currently ships for 1.21.10 and 1.21.11, so profiles are built for those
				versions.
			</p>
		</div>

		<!-- Profile grid -->
		<section class="grid">
			<article
				v-for="p in profiles"
				:key="p.id"
				class="card"
				:style="{ '--accent': p.accent }"
				:class="{ ready: !!instanceFor(p) }"
			>
				<div class="card-top">
					<div class="glyph" aria-hidden="true">
						<svg v-if="p.glyph === 'flame'" viewBox="0 0 24 24"><path d="M12 2c1 4 6 6 6 12a6 6 0 0 1-12 0c0-3 2-4 3-6 .5 2 1.5 3 2.5 3C11 8 10.5 5 12 2z" /></svg>
						<svg v-else-if="p.glyph === 'bolt'" viewBox="0 0 24 24"><path d="M13 2 4 14h6l-1 8 9-12h-6z" /></svg>
						<svg v-else-if="p.glyph === 'shield'" viewBox="0 0 24 24"><path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5z" /></svg>
						<svg v-else-if="p.glyph === 'film'" viewBox="0 0 24 24"><path d="M4 4h16v16H4zM7 4v16M17 4v16M4 9h3M4 15h3M17 9h3M17 15h3" fill="none" stroke="currentColor" stroke-width="1.8" /></svg>
						<svg v-else viewBox="0 0 24 24"><path d="M20 4C10 4 5 9 5 16l-1 4 4-1c7 0 12-5 12-15zM8 16l8-8" fill="none" stroke="currentColor" stroke-width="1.8" /></svg>
					</div>
					<div class="titles">
						<h2 class="card-title">{{ p.name }}</h2>
						<p class="card-tag">{{ p.tagline }}</p>
					</div>
					<span v-if="instanceFor(p)" class="installed"><CheckIcon /> Installed</span>
				</div>

				<p class="card-desc">{{ p.description }}</p>

				<div class="chips">
					<span v-for="t in p.tags" :key="t" class="chip">{{ t }}</span>
					<span class="chip chip-inferno">+ Inferno Client</span>
				</div>

				<button class="more" :aria-expanded="!!expanded[p.id]" @click="expanded[p.id] = !expanded[p.id]">
					{{ p.mods.length + 1 }} mods
					<ChevronDownIcon class="chev" :class="{ flip: expanded[p.id] }" />
				</button>
				<ul v-if="expanded[p.id]" class="mods">
					<li class="mod mod-inferno">
						<strong>Inferno Client</strong>
						<span>Zoom, freelook, toggle sprint, fullbright, CPS &amp; armour HUD, elytra swap.</span>
					</li>
					<li v-for="m in p.mods" :key="m.slug" class="mod">
						<strong>{{ m.label }}</strong>
						<span>{{ m.why }}</span>
					</li>
					<li class="mod mod-dep"><strong>Fabric API</strong><span>Added automatically as a dependency.</span></li>
				</ul>

				<p v-if="p.note" class="note">{{ p.note }}</p>

				<ul v-if="skippedFor(p).length" class="skipped">
					<li v-for="s in skippedFor(p)" :key="s.slug">
						Skipped <strong>{{ s.label }}</strong> - no {{ p.gameVersion }} build found.
					</li>
				</ul>

				<div class="card-actions">
					<template v-if="busy[p.id]">
						<div class="progress" role="progressbar" :aria-valuenow="busy[p.id]!.pct" aria-valuemin="0" aria-valuemax="100">
							<div class="bar" :style="{ width: busy[p.id]!.pct + '%' }" />
							<span class="progress-text"><SpinnerIcon class="spin" /> {{ busy[p.id]!.stage }}</span>
						</div>
					</template>
					<template v-else-if="instanceFor(p)">
						<button class="btn btn-play" @click="play(p)"><PlayIcon /> Play</button>
						<button class="btn btn-ghost" @click="openProfile(p)">Manage</button>
						<button class="btn btn-ghost" title="Build a fresh copy" @click="build(p)">Rebuild</button>
					</template>
					<template v-else>
						<button class="btn btn-build" @click="build(p)"><DownloadIcon /> Build profile</button>
					</template>
				</div>
			</article>
		</section>

		<footer class="foot">
			<p>
				Server and tournament rules differ. Inferno's Freelook and Fullbright are accepted on many
				servers but not all - check the rules of where you play.
			</p>
		</footer>
	</div>
</template>

<style scoped lang="scss">
.arena {
	--accent: var(--color-brand);
	padding: 1.5rem 1.75rem 3rem;
	display: flex;
	flex-direction: column;
	gap: 1.5rem;
	max-width: 78rem;
	margin: 0 auto;
}

.hero {
	position: relative;
	overflow: hidden;
	display: flex;
	justify-content: space-between;
	align-items: center;
	gap: 2rem;
	padding: 2.25rem 2.5rem;
	border-radius: var(--radius-lg);
	border: 1px solid rgba(255, 120, 60, 0.25);
	background:
		radial-gradient(120% 140% at 100% 100%, rgba(255, 90, 31, 0.35) 0%, transparent 55%),
		radial-gradient(80% 100% at 0% 0%, rgba(255, 179, 71, 0.12) 0%, transparent 60%),
		linear-gradient(135deg, var(--surface-3), var(--surface-1-5));
	box-shadow: var(--inferno-glow);
}
.eyebrow {
	margin: 0 0 0.5rem;
	font-family: var(--font-display);
	font-size: 0.8rem;
	font-weight: 700;
	letter-spacing: 0.28em;
	text-transform: uppercase;
	color: var(--color-brand);
}
.hero-title {
	margin: 0;
	font-size: clamp(2rem, 4.2vw, 3.1rem);
	line-height: 1.02;
	font-weight: 800;
	text-transform: uppercase;
	color: var(--color-contrast);
}
.ember {
	background: linear-gradient(90deg, #ff5a1f, #ffb347);
	-webkit-background-clip: text;
	background-clip: text;
	color: transparent;
}
.hero-sub {
	margin: 1rem 0 0;
	max-width: 34rem;
	font-size: 1rem;
	line-height: 1.5;
	color: var(--color-base);
	strong {
		color: var(--color-contrast);
	}
}
.hero-art {
	flex: none;
	width: 8.5rem;
	filter: drop-shadow(0 0 28px rgba(255, 90, 31, 0.55));
	.flame {
		width: 100%;
		height: auto;
		animation: flicker 2.6s ease-in-out infinite;
		transform-origin: 50% 90%;
	}
}
@keyframes flicker {
	0%,
	100% {
		transform: scale(1) rotate(-1deg);
	}
	40% {
		transform: scale(1.04, 1.07) rotate(1.2deg);
	}
	70% {
		transform: scale(0.98, 1.03) rotate(-0.6deg);
	}
}
@media (prefers-reduced-motion: reduce) {
	.hero-art .flame {
		animation: none;
	}
}

.toolbar {
	display: flex;
	align-items: center;
	flex-wrap: wrap;
	gap: 1rem;
}
.seg {
	display: inline-flex;
	padding: 3px;
	gap: 3px;
	background: var(--surface-2);
	border: 1px solid var(--surface-5);
	border-radius: var(--radius-md);
}
.seg-btn {
	border: 0;
	cursor: pointer;
	padding: 0.45rem 0.95rem;
	border-radius: var(--radius-sm);
	background: transparent;
	color: var(--color-base);
	font-family: var(--font-display);
	font-weight: 700;
	letter-spacing: 0.04em;
	&:hover {
		color: var(--color-contrast);
	}
	&.on {
		background: var(--color-brand);
		color: #fff;
	}
}
.toolbar-note {
	margin: 0;
	font-size: 0.85rem;
	color: var(--color-secondary);
}

.grid {
	display: grid;
	grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
	gap: 1.1rem;
}
.card {
	position: relative;
	display: flex;
	flex-direction: column;
	gap: 0.8rem;
	padding: 1.15rem 1.2rem 1.2rem;
	border-radius: var(--radius-lg);
	background: linear-gradient(180deg, var(--surface-3), var(--surface-2));
	border: 1px solid var(--surface-5);
	overflow: hidden;
	transition:
		transform 0.18s ease,
		border-color 0.18s ease,
		box-shadow 0.18s ease;
	&::before {
		content: '';
		position: absolute;
		inset: 0 0 auto 0;
		height: 3px;
		background: var(--accent);
		box-shadow: 0 0 18px var(--accent);
	}
	&:hover {
		transform: translateY(-2px);
		border-color: color-mix(in srgb, var(--accent) 55%, var(--surface-5));
		box-shadow: 0 10px 30px -14px var(--accent);
	}
	&.ready {
		border-color: color-mix(in srgb, var(--accent) 40%, var(--surface-5));
	}
}
.card-top {
	display: flex;
	align-items: center;
	gap: 0.8rem;
}
.glyph {
	flex: none;
	display: grid;
	place-items: center;
	width: 2.6rem;
	height: 2.6rem;
	border-radius: var(--radius-md);
	color: var(--accent);
	background: color-mix(in srgb, var(--accent) 16%, transparent);
	border: 1px solid color-mix(in srgb, var(--accent) 40%, transparent);
	svg {
		width: 1.4rem;
		height: 1.4rem;
		fill: currentColor;
	}
}
.titles {
	min-width: 0;
	flex: 1;
}
.card-title {
	margin: 0;
	font-size: 1.15rem;
	font-weight: 800;
	text-transform: uppercase;
	letter-spacing: 0.03em;
	color: var(--color-contrast);
}
.card-tag {
	margin: 0.1rem 0 0;
	font-size: 0.82rem;
	color: var(--accent);
	font-weight: 600;
}
.installed {
	display: inline-flex;
	align-items: center;
	gap: 0.2rem;
	font-size: 0.72rem;
	font-weight: 700;
	text-transform: uppercase;
	letter-spacing: 0.06em;
	padding: 0.2rem 0.5rem;
	border-radius: var(--radius-sm);
	color: var(--color-green);
	background: var(--color-green-bg);
	svg {
		width: 0.85rem;
		height: 0.85rem;
	}
}
.card-desc {
	margin: 0;
	font-size: 0.92rem;
	line-height: 1.45;
	color: var(--color-base);
}
.chips {
	display: flex;
	flex-wrap: wrap;
	gap: 0.35rem;
}
.chip {
	font-size: 0.72rem;
	font-weight: 600;
	padding: 0.18rem 0.5rem;
	border-radius: var(--radius-sm);
	background: var(--surface-4);
	color: var(--color-base);
}
.chip-inferno {
	background: var(--color-brand-highlight);
	color: var(--color-brand);
}
.more {
	align-self: flex-start;
	display: inline-flex;
	align-items: center;
	gap: 0.3rem;
	background: none;
	border: 0;
	padding: 0;
	cursor: pointer;
	font-weight: 700;
	font-size: 0.85rem;
	color: var(--color-base);
	&:hover {
		color: var(--color-contrast);
	}
	.chev {
		width: 1rem;
		height: 1rem;
		transition: transform 0.18s ease;
		&.flip {
			transform: rotate(180deg);
		}
	}
}
.mods {
	list-style: none;
	margin: 0;
	padding: 0.2rem 0 0;
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
}
.mod {
	display: flex;
	flex-direction: column;
	padding-left: 0.65rem;
	border-left: 2px solid var(--surface-5);
	font-size: 0.82rem;
	color: var(--color-secondary);
	strong {
		color: var(--color-contrast);
		font-size: 0.86rem;
	}
	&.mod-inferno {
		border-left-color: var(--color-brand);
	}
	&.mod-dep {
		opacity: 0.8;
	}
}
.note {
	margin: 0;
	font-size: 0.78rem;
	line-height: 1.4;
	color: var(--color-secondary);
	padding: 0.5rem 0.65rem;
	border-radius: var(--radius-sm);
	background: var(--surface-1-5);
	border: 1px dashed var(--surface-5);
}
.skipped {
	margin: 0;
	padding-left: 1.1rem;
	font-size: 0.8rem;
	color: var(--color-orange);
}
.card-actions {
	margin-top: auto;
	display: flex;
	gap: 0.5rem;
	align-items: stretch;
	padding-top: 0.2rem;
}
.btn {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 0.4rem;
	border: 0;
	cursor: pointer;
	height: 2.6rem;
	padding: 0 1rem;
	border-radius: var(--radius-md);
	font-family: var(--font-display);
	font-weight: 800;
	letter-spacing: 0.06em;
	text-transform: uppercase;
	color: #fff;
	transition:
		filter 0.15s ease,
		transform 0.1s ease;
	svg {
		width: 1.1rem;
		height: 1.1rem;
	}
	&:hover {
		filter: brightness(1.12);
	}
	&:active {
		transform: translateY(1px);
	}
}
.btn-build,
.btn-play {
	flex: 1;
	background: linear-gradient(180deg, #ff6a2f, #e8460e);
	box-shadow: 0 6px 18px -8px #ff5a1f;
}
.btn-ghost {
	background: var(--surface-4);
	color: var(--color-base);
	&:hover {
		color: var(--color-contrast);
	}
}
.progress {
	position: relative;
	flex: 1;
	height: 2.6rem;
	border-radius: var(--radius-md);
	background: var(--surface-4);
	overflow: hidden;
}
.bar {
	position: absolute;
	inset: 0 auto 0 0;
	background: linear-gradient(90deg, rgba(255, 90, 31, 0.55), rgba(255, 179, 71, 0.55));
	transition: width 0.35s ease;
}
.progress-text {
	position: relative;
	display: flex;
	align-items: center;
	gap: 0.5rem;
	height: 100%;
	padding: 0 0.85rem;
	font-size: 0.82rem;
	font-weight: 600;
	color: var(--color-contrast);
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}
.spin {
	width: 1rem;
	height: 1rem;
	flex: none;
	animation: spin 0.9s linear infinite;
}
@keyframes spin {
	to {
		transform: rotate(360deg);
	}
}
.foot {
	font-size: 0.8rem;
	color: var(--color-secondary);
	p {
		margin: 0;
	}
}

@media (max-width: 900px) {
	.hero-art {
		display: none;
	}
}
</style>
