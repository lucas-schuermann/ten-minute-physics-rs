import GUI, { Controller } from 'lil-gui';
import * as THREE from 'three';

import { SoftBodiesSimulation } from '../pkg';
import { memory } from '../pkg/index_bg.wasm';
import { Demo, Scene3D, Scene3DConfig, Grabber } from './lib';

const DEFAULT_NUM_SOLVER_SUBSTEPS = 10;
const DEFAULT_EDGE_COMPLIANCE = 100.0;
const DEFAULT_VOL_COMPLIANCE = 0.0;

type SoftBodiesDemoProps = {
    tets: number;
    animate: boolean;
    substeps: number;
    volumeCompliance: number;
    edgeCompliance: number;
    squash: () => void;
    addBody: () => void;
};

const SoftBodiesDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [1, 2],
    cameraLookAt: new THREE.Vector3(0, 0, 0),
}

class SoftBodiesDemo implements Demo<SoftBodiesSimulation, SoftBodiesDemoProps> {
    sim: SoftBodiesSimulation;
    scene: Scene3D;
    props: SoftBodiesDemoProps;

    private tetsController: Controller;
    private grabber: Grabber;
    private surfaceMeshes: THREE.Mesh[];

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.sim = new rust_wasm.SoftBodiesSimulation(DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_EDGE_COMPLIANCE, DEFAULT_VOL_COMPLIANCE);
        this.scene = scene;
        this.surfaceMeshes = [];
        this.initControls(folder, canvas);
    }

    init() {
        this.initMesh();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMeshes();
            this.grabber.increaseTime(this.sim.dt());
        }
    }

    reset() {
        this.sim.reset();
        this.surfaceMeshes.forEach(mesh => {
            this.scene.scene.remove(mesh);
        });
        this.surfaceMeshes = [];
        this.initMesh();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        let animateController: Controller;
        this.props = {
            tets: this.sim.num_tets(),
            animate: true,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            volumeCompliance: DEFAULT_VOL_COMPLIANCE,
            edgeCompliance: DEFAULT_EDGE_COMPLIANCE,
            squash: () => {
                this.sim.squash();
                this.props.animate = false;
                animateController.updateDisplay();
                this.updateMeshes();
            },
            addBody: () => {
                this.sim.add_body();
                this.initMesh();
            },
        };
        this.tetsController = folder.add(this.props, 'tets').name('tetrahedra').disable();
        folder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => this.sim.set_solver_substeps(v));
        folder.add(this.props, 'volumeCompliance').name('volume compliance').min(0).max(500).step(5).onChange((v: number) => this.sim.set_volume_compliance(v));
        folder.add(this.props, 'edgeCompliance').name('edge compliance').min(0).max(500).step(5).onChange((v: number) => this.sim.set_edge_compliance(v));
        animateController = folder.add(this.props, 'animate');
        folder.add(this.props, 'squash');
        folder.add(this.props, 'addBody').name('add body');

        // grab interaction handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initMesh() {
        const surface_tri_ids = Array.from(this.sim.surface_tri_ids());
        const id = this.surfaceMeshes.length;

        // NOTE: since additional bodies can be created in this demo, we do not rely on the `positionsPtr` to
        // be a constant offset in the WASM linear heap. The position geometry attribute is rewritten each
        // step in `updateMesh` to point to the refeteched `particle_positions_ptr(id)`. There are probably
        // more efficient ways to do this, but a simple implementation works for this demo.
        const positionsPtr = this.sim.particle_positions_ptr(id);
        const positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_particles_per_body() * 3);

        // visual tri mesh
        const geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        geometry.setIndex(surface_tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xF02000, side: THREE.DoubleSide });
        visMaterial.flatShading = true;
        const surfaceMesh = new THREE.Mesh(geometry, visMaterial);
        surfaceMesh.castShadow = true;
        surfaceMesh.layers.enable(1);
        surfaceMesh.userData = { 'id': id }; // for raycasting
        this.scene.scene.add(surfaceMesh);
        this.surfaceMeshes.push(surfaceMesh);
        geometry.computeVertexNormals();

        this.props.tets = this.sim.num_tets();
        this.tetsController.updateDisplay();

        this.updateMesh(id);
    }

    private updateMesh(id: number) {
        const positionsPtr = this.sim.particle_positions_ptr(id);
        const positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_particles_per_body() * 3);
        this.surfaceMeshes[id].geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
        this.surfaceMeshes[id].geometry.computeVertexNormals();
        this.surfaceMeshes[id].geometry.attributes.position.needsUpdate = true;
        this.surfaceMeshes[id].geometry.computeBoundingSphere();
    }

    private updateMeshes() {
        for (let id = 0; id < this.surfaceMeshes.length; id++) {
            this.updateMesh(id);
        }
    }
}

export { SoftBodiesDemo, SoftBodiesDemoConfig };
