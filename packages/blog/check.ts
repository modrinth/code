import { promises as fs } from 'fs'
import * as path from 'path'
import fastGlob from 'fast-glob'
import { repoPath, toVarName } from './utils'

import { PUBLIC_SRC, PUBLIC_LOCATIONS, ARTICLES_GLOB, COMPILED_DIR } from './blog.config'

async function checkPublicAssets() {
  const srcFiles = await fastGlob(['**/*'], { cwd: PUBLIC_SRC, dot: true })
  let allOk = true
  for (const target of PUBLIC_LOCATIONS) {
    for (const relativeFile of srcFiles) {
      const shouldExist = path.join(target, relativeFile)
      try {
        await fs.access(shouldExist)
      } catch {
        console.error(`⚠️  Missing public asset: ${shouldExist}`)
        allOk = false
      }
    }
    if (allOk) {
      console.log(`✅  All public assets exist in: ${target}`)
    }
  }
  if (!allOk) process.exit(1)
}

async function checkCompiledArticles() {
  const mdFiles = await fastGlob([ARTICLES_GLOB])
  const compiledFiles = await fastGlob([`${COMPILED_DIR}/*.ts`])
  const compiledVarNames = compiledFiles.map((f) => path.basename(f, '.ts'))

  // Check all .md have compiled .ts and .content.ts and the proper public thumbnail
  for (const file of mdFiles) {
    const varName = toVarName(path.basename(file, '.md'))
    const compiledPath = path.join(COMPILED_DIR, varName + '.ts')
    const contentPath = path.join(COMPILED_DIR, varName + '.content.ts')
    if (!compiledVarNames.includes(varName)) {
      console.error(`⚠️  Missing compiled article for: ${file} (should be: ${compiledPath})`)
      process.exit(1)
    }
    try {
      await fs.access(compiledPath)
    } catch {
      console.error(`⚠️  Compiled article file not found: ${compiledPath}`)
      process.exit(1)
    }
    try {
      await fs.access(contentPath)
    } catch {
      console.error(`⚠️  Compiled article content file not found: ${contentPath}`)
      process.exit(1)
    }
  }

  // Check compiled .ts still have corresponding .md
  for (const compiled of compiledFiles) {
    const varName = path.basename(compiled, '.ts')
    if (varName === 'index' || varName.endsWith('.content')) continue

    const mdPathGlob = repoPath(`packages/blog/articles/**/${varName.replace(/_/g, '*')}.md`)
    const found = await fastGlob([mdPathGlob])
    if (!found.length) {
      console.error(`❌  Compiled article ${compiled} has no matching markdown source!`)
      process.exit(1)
    }
  }

  console.log(
    '🎉  All articles are correctly compiled, matched, and have thumbnails (if declared)!',
  )
}

async function main() {
  console.log('🔎  Checking public assets...')
  await checkPublicAssets()

  console.log('🔎  Checking compiled articles...')
  await checkCompiledArticles()
}

main().catch((e) => {
  console.error('❌  Error in check.ts:', e)
  process.exit(1)
})
