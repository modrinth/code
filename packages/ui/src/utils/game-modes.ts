import { BlocksIcon, CompassIcon, EyeIcon, PickaxeIcon, UnknownIcon } from '@modrinth/assets'
import { defineMessage } from '@vintl/vintl'

export const GAME_MODES = {
  survival: {
    icon: PickaxeIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.survival',
      defaultMessage: 'Survival mode',
    }),
  },
  creative: {
    icon: BlocksIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.creative',
      defaultMessage: 'Creative mode',
    }),
  },
  adventure: {
    icon: CompassIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.adventure',
      defaultMessage: 'Adventure mode',
    }),
  },
  spectator: {
    icon: EyeIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.spectator',
      defaultMessage: 'Spectator mode',
    }),
  },
  unknown: {
    icon: UnknownIcon,
    message: defineMessage({
      id: 'instance.worlds.game_mode.unknown',
      defaultMessage: 'Unknown game mode',
    }),
  },
}
