import {
  DEFAULT_OPTIONS,
  GPT_LIB_SCRIPT_ID,
  GPT_INIT_SCRIPT_ID,
  GEOEDGE_CONF_SCRIPT_ID,
  GEOEDGE_LIB_SCRIPT_ID,
} from './constants'

const { resolve } = require('path')

// eslint-disable-next-line require-await
module.exports = async function module(moduleOptions) {
  const options = Object.assign(
    DEFAULT_OPTIONS,
    this.options.ads,
    moduleOptions
  )

  const templatesOptions = {
    ...options,
    GPT_LIB_SCRIPT_ID,
    GPT_INIT_SCRIPT_ID,
    GEOEDGE_CONF_SCRIPT_ID,
    GEOEDGE_LIB_SCRIPT_ID,
  }

  this.addPlugin({
    src: resolve(__dirname, 'templates/plugin.js'),
    fileName: 'gpt-ads-module/plugin.js',
    options: templatesOptions,
  })

  this.addTemplate({
    src: resolve(__dirname, 'templates/component.js'),
    fileName: 'gpt-ads-module/component.js',
    options: templatesOptions,
  })
}
module.exports.meta = require('../package.json')
