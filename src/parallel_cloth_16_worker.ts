import * as Comlink from 'comlink';
import * as Stats from 'stats.js';
import { ParallelClothSimulation, SolverKind } from '../pkg-parallel';
import { Grabber, resizeThreeScene, Scene3D, Scene3DConfig } from './lib';
import { initThreeScene } from './lib';
import * as THREE from 'three';
import { EventDispatcher } from 'three';
import { Controller } from 'lil-gui';
import './parallel_cloth_16_transfer'; // must be included to extend Comlink transfer to events


const DEFAULT_NUM_SOLVER_SUBSTEPS = 30;
const DEFAULT_CLOTH_NUM_VERTICES_WIDTH = 256;
const DEFAULT_CLOTH_NUM_VERTICES_HEIGHT = 256;
const PARTICLE_POINT_SIZE = 0.01;

type ParallelClothDemoWorkerProps = {
    threads: number;
    triangles: number;
    vertices: number;
    constraints: number;
    animate: boolean;
    showVertices: boolean;
    solver: string; // enum string value
    substeps: number;
};

const noop = () => { };

// See comment in `parallel_cloth_16_transfer.ts`. Essentially, we need a way to allow
// event subscriptions in our worker context for the OrbitControls and Grabber that
// we define in ParallelClothDemoWorker. Here, we create an intermediate class
// which is extended by ParallelClothDemoWorker and itself extends EventDispatcher
// allowing the worker to be bound to main thread event listeners and allowing `this` 
// to be passed as the input element to OrbitControls and Grabber. In addition to 
// implementing EventDispatcher, a few properties and methods from HTMLElement are 
// necessary. See `parallel_cloth_16.ts` for the window event bindings, such as 
// `onmousedown`.
export class ProxiedHTMLElement extends EventDispatcher {
    style = {};
    width: number;
    height: number;
    top: number;
    left: number;

    constructor() {
        super();
    }

    handleEvent(e: Event) {
        e.preventDefault = noop;
        e.stopPropagation = noop;
        this.dispatchEvent(e);
    }

    setSize(left: number, top: number, width: number, height: number) {
        this.left = left;
        this.top = top;
        this.width = width;
        this.height = height;
    }

    get clientWidth() {
        return this.width;
    }

    get clientHeight() {
        return this.height;
    }

    getBoundingClientRect() {
        return {
            left: this.left,
            top: this.top,
            width: this.width,
            height: this.height,
            right: this.left + this.width,
            bottom: this.top + this.height,
        };
    }

    // Used by OrbitControls--could implement in the future
    setPointerCapture() { }
    releasePointerCapture() { }
}

// Exported using Comlink to define a Web Worker, which allows us to take advantage
// of threading for our multi-threaded parallel cloth solver.
// See https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API
export class ParallelClothDemoWorker extends ProxiedHTMLElement {
    sim: ParallelClothSimulation;
    scene: Scene3D;
    props: ParallelClothDemoWorkerProps;

    private freeFlag = false;
    private maxSimMs = 0;
    private grabber: Grabber;
    private triMesh: THREE.Mesh;
    private points: THREE.Points;
    private sphereMesh: THREE.Mesh;
    private positions: Float32Array; // mapped to WASM memory
    private stats: Stats;
    private simPanel: Stats.Panel;
    private wasmMemory: WebAssembly.Memory; // store reference returned from module.default()

    constructor(canvas: OffscreenCanvas, config: Scene3DConfig, stats: Stats, simPanel: Stats.Panel, devicePixelRatio: number) {
        super();

        // for `this` cast, see comment on ProxiedHTMLElement
        this.scene = initThreeScene(canvas, this as unknown as HTMLElement, config, devicePixelRatio);
        this.stats = stats;
        this.simPanel = simPanel;
    }

    async init() {
        const rust_wasm = await import('../pkg-parallel');

        // must be included to init rayon thread pool with web workers
        const numThreads = navigator.hardwareConcurrency;
        const { memory } = await rust_wasm.default();
        this.wasmMemory = memory;
        await rust_wasm.initThreadPool(numThreads);

        this.sim = new rust_wasm.ParallelClothSimulation(DEFAULT_NUM_SOLVER_SUBSTEPS, DEFAULT_CLOTH_NUM_VERTICES_WIDTH, DEFAULT_CLOTH_NUM_VERTICES_HEIGHT);

        this.props = {
            threads: numThreads,
            triangles: this.sim.num_tris,
            vertices: this.sim.num_particles,
            constraints: this.sim.num_dist_constraints,
            animate: true,
            showVertices: false,
            solver: SolverKind[this.sim.solver_kind],
            substeps: this.sim.num_substeps,
        };

        this.initMesh();
    }

    beginLoop(animateController: Controller) {
        // Grab interaction handler. For `this` cast, see comment on ProxiedHTMLElement
        this.grabber = new Grabber(this.sim, this as unknown as HTMLElement, this.scene, this.props, animateController);

        // Main loop on the worker thread. Execute until `free` is called to set `this.freeFlag`.
        const step = () => {
            this.stats.begin(); // collect perf data for stats.js
            let simTimeMs = performance.now();
            this.update();
            simTimeMs = performance.now() - simTimeMs;
            this.scene.renderer.render(this.scene.scene, this.scene.camera);
            this.simPanel.update(simTimeMs, (this.maxSimMs = Math.max(this.maxSimMs, simTimeMs)));
            this.stats.end();
            if (!this.freeFlag) {
                requestAnimationFrame(step);
            } else {
                delete this.sim;
                return;
            }
        }
        requestAnimationFrame(step);
    }

    free() {
        this.freeFlag = true;
    }

    // GUI in main thread cannot directly access `this.props`, so we provide helper methods
    // to set properties via messages through Comlink
    showVertices(s: boolean) {
        this.points.visible = s;
        this.triMesh.visible = !s;
    }
    setSubsteps(n: number) {
        this.sim.num_substeps = n;
    }
    setAnimate(b: boolean) {
        this.props.animate = b;
    }
    setSolver(s: string) {
        this.sim.solver_kind = Object.values(SolverKind).indexOf(s);
    }

    resize(width: number, height: number) {
        resizeThreeScene(this.scene, width, height, false);
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMesh(); // TODO: might want to move this out of the simulate timings?
            this.grabber.increaseTime(this.sim.dt);
        }
    }

    reset() {
        this.sim.reset();
        this.updateMesh();
    }

    private initMesh() {
        const tri_ids = Array.from(this.sim.tri_ids);

        // NOTE: ordering matters here. The above sim.*_ids getter is lazily implemented and 
        // allocates into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions could change. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to refresh the pointer
        // moving forward.
        const positionsPtr = this.sim.pos;
        this.positions = new Float32Array(this.wasmMemory.buffer, positionsPtr, this.sim.num_particles * 3);

        // visual tri mesh
        let geometry = new THREE.BufferGeometry();
        let positionAttrib = new THREE.BufferAttribute(this.positions, 3); // vertex positions shared by both mesh and points
        geometry.setAttribute('position', positionAttrib);
        geometry.setIndex(tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.DoubleSide });
        this.triMesh = new THREE.Mesh(geometry, visMaterial);
        this.triMesh.castShadow = true;
        this.triMesh.layers.enable(1);
        this.scene.scene.add(this.triMesh);
        geometry.computeBoundingSphere();

        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', positionAttrib);
        const pointsMaterial = new THREE.PointsMaterial({ color: 0xff0000, size: PARTICLE_POINT_SIZE });
        this.points = new THREE.Points(geometry, pointsMaterial);
        this.points.castShadow = false;
        this.points.visible = false;
        this.scene.scene.add(this.points);

        geometry = new THREE.SphereGeometry(this.sim.obstacle_radius);
        const obstacleMaterial = new THREE.MeshPhongMaterial({ color: 0xf78a1d, side: THREE.FrontSide });
        this.sphereMesh = new THREE.Mesh(geometry, obstacleMaterial);
        this.sphereMesh.castShadow = true;
        this.sphereMesh.receiveShadow = true;
        this.sphereMesh.position.fromArray(this.sim.obstacle_pos); // static, so only need to set upon init
        this.scene.scene.add(this.sphereMesh);

        this.updateMesh();
    }

    private updateMesh() {
        this.triMesh.geometry.attributes.position.needsUpdate = true;
        this.points.geometry.attributes.position.needsUpdate = true;
        this.triMesh.geometry.computeVertexNormals(); // LVSTODO: here and elsewhere, maybe don't need to if we are displaying edges/vertices?
        this.triMesh.geometry.computeBoundingSphere();
    }
}

Comlink.expose(ParallelClothDemoWorker);