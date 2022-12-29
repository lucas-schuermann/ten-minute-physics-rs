import GUI from 'lil-gui';
import * as THREE from 'three';

import { ClothSimulation } from '../pkg';
import { memory } from '../pkg/index_bg.wasm';
import { Demo, Scene, SceneConfig, Grabber } from './lib';

const DEFAULT_NUM_SOLVER_SUBSTEPS = 15;
const DEFAULT_BENDING_COMPLIANCE = 1.0;
const DEFAULT_STRETCHING_COMPLIANCE = 0.0;

type ClothDemoProps = {
    triangles: number;
    vertices: number;
    animate: boolean;
    showEdges: boolean;
    substeps: number;
    bendingCompliance: number;
    stretchingCompliance: number;
};

const ClothDemoConfig: SceneConfig = {
    cameraYZ: [1, 1],
    cameraLookAt: new THREE.Vector3(0, 0.6, 0),
}

class ClothDemo implements Demo<ClothSimulation, ClothDemoProps> {
    sim: ClothSimulation;
    scene: Scene;
    props: ClothDemoProps;

    private grabber: Grabber;
    private edgeMesh: THREE.LineSegments;
    private triMesh: THREE.Mesh;
    private positions: Float32Array;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene, folder: GUI) {
        this.sim = new rust_wasm.ClothSimulation(DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_BENDING_COMPLIANCE, DEFAULT_STRETCHING_COMPLIANCE);
        this.scene = scene;
        this.initControls(folder, canvas);
    }

    init() {
        this.initMesh();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMesh();
            this.grabber.increaseTime(this.sim.dt());
        }
    }

    reset() {
        this.sim.reset();
        this.updateMesh();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            triangles: this.sim.num_tris(),
            vertices: this.sim.num_particles(),
            animate: true,
            showEdges: false,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            bendingCompliance: DEFAULT_BENDING_COMPLIANCE,
            stretchingCompliance: DEFAULT_STRETCHING_COMPLIANCE,
        };
        folder.add(this.props, 'triangles').disable();
        folder.add(this.props, 'vertices').disable();
        folder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => this.sim.set_solver_substeps(v));
        folder.add(this.props, 'bendingCompliance').name('bend compliance').min(0).max(10).step(0.1).onChange((v: number) => this.sim.set_bending_compliance(v));
        folder.add(this.props, 'stretchingCompliance').name('stretch compliance').min(0).max(1).step(0.01).onChange((v: number) => this.sim.set_stretching_compliance(v));
        folder.add(this.props, 'showEdges').name('show edges').onChange((s: boolean) => {
            this.edgeMesh.visible = s;
            this.triMesh.visible = !s;
        });
        const animateController = folder.add(this.props, 'animate');

        // grab interaction handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initMesh() {
        const tri_ids = Array.from(this.sim.mesh_tri_ids());
        const edge_ids = Array.from(this.sim.mesh_edge_ids());

        // NOTE: ordering matters here. The above sim.*_ids() getter methods are lazily implemented and 
        // allocate into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions could change. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to touch the array moving 
        // forward.
        const positionsPtr = this.sim.particle_positions_ptr();
        this.positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_particles() * 3);

        // edge mesh
        let geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(edge_ids);
        const lineMaterial = new THREE.LineBasicMaterial({ color: 0xff0000, linewidth: 2 });
        this.edgeMesh = new THREE.LineSegments(geometry, lineMaterial);
        this.edgeMesh.visible = false;
        this.scene.scene.add(this.edgeMesh);

        // visual tri mesh
        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.DoubleSide });
        this.triMesh = new THREE.Mesh(geometry, visMaterial);
        this.triMesh.castShadow = true;
        this.triMesh.layers.enable(1);
        this.scene.scene.add(this.triMesh);
        geometry.computeVertexNormals();

        this.updateMesh();
    }

    private updateMesh() {
        this.triMesh.geometry.computeVertexNormals();
        this.triMesh.geometry.attributes.position.needsUpdate = true;
        this.triMesh.geometry.computeBoundingSphere();
        this.edgeMesh.geometry.attributes.position.needsUpdate = true;
    }
}

export { ClothDemo, ClothDemoConfig };
