import GUI, { Controller } from 'lil-gui';
import * as THREE from 'three';

import { HeightFieldWaterSimulation } from '../pkg';
import { Demo, Grabber, Scene3D, Scene3DConfig } from './lib';

// TODO: caps and think of which to put in rs vs here
const tankSize = { x: 2.5, y: 1.0, z: 3.0 };
const tankBorder = 0.03;
const waterHeight = 0.8;
const waterSpacing = 0.02;

type HeightFieldWaterDemoProps = {
    // bodies: number;
    animate: boolean;
    // showCollisions: boolean;
};

const HeightFieldWaterDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [2.1, 1.5],
    cameraLookAt: new THREE.Vector3(0, 0.8, 0),
}


class HeightFieldWaterDemo implements Demo<HeightFieldWaterSimulation, HeightFieldWaterDemoProps> {
    sim: HeightFieldWaterSimulation;
    scene: Scene3D;
    props: HeightFieldWaterDemoProps;

    private memory: WebAssembly.Memory;
    private balls: THREE.Mesh[];
    private surfaceMesh: THREE.Mesh;
    private renderTarget: THREE.WebGLRenderTarget;
    private positions: Float32Array; // mapped to WASM memory
    private grabber: Grabber;

    constructor(rust_wasm: any, memory: WebAssembly.Memory, canvas: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.memory = memory;
        this.sim = new rust_wasm.HeightFieldWaterSimulation(tankSize.x, tankSize.z, waterHeight, waterSpacing, tankBorder);
        this.scene = scene;
        this.balls = [];
        this.initControls(folder, canvas);
    }

    init() {
        this.initMeshes();
    }

    update() {
        if (this.props.animate) {
            const dt = 1.0 / 30.0; // TODO, compute real dt
            this.sim.step(dt);
            this.grabber.increaseTime(dt);
            this.updateMeshes();

            const renderer = this.scene.renderer;

            this.surfaceMesh.visible = false;
            renderer.setRenderTarget(this.renderTarget);
            renderer.clear();
            renderer.render(this.scene.scene, this.scene.camera);

            this.surfaceMesh.visible = true;
            renderer.setRenderTarget(null);
        }
    }

    reset() {
        this.sim.reset(waterHeight);
        this.updateMeshes();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        let animateController: Controller;
        this.props = {
            animate: true,
            // showCollisions: false,
        };
        animateController = folder.add(this.props, 'animate');

        // grab interaction handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initWaterMesh() {
        const vertexShader = `
            varying vec3 varNormal;
			varying vec2 varScreenPos;
			varying vec3 varPos;

			void main() {
				vec4 pos = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
				varScreenPos = vec2(0.5, 0.5) + 0.5 * vec2(pos) / pos.z;
				varPos = vec3(position);       
				varNormal = normal;	
				gl_Position = pos;
			}`;
        const fragmentShader = `
            uniform sampler2D background;
            varying vec3 varNormal;
            varying vec2 varScreenPos;
            varying vec3 varPos;			

            void main() {
                float r = 0.02;	// todo: should be distance dependent!
                vec2 uv = varScreenPos + r * vec2(varNormal.x, varNormal.z);
                vec4 color = texture2D(background, uv);
                color.z = min(color.z + 0.2, 1.0);

                vec3 L = normalize(vec3(10.0, 10.0, 10.0) - varPos);   
                float s = max(dot(varNormal,L), 0.0); 
                color *= (0.5 + 0.5 * s);
            
                gl_FragColor = color;
            }`;

        // water
        this.renderTarget = new THREE.WebGLRenderTarget(this.scene.renderer.domElement.width, this.scene.renderer.domElement.height,
            { minFilter: THREE.LinearFilter, magFilter: THREE.NearestFilter, format: THREE.RGBAFormat });
        const surfaceShaderMaterial = new THREE.ShaderMaterial({
            uniforms: { background: { value: this.renderTarget.texture } },
            vertexShader,
            fragmentShader,
        });

        const uvs = this.sim.uvs;
        const indices = this.sim.indices;

        // Here, we store the pointer to the positions buffer location after the simulation is
        // initialized (all allocations are completed). In the WASM linear heap, it will be constant 
        // thereafter, so we don't need to refresh the pointer moving forward.
        const positionsPtr = this.sim.positions;
        this.positions = new Float32Array(this.memory.buffer, positionsPtr, this.sim.num_cells * 3);

        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setAttribute('uv', new THREE.BufferAttribute(uvs, 2));
        geometry.setIndex(new THREE.BufferAttribute(indices, 1));
        this.surfaceMesh = new THREE.Mesh(geometry, surfaceShaderMaterial);

        this.updateSurfaceMesh();
        this.scene.scene.add(this.surfaceMesh);
    }

    private initMeshes() {
        this.initWaterMesh();

        // iterate over sim.ball_radii
        this.sim.ball_radii.forEach((radius, id) => {
            const geometry = new THREE.SphereGeometry(radius, 32, 32);
            const material = new THREE.MeshPhongMaterial({ color: 0xffff00 }); // TODO: vary color
            const mesh = new THREE.Mesh(geometry, material);
            //mesh.position.copy(pos);
            mesh.userData = { id }; // for raycasting
            mesh.layers.enable(1);
            this.scene.scene.add(mesh);
            this.balls.push(mesh);
        });

        // tank
        let wx = tankSize.x;
        let wy = tankSize.y;
        let wz = tankSize.z;
        let b = tankBorder;
        const tankMaterial = new THREE.MeshPhongMaterial({ color: 0x909090 });
        var boxGeometry = new THREE.BoxGeometry(b, wy, wz);
        var box = new THREE.Mesh(boxGeometry, tankMaterial);
        box.position.set(-0.5 * wx, wy * 0.5, 0.0)
        this.scene.scene.add(box);
        var box = new THREE.Mesh(boxGeometry, tankMaterial);
        box.position.set(0.5 * wx, 0.5 * wy, 0.0)
        this.scene.scene.add(box);
        var boxGeometry = new THREE.BoxGeometry(wx, wy, b);
        var box = new THREE.Mesh(boxGeometry, tankMaterial);
        box.position.set(0.0, 0.5 * wy, - wz * 0.5)
        this.scene.scene.add(box);
        var box = new THREE.Mesh(boxGeometry, tankMaterial);
        box.position.set(0.0, 0.5 * wy, wz * 0.5)
        this.scene.scene.add(box);
    }

    private updateSurfaceMesh() {
        this.surfaceMesh.geometry.attributes.position.needsUpdate = true;
        this.surfaceMesh.geometry.computeVertexNormals();
        this.surfaceMesh.geometry.computeBoundingSphere();
    }

    private updateMeshes() {
        this.updateSurfaceMesh();

        // balls
        const ballPositions = this.sim.ball_positions;
        this.balls.forEach((ball, i) => {
            const pos = new THREE.Vector3(ballPositions[i * 3], ballPositions[i * 3 + 1], ballPositions[i * 3 + 2]);
            ball.position.copy(pos);
        });
    }
}

export { HeightFieldWaterDemo, HeightFieldWaterDemoConfig };
