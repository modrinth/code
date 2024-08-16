import { ref, watch } from 'vue'
import { get, set } from '@/helpers/settings.js'
import { mixpanel_opt_in_tracking, mixpanel_opt_out_tracking } from '@/helpers/mixpanel.js'
import { handleError } from '@/store/state'

export async function useSettings() {
  const rawSettings = await get().catch(handleError)

  rawSettings.launchArgs = rawSettings.extra_launch_args.join(' ')
  rawSettings.envVars = rawSettings.custom_env_vars.map((x) => x.join('=')).join(' ')

  const settings = ref(rawSettings)

  watch(
    settings,
    async (oldSettings, newSettings) => {
      if (oldSettings.loaded_config_dir !== newSettings.loaded_config_dir) {
        return
      }

      const setSettings = JSON.parse(JSON.stringify(newSettings))

      if (setSettings.telemetry) {
        mixpanel_opt_out_tracking()
      } else {
        mixpanel_opt_in_tracking()
      }

      setSettings.extra_launch_args = setSettings.launchArgs.trim().split(/\s+/).filter(Boolean)
      setSettings.custom_env_vars = setSettings.envVars
        .trim()
        .split(/\s+/)
        .filter(Boolean)
        .map((x) => x.split('=').filter(Boolean))

      if (!setSettings.hooks.pre_launch) {
        setSettings.hooks.pre_launch = null
      }
      if (!setSettings.hooks.wrapper) {
        setSettings.hooks.wrapper = null
      }
      if (!setSettings.hooks.post_exit) {
        setSettings.hooks.post_exit = null
      }

      if (!setSettings.custom_dir) {
        setSettings.custom_dir = null
      }

      await set(setSettings)
    },
    { deep: true },
  )

  return settings
}
