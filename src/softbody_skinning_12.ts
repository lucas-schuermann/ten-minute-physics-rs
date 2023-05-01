import GUI, { Controller } from 'lil-gui';
import * as THREE from 'three';

import { SkinnedSoftbodySimulation } from '../pkg';
import { Demo, Scene3D, Scene3DConfig, Grabber } from './lib';

const DEFAULT_NUM_SOLVER_SUBSTEPS = 10;
const DEFAULT_EDGE_COMPLIANCE = 0.0;
const DEFAULT_VOL_COMPLIANCE = 0.0;

type SkinnedSoftbodyDemoProps = {
    tets: number;
    triangles: number;
    vertices: number;
    animate: boolean;
    substeps: number;
    volumeCompliance: number;
    edgeCompliance: number;
    showTets: boolean;
    squash: () => void;
};

const SkinnedSoftbodyDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [1.5, 3],
    cameraLookAt: new THREE.Vector3(0, 0, 0),
}

class SkinnedSoftbodyDemo implements Demo<SkinnedSoftbodySimulation, SkinnedSoftbodyDemoProps> {
    sim: SkinnedSoftbodySimulation;
    scene: Scene3D;
    props: SkinnedSoftbodyDemoProps;

    private memory: WebAssembly.Memory;
    private grabber: Grabber;
    private tetMesh: THREE.LineSegments;
    private surfaceMesh: THREE.Mesh;
    private tetPositions: Float32Array; // mapped to WASM memory
    private surfacePositions: Float32Array; // mapped to WASM memory

    constructor(rust_wasm: any, memory: WebAssembly.Memory, canvas: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.memory = memory;
        this.sim = new rust_wasm.SkinnedSoftbodySimulation(DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_EDGE_COMPLIANCE, DEFAULT_VOL_COMPLIANCE);
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
            this.grabber.increaseTime(this.sim.dt);
        }
    }

    reset() {
        this.sim.reset();
        this.updateMesh();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        let animateController: Controller;
        this.props = {
            tets: this.sim.num_tets,
            triangles: this.sim.num_tris,
            vertices: this.sim.num_surface_verts,
            animate: true,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            volumeCompliance: DEFAULT_VOL_COMPLIANCE,
            edgeCompliance: DEFAULT_EDGE_COMPLIANCE,
            showTets: false,
            squash: () => {
                this.sim.squash();
                this.props.animate = false;
                animateController.updateDisplay();
                this.updateMesh();
            },
        };
        folder.add(this.props, 'tets').name('tetrahedra').disable();
        folder.add(this.props, 'triangles').disable();
        folder.add(this.props, 'vertices').disable();
        folder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => (this.sim.solver_substeps = v));
        folder.add(this.props, 'volumeCompliance').name('volume compliance').min(0).max(250).step(2.5).onChange((v: number) => (this.sim.vol_compliance = v));
        folder.add(this.props, 'edgeCompliance').name('edge compliance').min(0).max(100).step(1).onChange((v: number) => (this.sim.edge_compliance = v));
        folder.add(this.props, 'showTets').name('show tetrahedra').onChange((s: boolean) => {
            this.tetMesh.visible = s;
        });
        animateController = folder.add(this.props, 'animate');
        folder.add(this.props, 'squash');

        // grab interaction handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initMesh() {
        const tet_edge_ids = Array.from(this.sim.edge_ids);
        const surface_tri_ids = Array.from(this.sim.surface_tri_ids);

        // NOTE: ordering matters here. The above sim.*_ids getters are lazily implemented and 
        // allocate into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions could change. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to refresh the pointer
        // moving forward.
        const tetPositionsPtr = this.sim.pos;
        this.tetPositions = new Float32Array(this.memory.buffer, tetPositionsPtr, this.sim.num_particles * 3);
        const surfacePositionsPtr = this.sim.surface_pos;
        this.surfacePositions = new Float32Array(this.memory.buffer, surfacePositionsPtr, this.sim.num_surface_verts * 3);

        // visual tet mesh
        let geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.tetPositions, 3));
        geometry.setIndex(tet_edge_ids);
        const lineMaterial = new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 2 });
        this.tetMesh = new THREE.LineSegments(geometry, lineMaterial);
        this.tetMesh.visible = false;
        this.scene.scene.add(this.tetMesh);

        // visual tri mesh
        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.surfacePositions, 3));
        geometry.setIndex(surface_tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xf78a1d });
        this.surfaceMesh = new THREE.Mesh(geometry, visMaterial);
        this.surfaceMesh.castShadow = true;
        this.surfaceMesh.layers.enable(1);
        this.scene.scene.add(this.surfaceMesh);
        geometry.computeVertexNormals();
        geometry.computeBoundingSphere();

        this.updateMesh();
    }

    private updateMesh() {
        this.surfaceMesh.geometry.attributes.position.needsUpdate = true;
        this.tetMesh.geometry.attributes.position.needsUpdate = true;
        this.surfaceMesh.geometry.computeVertexNormals();
        this.surfaceMesh.geometry.computeBoundingSphere();
    }
}

export { SkinnedSoftbodyDemo, SkinnedSoftbodyDemoConfig };
