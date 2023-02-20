import * as Comlink from 'comlink';
import * as Stats from 'stats.js';
//import { memory } from '../pkg-parallel/parallel_bg.wasm';
import { ParallelClothSimulation } from '../pkg-parallel';
import { Scene3D, Scene3DConfig } from './lib';
import { initThreeScene } from './lib';
import * as THREE from 'three';
import GUI from 'lil-gui';

const DEFAULT_NUM_SOLVER_SUBSTEPS = 30;

export type HandlersWrap = {
    handlers: Handlers;
}

export type Handlers = {
    demo: ParallelClothDemoInternal;
    numThreads: number;
    init: (canvas: OffscreenCanvas, canvasElement: HTMLElement, config: Scene3DConfig, folder: GUI, stats: Stats, simPanel: Stats.Panel) => void;
    reset: () => void;
};

type ParallelClothDemoInternalProps = {
    triangles: number;
    vertices: number;
    animate: boolean;
    showEdges: boolean;
    substeps: number;
};

class ParallelClothDemoInternal {
    sim: ParallelClothSimulation;
    scene: Scene3D;
    props: ParallelClothDemoInternalProps;

    //private grabber: Grabber;
    private triMesh: THREE.Mesh;
    private points: THREE.Points;
    private sphereMesh: THREE.Mesh;
    private positions: Float32Array; // mapped to WASM memory

    constructor(rust_wasm: any, canvas: OffscreenCanvas, canvasElement: HTMLElement, config: Scene3DConfig, folder: GUI) {
        this.scene = initThreeScene(canvas, canvasElement, config);

        this.sim = new rust_wasm.ParallelClothSimulation(30, 256, 256); // LVSTODO pass params
        this.initControls(folder, canvas);
    }

    async init() {
        this.initMesh();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMesh();
            //this.grabber.increaseTime(this.sim.dt);
        }
    }

    reset() {
        this.sim.reset();
        this.updateMesh();
    }

    private initMesh() {
        const tri_ids = Array.from(this.sim.tri_ids);

        // NOTE: ordering matters here. The above sim.*_ids getters are lazily implemented and 
        // allocate into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions could change. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to refresh the pointer
        // moving forward.
        //const positionsPtr = this.sim.pos;
        //this.positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_particles * 3);
        this.positions = new Float32Array(this.sim.num_particles * 3);

        // visual tri mesh
        let geometry = new THREE.BufferGeometry();
        let positionAttrib = new THREE.BufferAttribute(this.positions, 3);
        geometry.setAttribute('position', positionAttrib);
        geometry.setIndex(tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.DoubleSide });
        this.triMesh = new THREE.Mesh(geometry, visMaterial);
        this.triMesh.castShadow = true;
        this.triMesh.visible = false; // LVSTODO: testing
        this.triMesh.layers.enable(1);
        this.scene.scene.add(this.triMesh);
        geometry.computeBoundingSphere();

        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', positionAttrib);
        const pointsMaterial = new THREE.PointsMaterial({ color: 0x0000ff, size: 0.015 });
        this.points = new THREE.Points(geometry, pointsMaterial);
        this.points.castShadow = false;
        this.points.visible = true; // LVSTODO: testing
        this.scene.scene.add(this.points);

        geometry = new THREE.SphereGeometry(this.sim.obstacle_radius);
        const obstacleMaterial = new THREE.MeshBasicMaterial({ color: 0x00ff00, side: THREE.FrontSide });
        this.sphereMesh = new THREE.Mesh(geometry, obstacleMaterial);
        this.sphereMesh.castShadow = true;
        this.sphereMesh.receiveShadow = true;
        this.scene.scene.add(this.sphereMesh);
        geometry.computeVertexNormals();

        this.updateMesh();
    }

    private updateMesh() {
        this.sphereMesh.position.fromArray(this.sim.obstacle_pos);

        //this.triMesh.geometry.computeVertexNormals();
        this.positions = this.sim.get_pos_copy();
        (this.triMesh.geometry.attributes.position as THREE.BufferAttribute).copyArray(this.positions);
        this.triMesh.geometry.attributes.position.needsUpdate = true;
        this.triMesh.geometry.computeBoundingSphere();
        this.points.geometry.attributes.position.needsUpdate = true;
    }

    private initControls(folder: GUI, _canvas: OffscreenCanvas) {
        console.log(folder);

        this.props = {
            triangles: this.sim.num_tris,
            vertices: this.sim.num_particles,
            animate: true,
            showEdges: false,
            substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
        };
        folder.add(this.props, 'triangles'); // disable doesnt work for some reason?
        folder.add(this.props, 'vertices');

        /*
        //folder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => (this.sim.solver_substeps = v));
        folder.add(this.props, 'showEdges').name('show edges').onChange((s: boolean) => {
            this.points.visible = s;
            this.triMesh.visible = !s;
        });
        const animateController = folder.add(this.props, 'animate');
        */

        // grab interaction handler
        //this.grabber = new Grabber(this.sim, canvas, this.scene, this.props, animateController);
    }
}

const initHandlers = async (): Promise<Handlers> => {
    const rust_wasm = await import('../pkg-parallel');
    let maxSimMs = 0;

    // must be included to init rayon thread pool with web workers
    const numThreads = navigator.hardwareConcurrency;
    await rust_wasm.default();
    await rust_wasm.initThreadPool(numThreads);

    return Comlink.proxy({
        demo: null,
        numThreads: numThreads,
        init(canvas: OffscreenCanvas, canvasElement: HTMLElement, config: Scene3DConfig, folder: GUI, stats: Stats, simPanel: Stats.Panel) {
            this.demo = new ParallelClothDemoInternal(rust_wasm, canvas, canvasElement, config, folder);
            this.demo.init();


            const step = () => {
                stats.begin(); // collect perf data for stats.js
                let simTimeMs = performance.now();
                this.demo.update();
                simTimeMs = performance.now() - simTimeMs;
                this.demo.scene.renderer.render(this.demo.scene.scene, this.demo.scene.camera);
                simPanel.update(simTimeMs, (maxSimMs = Math.max(maxSimMs, simTimeMs)));
                stats.end();
                requestAnimationFrame(step);
            }
            requestAnimationFrame(step);
        },
        reset() {
            this.sim.reset();
        }
    });
};

Comlink.expose({ handlers: initHandlers() });