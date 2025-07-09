import { execSync } from 'child_process'
import * as path from 'path'

let REPO_ROOT_CACHE: string | null = null
export function getRepoRoot(): string {
  if (REPO_ROOT_CACHE) return REPO_ROOT_CACHE
  return (REPO_ROOT_CACHE = execSync('git rev-parse --show-toplevel').toString().trim())
}

export function repoPath(...segments: string[]): string {
  return path.join(getRepoRoot(), ...segments)
}

export async function copyDir(
  src: string,
  dest: string,
  logFn: (src: string, dest: string) => void = () => {},
): Promise<void> {
  const { promises: fs } = await import('fs')
  await fs.mkdir(dest, { recursive: true })
  const entries = await fs.readdir(src, { withFileTypes: true })
  for (const entry of entries) {
    const srcPath = path.join(src, entry.name)
    const destPath = path.join(dest, entry.name)
    if (entry.isDirectory()) {
      await copyDir(srcPath, destPath, logFn)
    } else if (entry.isFile()) {
      await fs.copyFile(srcPath, destPath)
      logFn(srcPath, destPath)
    }
  }
}

export function toVarName(file: string): string {
  return file
    .replace(/\.md$/, '')
    .replace(/[^a-zA-Z0-9]/g, '_')
    .replace(/^_+/, '')
    .replace(/_+$/, '')
}
