import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { join } from 'node:path'
import { test } from 'node:test'

import { installPvpProfile, uniqueInstanceName, type InstallerDeps } from '../src/features/pvp/installer'
import { INFERNO_JARS, PVP_PROFILES, profilesForVersion } from '../src/features/pvp/profiles'

const root = join(process.cwd(), '..', 'app')

function mockDeps(over: Partial<InstallerDeps> = {}) {
	const calls: string[] = []
	const deps: InstallerDeps = {
		async createInstance(a) {
			calls.push(`create:${a.name}:${a.gameVersion}:${a.loader}`)
			return 'inst-1'
		},
		async resolveProjectId(slug) {
			return `id-${slug}`
		},
		async installMod(a) {
			calls.push(`mod:${a.projectId}:${a.gameVersion}`)
		},
		async tuneInstance(a) {
			calls.push(`tune:${a.instanceId}:${a.memoryMb}:${a.jvmArgs.includes('-XX:+UseG1GC')}`)
		},
		async resolveBundledResource(p) {
			return `/res/${p}`
		},
		async addLocalMod(id, path) {
			calls.push(`local:${id}:${path}`)
		},
		async removeInstance(id) {
			calls.push(`remove:${id}`)
		},
		...over,
	}
	return { deps, calls }
}

test('happy path installs every mod then Inferno for the right version', async () => {
	for (const v of ['1.21.11', '1.21.10'] as const) {
		const p = profilesForVersion(v)[0]
		const { deps, calls } = mockDeps()
		const stages: string[] = []
		const r = await installPvpProfile(p, [], deps, (s) => stages.push(s.kind))
		assert.equal(r.instanceId, 'inst-1')
		assert.equal(r.infernoInstalled, true)
		assert.ok(r.outcomes.every((o) => o.status === 'installed'))
		assert.equal(calls.filter((c) => c.startsWith('mod:')).length, p.mods.length)
		assert.ok(calls.includes(`local:inst-1:/res/${INFERNO_JARS[v]}`))
		assert.ok(calls.every((c) => !c.startsWith('mod:') || c.endsWith(`:${v}`)))
		assert.deepEqual([stages[0], stages[stages.length - 1]], ['creating', 'done'])
	}
})

test('a missing mod is skipped and reported, the rest continue', async () => {
	const p = profilesForVersion('1.21.11')[0]
	const { deps, calls } = mockDeps({
		async resolveProjectId(slug) {
			if (slug === 'sodium-extra') throw new Error('not found')
			return `id-${slug}`
		},
	})
	const r = await installPvpProfile(p, [], deps)
	const skipped = r.outcomes.filter((o) => o.status === 'skipped')
	assert.equal(skipped.length, 1)
	assert.equal(skipped[0].slug, 'sodium-extra')
	assert.equal(calls.filter((c) => c.startsWith('mod:')).length, p.mods.length - 1)
	assert.equal(r.infernoInstalled, true)
})

test('install failure for one mod (no compatible build) is skipped, not fatal', async () => {
	const p = profilesForVersion('1.21.10')[1]
	const { deps } = mockDeps({
		async installMod(a) {
			if (a.projectId === 'id-dynamic-fps') throw { message: 'no compatible version' }
		},
	})
	const r = await installPvpProfile(p, [], deps)
	const s = r.outcomes.find((o) => o.slug === 'dynamic-fps')
	assert.ok(s && s.status === 'skipped' && s.reason === 'no compatible version')
})

test('if Inferno cannot be added the half-built instance is removed and an error thrown', async () => {
	const p = profilesForVersion('1.21.11')[0]
	const { deps, calls } = mockDeps({
		async addLocalMod() {
			throw new Error('disk full')
		},
	})
	await assert.rejects(installPvpProfile(p, [], deps), /Could not install Inferno Client: disk full/)
	assert.ok(calls.includes('remove:inst-1'))
	assert.equal(calls.filter((c) => c.startsWith('remove:')).length, 1)
})

test('tuning applies JVM flags and memory; a tuning failure is non-fatal', async () => {
	const p = profilesForVersion('1.21.11')[0]
	const ok = mockDeps()
	const r1 = await installPvpProfile(p, [], ok.deps)
	assert.equal(r1.tuned, true)
	assert.ok(ok.calls.includes(`tune:inst-1:${p.memoryMb}:true`))

	const bad = mockDeps({
		async tuneInstance() {
			throw new Error('nope')
		},
	})
	const r2 = await installPvpProfile(p, [], bad.deps)
	assert.equal(r2.tuned, false)
	assert.equal(r2.infernoInstalled, true)
})

test('unique names do not collide with existing instances (case-insensitive)', () => {
	assert.equal(uniqueInstanceName('Inferno Duels', []), 'Inferno Duels')
	assert.equal(uniqueInstanceName('Inferno Duels', ['inferno duels']), 'Inferno Duels 2')
	assert.equal(
		uniqueInstanceName('Inferno Duels', ['Inferno Duels', 'Inferno Duels 2']),
		'Inferno Duels 3',
	)
})

test('Duels, Max FPS and Clip Lab carry the expanded mod sets', () => {
	for (const v of ['1.21.11', '1.21.10'] as const) {
		const byName = Object.fromEntries(profilesForVersion(v).map((p) => [p.id.split('-' + v.replace(/\./g, ''))[0], p]))
		for (const id of ['inferno-duels', 'max-fps', 'clip-lab']) {
			const slugs = byName[id].mods.map((m) => m.slug)
			assert.ok(slugs.includes('appleskin'), `${id} missing appleskin`)
			assert.ok(slugs.includes('sodium') && slugs.includes('lithium'))
			assert.ok(byName[id].mods.length >= 14, `${id} only has ${byName[id].mods.length} mods`)
		}
		assert.ok(byName['clean-slate'].mods.length === 1)
	}
})

test('profile data is sane', () => {
	const ids = new Set(PVP_PROFILES.map((p) => p.id))
	assert.equal(ids.size, PVP_PROFILES.length, 'duplicate profile ids')
	for (const p of PVP_PROFILES) {
		const slugs = p.mods.map((m) => m.slug)
		assert.equal(new Set(slugs).size, slugs.length, `${p.id} has duplicate mods`)
		assert.ok(p.includeInferno)
		assert.ok(INFERNO_JARS[p.gameVersion])
		assert.ok(p.memoryMb >= 2048 && p.memoryMb <= 8192)
	}
})

test('no cheat / hacked-client mods in any profile', () => {
	const banned = /(aura|killaura|reach|velocity|xray|x-ray|esp|tracer|autoclick|triggerbot|aimbot|scaffold|bhop|speedhack|nofall|wurst|meteor|impact|aristois|liquidbounce|freecam|baritone|hitbox)/i
	for (const p of PVP_PROFILES) {
		for (const m of p.mods) {
			assert.ok(!banned.test(m.slug) && !banned.test(m.label), `${p.id}: suspicious mod ${m.slug}`)
		}
	}
})

test('bundled Inferno jars exist and their fabric.mod.json targets the profile version', () => {
	for (const [version, rel] of Object.entries(INFERNO_JARS)) {
		const file = join(root, 'resources', rel)
		assert.ok(existsSync(file), `${file} missing`)
		const manifest = JSON.parse(
			execFileSync('unzip', ['-p', file, 'fabric.mod.json'], { encoding: 'utf8' }),
		)
		assert.equal(manifest.id, 'inferno_client')
		assert.equal(manifest.depends.minecraft, `~${version}`)
	}
})
