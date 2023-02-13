import GUI, { Controller } from 'lil-gui';
import * as THREE from 'three';

import { BodyChainSimulation } from '../pkg';
import { memory } from '../pkg/index_bg.wasm';
import { Demo, Scene3D, Scene3DConfig, Grabber } from './lib';

const DEFAULT_NUM_OBJECTS = 100;
const DEFAULT_OBJECT_SIZE = new THREE.Vector3(0.02, 0.04, 0.02);
const DEFAULT_LAST_OBJECT_SIZE = new THREE.Vector3(0.2, 0.04, 0.2);
const DEFAULT_NUM_SOLVER_SUBSTEPS = 40;
const DEFAULT_ROT_DAMPING = 1000.0;
const DEFAULT_POS_DAMPING = 1000.0;
const DEFAULT_COMPLIANCE = 0.0;

type BodyChainDemoProps = {
    bodies: number;
    animate: boolean;
    substeps: number;
    rot_damping: number;
    pos_damping: number;
    compliance: number;
};

const BodyChainDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [3, 4],
    cameraLookAt: new THREE.Vector3(0, 3.0, 0),
}

class BodyChainDemo implements Demo<BodyChainSimulation, BodyChainDemoProps> {
    sim: BodyChainSimulation;
    scene: Scene3D;
    props: BodyChainDemoProps;

    private grabber: Grabber;
    private boxMeshes: THREE.Mesh[];
    private boxPoses: Float32Array;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.sim = new rust_wasm.BodyChainSimulation(DEFAULT_NUM_OBJECTS, DEFAULT_OBJECT_SIZE.toArray(), DEFAULT_LAST_OBJECT_SIZE.toArray(), DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_ROT_DAMPING, DEFAULT_POS_DAMPING, DEFAULT_COMPLIANCE);
        this.scene = scene;
        this.initControls(folder, canvas);

        this.boxMeshes = [];
    }

    init() {
        this.initMeshes();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMeshes();
            this.grabber.increaseTime(this.sim.dt);
        }
    }

    reset() {
        this.sim.reset(DEFAULT_OBJECT_SIZE.toArray() as unknown as Float32Array, DEFAULT_LAST_OBJECT_SIZE.toArray() as unknown as Float32Array);
        this.updateMeshes();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        let animateController: Controller;
        this.props = {
            bodies: this.sim.num_objects,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            rot_damping: DEFAULT_ROT_DAMPING,
            pos_damping: DEFAULT_POS_DAMPING,
            compliance: DEFAULT_COMPLIANCE,
            animate: true,
        };
        folder.add(this.props, 'bodies').disable();
        folder.add(this.props, 'substeps').min(1).max(100).step(1).onChange((v: number) => (this.sim.num_substeps = v));
        folder.add(this.props, 'rot_damping').name('rotation damping').min(0).max(2000).step(10).onChange((v: number) => (this.sim.pos_damping = v));
        folder.add(this.props, 'pos_damping').name('position damping').min(0).max(2000).step(10).onChange((v: number) => (this.sim.rot_damping = v));
        folder.add(this.props, 'compliance').min(0).max(0.25).step(0.05).onChange((v: number) => (this.sim.compliance = v));
        animateController = folder.add(this.props, 'animate');

        // grab interaction handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initMeshes() {
        for (let id = 0; id < this.sim.num_objects; id++) {
            let boxMesh = new THREE.Mesh(new THREE.BoxGeometry(), new THREE.MeshPhongMaterial({ color: 0xf78a1d }));
            let size = DEFAULT_OBJECT_SIZE;
            if (id === this.sim.num_objects - 1) {
                size = DEFAULT_LAST_OBJECT_SIZE;
            }
            boxMesh.scale.set(size.x, size.y, size.z);

            boxMesh.userData = { id }; // for raycasting
            boxMesh.layers.enable(1);
            boxMesh.castShadow = true;
            boxMesh.receiveShadow = true;
            this.scene.scene.add(boxMesh);
            this.boxMeshes.push(boxMesh);
        }

        // Here, we store the pointer to the positions buffer location after the simulation is
        // initialized (all allocations are completed). In the WASM linear heap, it will be constant 
        // thereafter, so we don't need to refresh the pointer moving forward.
        const posesPtr = this.sim.poses;
        this.boxPoses = new Float32Array(memory.buffer, posesPtr, this.sim.num_objects * 7);
    }

    private updateMeshes() {
        for (let id = 0; id < this.boxMeshes.length; id++) {
            // mapped to WASM memory, see comment in `initMeshes`
            const p = this.boxPoses.slice(id * 7, id * 7 + 3);
            const q = this.boxPoses.slice(id * 7 + 3, id * 7 + 7);
            this.boxMeshes[id].position.set(p[0], p[1], p[2]);
            this.boxMeshes[id].quaternion.set(q[0], q[1], q[2], q[3]);
        }
    }
}

export { BodyChainDemo, BodyChainDemoConfig };
