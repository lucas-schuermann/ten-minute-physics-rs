import GUI from 'lil-gui'
import * as THREE from 'three';

import { memory } from '../pkg/index_bg.wasm';
import { HashSim } from '../pkg';

type HashDemoProps = {
    bodies: number;
    animate: boolean;
    showCollisions: boolean;
    reset: () => void;
};

class HashDemo {
    sim: HashSim;
    scene: THREE.Scene;
    camera: THREE.Camera;
    props: HashDemoProps;
    guiFolder: GUI;
    mesh: any;
    translationMatrix: any;
    colors: any;
    positions: any;
    collisions: any;
    baseColor: THREE.Color;
    collisionColor: THREE.Color;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, guiFolder: GUI, scene: THREE.Scene, camera: THREE.Camera) {
        this.sim = new rust_wasm.HashSim(canvas);

        // bound
        this.scene = scene;
        this.camera = camera;

        // members
        this.guiFolder = guiFolder;
        this.baseColor = new THREE.Color(0xFF0000);
        this.collisionColor = new THREE.Color(0xFF8000);
    }

    init() {
        this.initControls();
        this.initMesh();
    }

    initControls() {
        // populate controls
        this.props = {
            bodies: this.sim.num_bodies(),
            animate: true,
            showCollisions: false,
            reset: () => {
                this.sim.reset();
                this.updateMesh();
            },
        };
        this.guiFolder.add(this.props, 'bodies').disable();
        this.guiFolder.add(this.props, 'showCollisions').name('show collisions');
        this.guiFolder.add(this.props, 'animate');
        this.guiFolder.add(this.props, 'reset').name('reset simulation');
    }

    initMesh() {
        const material = new THREE.MeshPhongMaterial();
        const geometry = new THREE.SphereGeometry(this.sim.radius(), 8, 8);
        this.mesh = new THREE.InstancedMesh(geometry, material, this.sim.num_bodies());
        this.translationMatrix = new THREE.Matrix4();
        this.mesh.instanceMatrix.setUsage(THREE.DynamicDrawUsage);
        this.colors = new Float32Array(3 * this.sim.num_bodies());
        this.mesh.instanceColor = new THREE.InstancedBufferAttribute(this.colors, 3, false, 1);
        this.scene.add(this.mesh);

        // Here, we store the pointer to the positions buffer location after the simulation is
        // initialized (all allocations are completed). In the WASM linear heap, it will be 
        // constant thereafter, so we don't need to touch the array moving forward.
        const positionsPtr = this.sim.body_positions_ptr();
        this.positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_bodies() * 3);
        const collisionsPtr = this.sim.body_collisions_ptr();
        this.collisions = new Uint8Array(memory.buffer, collisionsPtr, this.sim.num_bodies());

        this.updateMesh();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMesh();
        }
    }

    updateMesh() {
        for (let i = 0; i < this.positions.length; i++) {
            this.translationMatrix.makeTranslation(this.positions[3 * i], this.positions[3 * i + 1], this.positions[3 * i + 2]);
            this.mesh.setMatrixAt(i, this.translationMatrix);
            if (this.props.showCollisions && this.collisions[i] === 1) {
                //this.mesh.setColorAt(i, this.collisionColor);
                this.colors[3 * i] = this.collisionColor.r;
                this.colors[3 * i + 1] = this.collisionColor.g;
                this.colors[3 * i + 2] = this.collisionColor.b;
            } else {
                //this.mesh.setColorAt(i, this.baseColor);
                this.colors[3 * i] = this.baseColor.r;
                this.colors[3 * i + 1] = this.baseColor.g;
                this.colors[3 * i + 2] = this.baseColor.b;
            }
        }
        this.mesh.instanceMatrix.needsUpdate = true;
        this.mesh.instanceColor.needsUpdate = true;
    }
}

export { HashDemo };