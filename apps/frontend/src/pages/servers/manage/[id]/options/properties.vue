<template>
  <section class="card">
    <h2 class="text-3xl font-bold">{{ formatMessage(messages.title) }}</h2>
    <div
      v-for="(property, index) in properties"
      :key="index"
      class="mb-4 flex justify-between gap-4"
    >
      <label :for="index" class="block text-lg font-semibold">{{ index.split("-").map(word => word.charAt(0).toUpperCase() + word.slice(1)).join(" ") }}</label>
      <div v-if="typeof property === 'boolean'">
        <Checkbox id="property.id" :model-value="property" />
      </div>
      <div v-else-if="typeof property === 'number'">
        <input
          :id="index"
          type="number"
          class="w-full rounded border p-2"
          v-model.number="properties[index]"
        />
      </div>
      <div v-else-if="typeof property === 'object'">
        <textarea
          :id="index"
          :value="JSON.stringify(property, null, 2)"
          class="w-full rounded border p-2"
        ></textarea>
      </div>
      <div v-else>
        <input
          :id="index"
          :value="property"
          type="text"
          class="w-full rounded border p-2"
        />
      </div>
    </div>
    <button type="submit" class="btn btn-primary">Save</button>
  </section>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";
import Checkbox from "~/components/ui/Checkbox.vue";

const changedPropertiesState = useState('changedProperties', () => shallowRef({}));

const { formatMessage } = useVIntl();
const messages = defineMessages({
  title: {
    id: "server.options.props.title",
    defaultMessage: "Server Properties",
  },
});

const properties = ref({
  "accepts-transfers": false,
  "allow-flight": false,
  "allow-nether": true,
  "broadcast-console-to-ops": true,
  "broadcast-rcon-to-ops": true,
  "difficulty": "hard",
  "enable-command-block": false,
  "enable-jmx-monitoring": false,
  "enable-query": true,
  "enable-rcon": false,
  "enable-status": true,
  "enforce-secure-profile": true,
  "enforce-whitelist": false,
  "entity-broadcast-range-percentage": 100,
  "force-gamemode": false,
  "function-permission-level": 2,
  "gamemode": "survival",
  "generate-structures": true,
  "generator-settings": {},
  "hardcore": false,
  "hide-online-players": false,
  "initial-disabled-packs": "",
  "initial-enabled-packs": "vanilla",
  "level-name": "world",
  "level-seed": 4680095729451076540,
  "level-type": "minecraft:normal",
  "log-ips": true,
  "max-chained-neighbor-updates": 1000000,
  "max-players": 40,
  "max-tick-time": 60000,
  "max-world-size": 29999984,
  "motd": "Welcome to§r\n§3Deep Season SMP",
  "network-compression-threshold": 256,
  "online-mode": true,
  "op-permission-level": 4,
  "player-idle-timeout": 0,
  "prevent-proxy-connections": false,
  "pvp": true,
  "query.port": 25578,
  "rate-limit": 0,
  "rcon.password": "",
  "rcon.port": 25575,
  "region-file-compression": "deflate",
  "require-resource-pack": true,
  "resource-pack": "http://45.153.48.36:25567/7618426846192410215.zip",
  "resource-pack-id": "",
  "resource-pack-prompt": "",
  "resource-pack-sha1": "1802a5eef0b222196c40541ccf40f87f82a7b250",
  "server-ip": "0.0.0.0",
  "server-port": 25565,
  "simulation-distance": 10,
  "spawn-animals": true,
  "spawn-monsters": true,
  "spawn-npcs": true,
  "spawn-protection": 0,
  "sync-chunk-writes": true,
  "text-filtering-config": "",
  "use-native-transport": true,
  "view-distance": 10,
  "white-list": true
});

const updateProperty = (key: string, value: any) => {
};

// TODO: EVERYTHING HERE IS BROKEN. HI WANDER
watch(properties, async (newProperties, oldProperties) => {
  console.log(newProperties);
  console.log(oldProperties);
  let changed
  for (const key in newProperties) {
    if (newProperties[key] !== oldProperties[key]) {
      console.log("hi");
      changed = key;
      break;
    }
  }
  console.log(changed);
  changedPropertiesState[changed] = newProperties[changed];
  properties[changed] = oldProperties[changed];
  console.log(changedPropertiesState.value);
  console.log(properties.value);
}, { deep: true });
</script>
