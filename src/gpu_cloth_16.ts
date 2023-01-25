import GUI from 'lil-gui';
import * as THREE from 'three';
import { GPUComputationRenderer, Variable } from 'three/examples/jsm/misc/GPUComputationRenderer.js';

import { GPUClothSimulation } from '../pkg';
//import { memory } from '../pkg/index_bg.wasm';
import { Demo, Scene3D, Scene3DConfig } from './lib';

type GPUClothDemoProps = {
    animate: boolean;
};

const GPUClothDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [1, 1],
    cameraLookAt: new THREE.Vector3(0, 0.6, 0),
}

// const baseColor = new THREE.Color(0xFF0000);
// const collisionColor = new THREE.Color(0xFF8000);


// Texture width for simulation (each texel is a debris particle)
const WIDTH = 64;
const PARTICLES = WIDTH * WIDTH;

const effectController = {
    // Can be changed dynamically
    gravityConstant: 100.0,
    density: 0.45,

    // Must restart simulation
    radius: 300,
    height: 8,
    exponent: 0.4,
    maxMass: 15.0,
    velocity: 70,
    velocityExponent: 0.2,
    randVelocity: 0.001
};


type ParticleUniforms = {
    texturePosition: { value: THREE.DataTexture };
    textureVelocity: { value: THREE.DataTexture };
    cameraConstant: { value: number; };
    density: { value: number };
}

class GPUClothDemo implements Demo<GPUClothSimulation, GPUClothDemoProps> {
    sim: GPUClothSimulation;
    scene: Scene3D;
    props: GPUClothDemoProps;

    gpuCompute: GPUComputationRenderer;
    velocityVariable: Variable;
    positionVariable: Variable;
    dtPosition: THREE.DataTexture;
    dtVelocity: THREE.DataTexture;
    particleUniforms: ParticleUniforms;

    //private edgeMesh: THREE.LineSegments;
    private triMesh: THREE.Mesh;
    private positions: Float32Array; // mapped to GPU compute memory

    // private mesh: THREE.InstancedMesh;
    // private translationMatrix: THREE.Matrix4;
    // private colors: Float32Array;
    // private positions: Float32Array; // mapped to WASM memory
    // private collisions: Uint8Array; // mapped to WASM memory

    constructor(rust_wasm: any, _: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.sim = new rust_wasm.GPUClothSimulation();
        this.scene = scene;
        this.initControls(folder);
    }

    init() {
        this.initMesh();
        //this.initCompute();

        /*
        let geometry = new THREE.BufferGeometry();

        const positions = new Float32Array(PARTICLES * 3);
        let p = 0;

        for (let i = 0; i < PARTICLES; i++) {

            positions[p++] = (Math.random() * 2 - 1) * effectController.radius;
            positions[p++] = 0; //( Math.random() * 2 - 1 ) * effectController.radius;
            positions[p++] = (Math.random() * 2 - 1) * effectController.radius;

        }

        const uvs = new Float32Array(PARTICLES * 2);
        p = 0;

        for (let j = 0; j < WIDTH; j++) {

            for (let i = 0; i < WIDTH; i++) {

                uvs[p++] = i / (WIDTH - 1);
                uvs[p++] = j / (WIDTH - 1);

            }

        }

        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));

        this.particleUniforms = {
            'texturePosition': { value: null },
            'textureVelocity': { value: null },
            'cameraConstant': { value: this.scene.renderer.domElement.height / (Math.tan(THREE.MathUtils.DEG2RAD * 0.5 * this.scene.camera.fov) / this.scene.camera.zoom) },
            'density': { value: effectController.density }
        };

        // THREE.ShaderMaterial
        const material = new THREE.ShaderMaterial({
            uniforms: this.particleUniforms,
            vertexShader: particleVertexShaderSrc,
            fragmentShader: particleFragmentShaderSrc
        });

        material.extensions.drawBuffers = true;

        const particles = new THREE.Points(geometry, material);
        particles.matrixAutoUpdate = false;
        particles.updateMatrix();

        this.scene.scene.add(particles);
        */
    }

    update() {
        //this.gpuCompute.compute();

        //this.particleUniforms.texturePosition.value = this.gpuCompute.getCurrentRenderTarget(this.positionVariable).texture as THREE.DataTexture;
        //this.particleUniforms.textureVelocity.value = this.gpuCompute.getCurrentRenderTarget(this.velocityVariable).texture as THREE.DataTexture;
    }

    reset() {
    }

    private initControls(_folder: GUI) {
    }

    private initMesh() {
        //const tri_ids = Array.from(this.sim.tri_ids);
        //const edge_ids = Array.from(this.sim.edge_ids);

        this.positions = new Float32Array(this.sim.num_particles * 3);

        // edge mesh
        // let geometry = new THREE.BufferGeometry();
        // geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        // geometry.setIndex(edge_ids);
        // const lineMaterial = new THREE.LineBasicMaterial({ color: 0xff0000, linewidth: 2 });
        // this.edgeMesh = new THREE.LineSegments(geometry, lineMaterial);
        // this.edgeMesh.visible = false;
        // this.scene.scene.add(this.edgeMesh);

        /*
        // visual tri mesh
        let geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.DoubleSide });
        this.triMesh = new THREE.Mesh(geometry, visMaterial);
        this.triMesh.castShadow = true;
        this.triMesh.layers.enable(1);
        this.scene.scene.add(this.triMesh);
        geometry.computeVertexNormals();
        geometry.computeBoundingSphere();

        this.updateMesh();
        */
    }

    private updateMesh() {
        //this.triMesh.geometry.computeVertexNormals();
        //this.triMesh.geometry.attributes.position.needsUpdate = true;
        //this.triMesh.geometry.computeBoundingSphere();
        //this.edgeMesh.geometry.attributes.position.needsUpdate = true;
    }

    fillTextures() {
        const posArray = this.dtPosition.image.data;
        const velArray = this.dtVelocity.image.data;

        const radius = effectController.radius;
        const height = effectController.height;
        const exponent = effectController.exponent;
        const maxMass = effectController.maxMass * 1024 / PARTICLES;
        const maxVel = effectController.velocity;
        const velExponent = effectController.velocityExponent;
        const randVel = effectController.randVelocity;

        for (let k = 0, kl = posArray.length; k < kl; k += 4) {

            // Position
            let x, z, rr;

            do {

                x = (Math.random() * 2 - 1);
                z = (Math.random() * 2 - 1);
                rr = x * x + z * z;

            } while (rr > 1);

            rr = Math.sqrt(rr);

            const rExp = radius * Math.pow(rr, exponent);

            // Velocity
            const vel = maxVel * Math.pow(rr, velExponent);

            const vx = vel * z + (Math.random() * 2 - 1) * randVel;
            const vy = (Math.random() * 2 - 1) * randVel * 0.05;
            const vz = - vel * x + (Math.random() * 2 - 1) * randVel;

            x *= rExp;
            z *= rExp;
            const y = (Math.random() * 2 - 1) * height;

            const mass = Math.random() * maxMass + 1;

            // Fill in texture values
            posArray[k + 0] = x;
            posArray[k + 1] = y;
            posArray[k + 2] = z;
            posArray[k + 3] = 1;

            velArray[k + 0] = vx;
            velArray[k + 1] = vy;
            velArray[k + 2] = vz;
            velArray[k + 3] = mass;

        }

    }

    initCompute() {
        this.gpuCompute = new GPUComputationRenderer(100, 100, this.scene.renderer);

        if (this.scene.renderer.capabilities.isWebGL2 === false) {
            this.gpuCompute.setDataType(THREE.HalfFloatType);
        }

        this.dtPosition = this.gpuCompute.createTexture();
        this.dtVelocity = this.gpuCompute.createTexture();

        this.fillTextures();

        this.velocityVariable = this.gpuCompute.addVariable('textureVelocity', computeShaderVelocitySrc, this.dtVelocity);
        this.positionVariable = this.gpuCompute.addVariable('texturePosition', computeShaderPositionSrc, this.dtPosition);

        this.gpuCompute.setVariableDependencies(this.velocityVariable, [this.positionVariable, this.velocityVariable]);
        this.gpuCompute.setVariableDependencies(this.positionVariable, [this.positionVariable, this.velocityVariable]);

        let velocityUniforms = this.velocityVariable.material.uniforms;
        velocityUniforms['gravityConstant'] = { value: effectController.gravityConstant };
        velocityUniforms['density'] = { value: effectController.density };

        const error = this.gpuCompute.init();
        if (error !== null) {
            console.error(error);
        }
    }
}

const computeShaderPositionSrc = `

#define delta (1.0 / 60.0)

void main() {

				vec2 uv = gl_FragCoord.xy / resolution.xy;

				vec4 tmpPos = texture2D(texturePosition, uv);
				vec3 pos = tmpPos.xyz;

				vec4 tmpVel = texture2D(textureVelocity, uv);
				vec3 vel = tmpVel.xyz;
				float mass = tmpVel.w;

    if (mass == 0.0) {
        vel = vec3(0.0);
    }

    // Dynamics
    pos += vel * delta;

    gl_FragColor = vec4(pos, 1.0);

}`;

const computeShaderVelocitySrc = `
// For PI declaration:
#include <common>

#define delta ( 1.0 / 60.0 )

uniform float gravityConstant;
uniform float density;

const float width = resolution.x;
const float height = resolution.y;

float radiusFromMass( float mass ) {
    // Calculate radius of a sphere from mass and density
    return pow( ( 3.0 / ( 4.0 * PI ) ) * mass / density, 1.0 / 3.0 );
}

void main()	{

    vec2 uv = gl_FragCoord.xy / resolution.xy;
    float idParticle = uv.y * resolution.x + uv.x;

    vec4 tmpPos = texture2D( texturePosition, uv );
    vec3 pos = tmpPos.xyz;

    vec4 tmpVel = texture2D( textureVelocity, uv );
    vec3 vel = tmpVel.xyz;
    float mass = tmpVel.w;

    if ( mass > 0.0 ) {

        float radius = radiusFromMass( mass );

        vec3 acceleration = vec3( 0.0 );

        // Gravity interaction
        for ( float y = 0.0; y < height; y++ ) {

            for ( float x = 0.0; x < width; x++ ) {

                vec2 secondParticleCoords = vec2( x + 0.5, y + 0.5 ) / resolution.xy;
                vec3 pos2 = texture2D( texturePosition, secondParticleCoords ).xyz;
                vec4 velTemp2 = texture2D( textureVelocity, secondParticleCoords );
                vec3 vel2 = velTemp2.xyz;
                float mass2 = velTemp2.w;

                float idParticle2 = secondParticleCoords.y * resolution.x + secondParticleCoords.x;

                if ( idParticle == idParticle2 ) {
                    continue;
                }

                if ( mass2 == 0.0 ) {
                    continue;
                }

                vec3 dPos = pos2 - pos;
                float distance = length( dPos );
                float radius2 = radiusFromMass( mass2 );

                if ( distance == 0.0 ) {
                    continue;
                }

                // Checks collision

                if ( distance < radius + radius2 ) {

                    if ( idParticle < idParticle2 ) {

                        // This particle is aggregated by the other
                        vel = ( vel * mass + vel2 * mass2 ) / ( mass + mass2 );
                        mass += mass2;
                        radius = radiusFromMass( mass );

                    }
                    else {

                        // This particle dies
                        mass = 0.0;
                        radius = 0.0;
                        vel = vec3( 0.0 );
                        break;

                    }

                }

                float distanceSq = distance * distance;

                float gravityField = gravityConstant * mass2 / distanceSq;

                gravityField = min( gravityField, 1000.0 );

                acceleration += gravityField * normalize( dPos );

            }

            if ( mass == 0.0 ) {
                break;
            }
        }

        // Dynamics
        vel += delta * acceleration;

    }

    gl_FragColor = vec4( vel, mass );

}`;

const particleVertexShaderSrc = `
// For PI declaration:
#include <common>

uniform sampler2D texturePosition;
uniform sampler2D textureVelocity;

uniform float cameraConstant;
uniform float density;

varying vec4 vColor;

float radiusFromMass( float mass ) {
    // Calculate radius of a sphere from mass and density
    return pow( ( 3.0 / ( 4.0 * PI ) ) * mass / density, 1.0 / 3.0 );
}


void main() {


    vec4 posTemp = texture2D( texturePosition, uv );
    vec3 pos = posTemp.xyz;

    vec4 velTemp = texture2D( textureVelocity, uv );
    vec3 vel = velTemp.xyz;
    float mass = velTemp.w;

    vColor = vec4( 1.0, mass / 250.0, 0.0, 1.0 );

    vec4 mvPosition = modelViewMatrix * vec4( pos, 1.0 );

    // Calculate radius of a sphere from mass and density
    //float radius = pow( ( 3.0 / ( 4.0 * PI ) ) * mass / density, 1.0 / 3.0 );
    float radius = radiusFromMass( mass );

    // Apparent size in pixels
    if ( mass == 0.0 ) {
        gl_PointSize = 0.0;
    }
    else {
        gl_PointSize = radius * cameraConstant / ( - mvPosition.z );
    }

    gl_Position = projectionMatrix * mvPosition;

}`;

const particleFragmentShaderSrc = `
varying vec4 vColor;

void main() {

    if ( vColor.y == 0.0 ) discard;

    float f = length( gl_PointCoord - vec2( 0.5, 0.5 ) );
    if ( f > 0.5 ) {
        discard;
    }
    gl_FragColor = vColor;

}
`;

export { GPUClothDemo, GPUClothDemoConfig };
