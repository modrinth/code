import type { Component } from 'vue'
import { type MessageDescriptor, defineMessage } from '@vintl/vintl'
import type { VirtualProjectType } from '@modrinth/utils'
import { BoxIcon } from '@modrinth/assets'

type ProjectTypeMessageType = 'body' | 'title' | 'category'

type ProjectTypeMetadata = {
  icon: Component
  messages: Record<ProjectTypeMessageType, MessageDescriptor>
}

const PROJECT_TYPE_METADATA: Record<VirtualProjectType | 'project', ProjectTypeMetadata> = {
  project: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.project.body', defaultMessage: '{count, plural, one {project} other {projects}}' }),
      title: defineMessage({ id: 'project-type.project.title', defaultMessage: '{count, plural, one {Project} other {Projects}}' }),
      category: defineMessage({ id: 'project-type.project.category', defaultMessage: 'Projects' }),
    }
  },
  mod: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.mod.body', defaultMessage: '{count, plural, one {mod} other {mods}}' }),
      title: defineMessage({ id: 'project-type.mod.title', defaultMessage: '{count, plural, one {Mod} other {Mods}}' }),
      category: defineMessage({ id: 'project-type.mod.category', defaultMessage: 'Mods' }),
    }
  },
  modpack: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.modpack.body', defaultMessage: '{count, plural, one {modpack} other {modpacks}}' }),
      title: defineMessage({ id: 'project-type.modpack.title', defaultMessage: '{count, plural, one {Modpack} other {Modpacks}}' }),
      category: defineMessage({ id: 'project-type.modpack.category', defaultMessage: 'Modpacks' }),
    }
  },
  resourcepack: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.resourcepack.body', defaultMessage: '{count, plural, one {resource pack} other {resource packs}}' }),
      title: defineMessage({ id: 'project-type.resourcepack.title', defaultMessage: '{count, plural, one {Resource Pack} other {Resource Packs}}' }),
      category: defineMessage({ id: 'project-type.resourcepack.category', defaultMessage: 'Resource Packs' }),
    }
  },
  shader: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.shader.body', defaultMessage: '{count, plural, one {shader} other {shaders}}' }),
      title: defineMessage({ id: 'project-type.shader.title', defaultMessage: '{count, plural, one {Shader} other {Shaders}}' }),
      category: defineMessage({ id: 'project-type.shader.category', defaultMessage: 'Shaders' }),
    }
  },
  plugin: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.plugin.body', defaultMessage: '{count, plural, one {plugin} other {plugins}}' }),
      title: defineMessage({ id: 'project-type.plugin.title', defaultMessage: '{count, plural, one {Plugin} other {Plugins}}' }),
      category: defineMessage({ id: 'project-type.plugin.category', defaultMessage: 'Plugins' }),
    }
  },
  datapack: {
    icon: BoxIcon,
    messages: {
      body: defineMessage({ id: 'project-type.datapack.body', defaultMessage: '{count, plural, one {data pack} other {data packs}}' }),
      title: defineMessage({ id: 'project-type.datapack.title', defaultMessage: '{count, plural, one {Data Pack} other {Data Packs}}' }),
      category: defineMessage({ id: 'project-type.datapack.category', defaultMessage: 'Data Packs' }),
    }
  },
}


export const getProjectTypeMessage = (type: VirtualProjectType | 'project' | null | undefined, message: ProjectTypeMessageType = 'category' ): MessageDescriptor => {
  return PROJECT_TYPE_METADATA[type ?? 'project'].messages[message]
}

export const ANY_PROJECT_TYPE_MESSAGE = defineMessage({ id: 'project-type.any', defaultMessage: 'Any type' })
