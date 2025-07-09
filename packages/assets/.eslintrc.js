module.exports = {
  root: true,
  extends: ['custom/library'],
  ignorePatterns: ['**/*.scss', '**/*.svg', 'node_modules/', 'dist/', '**/*.gltf'],
  env: {
    node: true,
  },
}
