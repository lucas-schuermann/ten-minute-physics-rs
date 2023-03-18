import * as THREE from 'three';
import * as Comlink from 'comlink';
import GUI, { Controller } from 'lil-gui';
import { ParallelClothDemoWorker } from './parallel_cloth_16_worker';
import { SolverKind } from '../pkg-parallel';
import { Demo, enumToValueList, Scene3D, Scene3DConfig } from './lib';
import './parallel_cloth_16_transfer'; // must be included to extend Comlink transfer to events

// defined in worker
type ParallelClothDemoProps = {};

const ParallelClothDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [1.0, 3.0],
    cameraLookAt: new THREE.Vector3(0, 1.0, 0), // should match `DEFAULT_OBSTACLE_POS`
    offscreen: true,
}

class ParallelClothDemo implements Demo<any, ParallelClothDemoProps> {
    sim: Comlink.Remote<ParallelClothDemoWorker>; // reference to demo worker thread
    scene: Scene3D;
    props: ParallelClothDemoProps;

    private canvas: OffscreenCanvas;
    private inputElement: HTMLElement;
    private config: Scene3DConfig;
    private folder: GUI;
    private stats: Stats;
    private simPanel: Stats.Panel;

    constructor(_rust_wasm: any, canvas: OffscreenCanvas, inputElement: HTMLElement, config: Scene3DConfig, folder: GUI, stats: Stats, simPanel: Stats.Panel) {
        this.canvas = canvas;
        this.inputElement = inputElement;
        this.config = config;
        this.folder = folder;
        this.stats = stats;
        this.simPanel = simPanel;
    }

    async init() {
        // create WASM web worker
        const RemoteParallelClothDemoWorker = await Comlink.wrap<typeof ParallelClothDemoWorker>(
            new Worker(new URL('./parallel_cloth_16_worker', import.meta.url), {
                type: 'module'
            })
        );

        // launch WASM web worker
        this.sim = await new RemoteParallelClothDemoWorker(Comlink.transfer(this.canvas, [this.canvas]), this.config, Comlink.proxy(this.stats), Comlink.proxy(this.simPanel), window.devicePixelRatio);
        await this.sim.init();
        const animateController = await this.initControls(this.folder);

        // used by OrbitControls
        this.inputElement.addEventListener("pointerdown", this.sim.handleEvent.bind(this.sim));
        this.inputElement.addEventListener("pointermove", this.sim.handleEvent.bind(this.sim));
        this.inputElement.addEventListener("pointerup", this.sim.handleEvent.bind(this.sim));
        this.inputElement.addEventListener("wheel", this.sim.handleEvent.bind(this.sim), { passive: false });
        this.inputElement.addEventListener("contextmenu", e => e.preventDefault());
        // used by Grabber
        this.inputElement.addEventListener("mousedown", this.sim.handleEvent.bind(this.sim));
        this.inputElement.addEventListener("mousemove", this.sim.handleEvent.bind(this.sim));
        this.inputElement.addEventListener("mouseup", this.sim.handleEvent.bind(this.sim));

        // LVSTODO: handle resizing
        const rect = this.inputElement.getBoundingClientRect();
        this.sim.setSize(rect.left, rect.top, this.inputElement.clientWidth, this.inputElement.clientHeight);

        // enter main loop on worker thread
        this.sim.beginLoop(Comlink.proxy(animateController));
    }

    private async initControls(folder: GUI): Promise<Controller> {
        // does not support two-way binding, so we use helper methods,
        // e.g. `this.sim.setSubsteps`
        const props = await this.sim.props;
        folder.add(props, 'threads').disable();
        folder.add(props, 'triangles').disable();
        folder.add(props, 'vertices').disable();
        folder.add(props, 'constraints').disable();
        folder.add(props, 'solver', enumToValueList(SolverKind)).name('solver').onChange((s: string) => { this.sim.setSolver(s) });
        folder.add(props, 'substeps').min(20).max(40).step(1).onChange((v: number) => { this.sim.setSubsteps(v) });
        folder.add(props, 'showVertices').name('show vertices').onChange((s: boolean) => { this.sim.showVertices(s) });
        return folder.add(props, 'animate').onChange((a: boolean) => { this.sim.setAnimate(a) });
    }

    // noop since main loop is in worker
    update() { }

    async reset() {
        await this.sim.reset();
    }

    async free() {
        await this.sim.free();
    }

    async resize(width: number, height: number) {
        const rect = this.inputElement.getBoundingClientRect();
        this.sim.setSize(rect.left, rect.top, this.inputElement.clientWidth, this.inputElement.clientHeight);
        await this.sim.resize(width, height);
    }
}

export { ParallelClothDemo, ParallelClothDemoConfig };
