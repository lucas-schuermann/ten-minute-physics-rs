import GUI from 'lil-gui';
import * as THREE from 'three';

import { SelfCollisionSimulation } from '../pkg';
import { Demo, Scene3D, Scene3DConfig, Grabber, enumToValueList } from './lib';

const DEFAULT_NUM_SOLVER_SUBSTEPS = 10;
const DEFAULT_BENDING_COMPLIANCE = 1.0;
const DEFAULT_STRETCH_COMPLIANCE = 0.0;
const DEFAULT_SHEAR_COMPLIANCE = 0.0001;
const DEFAULT_FRICTION = 0.0;

enum SceneType {
    Freefall,
    PinnedTop,
}
const DEFAULT_SCENE = SceneType.Freefall;

type SelfCollisionDemoProps = {
    scene: string; // enum string value
    triangles: number;
    vertices: number;
    animate: boolean;
    handleCollisions: boolean;
    showEdges: boolean;
    substeps: number;
    bendingCompliance: number;
    stretchCompliance: number;
    shearCompliance: number;
    friction: number;
};

const SelfCollisionDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [0.3, 0.5],
    cameraLookAt: new THREE.Vector3(0, 0.1, 0),
}

class SelfCollisionDemo implements Demo<SelfCollisionSimulation, SelfCollisionDemoProps> {
    sim: SelfCollisionSimulation;
    scene: Scene3D;
    props: SelfCollisionDemoProps;

    private memory: WebAssembly.Memory;
    private grabber: Grabber;
    private edgeMesh: THREE.LineSegments;
    private frontMesh: THREE.Mesh;
    private backMesh: THREE.Mesh;
    private positions: Float32Array; // mapped to WASM memory

    constructor(rust_wasm: any, memory: WebAssembly.Memory, canvas: HTMLCanvasElement, scene: Scene3D, folder: GUI) {
        this.memory = memory;
        this.sim = new rust_wasm.SelfCollisionSimulation(DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_BENDING_COMPLIANCE, DEFAULT_STRETCH_COMPLIANCE, DEFAULT_SHEAR_COMPLIANCE, DEFAULT_FRICTION);
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
        this.sim.reset(this.props.scene === SceneType[SceneType.PinnedTop]);
        this.updateMesh();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            scene: SceneType[DEFAULT_SCENE],
            triangles: this.sim.num_tris,
            vertices: this.sim.num_particles,
            animate: true,
            handleCollisions: true,
            showEdges: false,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            bendingCompliance: DEFAULT_BENDING_COMPLIANCE,
            stretchCompliance: DEFAULT_STRETCH_COMPLIANCE,
            shearCompliance: DEFAULT_SHEAR_COMPLIANCE,
            friction: DEFAULT_FRICTION,
        };
        folder.add(this.props, 'scene', enumToValueList(SceneType)).onChange(() => this.reset());
        folder.add(this.props, 'triangles').disable();
        folder.add(this.props, 'vertices').disable();
        folder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => (this.sim.solver_substeps = v));
        folder.add(this.props, 'bendingCompliance').name('bend compliance').min(0).max(10).step(0.1).onChange((v: number) => (this.sim.bending_compliance = v));
        folder.add(this.props, 'stretchCompliance').name('stretch compliance').min(0).max(1).step(0.01).onChange((v: number) => (this.sim.stretch_compliance = v));
        folder.add(this.props, 'shearCompliance').name('shear compliance').min(0).max(1).step(0.01).onChange((v: number) => (this.sim.shear_compliance = v));
        folder.add(this.props, 'friction').min(0).max(0.2).step(0.002).onChange((v: number) => (this.sim.friction = v));
        folder.add(this.props, 'handleCollisions').name('handle collisions').onChange((v: boolean) => {
            this.sim.handle_collisions = v;
            this.reset();
        });
        folder.add(this.props, 'showEdges').name('show edges').onChange((s: boolean) => {
            this.edgeMesh.visible = s;
            this.frontMesh.visible = !s;
            this.backMesh.visible = !s;
        });
        const animateController = folder.add(this.props, 'animate');

        // grab handler
        this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }

    private initMesh() {
        const triIds = Array.from(this.sim.tri_ids);
        const edgeIds = Array.from(this.sim.edge_ids);

        // NOTE: ordering matters here. The above sim.*_ids getters are lazily implemented and 
        // allocate into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions could change. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to refresh the pointer
        // moving forward.
        const positionsPtr = this.sim.pos;
        this.positions = new Float32Array(this.memory.buffer, positionsPtr, this.sim.num_particles * 3);

        // visual edge mesh
        let geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(edgeIds);
        const lineMaterial = new THREE.LineBasicMaterial({ color: 0xff0000, linewidth: 2 });
        this.edgeMesh = new THREE.LineSegments(geometry, lineMaterial);
        this.edgeMesh.visible = false;
        this.scene.scene.add(this.edgeMesh);

        // visual tri mesh
        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(triIds);
        const frontMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.FrontSide });
        this.frontMesh = new THREE.Mesh(geometry, frontMaterial);
        this.frontMesh.castShadow = true;
        this.frontMesh.layers.enable(1);
        this.scene.scene.add(this.frontMesh);
        const backMaterial = new THREE.MeshPhongMaterial({ color: 0xff8000, side: THREE.BackSide });
        this.backMesh = new THREE.Mesh(geometry, backMaterial);
        this.backMesh.castShadow = true;
        this.backMesh.layers.enable(1);
        this.scene.scene.add(this.backMesh);
        geometry.computeVertexNormals();
        geometry.computeBoundingSphere()

        this.updateMesh();
    }

    private updateMesh() {
        this.frontMesh.geometry.attributes.position.needsUpdate = true;
        this.edgeMesh.geometry.attributes.position.needsUpdate = true;
        if (!this.props.showEdges) {
            this.frontMesh.geometry.computeVertexNormals();
        }
        this.frontMesh.geometry.computeBoundingSphere();
    }
}

export { SelfCollisionDemo, SelfCollisionDemoConfig };
