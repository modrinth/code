<template>
  <svg
    v-if="loader === 'Fabric'"
    xmlns="http://www.w3.org/2000/svg"
    xml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    clip-rule="evenodd"
    viewBox="0 0 24 24"
  >
    <path fill="none" d="M0 0h24v24H0z" />
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="23"
      d="m820 761-85.6-87.6c-4.6-4.7-10.4-9.6-25.9 1-19.9 13.6-8.4 21.9-5.2 25.4 8.2 9 84.1 89 97.2 104 2.5 2.8-20.3-22.5-6.5-39.7 5.4-7 18-12 26-3 6.5 7.3 10.7 18-3.4 29.7-24.7 20.4-102 82.4-127 103-12.5 10.3-28.5 2.3-35.8-6-7.5-8.9-30.6-34.6-51.3-58.2-5.5-6.3-4.1-19.6 2.3-25 35-30.3 91.9-73.8 111.9-90.8"
      transform="matrix(.08671 0 0 .0867 -49.8 -56)"
    />
  </svg>
  <svg
    v-else-if="loader === 'Quilt'"
    xmlns:xlink="http://www.w3.org/1999/xlink"
    xml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-miterlimit="2"
    clip-rule="evenodd"
    viewBox="0 0 24 24"
  >
    <defs>
      <path
        id="quilt"
        fill="none"
        stroke="currentColor"
        stroke-width="65.6"
        d="M442.5 233.9c0-6.4-5.2-11.6-11.6-11.6h-197c-6.4 0-11.6 5.2-11.6 11.6v197c0 6.4 5.2 11.6 11.6 11.6h197c6.4 0 11.6-5.2 11.6-11.7v-197Z"
      ></path>
    </defs>
    <path fill="none" d="M0 0h24v24H0z"></path>
    <use
      xlink:href="#quilt"
      stroke-width="65.6"
      transform="matrix(.03053 0 0 .03046 -3.2 -3.2)"
    ></use>
    <use xlink:href="#quilt" stroke-width="65.6" transform="matrix(.03053 0 0 .03046 -3.2 7)"></use>
    <use
      xlink:href="#quilt"
      stroke-width="65.6"
      transform="matrix(.03053 0 0 .03046 6.9 -3.2)"
    ></use>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="70.4"
      d="M442.5 234.8c0-7-5.6-12.5-12.5-12.5H234.7c-6.8 0-12.4 5.6-12.4 12.5V430c0 6.9 5.6 12.5 12.4 12.5H430c6.9 0 12.5-5.6 12.5-12.5V234.8Z"
      transform="rotate(45 3.5 24) scale(.02843 .02835)"
    ></path>
  </svg>
  <svg
    v-else-if="loader === 'Forge'"
    ml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-miterlimit="1.5"
    clip-rule="evenodd"
    viewBox="0 0 24 24"
  >
    <path fill="none" d="M0 0h24v24H0z"></path>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      d="M2 7.5h8v-2h12v2s-7 3.4-7 6 3.1 3.1 3.1 3.1l.9 3.9H5l1-4.1s3.8.1 4-2.9c.2-2.7-6.5-.7-8-6Z"
    ></path>
  </svg>
  <svg
    v-else-if="loader === 'NeoForge'"
    enable-background="new 0 0 24 24"
    version="1.1"
    viewBox="0 0 24 24"
    xml:space="preserve"
    xmlns="http://www.w3.org/2000/svg"
  >
    <g
      fill="none"
      stroke="currentColor"
      stroke-linecap="round"
      stroke-linejoin="round"
      stroke-width="2"
    >
      <path d="m12 19.2v2m0-2v2" />
      <path
        d="m8.4 1.3c0.5 1.5 0.7 3 0.1 4.6-0.2 0.5-0.9 1.5-1.6 1.5m8.7-6.1c-0.5 1.5-0.7 3-0.1 4.6 0.2 0.6 0.9 1.5 1.6 1.5"
      />
      <path d="m3.6 15.8h-1.7m18.5 0h1.7" />
      <path d="m3.2 12.1h-1.7m19.3 0h1.8" />
      <path d="m8.1 12.7v1.6m7.8-1.6v1.6" />
      <path d="m10.8 18h1.2m0 1.2-1.2-1.2m2.4 0h-1.2m0 1.2 1.2-1.2" />
      <path
        d="m4 9.7c-0.5 1.2-0.8 2.4-0.8 3.7 0 3.1 2.9 6.3 5.3 8.2 0.9 0.7 2.2 1.1 3.4 1.1m0.1-17.8c-1.1 0-2.1 0.2-3.2 0.7m11.2 4.1c0.5 1.2 0.8 2.4 0.8 3.7 0 3.1-2.9 6.3-5.3 8.2-0.9 0.7-2.2 1.1-3.4 1.1m-0.1-17.8c1.1 0 2.1 0.2 3.2 0.7"
      />
      <path
        d="m4 9.7c-0.2-1.8-0.3-3.7 0.5-5.5s2.2-2.6 3.9-3m11.6 8.5c0.2-1.9 0.3-3.7-0.5-5.5s-2.2-2.6-3.9-3"
      />
      <path d="m12 21.2-2.4 0.4m2.4-0.4 2.4 0.4" />
    </g>
  </svg>
  <svg
    v-else-if="loader === 'Paper'"
    xml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-miterlimit="1.5"
    clip-rule="evenodd"
    viewBox="0 0 24 24"
  >
    <path fill="none" d="M0 0h24v24H0z" />
    <path fill="none" stroke="currentColor" stroke-width="2" d="m12 18 6 2 3-17L2 14l6 2" />
    <path stroke="currentColor" stroke-width="2" d="m9 21-1-5 4 2-3 3Z" />
    <path fill="currentColor" d="m12 18-4-2 10-9-6 11Z" />
  </svg>
  <svg
    v-else-if="loader === 'Spigot'"
    viewBox="0 0 332 284"
    style="
      fill-rule: evenodd;
      clip-rule: evenodd;
      stroke-linejoin: round;
      fill: none;
      fill-rule: nonzero;
      stroke-width: 24px;
    "
    stroke="currentColor"
  >
    <path
      d="M147.5,27l27,-15l27.5,15l66.5,0l0,33.5l-73,-0.912l0,45.5l26,-0.088l0,31.5l-12.5,0l0,15.5l16,21.5l35,0l0,-21.5l35.5,0l0,21.5l24.5,0l0,55.5l-24.5,0l0,17l-35.5,0l0,-27l-35,0l-55.5,14.5l-67.5,-14.5l-15,14.5l18,12.5l-3,24.5l-41.5,1.5l-48.5,-19.5l6,-19l24.5,-4.5l16,-41l79,-36l-7,-15.5l0,-31.5l23.5,0l0,-45.5l-73.5,0l0,-32.5l67,0Z"
    />
  </svg>
  <svg
    v-else-if="loader === 'Bukkit'"
    viewBox="0 0 292 319"
    style="fill-rule: evenodd; clip-rule: evenodd; stroke-linecap: round; stroke-linejoin: round"
    stroke="currentColor"
  >
    <g transform="matrix(1,0,0,1,0,-5)">
      <path
        d="M12,109.5L12,155L34.5,224L57.5,224L57.5,271L81,294L160,294L160,172L259.087,172L265,155L265,109.5M12,109.5L12,64L34.5,64L34.5,41L81,17L195.5,17L241,41L241,64L265,64L265,109.5M12,109.5L81,109.5L81,132L195.5,132L195.5,109.5L265,109.5M264.087,204L264.087,244M207.5,272L207.5,312M250,272L250,312L280,312L280,272L250,272ZM192.5,204L192.5,244L222.5,244L222.5,204L192.5,204Z"
        style="fill: none; fill-rule: nonzero; stroke-width: 24px"
      />
    </g>
  </svg>
  <svg
    v-else-if="loader === 'Purpur'"
    xml:space="preserve"
    fill-rule="evenodd"
    stroke-linecap="round"
    stroke-linejoin="round"
    stroke-miterlimit="1.5"
    clip-rule="evenodd"
    viewBox="0 0 24 24"
  >
    <defs>
      <path
        id="purpur"
        fill="none"
        stroke="currentColor"
        stroke-width="1.68"
        d="m264 41.95 8-4v8l-8 4v-8Z"
      ></path>
    </defs>
    <path fill="none" d="M0 0h24v24H0z"></path>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="1.77"
      d="m264 29.95-8 4 8 4.42 8-4.42-8-4Z"
      transform="matrix(1.125 0 0 1.1372 -285 -31.69)"
    ></path>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="1.77"
      d="m272 38.37-8 4.42-8-4.42"
      transform="matrix(1.125 0 0 1.1372 -285 -31.69)"
    ></path>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="1.77"
      d="m260 31.95 8 4.21V45"
      transform="matrix(1.125 0 0 1.1372 -285 -31.69)"
    ></path>
    <path
      fill="none"
      stroke="currentColor"
      stroke-width="1.77"
      d="M260 45v-8.84l8-4.21"
      transform="matrix(1.125 0 0 1.1372 -285 -31.69)"
    ></path>
    <use
      xlink:href="#purpur"
      stroke-width="1.68"
      transform="matrix(1.125 0 0 1.2569 -285 -40.78)"
    ></use>
    <use
      xlink:href="#purpur"
      stroke-width="1.68"
      transform="matrix(-1.125 0 0 1.2569 309 -40.78)"
    ></use>
  </svg>
  <svg v-else-if="loader === 'Vanilla'" viewBox="0 0 20 20" fill="currentColor">
    <path
      fill-rule="evenodd"
      d="M9.504 1.132a1 1 0 01.992 0l1.75 1a1 1 0 11-.992 1.736L10 3.152l-1.254.716a1 1 0 11-.992-1.736l1.75-1zM5.618 4.504a1 1 0 01-.372 1.364L5.016 6l.23.132a1 1 0 11-.992 1.736L4 7.723V8a1 1 0 01-2 0V6a.996.996 0 01.52-.878l1.734-.99a1 1 0 011.364.372zm8.764 0a1 1 0 011.364-.372l1.733.99A1.002 1.002 0 0118 6v2a1 1 0 11-2 0v-.277l-.254.145a1 1 0 11-.992-1.736l.23-.132-.23-.132a1 1 0 01-.372-1.364zm-7 4a1 1 0 011.364-.372L10 8.848l1.254-.716a1 1 0 11.992 1.736L11 10.58V12a1 1 0 11-2 0v-1.42l-1.246-.712a1 1 0 01-.372-1.364zM3 11a1 1 0 011 1v1.42l1.246.712a1 1 0 11-.992 1.736l-1.75-1A1 1 0 012 14v-2a1 1 0 011-1zm14 0a1 1 0 011 1v2a1 1 0 01-.504.868l-1.75 1a1 1 0 11-.992-1.736L16 13.42V12a1 1 0 011-1zm-9.618 5.504a1 1 0 011.364-.372l.254.145V16a1 1 0 112 0v.277l.254-.145a1 1 0 11.992 1.736l-1.735.992a.995.995 0 01-1.022 0l-1.735-.992a1 1 0 01-.372-1.364z"
      clip-rule="evenodd"
    ></path>
  </svg>
  <LoaderIcon v-else />
</template>

<script setup lang="ts">
import { LoaderIcon } from "@modrinth/assets";
import type { Loaders } from "~/types/servers";

defineProps<{
  loader: Loaders;
}>();
</script>
