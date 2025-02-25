<template>
  <div ref="container" class="relative h-[400px] w-full cursor-move lg:h-[600px]">
    <div
      v-for="location in locations"
      :key="location.name"
      :class="{
        'opacity-0': !showLabels,
        hidden: !isLocationVisible(location),
        'z-40': location.clicked,
      }"
      :style="{
        position: 'absolute',
        left: `${location.screenPosition?.x || 0}px`,
        top: `${location.screenPosition?.y || 0}px`,
      }"
      class="location-button center-on-top-left flex transform cursor-pointer items-center rounded-full bg-bg px-3 outline-1 outline-red transition-opacity duration-200 hover:z-50"
      @click="toggleLocationClicked(location)"
    >
      <div
        :class="{
          'animate-pulse': location.active,
          'border-gray-400': !location.active,
          'border-purple bg-purple': location.active,
          'border-dashed': !location.active,
          'opacity-40': !location.active,
        }"
        class="my-3 size-2.5 shrink-0 rounded-full border-2"
      ></div>
      <div
        class="expanding-item"
        :class="{
          expanded: location.clicked,
        }"
      >
        <div class="whitespace-nowrap text-sm">
          <span class="ml-2"> {{ location.name }} </span>
          <span v-if="!location.active" class="ml-1 text-xs text-secondary">(Coming Soon)</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";
import { ref, onMounted, onUnmounted } from "vue";

const container = ref(null);
const showLabels = ref(false);

const locations = ref([
  // Active locations
  { name: "New York", lat: 40.7128, lng: -74.006, active: true, clicked: false },
  { name: "Los Angeles", lat: 34.0522, lng: -118.2437, active: true, clicked: false },
  { name: "Miami", lat: 25.7617, lng: -80.1918, active: true, clicked: false },
  { name: "Spokane", lat: 47.667309, lng: -117.411922, active: true, clicked: false },
  { name: "Dallas", lat: 32.78372, lng: -96.7947, active: true, clicked: false },
  // Future Locations
  // { name: "London", lat: 51.5074, lng: -0.1278, active: false, clicked: false },
  // { name: "Frankfurt", lat: 50.1109, lng: 8.6821, active: false, clicked: false },
  // { name: "Amsterdam", lat: 52.3676, lng: 4.9041, active: false, clicked: false },
  // { name: "Paris", lat: 48.8566, lng: 2.3522, active: false, clicked: false },
  // { name: "Singapore", lat: 1.3521, lng: 103.8198, active: false, clicked: false },
  // { name: "Tokyo", lat: 35.6762, lng: 139.6503, active: false, clicked: false },
  // { name: "Sydney", lat: -33.8688, lng: 151.2093, active: false, clicked: false },
  // { name: "São Paulo", lat: -23.5505, lng: -46.6333, active: false, clicked: false },
  // { name: "Toronto", lat: 43.6532, lng: -79.3832, active: false, clicked: false },
]);

const isLocationVisible = (location) => {
  if (!location.screenPosition || !globe) return false;

  const vector = latLngToVector3(location.lat, location.lng).clone();
  vector.applyMatrix4(globe.matrixWorld);

  const cameraVector = new THREE.Vector3();
  camera.getWorldPosition(cameraVector);

  const viewVector = vector.clone().sub(cameraVector).normalize();

  const normal = vector.clone().normalize();

  const dotProduct = normal.dot(viewVector);

  return dotProduct < -0.15;
};

const toggleLocationClicked = (location) => {
  console.log("clicked", location.name);
  locations.value.find((loc) => loc.name === location.name).clicked = !location.clicked;
};

let scene, camera, renderer, globe, controls;
let animationFrame;

const init = () => {
  scene = new THREE.Scene();
  camera = new THREE.PerspectiveCamera(
    45,
    container.value.clientWidth / container.value.clientHeight,
    0.1,
    1000,
  );
  renderer = new THREE.WebGLRenderer({
    antialias: true,
    alpha: true,
    powerPreference: "low-power",
  });
  renderer.setPixelRatio(window.devicePixelRatio);
  renderer.setSize(container.value.clientWidth, container.value.clientHeight);
  container.value.appendChild(renderer.domElement);

  const geometry = new THREE.SphereGeometry(5, 64, 64);
  const outlineTexture = new THREE.TextureLoader().load("/earth-outline.png");
  outlineTexture.minFilter = THREE.LinearFilter;
  outlineTexture.magFilter = THREE.LinearFilter;

  const material = new THREE.ShaderMaterial({
    uniforms: {
      outlineTexture: { value: outlineTexture },
      globeColor: { value: new THREE.Color("#60fbb5") },
    },
    vertexShader: `
      varying vec2 vUv;
      void main() {
        vUv = uv;
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      uniform sampler2D outlineTexture;
      uniform vec3 globeColor;
      varying vec2 vUv;
      void main() {
        vec4 texColor = texture2D(outlineTexture, vUv);

        float brightness = max(max(texColor.r, texColor.g), texColor.b);
        gl_FragColor = vec4(globeColor, brightness * 0.8);
      }
    `,
    transparent: true,
    side: THREE.FrontSide,
  });

  globe = new THREE.Mesh(geometry, material);
  scene.add(globe);

  const atmosphereGeometry = new THREE.SphereGeometry(5.2, 64, 64);
  const atmosphereMaterial = new THREE.ShaderMaterial({
    transparent: true,
    side: THREE.BackSide,
    uniforms: {
      color: { value: new THREE.Color("#56f690") },
      viewVector: { value: camera.position },
    },
    vertexShader: `
      uniform vec3 viewVector;
      varying float intensity;
      void main() {
        vec3 vNormal = normalize(normalMatrix * normal);
        vec3 vNormel = normalize(normalMatrix * viewVector);
        intensity = pow(0.7 - dot(vNormal, vNormel), 2.0);
        gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
      }
    `,
    fragmentShader: `
      uniform vec3 color;
      varying float intensity;
      void main() {
        gl_FragColor = vec4(color, intensity * 0.4);
      }
    `,
  });

  const atmosphere = new THREE.Mesh(atmosphereGeometry, atmosphereMaterial);
  scene.add(atmosphere);

  const ambientLight = new THREE.AmbientLight(0x404040, 0.5);
  scene.add(ambientLight);

  camera.position.z = 15;

  controls = new OrbitControls(camera, renderer.domElement);
  controls.enableDamping = true;
  controls.dampingFactor = 0.05;
  controls.rotateSpeed = 0.3;
  controls.enableZoom = false;
  controls.enablePan = false;
  controls.autoRotate = true;
  controls.autoRotateSpeed = 0.05;
  controls.minPolarAngle = Math.PI * 0.3;
  controls.maxPolarAngle = Math.PI * 0.7;

  globe.rotation.y = Math.PI * 1.9;
  globe.rotation.x = Math.PI * 0.15;
};

const animate = () => {
  animationFrame = requestAnimationFrame(animate);
  controls.update();

  locations.value.forEach((location) => {
    const position = latLngToVector3(location.lat, location.lng);
    const vector = position.clone();
    vector.applyMatrix4(globe.matrixWorld);

    const coords = vector.project(camera);
    const screenPosition = {
      x: (coords.x * 0.5 + 0.5) * container.value.clientWidth,
      y: (-coords.y * 0.5 + 0.5) * container.value.clientHeight,
    };
    location.screenPosition = screenPosition;
  });

  renderer.render(scene, camera);
};

const latLngToVector3 = (lat, lng) => {
  const phi = (90 - lat) * (Math.PI / 180);
  const theta = (lng + 180) * (Math.PI / 180);
  const radius = 5;

  return new THREE.Vector3(
    -radius * Math.sin(phi) * Math.cos(theta),
    radius * Math.cos(phi),
    radius * Math.sin(phi) * Math.sin(theta),
  );
};

const handleResize = () => {
  if (!container.value) return;
  camera.aspect = container.value.clientWidth / container.value.clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(container.value.clientWidth, container.value.clientHeight);
};

onMounted(() => {
  init();
  animate();
  window.addEventListener("resize", handleResize);

  setTimeout(() => {
    showLabels.value = true;
  }, 1000);
});

onUnmounted(() => {
  if (animationFrame) {
    cancelAnimationFrame(animationFrame);
  }
  window.removeEventListener("resize", handleResize);
  if (renderer) {
    renderer.dispose();
  }
  if (container.value) {
    container.value.innerHTML = "";
  }
});
</script>

<style scoped>
@keyframes pulse {
  0% {
    box-shadow: 0 0 0 0 rgba(27, 217, 106, 0.3);
  }
  70% {
    box-shadow: 0 0 0 4px rgba(27, 217, 106, 0);
  }
  100% {
    box-shadow: 0 0 0 0 rgba(27, 217, 106, 0);
  }
}

.animate-pulse {
  animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.center-on-top-left {
  transform: translate(-50%, -50%);
}

.expanding-item.expanded {
  grid-template-columns: 1fr;
}

@media (hover: hover) {
  .location-button:hover .expanding-item {
    grid-template-columns: 1fr;
  }
}

.expanding-item {
  display: grid;
  grid-template-columns: 0fr;
  transition: grid-template-columns 0.15s ease-in-out;
  overflow: hidden;

  > div {
    overflow: hidden;
  }
}

@media (prefers-reduced-motion) {
  .expanding-item {
    transition: none !important;
  }
}
</style>
