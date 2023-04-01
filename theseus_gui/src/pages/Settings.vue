<script setup>
import { ref } from 'vue'
import { Card, SunIcon, MoonIcon, Button, Slider } from 'omorphia'
import { useTheming } from '@/store/state'

const javaMemory = ref(1024)
const javaArgs = ref('')
const javaPath = ref('')
const fullscreen = ref(false)

const theme = useTheming()
</script>

<template>
  <div>
    <Card class="theming">
      <h2>Themes</h2>
      <div class="setting-row">
        <div class="description">
          <h3>Color theme</h3>
          <p>Change the global launcher color theme.</p>
        </div>
        <Button class="theme-toggle" @click="theme.toggleTheme">
          <div v-if="theme.darkTheme === false">
            <SunIcon />
            <span>Light</span>
          </div>
          <div v-else>
            <MoonIcon />
            <span>Dark</span>
          </div>
        </Button>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Java</h2>
      <div class="settings-group">
        <h3>Installation</h3>
        <input
          v-model="javaPath"
          type="text"
          class="input installation-input"
          placeholder="/Library/Java/JavaVirtualMachines/zulu-17.jdk/Contents/Home"
        />
        <span class="installation-buttons">
          <Button @click="saveJavaPath">
            <SearchIcon />
            Auto Detect
          </Button>
          <Button @click="saveJavaPath">
            <BrowseIcon />
            Browse
          </Button>
          <Button @click="saveJavaPath">
            <PlayIcon />
            Test
          </Button>
        </span>
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <h3>Arguments</h3>
        <input v-model="javaArgs" type="text" class="input installation-input" />
      </div>
      <hr class="card-divider" />
      <div class="settings-group">
        <div class="sliders">
          <span class="slider">
            Minimum Memory
            <Slider v-model="javaMemory" :min="1024" :max="8192" :step="1024" />
          </span>
          <span class="slider">
            Maximum Memory
            <Slider v-model="javaMemory" :min="1024" :max="8192" :step="1024" />
          </span>
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Window</h2>
      <div class="settings-group">
        <div class="settings-group">
          <div class="sliders">
            <span class="slider">
              Width
              <Slider v-model="javaMemory" :min="1024" :max="8192" :step="1024" />
            </span>
            <span class="slider">
              Height
              <Slider v-model="javaMemory" :min="1024" :max="8192" :step="1024" />
            </span>
          </div>
          <div class="toggle-setting">
            Start in Fullscreen
            <input
              id="fullscreen"
              v-model="fullscreen"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
        </div>
        <hr class="card-divider" />
        <div class="settings-group">
          <h3>Console</h3>
          <div class="toggle-setting">
            Show console while game is running
            <input
              id="fullscreen"
              v-model="fullscreen"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
          <div class="toggle-setting">
            Close console when game quits
            <input
              id="fullscreen"
              v-model="fullscreen"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
          <div class="toggle-setting">
            Show console when game crashes
            <input
              id="fullscreen"
              v-model="fullscreen"
              type="checkbox"
              name="fullscreen"
              class="switch stylized-toggle"
            />
          </div>
        </div>
      </div>
    </Card>
    <Card class="settings-card">
      <h2 class="settings-title">Commands</h2>
      <div class="settings-group">
        <div class="toggle-setting">
          Pre Launch
          <input v-model="javaArgs" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Wrapper
          <input v-model="javaArgs" type="text" class="input" />
        </div>
        <div class="toggle-setting">
          Post Launch
          <input v-model="javaArgs" type="text" class="input" />
        </div>
      </div>
    </Card>
  </div>
</template>

<style lang="scss">
.slider-input {
  width: 4rem !important;
  flex-basis: 5rem !important;
}

.installation-input {
  width: 100%;
}

.theming,
.settings-card {
  width: 90%;
  margin: 2rem auto;
  display: flex;
  flex-direction: column;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;

  .slider {
    width: 40%;
  }

  .description {
    line-height: 1.2rem;
    font-size: 1.05rem;
    width: 40%;

    h3 {
      font-size: 1.1rem;
      margin-bottom: 0.5rem;
    }
  }

  .input {
    width: 60%;
    flex-basis: 24rem;
  }

  .theme-toggle {
    display: flex;
    align-items: center;
    justify-content: space-evenly;
    width: 20%;
    font-size: 1.25rem;
    cursor: pointer;
    transition: all 0.1s ease-in-out;

    div {
      display: flex;
      align-items: inherit;
      justify-content: center;

      span {
        font-size: 1.1rem;
      }

      svg {
        width: 1.5rem;
        height: 1.5rem;
      }
    }
  }
}

.settings-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.settings-title {
  color: var(--color-contrast);
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
  gap: 0.5rem;
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
