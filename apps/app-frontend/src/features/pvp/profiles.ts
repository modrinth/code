/**
 * Inferno Launcher - pre-built PvP profiles.
 *
 * Design rules for everything in this file:
 *  - NO hacked-client / cheat mods (no reach, killaura, autoclickers, velocity, ESP/X-ray, ...).
 *  - Only performance, visibility and quality-of-life mods that are client-side and widely accepted.
 *  - The Inferno Client jar is bundled with the launcher (see apps/app/resources/inferno) and is
 *    added as a local file; every other mod is resolved live from Modrinth at install time, so
 *    each profile always gets the newest build that actually supports its Minecraft version.
 *
 * Mod entries use Modrinth project slugs. If a slug has no compatible build for a profile's
 * Minecraft version (or was renamed) the installer skips that single mod and reports it, instead of
 * failing the whole profile. `optional: true` just changes how loudly the UI reports a skip.
 */

export type PvpGameVersion = '1.21.11' | '1.21.10'

export interface PvpMod {
	/** Modrinth project slug (or id). */
	slug: string
	/** Human label shown on the profile card. */
	label: string
	/** What it does for PvP, one short line. */
	why: string
	/** A skip is reported quietly rather than as a warning. */
	optional?: boolean
}

export interface PvpProfile {
	id: string
	name: string
	tagline: string
	description: string
	gameVersion: PvpGameVersion
	loader: 'fabric'
	/** Accent colour for the card (any CSS colour). */
	accent: string
	/** lucide-ish glyph key resolved in Arena.vue */
	glyph: 'flame' | 'bolt' | 'shield' | 'film' | 'feather'
	tags: string[]
	includeInferno: boolean
	/** Max heap in MB written to the instance's memory setting. */
	memoryMb: number
	mods: PvpMod[]
	/** Honest per-profile caveat shown on the card. */
	note?: string
}

/** Resource paths are relative to the bundled `inferno/` directory (see tauri.conf.json). */
export const INFERNO_JARS: Record<PvpGameVersion, string> = {
	'1.21.11': 'inferno/inferno-client-1.21.11.jar',
	'1.21.10': 'inferno/inferno-client-1.21.10.jar',
}

/**
 * Java launch flags applied to every PvP profile (Java 17/21). A latency-focused G1 setup:
 * short, frequent collections instead of rare long pauses, which is what causes mid-fight stutter.
 */
export const PVP_JVM_ARGS: string[] = [
	'-XX:+UseG1GC',
	'-XX:+ParallelRefProcEnabled',
	'-XX:MaxGCPauseMillis=50',
	'-XX:+UnlockExperimentalVMOptions',
	'-XX:+DisableExplicitGC',
	'-XX:G1NewSizePercent=30',
	'-XX:G1MaxNewSizePercent=40',
	'-XX:G1HeapRegionSize=8M',
	'-XX:G1ReservePercent=20',
	'-XX:G1HeapWastePercent=5',
	'-XX:G1MixedGCCountTarget=4',
	'-XX:InitiatingHeapOccupancyPercent=15',
	'-XX:G1MixedGCLiveThresholdPercent=90',
	'-XX:G1RSetUpdatingPauseTimePercent=5',
	'-XX:SurvivorRatio=32',
	'-XX:MaxTenuringThreshold=1',
	'-XX:+PerfDisableSharedMem',
]

const SODIUM: PvpMod = {
	slug: 'sodium',
	label: 'Sodium',
	why: 'Rewrites the renderer for a big FPS and frame-time boost.',
}
const LITHIUM: PvpMod = {
	slug: 'lithium',
	label: 'Lithium',
	why: 'Game-logic optimisations; smoother ticks, no gameplay changes.',
}
const MODERNFIX: PvpMod = {
	slug: 'modernfix',
	label: 'ModernFix',
	why: 'Faster startup, lower memory use, fewer stutters.',
	optional: true,
}
const FERRITECORE: PvpMod = {
	slug: 'ferrite-core',
	label: 'FerriteCore',
	why: 'Cuts memory use so long sessions stay smooth.',
	optional: true,
}
const ENTITY_CULLING: PvpMod = {
	slug: 'entityculling',
	label: 'Entity Culling',
	why: 'Skips rendering what you cannot see; helps in crowded fights.',
}
const IMMEDIATELYFAST: PvpMod = {
	slug: 'immediatelyfast',
	label: 'ImmediatelyFast',
	why: 'Faster HUD, text and particle drawing.',
	optional: true,
}
const MORE_CULLING: PvpMod = {
	slug: 'moreculling',
	label: 'More Culling',
	why: 'Extra occlusion culling on top of vanilla.',
	optional: true,
}
const SODIUM_EXTRA: PvpMod = {
	slug: 'sodium-extra',
	label: 'Sodium Extra',
	why: 'Extra video options: fine control over particles, fog and more.',
	optional: true,
}
const REESES_OPTIONS: PvpMod = {
	slug: 'reeses-sodium-options',
	label: "Reese's Sodium Options",
	why: 'A cleaner, single-page video settings screen.',
	optional: true,
}
const SCALABLELUX: PvpMod = {
	slug: 'scalablelux',
	label: 'ScalableLux',
	why: 'Much faster lighting engine; fewer lag spikes when blocks change.',
	optional: true,
}
const KRYPTON: PvpMod = {
	slug: 'krypton',
	label: 'Krypton',
	why: 'Optimises the network stack for steadier ping handling.',
	optional: true,
}
const CULL_LESS_LEAVES: PvpMod = {
	slug: 'cull-less-leaves',
	label: 'Cull Less Leaves',
	why: 'Faster leaf rendering while keeping them looking right.',
	optional: true,
}
const ENHANCED_BLOCK_ENTITIES: PvpMod = {
	slug: 'enhanced-block-entities',
	label: 'Enhanced Block Entities',
	why: 'Chests, signs and beds render much faster.',
	optional: true,
}
const BAD_OPTIMIZATIONS: PvpMod = {
	slug: 'badoptimizations',
	label: 'BadOptimizations',
	why: 'Many small, safe client-side speed-ups.',
	optional: true,
}
const MEMORY_LEAK_FIX: PvpMod = {
	slug: 'memoryleakfix',
	label: 'Memory Leak Fix',
	why: 'Stops memory creeping up over long sessions.',
	optional: true,
}
const KSYXIS: PvpMod = {
	slug: 'ksyxis',
	label: 'Ksyxis',
	why: 'Faster world loading.',
	optional: true,
}
const DYNAMIC_FPS: PvpMod = {
	slug: 'dynamic-fps',
	label: 'Dynamic FPS',
	why: 'Drops resource use while the window is in the background.',
	optional: true,
}
const MOD_MENU: PvpMod = {
	slug: 'modmenu',
	label: 'Mod Menu',
	why: 'See and configure all your mods in one screen.',
	optional: true,
}
const APPLESKIN: PvpMod = {
	slug: 'appleskin',
	label: 'AppleSkin',
	why: 'Shows exactly how much hunger and saturation food restores; plan your golden apples.',
	optional: true,
}
const MOUSE_TWEAKS: PvpMod = {
	slug: 'mouse-tweaks',
	label: 'Mouse Tweaks',
	why: 'Faster inventory moves, handy when re-kitting mid-fight.',
	optional: true,
}
const BETTERF3: PvpMod = {
	slug: 'betterf3',
	label: 'BetterF3',
	why: 'A cleaner, colour-coded F3 debug screen.',
	optional: true,
}
const STATUS_EFFECT_BARS: PvpMod = {
	slug: 'status-effect-bars',
	label: 'Status Effect Bars',
	why: 'Clear timers on potion effects so you know when they run out.',
	optional: true,
}
const LAMBDYNAMICLIGHTS: PvpMod = {
	slug: 'lambdynamiclights',
	label: 'LambDynamicLights',
	why: 'Held torches and lava light up your surroundings in dark arenas.',
	optional: true,
}
const REPLAY_MOD: PvpMod = {
	slug: 'replay-mod',
	label: 'Replay Mod',
	why: 'Record and re-film your fights from any camera angle.',
	optional: true,
}

const INFERNO_NOTE =
	'Inferno includes Freelook and Fullbright. Some servers ban those in PvP - turn them off in the Inferno menu before joining servers with strict rules.'

function build(version: PvpGameVersion): PvpProfile[] {
	const v = version
	const suffix = v === '1.21.11' ? '' : ` (${v})`
	const idSuffix = v === '1.21.11' ? '' : `-${v.replace(/\./g, '')}`

	return [
		{
			id: `inferno-duels${idSuffix}`,
			name: `Inferno Duels${suffix}`,
			tagline: 'The all-round PvP loadout',
			description:
				'Balanced for sword, axe and mace duels: top-tier FPS, a clean HUD with CPS and armour readouts, and zoom/freelook from Inferno.',
			gameVersion: v,
			loader: 'fabric',
			accent: '#ff5a1f',
			glyph: 'flame',
			tags: ['Duels', 'Balanced', 'Fabric'],
			includeInferno: true,
			memoryMb: 4096,
			mods: [
				SODIUM,
				LITHIUM,
				MODERNFIX,
				FERRITECORE,
				ENTITY_CULLING,
				IMMEDIATELYFAST,
				MORE_CULLING,
				SODIUM_EXTRA,
				REESES_OPTIONS,
				SCALABLELUX,
				KRYPTON,
				APPLESKIN,
				MOUSE_TWEAKS,
				BETTERF3,
				STATUS_EFFECT_BARS,
				LAMBDYNAMICLIGHTS,
				MOD_MENU,
			],
			note: INFERNO_NOTE,
		},
		{
			id: `max-fps${idSuffix}`,
			name: `Max FPS Arena${suffix}`,
			tagline: 'Every frame counts',
			description:
				'For crowded KitPvP, bedwars-style rushes and low-end PCs. Stacks every safe performance mod so your fights never hitch.',
			gameVersion: v,
			loader: 'fabric',
			accent: '#ffb800',
			glyph: 'bolt',
			tags: ['FPS', 'Low-end PCs', 'Fabric'],
			includeInferno: true,
			memoryMb: 3072,
			mods: [
				SODIUM,
				LITHIUM,
				MODERNFIX,
				FERRITECORE,
				ENTITY_CULLING,
				IMMEDIATELYFAST,
				MORE_CULLING,
				CULL_LESS_LEAVES,
				ENHANCED_BLOCK_ENTITIES,
				BAD_OPTIMIZATIONS,
				MEMORY_LEAK_FIX,
				KSYXIS,
				SCALABLELUX,
				KRYPTON,
				DYNAMIC_FPS,
				SODIUM_EXTRA,
				REESES_OPTIONS,
				APPLESKIN,
				MOUSE_TWEAKS,
				MOD_MENU,
			],
			note: INFERNO_NOTE,
		},
		{
			id: `clean-slate${idSuffix}`,
			name: `Clean Slate${suffix}`,
			tagline: 'Just Inferno and a renderer',
			description:
				'The smallest possible setup: Inferno plus Sodium. Ideal when a server or tournament keeps a strict allowed-mods list.',
			gameVersion: v,
			loader: 'fabric',
			accent: '#8d96a0',
			glyph: 'feather',
			tags: ['Minimal', 'Tournament-safe', 'Fabric'],
			includeInferno: true,
			memoryMb: 3072,
			mods: [
				SODIUM,
			],
			note: 'Always check the specific server or tournament rules for the mods you intend to use.',
		},
		{
			id: `clip-lab${idSuffix}`,
			name: `Clip Lab${suffix}`,
			tagline: 'Look good, record better',
			description:
				'For content creators: performance mods plus Iris shader support so your PvP clips look great without tanking frame rate.',
			gameVersion: v,
			loader: 'fabric',
			accent: '#c04bff',
			glyph: 'film',
			tags: ['Shaders', 'Content', 'Fabric'],
			includeInferno: true,
			memoryMb: 6144,
			mods: [
				SODIUM,
				LITHIUM,
				MODERNFIX,
				FERRITECORE,
				{
					slug: 'iris',
					label: 'Iris Shaders',
					why: 'Shader pack support built on Sodium.',
				},
				ENTITY_CULLING,
				IMMEDIATELYFAST,
				MORE_CULLING,
				ENHANCED_BLOCK_ENTITIES,
				SODIUM_EXTRA,
				REESES_OPTIONS,
				SCALABLELUX,
				LAMBDYNAMICLIGHTS,
				APPLESKIN,
				BETTERF3,
				REPLAY_MOD,
				MOD_MENU,
			],
			note: 'Shaders are a visual change; they are not allowed on every server. Drop a shader pack into this profile and enable it in Video Settings.',
		},
	]
}

export const PVP_PROFILES: PvpProfile[] = [...build('1.21.11'), ...build('1.21.10')]

export function profilesForVersion(version: PvpGameVersion): PvpProfile[] {
	return PVP_PROFILES.filter((p) => p.gameVersion === version)
}
