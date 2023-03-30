<template>
  <Card class="settings-card">
    <h2 class="settings-title"> Java </h2>
    <div class="settings-group">
      <h3>Installation</h3>
      <input
          type="text"
          class="input installation-input"
          placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home"
          ref="javaPath"
          v-model="javaPath"
      />
      <span class="installation-buttons">
          <Button @click="saveJavaPath">
            <SearchIcon/>
            Auto Detect
          </Button>
          <Button @click="saveJavaPath">
            <BrowseIcon/>
            Browse
          </Button>
          <Button @click="saveJavaPath">
            <PlayIcon/>
            Test
          </Button>
        </span>
    </div>
    <hr class="card-divider">
    <div class="settings-group">
      <h3>Arguments</h3>
      <input
          type="text"
          class="input installation-input"
          ref="javaArgs"
          v-model="javaArgs"
      />
    </div>
    <hr class="card-divider">
    <div class="settings-group">
      <div class="sliders">
          <span class="slider">
            Minimum Memory
            <Slider
                v-model="javaMemory"
                :min="1024"
                :max="8192"
                :step="1024"
            />
          </span>
        <span class="slider">
            Maximum Memory
            <Slider
                v-model="javaMemory"
                :min="1024"
                :max="8192"
                :step="1024"
            />
          </span>
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title"> Window </h2>
    <div class="settings-group">
      <div class="settings-group">
        <div class="sliders">
          <span class="slider">
            Width
            <Slider
                v-model="javaMemory"
                :min="1024"
                :max="8192"
                :step="1024"
            />
          </span>
          <span class="slider">
            Height
            <Slider
                v-model="javaMemory"
                :min="1024"
                :max="8192"
                :step="1024"
            />
          </span>
        </div>
        <div class="toggle-setting">
          Start in Fullscreen
          <input
              type="checkbox"
              id="fullscreen"
              name="fullscreen"
              v-model="fullscreen"
              class="switch stylized-toggle"
          />
        </div>
      </div>
      <hr class="card-divider">
      <div class="settings-group">
        <h3>Console</h3>
        <div class="toggle-setting">
          Show console while game is running
          <input
              type="checkbox"
              id="fullscreen"
              name="fullscreen"
              v-model="fullscreen"
              class="switch stylized-toggle"
          />
        </div>
        <div class="toggle-setting">
          Close console when game quits
          <input
              type="checkbox"
              id="fullscreen"
              name="fullscreen"
              v-model="fullscreen"
              class="switch stylized-toggle"
          />
        </div>
        <div class="toggle-setting">
          Show console when game crashes
          <input
              type="checkbox"
              id="fullscreen"
              name="fullscreen"
              v-model="fullscreen"
              class="switch stylized-toggle"
          />
        </div>
      </div>
    </div>
  </Card>
  <Card class="settings-card">
    <h2 class="settings-title"> Commands </h2>
    <div class="settings-group">
      <div class="toggle-setting">
        Pre Launch
        <input
            type="text"
            class="input"
            ref="javaArgs"
            v-model="javaArgs"
        />
      </div>
      <div class="toggle-setting">
        Wrapper
        <input
            type="text"
            class="input"
            ref="javaArgs"
            v-model="javaArgs"
        />
      </div>
      <div class="toggle-setting">
        Post Launch
        <input
            type="text"
            class="input"
            ref="javaArgs"
            v-model="javaArgs"
        />
      </div>
    </div>
  </Card>
</template>

<script setup>
import {Card, Button, SearchIcon, Slider } from 'omorphia'
import {BrowseIcon, PlayIcon} from "@/assets/icons";
</script>

<style scoped lang="scss">
.settings-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.settings-title {
  color: var(--color-contrast)
}

.settings-group {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}

.installation-input {
  width: 100%;
}

.installation-buttons {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  align-items: center;
  gap: .5rem;
  margin: 0;

}

.sliders {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 1rem;
  width: 100%;

  .slider {
    flex-grow: 1;
  }
}

.toggle-setting {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
  gap: 0.5rem;
}

.card-divider {
  background-color: var(--color-button-bg);
  border: none;
  color: var(--color-button-bg);
  height: 1px;
  margin: var(--gap-sm) 0;
}
</style>
