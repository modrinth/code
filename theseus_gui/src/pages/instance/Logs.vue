<template>
  <Card class="log-card">
    <div class="button-row">
      <div>
        <Button>
          logs/latest.log
        </Button>
      </div>
      <div class="button-group">
        <Button>
          <ClipboardCopyIcon/>
          Copy
        </Button>
        <Button color="primary">
          <SendIcon/>
          Share
        </Button>
        <Button color="danger">
          <TrashIcon/>
          Delete
        </Button>
      </div>
    </div>
    <div class="log-text">
      <div v-for="line in fileContents.value.split('\n')"> {{ line }} </div>
    </div>
  </Card>
</template>

<script setup>
import { Card, Button, TrashIcon, SendIcon, ClipboardCopyIcon } from 'omorphia'
</script>
<script>

  export default {
    data() {
      return {
        fileContents: {
          value: "'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:02] [Server thread/INFO]: venashial lost connection: Disconnected\n" +
              "[22:13:02] [Server thread/INFO]: venashial left the game\n" +
              "[22:13:02] [Server thread/INFO]: Stopping singleplayer server as player logged out\n" +
              "[22:13:02] [Server thread/INFO]: Stopping server\n" +
              "[22:13:02] [Server thread/INFO]: Saving players\n" +
              "[22:13:02] [Server thread/INFO]: Saving worlds\n" +
              "[22:13:02] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:overworld\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_nether\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (New World): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM-1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage: All dimensions are saved\n" +
              "[22:13:06] [Render thread/INFO]: Stopping worker threads\n" +
              "[22:13:07] [Render thread/INFO]: Stopping!\n" +
              "[22:13:07] [CraftPresence-ShutDown-Handler/INFO]: Shutting down CraftPresence...\n" +
              "'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:02] [Server thread/INFO]: venashial lost connection: Disconnected\n" +
              "[22:13:02] [Server thread/INFO]: venashial left the game\n" +
              "[22:13:02] [Server thread/INFO]: Stopping singleplayer server as player logged out\n" +
              "[22:13:02] [Server thread/INFO]: Stopping server\n" +
              "[22:13:02] [Server thread/INFO]: Saving players\n" +
              "[22:13:02] [Server thread/INFO]: Saving worlds\n" +
              "[22:13:02] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:overworld\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_nether\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (New World): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM-1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage: All dimensions are saved\n" +
              "[22:13:06] [Render thread/INFO]: Stopping worker threads\n" +
              "[22:13:07] [Render thread/INFO]: Stopping!\n" +
              "[22:13:07] [CraftPresence-ShutDown-Handler/INFO]: Shutting down CraftPresence...\n" +
              "'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:02] [Server thread/INFO]: venashial lost connection: Disconnected\n" +
              "[22:13:02] [Server thread/INFO]: venashial left the game\n" +
              "[22:13:02] [Server thread/INFO]: Stopping singleplayer server as player logged out\n" +
              "[22:13:02] [Server thread/INFO]: Stopping server\n" +
              "[22:13:02] [Server thread/INFO]: Saving players\n" +
              "[22:13:02] [Server thread/INFO]: Saving worlds\n" +
              "[22:13:02] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:overworld\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_nether\n" +
              "[22:13:05] [Server thread/INFO]: Saving chunks for level 'ServerLevel[New World]'/minecraft:the_end\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (New World): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM-1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage (DIM1): All chunks are saved\n" +
              "[22:13:05] [Server thread/INFO]: ThreadedAnvilChunkStorage: All dimensions are saved\n" +
              "[22:13:06] [Render thread/INFO]: Stopping worker threads\n" +
              "[22:13:07] [Render thread/INFO]: Stopping!\n" +
              "[22:13:07] [CraftPresence-ShutDown-Handler/INFO]: Shutting down CraftPresence..."
        }
      };
    }
  };
</script>

<style scoped lang="scss">
.log-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.button-row {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
}

.button-group {
  display: flex;
  flex-direction: row;
  gap: 0.5rem;
}

.log-text {
  width: 100%;
  aspect-ratio: 2/1;
  font-family: var(--mono-font);
  background-color: var(--color-accent-contrast);
  color: var(--color-contrast);
  border-radius: var(--radius-lg);
  padding: 1.5rem;
  overflow: auto;
  white-space: normal;
}
</style>