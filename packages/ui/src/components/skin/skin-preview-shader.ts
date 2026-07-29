import * as THREE from 'three'

type DamageFlashMaterial = THREE.MeshStandardMaterial & {
	userData: THREE.MeshStandardMaterial['userData'] & {
		damageFlashShader?: THREE.Shader
		damageFlashShaderInstalled?: boolean
	}
}

const DAMAGE_FLASH_COLOR = new THREE.Color(0xbd2f2f)
const DAMAGE_FLASH_SHADER_KEY = 'skin-preview-damage-flash'

export function createRadialSpotlightShader() {
	return {
		uniforms: {
			innerColor: { value: new THREE.Color(0x000000) },
			outerColor: { value: new THREE.Color(0xffffff) },
			innerOpacity: { value: 0.3 },
			outerOpacity: { value: 0.0 },
			falloffPower: { value: 1.2 },
			shadowRadius: { value: 7 },
		},
		vertexShader: `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
		fragmentShader: `
    uniform vec3 innerColor;
    uniform vec3 outerColor;
    uniform float innerOpacity;
    uniform float outerOpacity;
    uniform float falloffPower;
    uniform float shadowRadius;

    varying vec2 vUv;

    void main() {
      vec2 center = vec2(0.5, 0.5);
      float dist = distance(vUv, center) * 2.0;

      float shadowFalloff = 1.0 - smoothstep(0.0, shadowRadius, dist);
      float spotlightFalloff = 1.0 - smoothstep(0.0, 1.0, pow(dist, falloffPower));

      vec3 color = mix(outerColor, innerColor, shadowFalloff);
      float opacity = mix(outerOpacity, innerOpacity * shadowFalloff, spotlightFalloff);

      gl_FragColor = vec4(color, opacity);
    }
  `,
		transparent: true,
		depthWrite: false,
		depthTest: false,
	}
}

function installDamageFlashShader(material: THREE.MeshStandardMaterial, intensity: number) {
	const damageMaterial = material as DamageFlashMaterial

	if (damageMaterial.userData.damageFlashShaderInstalled) {
		return
	}

	const previousOnBeforeCompile = material.onBeforeCompile.bind(material)
	const previousCustomProgramCacheKey = material.customProgramCacheKey.bind(material)

	material.onBeforeCompile = (shader, renderer) => {
		previousOnBeforeCompile(shader, renderer)

		shader.uniforms.uDamageFlashIntensity = { value: intensity }
		shader.uniforms.uDamageFlashColor = { value: DAMAGE_FLASH_COLOR }
		shader.fragmentShader = shader.fragmentShader.replace(
			'#include <common>',
			'#include <common>\nuniform float uDamageFlashIntensity;\nuniform vec3 uDamageFlashColor;',
		)
		shader.fragmentShader = shader.fragmentShader.replace(
			'#include <dithering_fragment>',
			'gl_FragColor.rgb = mix(gl_FragColor.rgb, uDamageFlashColor, uDamageFlashIntensity * gl_FragColor.a);\n#include <dithering_fragment>',
		)

		damageMaterial.userData.damageFlashShader = shader
	}

	material.customProgramCacheKey = () =>
		`${previousCustomProgramCacheKey()}|${DAMAGE_FLASH_SHADER_KEY}`
	damageMaterial.userData.damageFlashShaderInstalled = true
	material.needsUpdate = true
}

function syncDamageFlashMaterial(material: THREE.MeshStandardMaterial, intensity: number) {
	installDamageFlashShader(material, intensity)

	const shader = (material as DamageFlashMaterial).userData.damageFlashShader
	if (shader) {
		shader.uniforms.uDamageFlashIntensity.value = intensity
	}
}

export function syncDamageFlashShader(scene: THREE.Object3D | null, intensity: number) {
	if (!scene) return

	scene.traverse((object) => {
		const mesh = object as THREE.Mesh
		if (!mesh.isMesh || !mesh.material) return

		const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material]
		materials.forEach((material) => {
			if (!(material instanceof THREE.MeshStandardMaterial) || material.name === 'cape') return

			syncDamageFlashMaterial(material, intensity)
		})
	})
}
