<script setup>
import { Button, Card, UserIcon, LockIcon } from 'omorphia'
import {
  DiscordIcon,
  GithubIcon,
  MicrosoftIcon,
  GoogleIcon,
  SteamIcon,
  GitLabIcon,
} from '@/assets/external'
</script>

<template>
  <Card>
    <h1>Login to Modrinth</h1>
    <div class="button-grid">
      <Button class="discord" large>
        <DiscordIcon />
        Discord
      </Button>
      <Button class="github" large>
        <GithubIcon />
        Github
      </Button>
      <Button class="white" large>
        <MicrosoftIcon />
        Microsoft
      </Button>
      <Button class="google" large>
        <GoogleIcon />
        Google
      </Button>
      <Button class="white" large>
        <SteamIcon />
        Steam
      </Button>
      <Button class="gitlab" large>
        <GitLabIcon />
        GitLab
      </Button>
    </div>
    <div class="divider">
      <hr />
      <p>Or</p>
    </div>
    <div class="iconified-input username">
      <UserIcon />
      <input type="text" placeholder="Email or username" />
    </div>
    <div class="iconified-input">
      <LockIcon />
      <input type="password" placeholder="Password" />
    </div>
    <div class="link-row">
      <a class="button-base"> Create account </a>
      <a class="button-base"> Forgot password? </a>
    </div>
    <div class="button-row">
      <Button class="transparent" large> Back </Button>
      <Button color="primary" large> Login </Button>
      <Button class="transparent" large> Skip </Button>
    </div>
  </Card>
</template>

<style scoped lang="scss">
.card {
  width: 25rem;
}

.button-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-gap: var(--gap-md);

  .btn {
    width: 100%;
    justify-content: center;
  }

  .discord {
    background-color: #5865f2;
    color: var(--color-contrast);
  }

  .github {
    background-color: #8740f1;
    color: var(--color-contrast);
  }

  .white {
    background-color: var(--color-contrast);
    color: var(--color-accent-contrast);
  }

  .google {
    background-color: #4285f4;
    color: var(--color-contrast);
  }

  .gitlab {
    background-color: #fc6d26;
    color: var(--color-contrast);
  }
}

.divider {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  margin: var(--gap-md) 0;

  p {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background-color: var(--color-raised-bg);
    padding: 0 1rem;
    margin: 0;
  }

  hr {
    border: none;
    width: 100%;
    border-top: 2px solid var(--color-button-bg);
  }
}

.iconified-input {
  width: 100%;

  input {
    width: 100%;
    flex-basis: auto;
  }
}

.username {
  margin-bottom: var(--gap-sm);
}

.link-row {
  display: flex;
  justify-content: space-between;
  margin: var(--gap-md) 0;

  a {
    color: var(--color-blue);
    text-decoration: underline;

    &:hover {
      cursor: pointer;
    }
  }
}

.button-row {
  display: flex;
  justify-content: space-between;

  .btn {
    flex-basis: auto;
  }

  .transparent {
    padding: var(--gap-md) 0;
  }
}
</style>
