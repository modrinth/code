<script setup>
import { Button, LogInIcon, Card } from 'omorphia'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { handleError } from '@/store/notifications.js'
import mixpanel from 'mixpanel-browser'
import { ref } from 'vue'
const finalizedLogin = ref(false)

const props = defineProps({
  nextPage: {
    type: Function,
    required: true,
  },
  prevPage: {
    type: Function,
    required: true,
  },
})

async function login() {
  const loggedIn = await login_flow().catch(handleError)

  if (loggedIn) {
    await set_default_user(loggedIn.id).catch(handleError)
  }

  finalizedLogin.value = true
  await mixpanel.track('AccountLogIn')
  props.nextPage()
}
</script>

<template>
  <div class="login-card">
    <img
      src="https://launcher-files.modrinth.com/assets/default_profile.png"
      class="logo"
      alt="Minecraft art"
    />
    <Card class="logging-in">
      <h2>Sign into Minecraft</h2>
      <p>
        Sign in with your Microsoft account to launch Minecraft with your mods and modpacks. If you
        don't have a Minecraft account, you can purchase the game on the
        <a
          href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
          class="link"
        >
          Minecraft website
        </a>
      </p>
      <div class="action-row">
        <Button class="transparent" large @click="prevPage"> Back </Button>
        <div class="sign-in-pair">
          <Button color="primary" large @click="login">
            <LogInIcon v-if="!finalizedLogin" />
            {{ finalizedLogin ? 'Finish' : 'Sign in' }}
          </Button>
        </div>
        <Button class="transparent" large @click="nextPage()"> Finish</Button>
      </div>
    </Card>
  </div>
</template>

<style scoped lang="scss">
.login-card {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  margin: auto;
  padding: var(--gap-lg);
  width: 30rem;

  img {
    width: 100%;
    border-radius: var(--radius-lg) var(--radius-lg) 0 0;
  }
}

.logging-in {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  vertical-align: center;
  gap: var(--gap-md);
  background-color: var(--color-raised-bg);
  width: 100%;
  border-radius: 0 0 var(--radius-lg) var(--radius-lg);

  h2,
  p {
    margin: 0;
  }

  p {
    text-align: center;
  }
}

.link {
  color: var(--color-blue);
  text-decoration: underline;
}

.button-row {
  display: flex;
  flex-direction: row;
}

.action-row {
  width: 100%;
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  gap: var(--gap-md);
  margin-top: var(--gap-md);

  .transparent {
    padding: 0 var(--gap-md);
  }
}

.sign-in-pair {
  display: flex;
  flex-direction: column;
  gap: var(--gap-sm);
  align-items: center;
}
</style>
