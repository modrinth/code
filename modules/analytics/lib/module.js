import { DEFAULT_OPTIONS, CF_LIB_TAG_ID } from './constants'

const { resolve } = require('path')

// eslint-disable-next-line require-await
module.exports = async function module(moduleOptions) {
  const options = Object.assign(
    DEFAULT_OPTIONS,
    this.options.analytics,
    moduleOptions
  )

  const templatesOptions = {
    ...options,
    CF_LIB_TAG_ID,
  }

  this.addPlugin({
    src: resolve(__dirname, 'templates/plugin.js'),
    fileName: 'analytics/plugin.js',
    options: templatesOptions,
  })
}
module.exports.meta = require('../package.json')
