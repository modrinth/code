import * as THREE from 'three'

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
