<script setup>
import {
  TrashIcon,
  SearchIcon,
  BoxIcon,
  SendIcon,
  EditIcon,
  DownloadIcon,
  LinkIcon,
} from "@modrinth/assets";
import { Avatar, Checkbox, Badge } from "@modrinth/ui";
import LogoAnimated from "~/components/brand/LogoAnimated.vue";
import PrismIcon from "~/assets/images/external/prism.svg?component";
import ATLauncher from "~/assets/images/external/atlauncher.svg?component";
import CurseForge from "~/assets/images/external/curseforge.svg?component";
import LatestNewsRow from "~/components/ui/news/LatestNewsRow.vue";

import { homePageProjects } from "~/generated/state.json";

const os = ref(null);
const downloadWindows = ref(null);
const downloadLinux = ref(null);
const downloadSection = ref(null);
const windowsLink = ref(null);
const linuxLinks = {
  appImage: null,
  deb: null,
  rpm: null,
  thirdParty: "https://support.modrinth.com/en/articles/9298760",
};
const macLinks = {
  universal: null,
};

let downloadLauncher;

const newProjects = homePageProjects.slice(0, 40);
const val = Math.ceil(newProjects.length / 6);
const rows = ref([
  newProjects.slice(0, val),
  newProjects.slice(val, val * 2),
  newProjects.slice(val * 2, val * 3),
  newProjects.slice(val * 3, val * 4),
  newProjects.slice(val * 4, val * 5),
]);

const [{ data: launcherUpdates }] = await Promise.all([
  await useAsyncData("launcherUpdates", () =>
    $fetch("https://launcher-files.modrinth.com/updates.json"),
  ),
]);

macLinks.universal = launcherUpdates.value.platforms["darwin-aarch64"].install_urls[0];
windowsLink.value = launcherUpdates.value.platforms["windows-x86_64"].install_urls[0];
linuxLinks.appImage = launcherUpdates.value.platforms["linux-x86_64"].install_urls[1];
linuxLinks.deb = launcherUpdates.value.platforms["linux-x86_64"].install_urls[0];
linuxLinks.rpm = launcherUpdates.value.platforms["linux-x86_64"].install_urls[2];

onMounted(() => {
  os.value = navigator?.platform.toString();
  os.value = os.value?.includes("Mac")
    ? "Mac"
    : os.value?.includes("Win")
      ? "Windows"
      : os.value?.includes("Linux")
        ? "Linux"
        : null;

  if (os.value === "Windows") {
    downloadLauncher = () => {
      downloadWindows.value.click();
    };
  } else if (os.value === "Linux") {
    downloadLauncher = () => {
      downloadLinux.value.click();
    };
  } else {
    downloadLauncher = () => {
      scrollToSection();
    };
  }
});

const scrollToSection = () => {
  nextTick(() => {
    window.scrollTo({
      top: downloadSection.value.offsetTop,
      behavior: "smooth",
    });
  });
};

const title = "Download the Modrinth App!";
const description =
  "The Modrinth App is a unique, open source launcher that allows you to play your favorite mods, and keep them up to date, all in one neat little package.";

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
});
</script>

<template>
  <div>
    <div class="landing-hero">
      <h1 class="main-header">
        Download Modrinth <br v-if="os" />
        App
        {{ os ? `for ${os}` : "" }}
      </h1>
      <h2 class="main-subheader">
        The Modrinth App is a unique, open source launcher that allows you to play your favorite
        mods, and keep them up to date, all in one neat little package.
      </h2>
      <div class="button-group">
        <button
          v-if="os"
          class="iconified-button brand-button btn btn-large"
          rel="noopener nofollow"
          @click="downloadLauncher"
        >
          <svg
            v-if="os === 'Linux'"
            class="light-icon"
            width="45"
            height="46"
            viewBox="0 0 45 46"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              id="Subtract"
              fill-rule="evenodd"
              clip-rule="evenodd"
              d="M23.5477 0H23.5461C23.2284 0 22.9077 0.0163711 22.5876 0.0415902C20.4638 0.208942 18.8991 0.910133 17.7996 2.0082C16.7124 3.09404 16.1751 4.47323 15.9172 5.82868C15.6602 7.18017 15.6685 8.57463 15.7244 9.76049C15.7464 10.2267 15.7737 10.6424 15.7981 11.0129L15.7981 11.0129C15.8393 11.6392 15.8719 12.1361 15.856 12.5288C15.7269 14.3647 15.3652 15.7513 14.1358 17.5116C12.4894 19.4715 10.1353 22.6851 9.00147 26.0938C8.50987 27.5652 8.23247 29.1259 8.37804 30.6595C8.14194 30.9205 7.94151 31.1909 7.77396 31.4209L7.69935 31.5235C7.53577 31.7486 7.41036 31.9212 7.28343 32.0672C7.22885 32.1165 7.13829 32.1747 6.95723 32.2529C6.85646 32.2965 6.74665 32.3386 6.60714 32.3918L6.60229 32.3937L6.60063 32.3943C6.4689 32.4446 6.31402 32.5038 6.15349 32.5722L6.14713 32.575C6.11461 32.5891 6.08029 32.6038 6.04439 32.6191L6.04194 32.6201C5.49857 32.8521 4.5956 33.2374 4.06765 34.2907L4.06313 34.2998L4.05878 34.3089C3.83111 34.787 3.71086 35.3126 3.71965 35.8518C3.72044 36.2784 3.77526 36.6973 3.83488 36.9965L3.8573 37.1494C3.90153 37.4502 3.93607 37.685 3.9521 37.8975C3.96952 38.1284 3.9557 38.2416 3.94333 38.2911C3.47501 39.5811 3.29341 40.7747 3.80981 41.7808L3.81261 41.7862C4.34311 42.8045 5.41437 43.1509 6.09597 43.3713L6.14873 43.3884L6.1831 43.3968C6.76559 43.5407 7.45296 43.6219 8.08651 43.6967L8.08657 43.6967C8.33286 43.7258 8.57102 43.7539 8.79172 43.7844C9.65223 43.9032 10.4057 44.0681 11.0346 44.4066L11.0467 44.4131L11.0589 44.4193C12.8487 45.3199 14.8309 45.8073 16.5413 45.3594C17.5947 45.1214 18.5098 44.4944 19.1114 43.6161C19.5314 43.5681 19.9501 43.481 20.3513 43.3976L20.4887 43.3691C21.1456 43.2333 21.8552 43.0939 22.7255 43.039L22.7353 43.0384L22.7452 43.0376C23.2171 42.9984 23.7676 43.0823 24.5478 43.2012L24.6654 43.2192C25.2572 43.3092 25.9398 43.4072 26.6797 43.4223L26.7366 43.5729L26.7865 43.6228C27.7132 45.3272 29.3865 46.1223 31.1344 45.9841C32.8861 45.8473 34.6161 44.7911 35.9501 43.2465L35.9575 43.2379L35.9647 43.2291C36.4178 42.6799 37.0511 42.2572 37.7983 41.8603C38.1016 41.6992 38.4052 41.5515 38.7175 41.3996L38.7179 41.3994L38.936 41.2932C39.3111 41.1099 39.7022 40.9143 40.0596 40.6996C40.783 40.2838 41.6765 39.5491 41.7413 38.3282C41.8014 37.2663 41.2531 36.3256 40.4338 35.3719V35.1588L40.1744 34.8993C40.1374 34.8477 40.0795 34.7411 40.0113 34.5353C39.934 34.302 39.8672 34.0188 39.7856 33.6562C39.6368 32.9545 39.4127 31.9102 38.5493 31.186L38.3995 31.0604C38.9576 28.7316 38.5908 26.4633 37.8259 24.46C36.7898 21.7201 34.9984 19.3692 33.6785 17.7973L33.6778 17.7965C32.1849 15.9116 31.019 14.4397 31.0434 12.2669C31.0453 12.1131 31.0478 11.9522 31.0504 11.7849C31.0801 9.87812 31.1226 7.15021 30.3612 4.83914C29.9387 3.55665 29.2488 2.33019 28.115 1.42642C26.9706 0.514298 25.4646 0.00304389 23.5477 0ZM22.666 1.0385C22.9685 1.01467 23.2618 1 23.546 1C30.2102 1.01058 30.1098 7.76032 30.0499 11.7866C30.0475 11.9472 30.0452 12.1035 30.0433 12.2548C30.0146 14.7849 31.382 16.51 32.8074 18.3084C32.8393 18.3487 32.8713 18.3891 32.9033 18.4295C34.205 19.9787 35.9137 22.23 36.8908 24.815C37.692 26.9123 37.9982 29.2443 37.208 31.5873C37.3329 31.5924 37.4538 31.6331 37.5563 31.7047C37.6223 31.7737 37.6877 31.8118 37.7513 31.8488C37.8026 31.8787 37.8527 31.908 37.901 31.9522H37.9065C38.4748 32.4288 38.6527 33.1347 38.8085 33.8698C38.9698 34.5867 39.1165 35.2008 39.4282 35.5675L39.4337 35.573V35.7508C40.376 36.7867 40.7848 37.5402 40.7427 38.2735C40.706 38.9775 40.1908 39.4725 39.5528 39.8373C39.1757 40.0647 38.7412 40.276 38.2863 40.4973C37.2045 41.0234 36.0076 41.6055 35.1932 42.5928C33.974 44.0045 32.4688 44.8772 31.0553 44.9872C29.6418 45.099 28.3182 44.45 27.6013 43.0237L27.5958 43.0182C27.5697 42.949 27.5454 42.89 27.523 42.8356C27.4653 42.6951 27.4199 42.5847 27.3868 42.4077C26.4008 42.4735 25.4823 42.3328 24.6448 42.2044C23.9204 42.0934 23.2567 41.9917 22.6623 42.041C21.6653 42.1039 20.8661 42.2696 20.1753 42.4127C19.5578 42.5407 19.027 42.6507 18.519 42.6533C18.0827 43.537 17.2687 44.175 16.3043 44.3877C14.9293 44.7543 13.206 44.3803 11.5083 43.526C10.4773 42.9711 9.26303 42.8279 8.13135 42.6944C7.52434 42.6228 6.9411 42.554 6.42267 42.426C5.68017 42.1858 5.01833 41.9365 4.69933 41.3242C4.38033 40.7027 4.43716 39.8502 4.89366 38.6035C5.02207 38.1946 4.94184 37.6503 4.84524 36.995C4.83702 36.9392 4.82868 36.8826 4.82033 36.8252C4.769 36.5758 4.7195 36.2073 4.7195 35.8425C4.71216 35.4612 4.7965 35.0853 4.9615 34.7388C5.32126 34.021 5.91228 33.7658 6.46354 33.5276C6.49097 33.5158 6.5183 33.504 6.5455 33.4922C6.68913 33.4309 6.82976 33.3772 6.96561 33.3253C7.37077 33.1706 7.73344 33.0321 8.00667 32.7588C8.17742 32.5672 8.34012 32.343 8.50901 32.1103C8.72634 31.8108 8.95394 31.4971 9.22217 31.2207C9.27154 31.1251 9.34021 31.0408 9.42383 30.9732C9.19833 29.4973 9.44033 27.9353 9.95 26.41C11.0298 23.1632 13.3068 20.0483 14.9293 18.1215C16.3043 16.1653 16.715 14.5868 16.8543 12.5848C16.8738 12.1381 16.8352 11.5454 16.7908 10.8636C16.5636 7.37332 16.1844 1.54924 22.666 1.0385ZM24.5397 7.2425H24.5158C24.1226 7.24289 23.7402 7.3698 23.4268 7.60733C23.065 7.83891 22.7883 8.18188 22.6385 8.5845C22.4431 9.02748 22.3491 9.50854 22.3635 9.9925V10.0292C22.367 10.1519 22.378 10.2744 22.3965 10.3958C22.582 10.4694 22.8174 10.5436 23.0524 10.6176C23.2099 10.6673 23.3673 10.7169 23.5093 10.7662C23.4873 10.6048 23.4763 10.4398 23.4727 10.2767V10.2418C23.4845 9.98963 23.5403 9.74144 23.6377 9.5085C23.7102 9.27639 23.8394 9.06596 24.0135 8.89617C24.1712 8.70917 24.3233 8.65417 24.4938 8.65417H24.5232C24.7047 8.65417 24.8557 8.77281 25.0081 8.89261L25.0127 8.89617C25.0488 8.95476 25.0822 9.00626 25.1132 9.0541C25.2112 9.20535 25.2855 9.32003 25.3482 9.50667C25.4263 9.7431 25.4641 9.99101 25.46 10.24V10.2767C25.449 10.5297 25.3958 10.779 25.2987 11.0137C25.2573 11.0863 25.1757 11.1644 25.0976 11.2393C25.0476 11.2872 24.9989 11.3337 24.9632 11.3767C25.1009 11.4051 25.2359 11.4456 25.3665 11.4977C25.4774 11.5867 25.5956 11.6303 25.7223 11.6771C25.7757 11.6968 25.8306 11.717 25.8872 11.7415C25.9426 11.7634 25.9966 11.7892 26.0485 11.8185C26.2203 11.6435 26.3534 11.4342 26.439 11.2043C26.6168 10.7958 26.7103 10.3555 26.714 9.91L26.7213 9.954C26.7222 9.96709 26.7246 9.98002 26.7287 9.9925V9.8C26.7287 9.81833 26.7259 9.83667 26.7232 9.855C26.7204 9.87333 26.7177 9.89167 26.7177 9.91C26.703 9.42417 26.6058 9.0575 26.4133 8.58267C26.2153 8.21417 25.9587 7.853 25.6103 7.6055C25.2657 7.35617 24.9302 7.2425 24.5397 7.2425ZM19.1515 7.35067H19.0855C18.7958 7.35067 18.5538 7.4735 18.3192 7.71733C18.0782 8.00351 17.9013 8.33795 17.8003 8.69817C17.6731 9.09315 17.6233 9.50898 17.6537 9.92283C17.668 10.345 17.7645 10.7604 17.9378 11.1457C18.0826 11.4065 18.2514 11.6532 18.442 11.8827C18.5311 11.8615 18.6136 11.8187 18.6822 11.758C18.748 11.6843 18.8058 11.6467 18.8698 11.6049C18.9109 11.578 18.9547 11.5495 19.0048 11.5087C18.9792 11.4977 18.9608 11.4812 18.9425 11.4647C18.8178 11.34 18.7225 11.219 18.6382 10.9733C18.6295 10.9464 18.6213 10.9211 18.6135 10.8969C18.5496 10.7 18.5111 10.5814 18.4915 10.361V10.3335C18.4698 10.1309 18.4841 9.92611 18.5337 9.7285C18.5607 9.54657 18.6367 9.37544 18.7537 9.2335C18.8563 9.06483 18.9645 9.0025 19.0947 8.9915H19.1332C19.1941 8.98752 19.2552 8.99587 19.3129 9.01602C19.3705 9.03618 19.4235 9.06775 19.4687 9.10883C19.6109 9.24277 19.7158 9.41148 19.773 9.59833C19.7858 9.63881 19.7979 9.67596 19.8092 9.71087C19.8662 9.88628 19.905 10.0055 19.9233 10.2088V10.2363C19.9417 10.3958 19.9398 10.5608 19.9178 10.7258C19.9701 10.6973 20.0226 10.6672 20.0762 10.6365C20.2463 10.5391 20.4264 10.4359 20.6383 10.3592C20.6632 10.3536 20.6875 10.3458 20.7126 10.3378C20.7375 10.3298 20.7631 10.3216 20.7905 10.3152V10.1685C20.8052 10.0475 20.807 9.9265 20.7942 9.68083V9.6735C20.7703 9.184 20.6787 8.8155 20.5137 8.45067C20.367 8.12617 20.1507 7.83467 19.883 7.59817C19.6465 7.43867 19.4118 7.35067 19.1515 7.35067ZM23.8265 11.2098C22.941 10.5975 22.2058 10.4783 21.5972 10.4783H21.599C21.2837 10.4783 20.9995 10.5315 20.7447 10.5975C20.0058 10.8377 19.5163 11.329 19.2065 11.6993V11.7048H19.2028C19.1423 11.758 19.0708 11.8093 18.8802 11.945C18.8088 11.99 18.7236 12.0537 18.623 12.1287C18.4574 12.2523 18.2501 12.407 17.9947 12.561C17.6317 12.7993 17.5125 13.1697 17.6408 13.661C17.7692 14.024 18.1652 14.5117 18.8985 14.9627H18.9077V14.9718C19.1865 15.1208 19.4099 15.3135 19.6231 15.4973C19.7589 15.6145 19.8905 15.728 20.0297 15.8243C20.2199 15.9557 20.4169 16.0769 20.62 16.1873C20.8828 16.2829 21.1621 16.3246 21.4413 16.3102C22.2045 16.3552 22.7698 16.0785 23.269 15.8342L23.2967 15.8207C23.5576 15.695 23.7982 15.5366 24.0381 15.3788C24.2601 15.2327 24.4814 15.0871 24.7175 14.9682V14.9645C25.7203 14.5997 26.4408 13.9892 26.6663 13.4208C26.7229 13.2834 26.7496 13.1354 26.7445 12.9868C26.7394 12.8383 26.7028 12.6925 26.637 12.5592V12.7993C26.5032 12.5555 26.2282 12.3172 25.7607 12.0678H25.7552C25.6086 12.0127 25.4746 11.963 25.3509 11.9171C24.6553 11.6593 24.2825 11.5211 23.8265 11.2098ZM29.9113 22.6058C28.9232 20.7762 27.3868 17.0032 26.7305 14.4053H26.7287C26.3455 14.9517 25.6708 15.4412 24.8147 15.6887H24.8055C24.5824 15.7556 24.3538 15.8976 24.104 16.0528C23.9021 16.1782 23.6865 16.3122 23.4488 16.422C22.9593 16.6677 22.3763 16.9133 21.621 16.9133C21.5616 16.9133 21.5005 16.9086 21.435 16.9035L21.4193 16.9023C20.6878 16.9023 20.1983 16.6567 19.8133 16.2882C19.626 16.1704 19.4595 16.0231 19.2918 15.8748C19.115 15.7184 18.9368 15.5608 18.7317 15.4357C18.6433 15.3534 18.4753 15.2449 18.3474 15.1624C18.2843 15.1216 18.2309 15.0872 18.2018 15.0653C18.0598 17.7587 16.5369 21.0396 15.4344 23.4147C15.2131 23.8914 15.0087 24.3316 14.8358 24.7233C14.1222 26.3938 13.7202 28.1808 13.6497 29.996C11.7833 27.5082 13.1473 24.3072 13.8733 22.9303C14.6892 21.3922 14.8175 21.0255 14.6213 21.1447C13.8862 22.3418 12.7348 24.2485 12.2857 26.2102C12.051 27.2368 12.0107 28.247 12.3132 29.226C12.6138 30.1903 13.2665 31.052 14.4215 31.7853C15.9927 32.7497 17.067 33.7177 17.6995 34.5738C18.332 35.4282 18.53 36.2165 18.332 36.706C18.2262 36.9767 18.02 37.196 17.7563 37.3183C17.5235 37.3862 17.2375 37.4412 16.9038 37.4412C17.1014 37.6184 17.2714 37.8241 17.408 38.0517C17.694 38.345 17.9416 38.6733 18.145 39.0288C21.3863 41.2362 25.1997 40.3818 27.9497 38.4165C27.9605 38.3792 27.9714 38.3419 27.9822 38.3047C28.2443 37.4049 28.4993 36.5295 28.5345 35.9415V35.936C28.5987 34.6288 28.672 33.4867 28.9103 32.5095C29.1505 31.5598 29.5758 30.8247 30.3642 30.3333C30.4616 30.318 30.56 30.2827 30.6563 30.2481C30.6869 30.2371 30.7173 30.2262 30.7473 30.216C30.7558 30.0996 30.7642 30.0546 30.7769 29.9868C30.778 29.9809 30.7792 29.9748 30.7803 29.9685C31.0077 28.5 32.3478 28.3827 34.0253 29.116C35.6442 29.8475 36.2547 30.5038 35.9778 31.36H36.0072C36.0438 31.36 36.0805 31.3594 36.1172 31.3588C36.1905 31.3576 36.2638 31.3563 36.3372 31.36V31.4388C36.6727 30.3407 35.9742 29.6 34.0822 28.621C33.9694 28.6097 33.8595 28.5631 33.7512 28.5171C33.7362 28.5108 33.7213 28.5044 33.7063 28.4982C33.9612 27.4605 33.9062 26.4173 33.6678 25.4365C33.1527 23.264 31.7318 21.361 30.6502 20.382C30.4503 20.382 30.4723 20.6295 30.8757 20.9962C31.8712 21.911 34.0565 25.2037 32.8722 28.2672C32.5367 28.1828 32.2177 28.1407 31.9317 28.1498C31.4788 25.6473 30.4357 23.5848 29.9113 22.6058ZM32.3533 33.3528C31.3652 33.1108 30.8042 31.9283 30.7363 30.8283V30.832L30.6538 30.8723C30.0433 31.2152 29.6987 31.8238 29.4787 32.6763C29.2605 33.5545 29.189 34.693 29.1212 36.002C29.0756 36.7223 28.8292 37.5563 28.5746 38.4181C28.4709 38.7694 28.3658 39.1252 28.2723 39.4798C27.9497 40.699 27.7847 41.8448 28.1697 42.5782L28.1788 42.5837C28.7967 43.8633 29.8472 44.3272 31.0645 44.23C32.2855 44.1365 33.6642 43.3977 34.7972 42.0465C35.7047 40.9513 36.9722 40.3422 38.0484 39.8251C38.5178 39.5996 38.9508 39.3915 39.3017 39.1682C39.8755 38.8033 40.1872 38.5595 40.2147 38.0792L40.2202 38.0993V38.0663L40.2147 38.0792C40.2312 37.6502 39.9397 36.9553 39.0487 35.9892C38.6065 35.4932 38.4694 34.8431 38.324 34.1536C38.3156 34.1142 38.3073 34.0745 38.2988 34.0348C38.1412 33.4005 37.9633 32.7937 37.582 32.4288L37.5765 32.4233C37.0815 31.9503 36.6012 31.9192 36.0237 31.932L35.6368 31.9503C34.9823 32.7423 33.4313 33.5985 32.3533 33.3528ZM10.929 30.9182H10.9107C10.3918 30.9182 10.0435 31.1657 9.69517 31.5342C9.5004 31.7412 9.31978 31.9881 9.13328 32.2431C8.92831 32.5233 8.71624 32.8132 8.4705 33.0705V33.0742H8.46683C8.1027 33.429 7.67066 33.5917 7.25569 33.7479C7.0961 33.808 6.93904 33.8671 6.78933 33.9358C6.72367 33.9655 6.65958 33.9934 6.59713 34.0206C6.14698 34.2164 5.78193 34.3753 5.52433 34.913C5.29333 35.3347 5.34467 35.9012 5.4455 36.6363C5.45614 36.7047 5.46735 36.774 5.47868 36.844C5.57861 37.4618 5.68765 38.136 5.495 38.686L5.49133 38.6915V38.6988C5.05867 39.8502 5.06417 40.5872 5.26583 40.9538C5.473 41.3205 5.91483 41.5662 6.59133 41.6872C6.96665 41.7547 7.39979 41.7941 7.86764 41.8367C9.08866 41.9478 10.5461 42.0804 11.8292 42.7908C13.4333 43.6452 15.0375 43.889 16.22 43.6415C17.4025 43.3738 18.178 42.6332 18.2825 41.2692V41.2582C18.3577 40.4533 17.7417 39.4762 16.946 38.3798C16.7146 38.075 16.4714 37.7766 16.2319 37.4828C15.6462 36.764 15.0824 36.0723 14.7662 35.3787L14.7607 35.3732L13.0923 32.3225C12.5112 31.5543 11.9062 31.0447 11.2168 30.9438C11.1215 30.9273 11.0262 30.9182 10.929 30.9182ZM23.5083 14.5427C23.6897 14.4504 23.8657 14.3609 24.0355 14.2825L24.0337 14.2843C24.4698 14.0202 24.892 13.7339 25.2987 13.4263C25.6305 13.0633 25.8028 12.847 25.5682 12.8177C25.4293 12.8177 25.3757 12.8992 25.2985 13.0166C25.2433 13.1007 25.176 13.2031 25.0567 13.3072C24.8433 13.4469 24.6017 13.6265 24.3669 13.8009C24.1922 13.9307 24.0212 14.0577 23.8687 14.1633C23.2362 14.5282 22.1948 15.0177 21.3057 15.0177C20.4161 15.0177 19.7068 14.5318 19.1721 14.1656L19.1662 14.1615C19.0678 14.0713 18.976 13.9809 18.8905 13.8966C18.7447 13.753 18.6171 13.6272 18.5062 13.551C18.4612 13.5086 18.4274 13.4544 18.394 13.4007C18.332 13.3013 18.2711 13.2034 18.1413 13.1843C18.013 13.1843 17.9763 13.5492 18.277 13.7948C18.3752 13.8656 18.4936 13.9772 18.6302 14.106C18.7307 14.2008 18.8412 14.3049 18.9608 14.409C19.5292 14.7738 20.3138 15.2633 21.3075 15.2633C22.0911 15.2633 22.8372 14.8839 23.5083 14.5427ZM22.0518 11.34V11.3803C22.0735 11.4531 22.1637 11.4663 22.2486 11.4788C22.2856 11.4842 22.3217 11.4895 22.3507 11.4995C22.3795 11.5142 22.4061 11.5324 22.4323 11.5502C22.4875 11.5879 22.5407 11.6242 22.6092 11.6242C22.7008 11.6242 22.8457 11.593 22.8567 11.5032C22.875 11.3785 22.6972 11.2557 22.5817 11.2557C22.435 11.1988 22.2407 11.1695 22.105 11.2483C22.0757 11.263 22.0408 11.3033 22.0518 11.34ZM20.8356 11.482C20.9223 11.4694 21.0157 11.4557 21.0417 11.3803V11.3767H21.0362V11.34C21.0472 11.3033 21.016 11.2648 20.9812 11.2483C20.8437 11.1713 20.653 11.2007 20.5063 11.2575C20.3927 11.2575 20.213 11.3803 20.2313 11.5013C20.2423 11.5893 20.389 11.626 20.4807 11.626C20.5494 11.626 20.6048 11.588 20.6612 11.5495C20.686 11.5325 20.711 11.5154 20.7373 11.5013C20.7656 11.4923 20.8 11.4872 20.8356 11.482Z"
              fill="white"
            />
          </svg>
          <svg
            v-if="os === 'Linux'"
            class="dark-icon"
            role="img"
            viewBox="0 0 24 24"
            xmlns="http://www.w3.org/2000/svg"
          >
            <path
              d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"
            />
          </svg>
          <svg
            v-else-if="os === 'Windows'"
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 4875 4875"
            fill="currentColor"
          >
            <path
              d="M0 0h2311v2310H0zm2564 0h2311v2310H2564zM0 2564h2311v2311H0zm2564 0h2311v2311H2564"
            />
          </svg>
          <svg
            v-else-if="os === 'Mac'"
            xmlns="http://www.w3.org/2000/svg"
            width="44"
            height="44"
            viewBox="0 0 44 44"
            fill="none"
          >
            <path
              d="M33.5752 37.7889C31.3308 39.8664 28.8802 39.5384 26.5212 38.5543C24.0248 37.5484 21.7345 37.5046 19.1007 38.5543C15.8027 39.9102 14.0621 39.5165 12.0924 37.7889C0.915868 26.789 2.56487 10.0377 15.253 9.42536C18.3449 9.57844 20.4978 11.0436 22.3071 11.1748C25.0096 10.65 27.5976 9.14107 30.4834 9.33789C33.9417 9.60031 36.5526 10.9124 38.2703 13.2742C31.1247 17.3637 32.8195 26.3516 39.3697 28.8665C38.0642 32.1468 36.3694 35.4052 33.5523 37.8108L33.5752 37.7889ZM22.078 9.29415C21.7345 4.41745 25.8799 0.393635 30.6437 0C31.3079 5.6421 25.2844 9.84086 22.078 9.29415Z"
              fill="currentColor"
            />
          </svg>
          Download the Modrinth App
        </button>
        <button class="iconified-button outline-button btn btn-large" @click="scrollToSection">
          More Download Options
        </button>
      </div>
      <img src="https://cdn-raw.modrinth.com/app-landing/app-screenshot.webp" alt="cube maze" />
      <div class="bottom-transition" />
    </div>
    <div class="features">
      <h1 class="subheader">
        Unlike any launcher <br />
        you've used before
      </h1>
      <div class="feature-grid">
        <div class="feature gradient-border mods">
          <div class="search-bar">
            <h4>Installed mods</h4>
            <div class="mini-input">
              <SearchIcon aria-hidden="true" />
              <div class="search">Search mods</div>
            </div>
          </div>
          <div class="header row">
            <div />
            <div class="cell">Name</div>
            <div class="cell">Version</div>
            <div class="cell">Actions</div>
          </div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/P7dR8mSH/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Fabric API</div>
                <div class="description">by modmuss</div>
              </div>
              <div class="cell important">0.86.1+1.20.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/AANobbMI/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Sodium</div>
                <div class="description">by jellysquid3</div>
              </div>
              <div class="cell">mc1.20.1-0.5.0</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/YL57xq9U/dc558eece920db435f9823ce86de0c4cde89800b.png"
                />
              </div>
              <div class="cell">
                <div class="name">Iris Shaders</div>
                <div class="description">by coderbot</div>
              </div>
              <div class="cell">1.6.5+1.20.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/gvQqBUqZ/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Lithium</div>
                <div class="description">by jellysquid3</div>
              </div>
              <div class="cell">mc1.20.1-0.11.2</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/mOgUt4GM/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Mod Menu</div>
                <div class="description">by Prospector</div>
              </div>
              <div class="cell">7.2.1</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/9s6osm5g/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Cloth Config API</div>
                <div class="description">by shedaniel</div>
              </div>
              <div class="cell">11.1.106+fabric</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar size="sm" src="https://cdn.modrinth.com/data/lhGA9TYQ/icon.png" />
              </div>
              <div class="cell">
                <div class="name">Architectury API</div>
                <div class="description">by shedaniel</div>
              </div>
              <div class="cell">9.1.12+fabric</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/nrJ2NpD0/0efcf28eb5c18bed0cc47d786879e32550861ca4.png"
                />
              </div>
              <div class="cell">
                <div class="name">Craftify</div>
                <div class="description">by ThatGravyBoat</div>
              </div>
              <div class="cell">8.5.2023</div>
              <div class="cell check">
                <Checkbox :model-value="true" tabindex="-1" />
                <button class="btn icon-only transparent" tabindex="-1">
                  <TrashIcon />
                </button>
              </div>
            </div>
          </div>
          <h3>Mod management</h3>
          <p>
            Modrinth makes it easy to manage all your mods in one place. You can install, uninstall,
            and update mods with a single click.
          </p>
        </div>
        <div class="feature gradient-border playing">
          <div class="text">
            <h3>Play with your favorite mods</h3>
            <p>Use the Modrinth App to download and play with your favorite mods and modpacks.</p>
          </div>
          <img
            src="https://cdn-raw.modrinth.com/app-landing/cobblemon-launcher.webp"
            alt="The Modrinth App playing Cobblemon for Fabric"
            class="launcher"
          />
          <img
            src="https://cdn-raw.modrinth.com/app-landing/cobblemon.webp"
            alt="Minecraft playing Cobblemon for Fabric"
            class="minecraft"
          />
        </div>
        <div class="feature gradient-border sharing">
          <div class="row header">Included mods <EditIcon /></div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/3ufwT9JF/2a15f23b7ffa2d50fc6ae1c42029a728ce3e2847.jpeg"
                />
              </div>
              <div class="cell">Ad Astra</div>
              <div class="cell">
                <div class="description">Live long and prosper, Ad Astra!</div>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/b1LdOZlE/465598dc5d89f67fb8f8de6def21240fa35e3a54.png"
                />
              </div>
              <div class="cell">Spirit</div>
              <div class="cell">
                <div class="description">
                  A mod exploring soul magic in tools, blocks, weapons and more.
                </div>
              </div>
            </div>
            <div class="row">
              <div class="cell">
                <Avatar
                  size="sm"
                  src="https://cdn.modrinth.com/data/MI1LWe93/d42fb7a69f1e7a86584fa1ed43520af98acec065.png"
                />
              </div>
              <div class="cell">Creeper Overhaul</div>
              <div class="cell">A mod which overhauls the vanilla creepers!</div>
            </div>
          </div>
          <h3>Share Modpacks</h3>
          <p>
            Build, share, and play modpacks with any of the thousands of mods and modpacks hosted
            here on Modrinth.
          </p>
          <div class="export-card">
            <Avatar
              src="https://cdn.modrinth.com/data/mY0lOQFc/81c6eff2b86220e12e62a4ad0d2f364a605c42c4.png"
            />
            <div class="info">
              <div class="exporting">
                <div class="tag"><BoxIcon /> Modpack</div>
                <div class="small-button">
                  Share
                  <SendIcon />
                </div>
              </div>
              <h4 class="name">All of Fabric | Orion</h4>
              <div class="author">by AK</div>
            </div>
          </div>
        </div>
        <div class="feature gradient-border performance">
          <div class="title">
            <h4>Activity monitor</h4>
            <Badge color="green" type="Good performance" />
          </div>
          <div class="header row">
            <div />
            <div class="cell">Process name</div>
            <div class="cell">% CPU</div>
            <div class="cell">RAM</div>
          </div>
          <div class="table">
            <div class="row first">
              <div class="cell">
                <div>
                  <div class="icon-logo modrinth">
                    <LogoAnimated class="icon" />
                  </div>
                </div>
              </div>
              <div class="cell important">Modrinth App</div>
              <div class="cell important">Small</div>
              <div class="cell important">{{ "< 150 MB" }}</div>
            </div>
            <div class="row">
              <div class="cell">
                <div>
                  <div class="icon-logo"></div>
                </div>
              </div>
              <div class="cell">Google Chrome</div>
              <div class="cell">150%</div>
              <div class="cell">∞ MB</div>
            </div>
            <div class="row">
              <div class="cell">
                <div>
                  <div class="icon-logo"></div>
                </div>
              </div>
              <div class="cell">Discord</div>
              <div class="cell">1 billion %</div>
              <div class="cell">∞ * ∞ MB</div>
            </div>
          </div>
          <h3>Performant</h3>
          <p>
            The Modrinth App performs better than many of the leading mod managers, using just 150mb
            of RAM!
          </p>
        </div>
        <div class="feature gradient-border website">
          <div class="icon-logo">
            <LogoAnimated class="icon" />
          </div>
          <div class="ellipsis" />
          <div class="projects-showcase">
            <div v-for="(row, index) in rows" :key="index" class="row">
              <div v-for="n in 2" :key="n" class="row__content" :class="{ offset: index % 2 }">
                <nuxt-link
                  v-for="project in row"
                  :key="project.id"
                  class="project button-animation gradient-border"
                  :to="`/${project.project_type}/${project.slug ? project.slug : project.id}`"
                >
                  <Avatar :src="project.icon_url" :alt="project.title" size="sm" loading="lazy" />
                  <div class="project-info">
                    <span class="title">
                      {{ project.title }}
                    </span>
                    <span class="description">
                      {{ project.description }}
                    </span>
                  </div>
                </nuxt-link>
              </div>
            </div>
          </div>
          <h3>Website Integration</h3>
          <p>
            The Modrinth App is fully integrated with the website, so you can access all your
            favorite projects from the app!
          </p>
        </div>
        <div class="feature gradient-border importing">
          <div class="text">
            <h3>Profile importing</h3>
            <p>
              Import all your favorite profiles from the launcher you were using before, and get
              started with the Modrinth App in seconds!
            </p>
          </div>
          <div class="inner-ring ring">
            <div class="icon-logo">
              <LogoAnimated class="icon" />
            </div>
            <div class="launcher-badge top-left">
              <img src="~/assets/images/external/gdlauncher.png" alt="GDLauncher" />
            </div>
            <div class="launcher-badge top-right">
              <img src="~/assets/images/external/multimc.webp" alt="MultiMC" />
            </div>
            <div class="launcher-badge bottom-left">
              <PrismIcon />
            </div>
            <div class="launcher-badge bottom-right">
              <ATLauncher />
            </div>
            <div class="launcher-badge bottom-middle">
              <CurseForge />
            </div>
            <div class="first-ring" />
            <div class="second-ring" />
            <div class="third-ring" />
          </div>
        </div>
      </div>
      <div class="feature-row">
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M21 8.25C21 8.70312 20.9258 9.13281 20.7773 9.53906C20.6289 9.94531 20.4141 10.3125 20.1328 10.6406C19.8516 10.9688 19.5273 11.2461 19.1602 11.4727C18.793 11.6992 18.3828 11.8516 17.9297 11.9297C17.8438 12.375 17.6914 12.7852 17.4727 13.1602C17.2539 13.5352 16.9766 13.8594 16.6406 14.1328C16.3047 14.4062 15.9375 14.6172 15.5391 14.7656C15.1406 14.9141 14.7109 14.9922 14.25 15H9.75C9.5 15 9.26172 15.0391 9.03516 15.1172C8.80859 15.1953 8.60156 15.3047 8.41406 15.4453C8.22656 15.5859 8.0625 15.7539 7.92188 15.9492C7.78125 16.1445 7.67578 16.3633 7.60547 16.6055C8.02734 16.707 8.41406 16.8711 8.76562 17.0977C9.11719 17.3242 9.42188 17.6016 9.67969 17.9297C9.9375 18.2578 10.1367 18.6172 10.2773 19.0078C10.418 19.3984 10.4922 19.8125 10.5 20.25C10.5 20.7656 10.4023 21.25 10.207 21.7031C10.0117 22.1562 9.74219 22.5547 9.39844 22.8984C9.05469 23.2422 8.65625 23.5117 8.20312 23.707C7.75 23.9023 7.26562 24 6.75 24C6.23438 24 5.75 23.9023 5.29688 23.707C4.84375 23.5117 4.44531 23.2461 4.10156 22.9102C3.75781 22.5742 3.48828 22.1758 3.29297 21.7148C3.09766 21.2539 3 20.7656 3 20.25C3 19.8047 3.07422 19.3789 3.22266 18.9727C3.37109 18.5664 3.57812 18.2031 3.84375 17.8828C4.10938 17.5625 4.42578 17.2852 4.79297 17.0508C5.16016 16.8164 5.5625 16.6602 6 16.582V7.41797C5.5625 7.33203 5.16016 7.17578 4.79297 6.94922C4.42578 6.72266 4.10938 6.44922 3.84375 6.12891C3.57812 5.80859 3.37109 5.44141 3.22266 5.02734C3.07422 4.61328 3 4.1875 3 3.75C3 3.23438 3.09766 2.75 3.29297 2.29688C3.48828 1.84375 3.75391 1.44922 4.08984 1.11328C4.42578 0.777344 4.82422 0.507812 5.28516 0.304688C5.74609 0.101562 6.23438 0 6.75 0C7.26562 0 7.75 0.0976563 8.20312 0.292969C8.65625 0.488281 9.05078 0.757812 9.38672 1.10156C9.72266 1.44531 9.99219 1.84375 10.1953 2.29688C10.3984 2.75 10.5 3.23438 10.5 3.75C10.5 4.19531 10.4258 4.62109 10.2773 5.02734C10.1289 5.43359 9.92188 5.79688 9.65625 6.11719C9.39062 6.4375 9.07422 6.71484 8.70703 6.94922C8.33984 7.18359 7.9375 7.33984 7.5 7.41797V14.2734C7.82812 14.0234 8.18359 13.832 8.56641 13.6992C8.94922 13.5664 9.34375 13.5 9.75 13.5H14.25C14.5 13.5 14.7383 13.4609 14.9648 13.3828C15.1914 13.3047 15.3984 13.1953 15.5859 13.0547C15.7734 12.9141 15.9375 12.7461 16.0781 12.5508C16.2188 12.3555 16.3242 12.1367 16.3945 11.8945C15.9727 11.793 15.5859 11.6289 15.2344 11.4023C14.8828 11.1758 14.5781 10.9023 14.3203 10.582C14.0625 10.2617 13.8633 9.90234 13.7227 9.50391C13.582 9.10547 13.5078 8.6875 13.5 8.25C13.5 7.73438 13.5977 7.25 13.793 6.79688C13.9883 6.34375 14.2539 5.94922 14.5898 5.61328C14.9258 5.27734 15.3242 5.00781 15.7852 4.80469C16.2461 4.60156 16.7344 4.5 17.25 4.5C17.7656 4.5 18.25 4.59766 18.7031 4.79297C19.1562 4.98828 19.5508 5.25781 19.8867 5.60156C20.2227 5.94531 20.4922 6.34375 20.6953 6.79688C20.8984 7.25 21 7.73438 21 8.25ZM4.5 3.75C4.5 4.0625 4.55859 4.35547 4.67578 4.62891C4.79297 4.90234 4.95312 5.14062 5.15625 5.34375C5.35938 5.54688 5.59766 5.70703 5.87109 5.82422C6.14453 5.94141 6.4375 6 6.75 6C7.0625 6 7.35547 5.94141 7.62891 5.82422C7.90234 5.70703 8.14062 5.54688 8.34375 5.34375C8.54688 5.14062 8.70703 4.90234 8.82422 4.62891C8.94141 4.35547 9 4.0625 9 3.75C9 3.4375 8.94141 3.14453 8.82422 2.87109C8.70703 2.59766 8.54688 2.35938 8.34375 2.15625C8.14062 1.95312 7.90234 1.79297 7.62891 1.67578C7.35547 1.55859 7.0625 1.5 6.75 1.5C6.4375 1.5 6.14453 1.55859 5.87109 1.67578C5.59766 1.79297 5.35938 1.95312 5.15625 2.15625C4.95312 2.35938 4.79297 2.59766 4.67578 2.87109C4.55859 3.14453 4.5 3.4375 4.5 3.75ZM9 20.25C9 19.9375 8.94141 19.6445 8.82422 19.3711C8.70703 19.0977 8.54688 18.8594 8.34375 18.6562C8.14062 18.4531 7.90234 18.293 7.62891 18.1758C7.35547 18.0586 7.0625 18 6.75 18C6.4375 18 6.14453 18.0586 5.87109 18.1758C5.59766 18.293 5.35938 18.4531 5.15625 18.6562C4.95312 18.8594 4.79297 19.0977 4.67578 19.3711C4.55859 19.6445 4.5 19.9375 4.5 20.25C4.5 20.5625 4.55859 20.8555 4.67578 21.1289C4.79297 21.4023 4.95312 21.6406 5.15625 21.8438C5.35938 22.0469 5.59766 22.207 5.87109 22.3242C6.14453 22.4414 6.4375 22.5 6.75 22.5C7.0625 22.5 7.35547 22.4414 7.62891 22.3242C7.90234 22.207 8.14062 22.0469 8.34375 21.8438C8.54688 21.6406 8.70703 21.4023 8.82422 21.1289C8.94141 20.8555 9 20.5625 9 20.25ZM17.25 10.5C17.5625 10.5 17.8555 10.4414 18.1289 10.3242C18.4023 10.207 18.6406 10.0469 18.8438 9.84375C19.0469 9.64062 19.207 9.40234 19.3242 9.12891C19.4414 8.85547 19.5 8.5625 19.5 8.25C19.5 7.9375 19.4414 7.64453 19.3242 7.37109C19.207 7.09766 19.0469 6.85938 18.8438 6.65625C18.6406 6.45312 18.4023 6.29297 18.1289 6.17578C17.8555 6.05859 17.5625 6 17.25 6C16.9375 6 16.6445 6.05859 16.3711 6.17578C16.0977 6.29297 15.8594 6.45312 15.6562 6.65625C15.4531 6.85938 15.293 7.09766 15.1758 7.37109C15.0586 7.64453 15 7.9375 15 8.25C15 8.5625 15.0586 8.85547 15.1758 9.12891C15.293 9.40234 15.4531 9.64062 15.6562 9.84375C15.8594 10.0469 16.0977 10.207 16.3711 10.3242C16.6445 10.4414 16.9375 10.5 17.25 10.5Z"
                fill="url(#paint0_linear_897_3796)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_897_3796"
                  x1="12"
                  y1="0"
                  x2="12"
                  y2="24"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M21 8.25C21 8.70312 20.9258 9.13281 20.7773 9.53906C20.6289 9.94531 20.4141 10.3125 20.1328 10.6406C19.8516 10.9688 19.5273 11.2461 19.1602 11.4727C18.793 11.6992 18.3828 11.8516 17.9297 11.9297C17.8438 12.375 17.6914 12.7852 17.4727 13.1602C17.2539 13.5352 16.9766 13.8594 16.6406 14.1328C16.3047 14.4062 15.9375 14.6172 15.5391 14.7656C15.1406 14.9141 14.7109 14.9922 14.25 15H9.75C9.5 15 9.26172 15.0391 9.03516 15.1172C8.80859 15.1953 8.60156 15.3047 8.41406 15.4453C8.22656 15.5859 8.0625 15.7539 7.92188 15.9492C7.78125 16.1445 7.67578 16.3633 7.60547 16.6055C8.02734 16.707 8.41406 16.8711 8.76562 17.0977C9.11719 17.3242 9.42188 17.6016 9.67969 17.9297C9.9375 18.2578 10.1367 18.6172 10.2773 19.0078C10.418 19.3984 10.4922 19.8125 10.5 20.25C10.5 20.7656 10.4023 21.25 10.207 21.7031C10.0117 22.1562 9.74219 22.5547 9.39844 22.8984C9.05469 23.2422 8.65625 23.5117 8.20312 23.707C7.75 23.9023 7.26562 24 6.75 24C6.23438 24 5.75 23.9023 5.29688 23.707C4.84375 23.5117 4.44531 23.2461 4.10156 22.9102C3.75781 22.5742 3.48828 22.1758 3.29297 21.7148C3.09766 21.2539 3 20.7656 3 20.25C3 19.8047 3.07422 19.3789 3.22266 18.9727C3.37109 18.5664 3.57812 18.2031 3.84375 17.8828C4.10938 17.5625 4.42578 17.2852 4.79297 17.0508C5.16016 16.8164 5.5625 16.6602 6 16.582V7.41797C5.5625 7.33203 5.16016 7.17578 4.79297 6.94922C4.42578 6.72266 4.10938 6.44922 3.84375 6.12891C3.57812 5.80859 3.37109 5.44141 3.22266 5.02734C3.07422 4.61328 3 4.1875 3 3.75C3 3.23438 3.09766 2.75 3.29297 2.29688C3.48828 1.84375 3.75391 1.44922 4.08984 1.11328C4.42578 0.777344 4.82422 0.507812 5.28516 0.304688C5.74609 0.101562 6.23438 0 6.75 0C7.26562 0 7.75 0.0976563 8.20312 0.292969C8.65625 0.488281 9.05078 0.757812 9.38672 1.10156C9.72266 1.44531 9.99219 1.84375 10.1953 2.29688C10.3984 2.75 10.5 3.23438 10.5 3.75C10.5 4.19531 10.4258 4.62109 10.2773 5.02734C10.1289 5.43359 9.92188 5.79688 9.65625 6.11719C9.39062 6.4375 9.07422 6.71484 8.70703 6.94922C8.33984 7.18359 7.9375 7.33984 7.5 7.41797V14.2734C7.82812 14.0234 8.18359 13.832 8.56641 13.6992C8.94922 13.5664 9.34375 13.5 9.75 13.5H14.25C14.5 13.5 14.7383 13.4609 14.9648 13.3828C15.1914 13.3047 15.3984 13.1953 15.5859 13.0547C15.7734 12.9141 15.9375 12.7461 16.0781 12.5508C16.2188 12.3555 16.3242 12.1367 16.3945 11.8945C15.9727 11.793 15.5859 11.6289 15.2344 11.4023C14.8828 11.1758 14.5781 10.9023 14.3203 10.582C14.0625 10.2617 13.8633 9.90234 13.7227 9.50391C13.582 9.10547 13.5078 8.6875 13.5 8.25C13.5 7.73438 13.5977 7.25 13.793 6.79688C13.9883 6.34375 14.2539 5.94922 14.5898 5.61328C14.9258 5.27734 15.3242 5.00781 15.7852 4.80469C16.2461 4.60156 16.7344 4.5 17.25 4.5C17.7656 4.5 18.25 4.59766 18.7031 4.79297C19.1562 4.98828 19.5508 5.25781 19.8867 5.60156C20.2227 5.94531 20.4922 6.34375 20.6953 6.79688C20.8984 7.25 21 7.73438 21 8.25ZM4.5 3.75C4.5 4.0625 4.55859 4.35547 4.67578 4.62891C4.79297 4.90234 4.95312 5.14062 5.15625 5.34375C5.35938 5.54688 5.59766 5.70703 5.87109 5.82422C6.14453 5.94141 6.4375 6 6.75 6C7.0625 6 7.35547 5.94141 7.62891 5.82422C7.90234 5.70703 8.14062 5.54688 8.34375 5.34375C8.54688 5.14062 8.70703 4.90234 8.82422 4.62891C8.94141 4.35547 9 4.0625 9 3.75C9 3.4375 8.94141 3.14453 8.82422 2.87109C8.70703 2.59766 8.54688 2.35938 8.34375 2.15625C8.14062 1.95312 7.90234 1.79297 7.62891 1.67578C7.35547 1.55859 7.0625 1.5 6.75 1.5C6.4375 1.5 6.14453 1.55859 5.87109 1.67578C5.59766 1.79297 5.35938 1.95312 5.15625 2.15625C4.95312 2.35938 4.79297 2.59766 4.67578 2.87109C4.55859 3.14453 4.5 3.4375 4.5 3.75ZM9 20.25C9 19.9375 8.94141 19.6445 8.82422 19.3711C8.70703 19.0977 8.54688 18.8594 8.34375 18.6562C8.14062 18.4531 7.90234 18.293 7.62891 18.1758C7.35547 18.0586 7.0625 18 6.75 18C6.4375 18 6.14453 18.0586 5.87109 18.1758C5.59766 18.293 5.35938 18.4531 5.15625 18.6562C4.95312 18.8594 4.79297 19.0977 4.67578 19.3711C4.55859 19.6445 4.5 19.9375 4.5 20.25C4.5 20.5625 4.55859 20.8555 4.67578 21.1289C4.79297 21.4023 4.95312 21.6406 5.15625 21.8438C5.35938 22.0469 5.59766 22.207 5.87109 22.3242C6.14453 22.4414 6.4375 22.5 6.75 22.5C7.0625 22.5 7.35547 22.4414 7.62891 22.3242C7.90234 22.207 8.14062 22.0469 8.34375 21.8438C8.54688 21.6406 8.70703 21.4023 8.82422 21.1289C8.94141 20.8555 9 20.5625 9 20.25ZM17.25 10.5C17.5625 10.5 17.8555 10.4414 18.1289 10.3242C18.4023 10.207 18.6406 10.0469 18.8438 9.84375C19.0469 9.64062 19.207 9.40234 19.3242 9.12891C19.4414 8.85547 19.5 8.5625 19.5 8.25C19.5 7.9375 19.4414 7.64453 19.3242 7.37109C19.207 7.09766 19.0469 6.85938 18.8438 6.65625C18.6406 6.45312 18.4023 6.29297 18.1289 6.17578C17.8555 6.05859 17.5625 6 17.25 6C16.9375 6 16.6445 6.05859 16.3711 6.17578C16.0977 6.29297 15.8594 6.45312 15.6562 6.65625C15.4531 6.85938 15.293 7.09766 15.1758 7.37109C15.0586 7.64453 15 7.9375 15 8.25C15 8.5625 15.0586 8.85547 15.1758 9.12891C15.293 9.40234 15.4531 9.64062 15.6562 9.84375C15.8594 10.0469 16.0977 10.207 16.3711 10.3242C16.6445 10.4414 16.9375 10.5 17.25 10.5Z"
                fill="url(#paint0_linear_944_4967)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4967"
                  x1="12"
                  y1="0"
                  x2="12"
                  y2="24"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Open source</h3>
          </div>
          <div class="description">
            Modrinth’s launcher is fully open source. You can view the source code on our
            <a href="https://github.com/modrinth/theseus" rel="noopener" :target="$external()"
              >GitHub</a
            >!
          </div>
        </div>
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <g clip-path="url(#clip0_897_3802)">
                <path
                  d="M4.39313 8.58984C2.31984 9.28359 0.75 10.9266 0.75 13.5C0.75 16.5938 3.28125 18.75 6.375 18.75H15.0173M21.9291 17.7066C22.7456 17.0297 23.25 16.013 23.25 14.625C23.25 11.8209 20.7656 10.605 18.75 10.5C18.3333 6.30281 15.4219 3.75 12 3.75C10.7738 3.75 9.71297 4.07484 8.83125 4.60031"
                  stroke="url(#paint0_linear_897_3802)"
                  stroke-width="2.34783"
                  stroke-linecap="round"
                  stroke-linejoin="round"
                />
                <path
                  d="M21 21L3 3"
                  stroke="url(#paint1_linear_897_3802)"
                  stroke-width="2.34783"
                  stroke-miterlimit="10"
                  stroke-linecap="round"
                />
              </g>
              <defs>
                <linearGradient
                  id="paint0_linear_897_3802"
                  x1="12"
                  y1="3.75"
                  x2="12"
                  y2="18.75"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
                <linearGradient
                  id="paint1_linear_897_3802"
                  x1="12"
                  y1="3"
                  x2="12"
                  y2="21"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
                <clipPath id="clip0_897_3802">
                  <rect width="24" height="24" fill="white" />
                </clipPath>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="26"
              height="24"
              viewBox="0 0 26 24"
              fill="none"
            >
              <path
                d="M5.39313 8.58984C3.31984 9.28359 1.75 10.9266 1.75 13.5C1.75 16.5938 4.28125 18.75 7.375 18.75H16.0173M22.9291 17.7066C23.7456 17.0297 24.25 16.013 24.25 14.625C24.25 11.8209 21.7656 10.605 19.75 10.5C19.3333 6.30281 16.4219 3.75 13 3.75C11.7738 3.75 10.713 4.07484 9.83125 4.60031"
                stroke="url(#paint0_linear_944_4973)"
                stroke-width="2.34783"
                stroke-linecap="round"
                stroke-linejoin="round"
              />
              <path
                d="M22 21L4 3"
                stroke="url(#paint1_linear_944_4973)"
                stroke-width="2.34783"
                stroke-miterlimit="10"
                stroke-linecap="round"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4973"
                  x1="13"
                  y1="3.75"
                  x2="13"
                  y2="18.75"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
                <linearGradient
                  id="paint1_linear_944_4973"
                  x1="13"
                  y1="3"
                  x2="13"
                  y2="21"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Offline mode</h3>
          </div>
          <div class="description">
            Play your mods, whether you are connected to the internet, or not.
          </div>
        </div>
        <div class="point">
          <div class="title">
            <svg
              class="dark-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M22.8533 11.7089C22.8205 11.6379 22.0264 9.94977 20.2611 8.25801C17.9089 6.00383 14.938 4.8125 11.668 4.8125C8.39796 4.8125 5.42702 6.00383 3.07483 8.25801C1.30952 9.94977 0.51171 11.6406 0.482647 11.7089C0.440003 11.8008 0.417969 11.9003 0.417969 12.0009C0.417969 12.1015 0.440003 12.201 0.482647 12.2929C0.51546 12.3639 1.30952 14.0511 3.07483 15.7429C5.42702 17.9962 8.39796 19.1875 11.668 19.1875C14.938 19.1875 17.9089 17.9962 20.2611 15.7429C22.0264 14.0511 22.8205 12.3639 22.8533 12.2929C22.8959 12.201 22.918 12.1015 22.918 12.0009C22.918 11.9003 22.8959 11.8008 22.8533 11.7089ZM11.668 17.75C8.78234 17.75 6.2614 16.7446 4.17452 14.7627C3.31825 13.9466 2.58976 13.0161 2.01171 12C2.58961 10.9838 3.31811 10.0532 4.17452 9.2373C6.2614 7.25535 8.78234 6.25 11.668 6.25C14.5536 6.25 17.0745 7.25535 19.1614 9.2373C20.0193 10.053 20.7494 10.9836 21.3289 12C20.653 13.2093 17.7083 17.75 11.668 17.75ZM11.668 7.6875C10.7779 7.6875 9.90792 7.94042 9.16789 8.41429C8.42787 8.88815 7.8511 9.56167 7.5105 10.3497C7.16991 11.1377 7.08079 12.0048 7.25443 12.8413C7.42806 13.6779 7.85664 14.4463 8.48598 15.0494C9.11532 15.6525 9.91714 16.0632 10.7901 16.2296C11.663 16.396 12.5678 16.3106 13.39 15.9842C14.2123 15.6578 14.9151 15.1051 15.4096 14.3959C15.904 13.6867 16.168 12.8529 16.168 12C16.1667 10.8566 15.6922 9.76041 14.8486 8.95192C14.0049 8.14342 12.8611 7.68869 11.668 7.6875ZM11.668 14.875C11.0746 14.875 10.4946 14.7064 10.0012 14.3905C9.5079 14.0746 9.12338 13.6256 8.89632 13.1002C8.66926 12.5749 8.60985 11.9968 8.7256 11.4391C8.84136 10.8814 9.12708 10.3691 9.54664 9.96707C9.9662 9.56499 10.5007 9.29117 11.0827 9.18024C11.6646 9.06931 12.2678 9.12624 12.816 9.34385C13.3642 9.56145 13.8327 9.92994 14.1624 10.4027C14.492 10.8755 14.668 11.4314 14.668 12C14.668 12.7625 14.3519 13.4938 13.7893 14.0329C13.2267 14.5721 12.4636 14.875 11.668 14.875Z"
                fill="url(#paint0_linear_897_3809)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_897_3809"
                  x1="11.668"
                  y1="4.8125"
                  x2="11.668"
                  y2="19.1875"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#C1E1B1" />
                  <stop offset="1" stop-color="#A7BDE6" />
                </linearGradient>
              </defs>
            </svg>
            <svg
              class="light-icon"
              xmlns="http://www.w3.org/2000/svg"
              width="24"
              height="24"
              viewBox="0 0 24 24"
              fill="none"
            >
              <path
                d="M22.8533 11.7089C22.8205 11.6379 22.0264 9.94977 20.2611 8.25801C17.9089 6.00383 14.938 4.8125 11.668 4.8125C8.39796 4.8125 5.42702 6.00383 3.07483 8.25801C1.30952 9.94977 0.51171 11.6406 0.482647 11.7089C0.440003 11.8008 0.417969 11.9003 0.417969 12.0009C0.417969 12.1015 0.440003 12.201 0.482647 12.2929C0.51546 12.3639 1.30952 14.0511 3.07483 15.7429C5.42702 17.9962 8.39796 19.1875 11.668 19.1875C14.938 19.1875 17.9089 17.9962 20.2611 15.7429C22.0264 14.0511 22.8205 12.3639 22.8533 12.2929C22.8959 12.201 22.918 12.1015 22.918 12.0009C22.918 11.9003 22.8959 11.8008 22.8533 11.7089ZM11.668 17.75C8.78234 17.75 6.2614 16.7446 4.17452 14.7627C3.31825 13.9466 2.58976 13.0161 2.01171 12C2.58961 10.9838 3.31811 10.0532 4.17452 9.2373C6.2614 7.25535 8.78234 6.25 11.668 6.25C14.5536 6.25 17.0745 7.25535 19.1614 9.2373C20.0193 10.053 20.7494 10.9836 21.3289 12C20.653 13.2093 17.7083 17.75 11.668 17.75ZM11.668 7.6875C10.7779 7.6875 9.90792 7.94042 9.16789 8.41429C8.42787 8.88815 7.8511 9.56167 7.5105 10.3497C7.16991 11.1377 7.08079 12.0048 7.25443 12.8413C7.42806 13.6779 7.85664 14.4463 8.48598 15.0494C9.11532 15.6525 9.91714 16.0632 10.7901 16.2296C11.663 16.396 12.5678 16.3106 13.39 15.9842C14.2123 15.6578 14.9151 15.1051 15.4096 14.3959C15.904 13.6867 16.168 12.8529 16.168 12C16.1667 10.8566 15.6922 9.76041 14.8486 8.95192C14.0049 8.14342 12.8611 7.68869 11.668 7.6875ZM11.668 14.875C11.0746 14.875 10.4946 14.7064 10.0012 14.3905C9.5079 14.0746 9.12338 13.6256 8.89632 13.1002C8.66926 12.5749 8.60985 11.9968 8.7256 11.4391C8.84136 10.8814 9.12708 10.3691 9.54664 9.96707C9.9662 9.56499 10.5007 9.29117 11.0827 9.18024C11.6646 9.06931 12.2678 9.12624 12.816 9.34385C13.3642 9.56145 13.8327 9.92994 14.1624 10.4027C14.492 10.8755 14.668 11.4314 14.668 12C14.668 12.7625 14.3519 13.4938 13.7893 14.0329C13.2267 14.5721 12.4636 14.875 11.668 14.875Z"
                fill="url(#paint0_linear_944_4980)"
              />
              <defs>
                <linearGradient
                  id="paint0_linear_944_4980"
                  x1="11.668"
                  y1="4.8125"
                  x2="11.668"
                  y2="19.1875"
                  gradientUnits="userSpaceOnUse"
                >
                  <stop stop-color="#A7D0FF" />
                  <stop offset="0.414928" stop-color="#00BD3C" />
                </linearGradient>
              </defs>
            </svg>
            <h3>Follow projects</h3>
          </div>
          <div class="description">Save content you love and receive updates with one click.</div>
        </div>
      </div>
    </div>
    <div ref="downloadSection" class="footer">
      <div class="section-badge">Download options</div>
      <div class="section-subheader">
        <div class="section-subheader-title">Download the Modrinth App</div>
        <div class="section-subheader-description">
          Our desktop app is available across all platforms, <br />
          choose your desired version.
        </div>
      </div>
      <div class="download-section">
        <div class="download-card">
          <div class="title">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 4875 4875" fill="currentColor">
              <path
                d="M0 0h2311v2310H0zm2564 0h2311v2310H2564zM0 2564h2311v2311H0zm2564 0h2311v2311H2564"
              />
            </svg>
            Windows
          </div>
          <div class="description">
            <a ref="downloadWindows" :href="windowsLink" download="">
              <DownloadIcon />
              <span> Download the beta </span>
            </a>
          </div>
        </div>
        <div class="divider" />
        <div class="download-card">
          <div class="title">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="44"
              height="44"
              viewBox="0 0 44 44"
              fill="none"
            >
              <path
                d="M33.5752 37.7889C31.3308 39.8664 28.8802 39.5384 26.5212 38.5543C24.0248 37.5484 21.7345 37.5046 19.1007 38.5543C15.8027 39.9102 14.0621 39.5165 12.0924 37.7889C0.915868 26.789 2.56487 10.0377 15.253 9.42536C18.3449 9.57844 20.4978 11.0436 22.3071 11.1748C25.0096 10.65 27.5976 9.14107 30.4834 9.33789C33.9417 9.60031 36.5526 10.9124 38.2703 13.2742C31.1247 17.3637 32.8195 26.3516 39.3697 28.8665C38.0642 32.1468 36.3694 35.4052 33.5523 37.8108L33.5752 37.7889ZM22.078 9.29415C21.7345 4.41745 25.8799 0.393635 30.6437 0C31.3079 5.6421 25.2844 9.84086 22.078 9.29415Z"
                fill="currentColor"
              />
            </svg>
            Mac
          </div>
          <div class="description apple">
            <a :href="macLinks.universal" download="">
              <DownloadIcon />
              <span> Download the beta </span>
            </a>
          </div>
        </div>
        <div class="divider" />
        <div class="download-card">
          <div class="title">
            <svg
              class="dark-icon"
              width="45"
              height="46"
              viewBox="0 0 45 46"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                id="Subtract"
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M23.5477 0H23.5461C23.2284 0 22.9077 0.0163711 22.5876 0.0415902C20.4638 0.208942 18.8991 0.910133 17.7996 2.0082C16.7124 3.09404 16.1751 4.47323 15.9172 5.82868C15.6602 7.18017 15.6685 8.57463 15.7244 9.76049C15.7464 10.2267 15.7737 10.6424 15.7981 11.0129L15.7981 11.0129C15.8393 11.6392 15.8719 12.1361 15.856 12.5288C15.7269 14.3647 15.3652 15.7513 14.1358 17.5116C12.4894 19.4715 10.1353 22.6851 9.00147 26.0938C8.50987 27.5652 8.23247 29.1259 8.37804 30.6595C8.14194 30.9205 7.94151 31.1909 7.77396 31.4209L7.69935 31.5235C7.53577 31.7486 7.41036 31.9212 7.28343 32.0672C7.22885 32.1165 7.13829 32.1747 6.95723 32.2529C6.85646 32.2965 6.74665 32.3386 6.60714 32.3918L6.60229 32.3937L6.60063 32.3943C6.4689 32.4446 6.31402 32.5038 6.15349 32.5722L6.14713 32.575C6.11461 32.5891 6.08029 32.6038 6.04439 32.6191L6.04194 32.6201C5.49857 32.8521 4.5956 33.2374 4.06765 34.2907L4.06313 34.2998L4.05878 34.3089C3.83111 34.787 3.71086 35.3126 3.71965 35.8518C3.72044 36.2784 3.77526 36.6973 3.83488 36.9965L3.8573 37.1494C3.90153 37.4502 3.93607 37.685 3.9521 37.8975C3.96952 38.1284 3.9557 38.2416 3.94333 38.2911C3.47501 39.5811 3.29341 40.7747 3.80981 41.7808L3.81261 41.7862C4.34311 42.8045 5.41437 43.1509 6.09597 43.3713L6.14873 43.3884L6.1831 43.3968C6.76559 43.5407 7.45296 43.6219 8.08651 43.6967L8.08657 43.6967C8.33286 43.7258 8.57102 43.7539 8.79172 43.7844C9.65223 43.9032 10.4057 44.0681 11.0346 44.4066L11.0467 44.4131L11.0589 44.4193C12.8487 45.3199 14.8309 45.8073 16.5413 45.3594C17.5947 45.1214 18.5098 44.4944 19.1114 43.6161C19.5314 43.5681 19.9501 43.481 20.3513 43.3976L20.4887 43.3691C21.1456 43.2333 21.8552 43.0939 22.7255 43.039L22.7353 43.0384L22.7452 43.0376C23.2171 42.9984 23.7676 43.0823 24.5478 43.2012L24.6654 43.2192C25.2572 43.3092 25.9398 43.4072 26.6797 43.4223L26.7366 43.5729L26.7865 43.6228C27.7132 45.3272 29.3865 46.1223 31.1344 45.9841C32.8861 45.8473 34.6161 44.7911 35.9501 43.2465L35.9575 43.2379L35.9647 43.2291C36.4178 42.6799 37.0511 42.2572 37.7983 41.8603C38.1016 41.6992 38.4052 41.5515 38.7175 41.3996L38.7179 41.3994L38.936 41.2932C39.3111 41.1099 39.7022 40.9143 40.0596 40.6996C40.783 40.2838 41.6765 39.5491 41.7413 38.3282C41.8014 37.2663 41.2531 36.3256 40.4338 35.3719V35.1588L40.1744 34.8993C40.1374 34.8477 40.0795 34.7411 40.0113 34.5353C39.934 34.302 39.8672 34.0188 39.7856 33.6562C39.6368 32.9545 39.4127 31.9102 38.5493 31.186L38.3995 31.0604C38.9576 28.7316 38.5908 26.4633 37.8259 24.46C36.7898 21.7201 34.9984 19.3692 33.6785 17.7973L33.6778 17.7965C32.1849 15.9116 31.019 14.4397 31.0434 12.2669C31.0453 12.1131 31.0478 11.9522 31.0504 11.7849C31.0801 9.87812 31.1226 7.15021 30.3612 4.83914C29.9387 3.55665 29.2488 2.33019 28.115 1.42642C26.9706 0.514298 25.4646 0.00304389 23.5477 0ZM22.666 1.0385C22.9685 1.01467 23.2618 1 23.546 1C30.2102 1.01058 30.1098 7.76032 30.0499 11.7866C30.0475 11.9472 30.0452 12.1035 30.0433 12.2548C30.0146 14.7849 31.382 16.51 32.8074 18.3084C32.8393 18.3487 32.8713 18.3891 32.9033 18.4295C34.205 19.9787 35.9137 22.23 36.8908 24.815C37.692 26.9123 37.9982 29.2443 37.208 31.5873C37.3329 31.5924 37.4538 31.6331 37.5563 31.7047C37.6223 31.7737 37.6877 31.8118 37.7513 31.8488C37.8026 31.8787 37.8527 31.908 37.901 31.9522H37.9065C38.4748 32.4288 38.6527 33.1347 38.8085 33.8698C38.9698 34.5867 39.1165 35.2008 39.4282 35.5675L39.4337 35.573V35.7508C40.376 36.7867 40.7848 37.5402 40.7427 38.2735C40.706 38.9775 40.1908 39.4725 39.5528 39.8373C39.1757 40.0647 38.7412 40.276 38.2863 40.4973C37.2045 41.0234 36.0076 41.6055 35.1932 42.5928C33.974 44.0045 32.4688 44.8772 31.0553 44.9872C29.6418 45.099 28.3182 44.45 27.6013 43.0237L27.5958 43.0182C27.5697 42.949 27.5454 42.89 27.523 42.8356C27.4653 42.6951 27.4199 42.5847 27.3868 42.4077C26.4008 42.4735 25.4823 42.3328 24.6448 42.2044C23.9204 42.0934 23.2567 41.9917 22.6623 42.041C21.6653 42.1039 20.8661 42.2696 20.1753 42.4127C19.5578 42.5407 19.027 42.6507 18.519 42.6533C18.0827 43.537 17.2687 44.175 16.3043 44.3877C14.9293 44.7543 13.206 44.3803 11.5083 43.526C10.4773 42.9711 9.26303 42.8279 8.13135 42.6944C7.52434 42.6228 6.9411 42.554 6.42267 42.426C5.68017 42.1858 5.01833 41.9365 4.69933 41.3242C4.38033 40.7027 4.43716 39.8502 4.89366 38.6035C5.02207 38.1946 4.94184 37.6503 4.84524 36.995C4.83702 36.9392 4.82868 36.8826 4.82033 36.8252C4.769 36.5758 4.7195 36.2073 4.7195 35.8425C4.71216 35.4612 4.7965 35.0853 4.9615 34.7388C5.32126 34.021 5.91228 33.7658 6.46354 33.5276C6.49097 33.5158 6.5183 33.504 6.5455 33.4922C6.68913 33.4309 6.82976 33.3772 6.96561 33.3253C7.37077 33.1706 7.73344 33.0321 8.00667 32.7588C8.17742 32.5672 8.34012 32.343 8.50901 32.1103C8.72634 31.8108 8.95394 31.4971 9.22217 31.2207C9.27154 31.1251 9.34021 31.0408 9.42383 30.9732C9.19833 29.4973 9.44033 27.9353 9.95 26.41C11.0298 23.1632 13.3068 20.0483 14.9293 18.1215C16.3043 16.1653 16.715 14.5868 16.8543 12.5848C16.8738 12.1381 16.8352 11.5454 16.7908 10.8636C16.5636 7.37332 16.1844 1.54924 22.666 1.0385ZM24.5397 7.2425H24.5158C24.1226 7.24289 23.7402 7.3698 23.4268 7.60733C23.065 7.83891 22.7883 8.18188 22.6385 8.5845C22.4431 9.02748 22.3491 9.50854 22.3635 9.9925V10.0292C22.367 10.1519 22.378 10.2744 22.3965 10.3958C22.582 10.4694 22.8174 10.5436 23.0524 10.6176C23.2099 10.6673 23.3673 10.7169 23.5093 10.7662C23.4873 10.6048 23.4763 10.4398 23.4727 10.2767V10.2418C23.4845 9.98963 23.5403 9.74144 23.6377 9.5085C23.7102 9.27639 23.8394 9.06596 24.0135 8.89617C24.1712 8.70917 24.3233 8.65417 24.4938 8.65417H24.5232C24.7047 8.65417 24.8557 8.77281 25.0081 8.89261L25.0127 8.89617C25.0488 8.95476 25.0822 9.00626 25.1132 9.0541C25.2112 9.20535 25.2855 9.32003 25.3482 9.50667C25.4263 9.7431 25.4641 9.99101 25.46 10.24V10.2767C25.449 10.5297 25.3958 10.779 25.2987 11.0137C25.2573 11.0863 25.1757 11.1644 25.0976 11.2393C25.0476 11.2872 24.9989 11.3337 24.9632 11.3767C25.1009 11.4051 25.2359 11.4456 25.3665 11.4977C25.4774 11.5867 25.5956 11.6303 25.7223 11.6771C25.7757 11.6968 25.8306 11.717 25.8872 11.7415C25.9426 11.7634 25.9966 11.7892 26.0485 11.8185C26.2203 11.6435 26.3534 11.4342 26.439 11.2043C26.6168 10.7958 26.7103 10.3555 26.714 9.91L26.7213 9.954C26.7222 9.96709 26.7246 9.98002 26.7287 9.9925V9.8C26.7287 9.81833 26.7259 9.83667 26.7232 9.855C26.7204 9.87333 26.7177 9.89167 26.7177 9.91C26.703 9.42417 26.6058 9.0575 26.4133 8.58267C26.2153 8.21417 25.9587 7.853 25.6103 7.6055C25.2657 7.35617 24.9302 7.2425 24.5397 7.2425ZM19.1515 7.35067H19.0855C18.7958 7.35067 18.5538 7.4735 18.3192 7.71733C18.0782 8.00351 17.9013 8.33795 17.8003 8.69817C17.6731 9.09315 17.6233 9.50898 17.6537 9.92283C17.668 10.345 17.7645 10.7604 17.9378 11.1457C18.0826 11.4065 18.2514 11.6532 18.442 11.8827C18.5311 11.8615 18.6136 11.8187 18.6822 11.758C18.748 11.6843 18.8058 11.6467 18.8698 11.6049C18.9109 11.578 18.9547 11.5495 19.0048 11.5087C18.9792 11.4977 18.9608 11.4812 18.9425 11.4647C18.8178 11.34 18.7225 11.219 18.6382 10.9733C18.6295 10.9464 18.6213 10.9211 18.6135 10.8969C18.5496 10.7 18.5111 10.5814 18.4915 10.361V10.3335C18.4698 10.1309 18.4841 9.92611 18.5337 9.7285C18.5607 9.54657 18.6367 9.37544 18.7537 9.2335C18.8563 9.06483 18.9645 9.0025 19.0947 8.9915H19.1332C19.1941 8.98752 19.2552 8.99587 19.3129 9.01602C19.3705 9.03618 19.4235 9.06775 19.4687 9.10883C19.6109 9.24277 19.7158 9.41148 19.773 9.59833C19.7858 9.63881 19.7979 9.67596 19.8092 9.71087C19.8662 9.88628 19.905 10.0055 19.9233 10.2088V10.2363C19.9417 10.3958 19.9398 10.5608 19.9178 10.7258C19.9701 10.6973 20.0226 10.6672 20.0762 10.6365C20.2463 10.5391 20.4264 10.4359 20.6383 10.3592C20.6632 10.3536 20.6875 10.3458 20.7126 10.3378C20.7375 10.3298 20.7631 10.3216 20.7905 10.3152V10.1685C20.8052 10.0475 20.807 9.9265 20.7942 9.68083V9.6735C20.7703 9.184 20.6787 8.8155 20.5137 8.45067C20.367 8.12617 20.1507 7.83467 19.883 7.59817C19.6465 7.43867 19.4118 7.35067 19.1515 7.35067ZM23.8265 11.2098C22.941 10.5975 22.2058 10.4783 21.5972 10.4783H21.599C21.2837 10.4783 20.9995 10.5315 20.7447 10.5975C20.0058 10.8377 19.5163 11.329 19.2065 11.6993V11.7048H19.2028C19.1423 11.758 19.0708 11.8093 18.8802 11.945C18.8088 11.99 18.7236 12.0537 18.623 12.1287C18.4574 12.2523 18.2501 12.407 17.9947 12.561C17.6317 12.7993 17.5125 13.1697 17.6408 13.661C17.7692 14.024 18.1652 14.5117 18.8985 14.9627H18.9077V14.9718C19.1865 15.1208 19.4099 15.3135 19.6231 15.4973C19.7589 15.6145 19.8905 15.728 20.0297 15.8243C20.2199 15.9557 20.4169 16.0769 20.62 16.1873C20.8828 16.2829 21.1621 16.3246 21.4413 16.3102C22.2045 16.3552 22.7698 16.0785 23.269 15.8342L23.2967 15.8207C23.5576 15.695 23.7982 15.5366 24.0381 15.3788C24.2601 15.2327 24.4814 15.0871 24.7175 14.9682V14.9645C25.7203 14.5997 26.4408 13.9892 26.6663 13.4208C26.7229 13.2834 26.7496 13.1354 26.7445 12.9868C26.7394 12.8383 26.7028 12.6925 26.637 12.5592V12.7993C26.5032 12.5555 26.2282 12.3172 25.7607 12.0678H25.7552C25.6086 12.0127 25.4746 11.963 25.3509 11.9171C24.6553 11.6593 24.2825 11.5211 23.8265 11.2098ZM29.9113 22.6058C28.9232 20.7762 27.3868 17.0032 26.7305 14.4053H26.7287C26.3455 14.9517 25.6708 15.4412 24.8147 15.6887H24.8055C24.5824 15.7556 24.3538 15.8976 24.104 16.0528C23.9021 16.1782 23.6865 16.3122 23.4488 16.422C22.9593 16.6677 22.3763 16.9133 21.621 16.9133C21.5616 16.9133 21.5005 16.9086 21.435 16.9035L21.4193 16.9023C20.6878 16.9023 20.1983 16.6567 19.8133 16.2882C19.626 16.1704 19.4595 16.0231 19.2918 15.8748C19.115 15.7184 18.9368 15.5608 18.7317 15.4357C18.6433 15.3534 18.4753 15.2449 18.3474 15.1624C18.2843 15.1216 18.2309 15.0872 18.2018 15.0653C18.0598 17.7587 16.5369 21.0396 15.4344 23.4147C15.2131 23.8914 15.0087 24.3316 14.8358 24.7233C14.1222 26.3938 13.7202 28.1808 13.6497 29.996C11.7833 27.5082 13.1473 24.3072 13.8733 22.9303C14.6892 21.3922 14.8175 21.0255 14.6213 21.1447C13.8862 22.3418 12.7348 24.2485 12.2857 26.2102C12.051 27.2368 12.0107 28.247 12.3132 29.226C12.6138 30.1903 13.2665 31.052 14.4215 31.7853C15.9927 32.7497 17.067 33.7177 17.6995 34.5738C18.332 35.4282 18.53 36.2165 18.332 36.706C18.2262 36.9767 18.02 37.196 17.7563 37.3183C17.5235 37.3862 17.2375 37.4412 16.9038 37.4412C17.1014 37.6184 17.2714 37.8241 17.408 38.0517C17.694 38.345 17.9416 38.6733 18.145 39.0288C21.3863 41.2362 25.1997 40.3818 27.9497 38.4165C27.9605 38.3792 27.9714 38.3419 27.9822 38.3047C28.2443 37.4049 28.4993 36.5295 28.5345 35.9415V35.936C28.5987 34.6288 28.672 33.4867 28.9103 32.5095C29.1505 31.5598 29.5758 30.8247 30.3642 30.3333C30.4616 30.318 30.56 30.2827 30.6563 30.2481C30.6869 30.2371 30.7173 30.2262 30.7473 30.216C30.7558 30.0996 30.7642 30.0546 30.7769 29.9868C30.778 29.9809 30.7792 29.9748 30.7803 29.9685C31.0077 28.5 32.3478 28.3827 34.0253 29.116C35.6442 29.8475 36.2547 30.5038 35.9778 31.36H36.0072C36.0438 31.36 36.0805 31.3594 36.1172 31.3588C36.1905 31.3576 36.2638 31.3563 36.3372 31.36V31.4388C36.6727 30.3407 35.9742 29.6 34.0822 28.621C33.9694 28.6097 33.8595 28.5631 33.7512 28.5171C33.7362 28.5108 33.7213 28.5044 33.7063 28.4982C33.9612 27.4605 33.9062 26.4173 33.6678 25.4365C33.1527 23.264 31.7318 21.361 30.6502 20.382C30.4503 20.382 30.4723 20.6295 30.8757 20.9962C31.8712 21.911 34.0565 25.2037 32.8722 28.2672C32.5367 28.1828 32.2177 28.1407 31.9317 28.1498C31.4788 25.6473 30.4357 23.5848 29.9113 22.6058ZM32.3533 33.3528C31.3652 33.1108 30.8042 31.9283 30.7363 30.8283V30.832L30.6538 30.8723C30.0433 31.2152 29.6987 31.8238 29.4787 32.6763C29.2605 33.5545 29.189 34.693 29.1212 36.002C29.0756 36.7223 28.8292 37.5563 28.5746 38.4181C28.4709 38.7694 28.3658 39.1252 28.2723 39.4798C27.9497 40.699 27.7847 41.8448 28.1697 42.5782L28.1788 42.5837C28.7967 43.8633 29.8472 44.3272 31.0645 44.23C32.2855 44.1365 33.6642 43.3977 34.7972 42.0465C35.7047 40.9513 36.9722 40.3422 38.0484 39.8251C38.5178 39.5996 38.9508 39.3915 39.3017 39.1682C39.8755 38.8033 40.1872 38.5595 40.2147 38.0792L40.2202 38.0993V38.0663L40.2147 38.0792C40.2312 37.6502 39.9397 36.9553 39.0487 35.9892C38.6065 35.4932 38.4694 34.8431 38.324 34.1536C38.3156 34.1142 38.3073 34.0745 38.2988 34.0348C38.1412 33.4005 37.9633 32.7937 37.582 32.4288L37.5765 32.4233C37.0815 31.9503 36.6012 31.9192 36.0237 31.932L35.6368 31.9503C34.9823 32.7423 33.4313 33.5985 32.3533 33.3528ZM10.929 30.9182H10.9107C10.3918 30.9182 10.0435 31.1657 9.69517 31.5342C9.5004 31.7412 9.31978 31.9881 9.13328 32.2431C8.92831 32.5233 8.71624 32.8132 8.4705 33.0705V33.0742H8.46683C8.1027 33.429 7.67066 33.5917 7.25569 33.7479C7.0961 33.808 6.93904 33.8671 6.78933 33.9358C6.72367 33.9655 6.65958 33.9934 6.59713 34.0206C6.14698 34.2164 5.78193 34.3753 5.52433 34.913C5.29333 35.3347 5.34467 35.9012 5.4455 36.6363C5.45614 36.7047 5.46735 36.774 5.47868 36.844C5.57861 37.4618 5.68765 38.136 5.495 38.686L5.49133 38.6915V38.6988C5.05867 39.8502 5.06417 40.5872 5.26583 40.9538C5.473 41.3205 5.91483 41.5662 6.59133 41.6872C6.96665 41.7547 7.39979 41.7941 7.86764 41.8367C9.08866 41.9478 10.5461 42.0804 11.8292 42.7908C13.4333 43.6452 15.0375 43.889 16.22 43.6415C17.4025 43.3738 18.178 42.6332 18.2825 41.2692V41.2582C18.3577 40.4533 17.7417 39.4762 16.946 38.3798C16.7146 38.075 16.4714 37.7766 16.2319 37.4828C15.6462 36.764 15.0824 36.0723 14.7662 35.3787L14.7607 35.3732L13.0923 32.3225C12.5112 31.5543 11.9062 31.0447 11.2168 30.9438C11.1215 30.9273 11.0262 30.9182 10.929 30.9182ZM23.5083 14.5427C23.6897 14.4504 23.8657 14.3609 24.0355 14.2825L24.0337 14.2843C24.4698 14.0202 24.892 13.7339 25.2987 13.4263C25.6305 13.0633 25.8028 12.847 25.5682 12.8177C25.4293 12.8177 25.3757 12.8992 25.2985 13.0166C25.2433 13.1007 25.176 13.2031 25.0567 13.3072C24.8433 13.4469 24.6017 13.6265 24.3669 13.8009C24.1922 13.9307 24.0212 14.0577 23.8687 14.1633C23.2362 14.5282 22.1948 15.0177 21.3057 15.0177C20.4161 15.0177 19.7068 14.5318 19.1721 14.1656L19.1662 14.1615C19.0678 14.0713 18.976 13.9809 18.8905 13.8966C18.7447 13.753 18.6171 13.6272 18.5062 13.551C18.4612 13.5086 18.4274 13.4544 18.394 13.4007C18.332 13.3013 18.2711 13.2034 18.1413 13.1843C18.013 13.1843 17.9763 13.5492 18.277 13.7948C18.3752 13.8656 18.4936 13.9772 18.6302 14.106C18.7307 14.2008 18.8412 14.3049 18.9608 14.409C19.5292 14.7738 20.3138 15.2633 21.3075 15.2633C22.0911 15.2633 22.8372 14.8839 23.5083 14.5427ZM22.0518 11.34V11.3803C22.0735 11.4531 22.1637 11.4663 22.2486 11.4788C22.2856 11.4842 22.3217 11.4895 22.3507 11.4995C22.3795 11.5142 22.4061 11.5324 22.4323 11.5502C22.4875 11.5879 22.5407 11.6242 22.6092 11.6242C22.7008 11.6242 22.8457 11.593 22.8567 11.5032C22.875 11.3785 22.6972 11.2557 22.5817 11.2557C22.435 11.1988 22.2407 11.1695 22.105 11.2483C22.0757 11.263 22.0408 11.3033 22.0518 11.34ZM20.8356 11.482C20.9223 11.4694 21.0157 11.4557 21.0417 11.3803V11.3767H21.0362V11.34C21.0472 11.3033 21.016 11.2648 20.9812 11.2483C20.8437 11.1713 20.653 11.2007 20.5063 11.2575C20.3927 11.2575 20.213 11.3803 20.2313 11.5013C20.2423 11.5893 20.389 11.626 20.4807 11.626C20.5494 11.626 20.6048 11.588 20.6612 11.5495C20.686 11.5325 20.711 11.5154 20.7373 11.5013C20.7656 11.4923 20.8 11.4872 20.8356 11.482Z"
                fill="white"
              />
            </svg>
            <svg
              class="light-icon"
              role="img"
              viewBox="0 0 24 24"
              xmlns="http://www.w3.org/2000/svg"
            >
              <path
                d="M12.504 0c-.155 0-.315.008-.48.021-4.226.333-3.105 4.807-3.17 6.298-.076 1.092-.3 1.953-1.05 3.02-.885 1.051-2.127 2.75-2.716 4.521-.278.832-.41 1.684-.287 2.489a.424.424 0 00-.11.135c-.26.268-.45.6-.663.839-.199.199-.485.267-.797.4-.313.136-.658.269-.864.68-.09.189-.136.394-.132.602 0 .199.027.4.055.536.058.399.116.728.04.97-.249.68-.28 1.145-.106 1.484.174.334.535.47.94.601.81.2 1.91.135 2.774.6.926.466 1.866.67 2.616.47.526-.116.97-.464 1.208-.946.587-.003 1.23-.269 2.26-.334.699-.058 1.574.267 2.577.2.025.134.063.198.114.333l.003.003c.391.778 1.113 1.132 1.884 1.071.771-.06 1.592-.536 2.257-1.306.631-.765 1.683-1.084 2.378-1.503.348-.199.629-.469.649-.853.023-.4-.2-.811-.714-1.376v-.097l-.003-.003c-.17-.2-.25-.535-.338-.926-.085-.401-.182-.786-.492-1.046h-.003c-.059-.054-.123-.067-.188-.135a.357.357 0 00-.19-.064c.431-1.278.264-2.55-.173-3.694-.533-1.41-1.465-2.638-2.175-3.483-.796-1.005-1.576-1.957-1.56-3.368.026-2.152.236-6.133-3.544-6.139zm.529 3.405h.013c.213 0 .396.062.584.198.19.135.33.332.438.533.105.259.158.459.166.724 0-.02.006-.04.006-.06v.105a.086.086 0 01-.004-.021l-.004-.024a1.807 1.807 0 01-.15.706.953.953 0 01-.213.335.71.71 0 00-.088-.042c-.104-.045-.198-.064-.284-.133a1.312 1.312 0 00-.22-.066c.05-.06.146-.133.183-.198.053-.128.082-.264.088-.402v-.02a1.21 1.21 0 00-.061-.4c-.045-.134-.101-.2-.183-.333-.084-.066-.167-.132-.267-.132h-.016c-.093 0-.176.03-.262.132a.8.8 0 00-.205.334 1.18 1.18 0 00-.09.4v.019c.002.089.008.179.02.267-.193-.067-.438-.135-.607-.202a1.635 1.635 0 01-.018-.2v-.02a1.772 1.772 0 01.15-.768c.082-.22.232-.406.43-.533a.985.985 0 01.594-.2zm-2.962.059h.036c.142 0 .27.048.399.135.146.129.264.288.344.465.09.199.14.4.153.667v.004c.007.134.006.2-.002.266v.08c-.03.007-.056.018-.083.024-.152.055-.274.135-.393.2.012-.09.013-.18.003-.267v-.015c-.012-.133-.04-.2-.082-.333a.613.613 0 00-.166-.267.248.248 0 00-.183-.064h-.021c-.071.006-.13.04-.186.132a.552.552 0 00-.12.27.944.944 0 00-.023.33v.015c.012.135.037.2.08.334.046.134.098.2.166.268.01.009.02.018.034.024-.07.057-.117.07-.176.136a.304.304 0 01-.131.068 2.62 2.62 0 01-.275-.402 1.772 1.772 0 01-.155-.667 1.759 1.759 0 01.08-.668 1.43 1.43 0 01.283-.535c.128-.133.26-.2.418-.2zm1.37 1.706c.332 0 .733.065 1.216.399.293.2.523.269 1.052.468h.003c.255.136.405.266.478.399v-.131a.571.571 0 01.016.47c-.123.31-.516.643-1.063.842v.002c-.268.135-.501.333-.775.465-.276.135-.588.292-1.012.267a1.139 1.139 0 01-.448-.067 3.566 3.566 0 01-.322-.198c-.195-.135-.363-.332-.612-.465v-.005h-.005c-.4-.246-.616-.512-.686-.71-.07-.268-.005-.47.193-.6.224-.135.38-.271.483-.336.104-.074.143-.102.176-.131h.002v-.003c.169-.202.436-.47.839-.601.139-.036.294-.065.466-.065zm2.8 2.142c.358 1.417 1.196 3.475 1.735 4.473.286.534.855 1.659 1.102 3.024.156-.005.33.018.513.064.646-1.671-.546-3.467-1.089-3.966-.22-.2-.232-.335-.123-.335.59.534 1.365 1.572 1.646 2.757.13.535.16 1.104.021 1.67.067.028.135.06.205.067 1.032.534 1.413.938 1.23 1.537v-.043c-.06-.003-.12 0-.18 0h-.016c.151-.467-.182-.825-1.065-1.224-.915-.4-1.646-.336-1.77.465-.008.043-.013.066-.018.135-.068.023-.139.053-.209.064-.43.268-.662.669-.793 1.187-.13.533-.17 1.156-.205 1.869v.003c-.02.334-.17.838-.319 1.35-1.5 1.072-3.58 1.538-5.348.334a2.645 2.645 0 00-.402-.533 1.45 1.45 0 00-.275-.333c.182 0 .338-.03.465-.067a.615.615 0 00.314-.334c.108-.267 0-.697-.345-1.163-.345-.467-.931-.995-1.788-1.521-.63-.4-.986-.87-1.15-1.396-.165-.534-.143-1.085-.015-1.645.245-1.07.873-2.11 1.274-2.763.107-.065.037.135-.408.974-.396.751-1.14 2.497-.122 3.854a8.123 8.123 0 01.647-2.876c.564-1.278 1.743-3.504 1.836-5.268.048.036.217.135.289.202.218.133.38.333.59.465.21.201.477.335.876.335.039.003.075.006.11.006.412 0 .73-.134.997-.268.29-.134.52-.334.74-.4h.005c.467-.135.835-.402 1.044-.7zm2.185 8.958c.037.6.343 1.245.882 1.377.588.134 1.434-.333 1.791-.765l.211-.01c.315-.007.577.01.847.268l.003.003c.208.199.305.53.391.876.085.4.154.78.409 1.066.486.527.645.906.636 1.14l.003-.007v.018l-.003-.012c-.015.262-.185.396-.498.595-.63.401-1.746.712-2.457 1.57-.618.737-1.37 1.14-2.036 1.191-.664.053-1.237-.2-1.574-.898l-.005-.003c-.21-.4-.12-1.025.056-1.69.176-.668.428-1.344.463-1.897.037-.714.076-1.335.195-1.814.12-.465.308-.797.641-.984l.045-.022zm-10.814.049h.01c.053 0 .105.005.157.014.376.055.706.333 1.023.752l.91 1.664.003.003c.243.533.754 1.064 1.189 1.637.434.598.77 1.131.729 1.57v.006c-.057.744-.48 1.148-1.125 1.294-.645.135-1.52.002-2.395-.464-.968-.536-2.118-.469-2.857-.602-.369-.066-.61-.2-.723-.4-.11-.2-.113-.602.123-1.23v-.004l.002-.003c.117-.334.03-.752-.027-1.118-.055-.401-.083-.71.043-.94.16-.334.396-.4.69-.533.294-.135.64-.202.915-.47h.002v-.002c.256-.268.445-.601.668-.838.19-.201.38-.336.663-.336zm7.159-9.074c-.435.201-.945.535-1.488.535-.542 0-.97-.267-1.28-.466-.154-.134-.28-.268-.373-.335-.164-.134-.144-.333-.074-.333.109.016.129.134.199.2.096.066.215.2.36.333.292.2.68.467 1.167.467.485 0 1.053-.267 1.398-.466.195-.135.445-.334.648-.467.156-.136.149-.267.279-.267.128.016.034.134-.147.332a8.097 8.097 0 01-.69.468zm-1.082-1.583V5.64c-.006-.02.013-.042.029-.05.074-.043.18-.027.26.004.063 0 .16.067.15.135-.006.049-.085.066-.135.066-.055 0-.092-.043-.141-.068-.052-.018-.146-.008-.163-.065zm-.551 0c-.02.058-.113.049-.166.066-.047.025-.086.068-.14.068-.05 0-.13-.02-.136-.068-.01-.066.088-.133.15-.133.08-.031.184-.047.259-.005.019.009.036.03.03.05v.02h.003z"
              />
            </svg>
            Linux
          </div>
          <div class="description apple">
            <a ref="downloadLinux" :href="linuxLinks.appImage" download="">
              <DownloadIcon />
              <span> Download the AppImage </span>
            </a>
            <a :href="linuxLinks.deb" download="">
              <DownloadIcon />
              <span> Download the DEB </span>
            </a>
            <a :href="linuxLinks.rpm" download="">
              <DownloadIcon />
              <span> Download the RPM </span>
            </a>
            <a :href="linuxLinks.thirdParty" download="">
              <LinkIcon />
              <span> Third-party packages </span>
            </a>
          </div>
        </div>
      </div>
      <p class="terms">
        By downloading the Modrinth App you agree to our
        <nuxt-link to="/legal/terms"> Terms</nuxt-link> and
        <nuxt-link to="/legal/privacy">Privacy Policy.</nuxt-link>
      </p>
    </div>
    <div class="bg-black">
      <LatestNewsRow />
    </div>
  </div>
</template>

<style scoped lang="scss">
.landing-hero {
  position: relative;
  background: #0f1121 url("https://cdn-raw.modrinth.com/app-landing/cube-black.png") no-repeat
    center 4rem;
  background-size: cover;
  padding: 6rem 1rem 12rem 1rem;
  margin-top: -5rem;
  display: flex;
  justify-content: center;
  align-items: center;
  text-align: center;
  flex-direction: column;
  isolation: isolate;

  .main-subheader {
    font-size: 1.625rem;
  }

  h2 {
    line-height: 125%;
    margin: 0 0 1.625rem;
    font-weight: 400;
    line-break: loose;
    color: var(--landing-color-subheading);
    max-width: 1096px;
    mask-image: none;
  }

  .button-group {
    width: fit-content;
    margin: 0 auto;
    justify-content: center;
    mask-image: none;

    .outline-button {
      color: var(--landing-color-heading);
      background: none;
      border: 1px var(--landing-color-heading) solid;
    }
  }

  img {
    width: 100%;
    max-width: 65rem;
    height: auto;
    mask-image: none;
    z-index: 1;
  }

  .bottom-transition {
    z-index: -1;
  }
}

.main-header {
  font-size: 5.25rem;
  font-weight: 600;
  line-height: 100%;
  margin: 2rem 0;
}

.subheader {
  font-size: 3.5rem;
  font-weight: 600;
  line-height: 100%;
  margin: 0 auto;
  padding: 0 4rem 4rem;
}

.features {
  position: relative;
  width: 100%;
  background: var(--landing-transition-gradient-end);
  align-content: center;
  justify-content: center;
  display: flex;
  flex-direction: column;

  h3 {
    font-weight: 500;
    font-size: var(--font-size-xl) !important;
  }

  p {
    font-size: var(--font-size-md) !important;
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    grid-template-rows: repeat(4, 1fr);
    max-width: min(100%, 1096px);
    width: min(100%, 1096px);
    gap: 16px;
    margin: 0 auto;
    padding: 1rem;

    .mods {
      grid-column: 1 / 2;
      grid-row: 1 / 3;

      .table {
        margin-bottom: 1rem;
        overflow: hidden;
        max-height: 32rem;
      }

      h3,
      p {
        text-align: center;
      }

      h4 {
        margin: 0;
        color: var(--color-contrast);
      }

      .search-bar {
        width: 100%;
        padding: var(--gap-sm);
        display: flex;
        flex-direction: row;
        justify-content: space-between;
        align-items: center;
        border-radius: var(--radius-md);
        border: 1px solid var(--landing-border-color);
        background: linear-gradient(0deg, #3b3f55 0%, #3b3f55 100%), rgba(59, 63, 85, 0.15);
        margin-bottom: 0.5rem;
        white-space: nowrap;
        font-size: var(--font-size-sm);

        .mini-input {
          display: flex;
          flex-direction: row;
          align-items: center;
          gap: 0.5rem;
          padding: var(--gap-sm) var(--gap-md);
          border-radius: var(--radius-sm);
          background-color: #1e202f;
          flex-grow: 1;
          max-width: 12rem;
        }

        h4 {
          font-weight: normal;
          margin-left: 0.5rem;
        }
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 2fr 1fr 3.75rem;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          flex-direction: column;
          justify-content: center;
          padding: var(--gap-sm) 0;
          font-size: var(--font-size-sm);

          .name {
            color: var(--color-contrast);
          }

          .description {
            font-size: var(--font-size-xs);
          }

          &.last {
            align-items: flex-end;
          }

          &.check {
            align-items: center;
            flex-direction: row;
          }
        }
      }

      .header {
        .cell {
          color: var(--color-base);
        }
      }
    }

    .playing {
      grid-column: 1 / 2;
      grid-row: 4 / 5;
      position: relative;
      min-height: 23rem;

      .minecraft {
        position: absolute;
        bottom: 6rem;
        right: var(--gap-xl);
        height: 13rem;
        mask-image: none;
      }

      .launcher {
        position: absolute;
        z-index: 12;
        top: var(--gap-xl);
        left: var(--gap-xl);
        height: 13rem;
        mask-image: none;
      }

      .text {
        position: absolute;
        left: 50%;
        bottom: 0;
        transform: translateX(-50%);
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        color: var(--color-contrast);
        text-align: center;
        padding: var(--gap-xl);
        mask-image: none;
        z-index: 10;
        width: 100%;
      }
    }

    .sharing {
      grid-column: 2 / 3;
      grid-row: 2 / 3;

      .table {
        height: 11rem;
        overflow: hidden;
        margin-bottom: 1rem;
      }

      h3,
      p {
        text-align: center;
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 1fr 2fr;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          align-items: center;
          padding: var(--gap-sm) 0;

          .description {
            display: -webkit-box;
            -webkit-line-clamp: 2;
            -webkit-box-orient: vertical;
            overflow: hidden;
          }
        }
      }

      .header {
        display: flex;
        flex-direction: row;
        gap: 0.25rem;
        padding: var(--gap-sm) var(--gap-sm);
      }

      .export-card {
        position: absolute;
        right: 1rem;
        top: 1.5rem;
        display: flex;
        flex-direction: row;
        align-items: center;
        padding: var(--gap-sm);
        gap: var(--gap-md);
        border-radius: var(--radius-md);
        background: linear-gradient(0deg, #3b3f55 0%, #3b3f55 100%), rgba(59, 63, 85, 0.15);
        border: 1px solid var(--landing-border-color);
        margin-bottom: 1rem;
        width: 20rem;
        font-size: var(--font-size-sm);

        img {
          width: 4.5rem;
          height: 4.5rem;
        }

        .info {
          width: 100%;
          display: flex;
          flex-direction: column;
          align-items: flex-start;
          gap: 0.25rem;

          .exporting {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
            gap: 0.5rem;
            width: 100%;

            .tag {
              display: flex;
              flex-direction: row;
              align-items: center;
              gap: 0.25rem;
              border-radius: var(--radius-sm);
            }

            .small-button {
              display: flex;
              flex-direction: row;
              align-items: center;
              gap: 0.25rem;
              padding: var(--gap-xs) var(--gap-md);
              border-radius: var(--radius-lg);
              border: 1px solid var(--landing-border-color);
              font-size: var(--font-size-xs);
            }
          }

          .name {
            font-size: var(--font-size-md);
            color: var(--color-contrast);
            margin: 0;
          }
        }
      }
    }

    .performance {
      grid-column: 1 / 2;
      grid-row: 3 / 4;

      .title {
        display: flex;
        justify-content: space-between;
        align-items: baseline;
        margin-bottom: 0.5rem;

        h3 {
          margin: 0;
          font-size: var(--font-size-lg);
        }
      }

      h3,
      h4,
      p {
        text-align: center;
        margin: 0;
      }

      h4,
      h3 {
        color: var(--color-contrast);
        margin-top: var(--gap-lg) !important;
      }

      h4 {
        font-size: var(--font-size-md);
        font-weight: normal;
      }

      .table {
        height: 10rem;
        overflow: hidden;
      }

      .row {
        display: grid;
        grid-template-columns: 3rem 2fr 1fr 1fr;
        padding: 0 var(--gap-sm);
        gap: 1rem;

        .cell {
          display: flex;
          align-items: center;
          padding: var(--gap-sm) 0;
          color: var(--color-gray);

          &.important {
            color: var(--color-contrast);
            font-weight: 500;
          }
        }
      }

      .header {
        font-size: var(--font-size-sm);
        .cell {
          color: var(--color-base);
        }
      }

      .icon-logo {
        width: 3rem;
        height: 3rem;
        padding: 0.5rem;
        border-radius: var(--radius-sm);
        background-color: var(--color-button-bg);

        &.modrinth {
          background-color: var(--color-brand) !important;
          border: 1px solid var(--color-accent-contrast) !important;

          .icon {
            width: 2rem;
            height: 2rem;
            z-index: 0;

            :deep(svg) {
              width: 2rem;
              height: 2rem;
            }

            :deep(path) {
              fill: var(--color-accent-contrast) !important;
            }
          }
        }
      }
    }

    .importing {
      position: relative;
      grid-column: 2 / 3;
      grid-row: 1 / 2;

      .icon-logo {
        position: absolute;
        top: 4rem;
        left: 50%;
        transform: translate(-50%, -50%);
        border-radius: 100%;
      }

      .outer-ring {
        position: absolute;
        top: 4rem;
        left: 50%;
        transform: translate(-50%, -50%);
      }

      .ring {
        position: relative;
        display: flex;
        justify-content: center;
        align-items: center;
        width: 100%;
        height: 100%;

        .base-ring {
          position: absolute;
          border-radius: 100%;
          top: 4rem;
          left: 50%;
          transform: translate(-50%, -50%);
          z-index: -1;
          border: 1px solid rgba(#a8b1ddbf, 0.25);
        }

        .first-ring {
          @extend .base-ring;
          width: 15rem;
          height: 15rem;
          background: radial-gradient(
            50% 50% at 50% 50%,
            rgba(5, 206, 69, 0.19) 0%,
            rgba(15, 19, 49, 0.25) 100%
          );
        }

        .second-ring {
          @extend .base-ring;
          width: 25rem;
          height: 25rem;
          opacity: 0.75;
          background: radial-gradient(
              50% 50% at 50% 50%,
              rgba(5, 206, 69, 0.19) 0%,
              rgba(15, 19, 49, 0.25) 100%
            ),
            radial-gradient(
              50% 50% at 50% 50%,
              rgba(44, 48, 79, 0.25) 0%,
              rgba(32, 35, 50, 0.19) 100%
            );
        }

        .third-ring {
          @extend .base-ring;
          width: 35rem;
          height: 35rem;
          opacity: 0.5;
          background: radial-gradient(
            50% 50% at 50% 50%,
            rgba(44, 48, 79, 0.25) 0%,
            rgba(32, 35, 50, 0.19) 100%
          );
        }
      }

      .inner-ring {
        position: relative;
      }

      .launcher-badge {
        position: absolute;
        background-color: var(--landing-transition-gradient-end);
        width: 3rem;
        height: 3rem;
        padding: 0.5rem;
        border-radius: 100%;
        z-index: 10;
        user-select: none;

        &:hover {
          cursor: default;
        }

        &.top-left {
          top: calc(15% - 3rem);
          left: calc(58% - 11rem);
        }

        &.top-right {
          top: calc(15% - 3rem);
          right: calc(58% - 11rem);
        }

        &.bottom-left {
          bottom: calc(41% + 3rem);
          left: calc(58% - 11rem);
        }

        &.bottom-right {
          bottom: calc(41% + 3rem);
          right: calc(58% - 11rem);
        }

        &.bottom-middle {
          bottom: calc(22% + 3rem);
          left: 50%;
          transform: translateX(-50%);
        }

        &.center {
          top: calc(50% - 3rem);
          left: 50%;
          transform: translate(-50%, -50%);
        }

        svg {
          width: 2rem;
          height: 2rem;
        }

        img {
          width: 2rem;
          height: 2rem;
        }
      }

      .text {
        position: absolute;
        text-align: center;
        left: 50%;
        bottom: 0;
        transform: translateX(-50%);
        width: 100%;
        padding: var(--gap-xl);
      }
    }

    .website {
      grid-column: 2 / 3;
      grid-row: 3 / 5;
      text-align: center;
      padding: 0 !important;

      position: relative;

      .projects-showcase {
        margin: calc(7rem + var(--gap-xl)) 0 var(--gap-xl);
        z-index: 3;
        text-align: left;

        .row {
          --gap: var(--gap-md);

          width: 100vw;
          gap: var(--gap);
          margin-bottom: var(--gap);
          display: flex;
          overflow: hidden;
          user-select: none;

          .row__content {
            flex-shrink: 0;
            display: flex;
            min-width: 100%;
            gap: var(--gap);
            transform: translateX(-15%);

            &.offset {
              transform: translateX(-130%);
            }
          }

          .project {
            position: relative;
            display: flex;

            cursor: pointer;
            padding: 1rem;
            gap: 1rem;
            border-radius: 1rem;
            border: 1px solid var(--landing-border-color);
            transition:
              background 0.5s ease-in-out,
              transform 0.05s ease-in-out;
            // Removed due to lag on mobile :(
            background: var(--landing-blob-gradient);

            img {
              height: 3rem;
            }

            .project-info {
              box-sizing: border-box;
            }

            .title {
              color: var(--landing-color-heading);
              max-width: 13.75rem;
              overflow: hidden;
              white-space: nowrap;
              text-overflow: ellipsis;
              margin: 0;
              font-weight: 600;
              font-size: 1.25rem;
              line-height: 110%;
              display: block;
            }

            .description {
              width: 13.75rem;

              display: -webkit-box;
              -webkit-line-clamp: 2;
              -webkit-box-orient: vertical;
              overflow: hidden;

              font-weight: 500;
              font-size: 0.875rem;
              line-height: 125%;
              margin: 0.25rem 0 0;
            }
          }
        }
      }

      .ellipsis {
        position: absolute;
        top: 2rem;
        left: 50%;
        height: 150%;
        z-index: -1;
        aspect-ratio: 1;
        border-radius: 100%;
        transform: translate(-50%, 2rem);
        background: var(--landing-blob-gradient);
        border: 1px solid var(--landing-border-color);
      }

      .icon-logo {
        position: absolute;
        top: 2rem;
        left: 50%;
        transform: translate(-50%, 0);
        z-index: 4;
      }

      p {
        padding: var(--gap-xl);
        padding-top: 0;
      }
    }

    .feature {
      padding: var(--gap-xl);
      z-index: 1;
      background: radial-gradient(
        50% 50% at 50% 50%,
        rgba(44, 48, 79, 0.35) 0%,
        rgba(32, 35, 50, 0.27) 100%
      );
      box-shadow:
        2px 2px 12px 0px rgba(0, 0, 0, 0.16),
        2px 2px 64px 0px rgba(57, 61, 94, 0.45) inset;
      backdrop-filter: blur(6px);
      -webkit-backdrop-filter: blur(6px);
      max-width: 540px;
      width: 100%;
      overflow: hidden;

      .additional-label {
        width: fit-content;
        padding: 0.5rem 0.75rem;
        margin-bottom: 0.5rem;
        background: var(--landing-blue-label-bg);
        color: var(--landing-blue-label);
        border-radius: 6px;
        font-weight: 700;
        font-size: 1rem;
      }

      h3,
      p {
        margin: 0;
      }

      h3 {
        font-size: var(--font-size-xl);
        color: var(--landing-color-heading);
        margin-bottom: 0.375rem;
      }

      p {
        color: var(--landing-color-subheading);
      }
    }
  }

  .feature-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--gap-lg);
    max-width: 1096px;
    margin: 0 auto;
    padding: calc(var(--gap-xl) * 2) 1rem;

    @media (max-width: 1024px) {
      grid-template-columns: repeat(1, 1fr);

      .point {
        text-align: center;

        .title {
          justify-content: center;
        }
      }
    }

    .point {
      display: flex;
      flex-direction: column;
      gap: var(--gap-md);
      padding: 1rem 0;

      svg {
        width: 1.5rem;
        height: 1.5rem;
      }

      .title {
        display: flex;
        align-items: center;
        gap: 0.5rem;
      }

      h3 {
        font-size: var(--font-size-lg);
        font-weight: normal;
        color: var(--landing-color-heading);
        margin: 0;
      }

      p {
        color: var(--landing-color-subheading);
        margin: 0;
      }

      a {
        text-decoration: underline;
      }
    }
  }
}

.table {
  display: grid;
  border: 1px solid rgba(#a8b1ddbf, 0.25);
  gap: 0.25rem;
  overflow: hidden;
  font-size: var(--font-size-sm);
  background: rgba(59, 63, 85, 0.15);
  box-shadow: 2px 2px 12px 0px rgba(0, 0, 0, 0.16);

  .btn,
  button {
    &:hover {
      cursor: default !important;
    }
  }

  .first {
    border-top: none !important;
  }

  .row {
    &:not(.header) {
      border-top: 1px solid rgba(#a8b1ddbf, 0.25);
    }
  }
}

.row,
.header,
.table,
.project,
.export-card {
  user-select: none;

  &:hover {
    cursor: default;
  }
}

.footer {
  padding: var(--gap-xl);
  background: var(--color-accent-contrast);
  color: var(--color-contrast);
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: var(--gap-xl);
  justify-content: center;
  align-items: center;

  .section-badge {
    background-color: var(--color-brand-highlight);
    color: var(--color-brand);
    border-radius: var(--radius-sm);
    width: min-content;
    padding: var(--gap-lg) var(--gap-xl);
    white-space: nowrap;
  }

  .section-subheader {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--gap-sm);
    font-size: var(--font-size-lg);
    margin: 2rem 0;

    .section-subheader-title {
      font-size: var(--font-size-xl);
      margin: 0;
    }

    .section-subheader-description {
      color: var(--color-base);
      margin: 0;
    }
  }

  .download-section {
    display: grid;
    grid-template-columns: 1fr 1px 1fr 1px 1fr;
    height: 100%;
    gap: var(--gap-lg);
    max-width: 1096px;
    margin: 0 auto;

    @media (max-width: 1024px) {
      grid-template-columns: repeat(1, 1fr);
      max-width: 340px;

      .divider {
        display: none;
      }
    }

    .divider {
      height: 13rem;
      width: 1px;
      background: var(--landing-border-color);
      margin: 0;
    }

    .download-card {
      display: flex;
      flex-direction: column;
      gap: calc(var(--gap-lg) * 2);
      padding: calc(var(--gap-lg) * 2);
      height: min-content;

      .title {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        font-size: var(--font-size-2xl);
        gap: var(--gap-sm);
        border-radius: var(--radius-md) var(--radius-md) 0 0;
        color: var(--color-contrast);
      }

      .description {
        display: flex;
        flex-direction: column;
        align-items: center;
        border-top: none;
        font-size: var(--font-size-md);
        color: var(--color-brand);
        gap: var(--gap-sm);

        a {
          display: flex;
          align-items: flex-start;
          gap: var(--gap-sm);
          justify-content: center;

          &:hover {
            cursor: pointer;
          }
        }

        &.apple {
          align-items: flex-start;
        }
      }

      :deep(.animated-dropdown) {
        color: var(--color-brand);
        width: 16rem;
        white-space: nowrap;

        .selected {
          border: 1px solid var(--color-brand);
          background-color: var(--color-accent-contrast);
        }

        .options {
          border: 1px solid var(--color-brand);
          border-radius: 0 0 var(--radius-md) var(--radius-md);
        }

        .option {
          background-color: var(--color-accent-contrast);
        }

        .selected-option {
          background-color: var(--color-brand);
        }
      }
    }
  }

  .terms {
    margin: var(--gap-xl);
    font-size: var(--font-size-lg);
    color: var(--landing-color-subheading);
    text-align: center;
    line-height: 1.5;

    a {
      text-decoration: underline;
    }
  }
}

.icon-logo {
  background: var(--color-accent-contrast);
  width: 5rem;
  height: 5rem;
  padding: 0.75rem;
  display: flex;
  justify-content: center;
  align-items: center;
  border: 1px solid var(--landing-border-color);
  border-radius: var(--radius-lg);

  .icon {
    margin: auto;
    height: 3.5rem;

    :deep(svg) {
      width: 3.5rem;
      height: 3.5rem;
    }

    :deep(.rotate) {
      animation: none !important;
    }
  }
}

.gradient-border {
  position: relative;
  border-radius: var(--radius-lg);

  &:before {
    content: "";
    position: absolute;
    inset: 0;
    padding: 1px;
    z-index: -1;
    border-radius: 1rem;
    background: var(--landing-border-gradient);

    -webkit-mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    mask:
      linear-gradient(#fff 0 0) content-box,
      linear-gradient(#fff 0 0);
    -webkit-mask-composite: xor;
    mask-composite: exclude;
  }
}

.bottom-transition {
  position: absolute;
  bottom: 0;
  width: 100%;
  height: 30rem;
  background: linear-gradient(
    0deg,
    var(--landing-transition-gradient-end) 0%,
    var(--landing-transition-gradient-start) 100%
  );
}

@media screen and (max-width: 1024px) {
  .feature-grid {
    grid-template-columns: 1fr !important;
    grid-template-rows: repeat(8, 1fr) !important;
    gap: var(--gap-lg);
    margin: 0 auto;
    align-content: center;

    .feature {
      width: 100% !important;
      margin: 0 auto;
    }

    .mods {
      grid-row: 1 / 3 !important;
      grid-column: 1 / 2 !important;
    }

    .importing {
      grid-row: 3 / 4 !important;
      grid-column: 1 / 2 !important;
    }

    .sharing {
      grid-row: 4 / 5 !important;
      grid-column: 1 / 2 !important;
    }

    .performance {
      grid-row: 5 / 6 !important;
      grid-column: 1 / 2 !important;
    }

    .playing {
      grid-row: 6 / 7 !important;
      grid-column: 1 / 2 !important;
    }

    .website {
      grid-row: 7 / 9 !important;
      grid-column: 1 / 2 !important;
    }
  }

  .main-header {
    font-size: 4rem !important;
  }

  .subheader {
    font-size: 1.5rem !important;
  }

  .main-subheader {
    font-size: 1.25rem !important;
  }
}

@media screen and (max-width: 746px) {
  .main-header {
    font-size: 3rem !important;
  }

  .subheader {
    font-size: 1.25rem !important;
  }

  .main-subheader {
    font-size: 1.1rem !important;
  }

  .logo-banner {
    padding: 3rem 1rem 3.75rem 1rem;

    .overlay {
      bottom: 3.5rem;
    }
  }
}

.dark-icon {
  display: block;
}

.light-icon {
  display: none;
}

.light-mode {
  .footer,
  .features {
    background: #f8f7f8;
  }

  .bottom-transition {
    background: linear-gradient(rgba(#f8f7f8, 0) 0%, #f8f7f8 100%);
  }

  .dark-icon {
    display: none;
  }

  .light-icon {
    display: block;
  }

  .feature {
    background: radial-gradient(
      50% 50% at 50% 50%,
      rgba(255, 255, 255, 0.35) 0%,
      rgba(255, 255, 255, 0.27) 100%
    ) !important;
    box-shadow:
      2px 2px 64px 0px rgba(255, 255, 255, 0.45) inset,
      2px 2px 12px 0px rgba(0, 0, 0, 0.16) !important;
    border: none !important;
  }

  .gradient-border {
    &:before {
      background: var(--landing-border-gradient-light);
    }
  }

  .search-bar {
    background: var(--color-raised-bg) !important;
    border: 2px solid var(--color-brand) !important;

    .mini-input {
      background: var(--color-raised-bg) !important;
      border: 2px solid var(--color-bg);
    }
  }

  .ellipsis {
    opacity: 0.75;
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .landing-hero {
    background: url("https://cdn-raw.modrinth.com/app-landing/cube-light.png") no-repeat center 4rem;
    background-size: cover;
  }

  .base-ring {
    border: 1px solid rgba(#a8b1ddbf, 0.25) !important;
  }

  .first-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .second-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .third-ring {
    background: linear-gradient(
      180deg,
      rgba(5, 206, 69, 0.15) 0%,
      rgba(5, 206, 69, 0) 100%
    ) !important;
  }

  .table {
    background: white;
  }

  .project {
    background: rgba(255, 255, 255, 0.8) !important;
  }

  .export-card {
    background: white !important;
  }
}

.main-header-strong {
  font-weight: 600;
  background-color: #00bd3c;
  background-image: linear-gradient(180deg, #a7d0ff 0%, var(--color-brand) 60%);
  background-size: 100%;
  background-clip: text;
  -webkit-text-fill-color: transparent;
  -moz-text-fill-color: transparent;
  color: transparent;
}

.logo-banner {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 4rem 1rem 6.75rem 1rem;
  overflow: hidden;
  width: 100%;
  background-color: var(--color-accent-contrast);

  svg {
    z-index: 2;
    width: auto;
    height: 32rem;
  }

  .overlay {
    z-index: 3;
    position: absolute;
    bottom: 5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1.5rem;
    white-space: nowrap;
  }
}
</style>
