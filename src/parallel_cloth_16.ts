import * as THREE from 'three';
import * as Comlink from 'comlink';
//import { threads } from 'wasm-feature-detect';
//import * as Stats from 'stats.js';
import GUI from 'lil-gui';
import { Handlers, HandlersWrap } from './parallel_cloth_16_worker';

import { Demo, Scene3D, Scene3DConfig } from './lib';

type ParallelClothDemoProps = {
};

const ParallelClothDemoConfig: Scene3DConfig = {
    kind: '3D',
    cameraYZ: [1.0, 3.0],
    cameraLookAt: new THREE.Vector3(0, 1.5, 0),
    offscreen: true,
}

class ParallelClothDemo implements Demo<any, ParallelClothDemoProps> {
    sim: Handlers;
    scene: Scene3D;
    props: ParallelClothDemoProps;

    private canvas: OffscreenCanvas;
    private canvasElement: HTMLElement;
    private config: Scene3DConfig;
    private folder: GUI;
    private stats: Stats;
    private simPanel: Stats.Panel;

    constructor(_rust_wasm: any, canvas: OffscreenCanvas, canvasElement: HTMLElement, config: Scene3DConfig, folder: GUI, stats: Stats, simPanel: Stats.Panel) {
        this.canvas = canvas;
        console.log("ctor canvas element", canvasElement);
        this.canvasElement = canvasElement;
        this.config = config;
        this.folder = folder;
        this.stats = stats;
        this.simPanel = simPanel;
    }

    async init() {
        // create WASM web worker and get handlers for interaction
        this.sim = await Comlink.wrap<HandlersWrap>(
            new Worker(new URL('./parallel_cloth_16_worker', import.meta.url), {
                type: 'module'
            })
        ).handlers;

        await this.sim.init(Comlink.transfer(this.canvas, [this.canvas]), Comlink.proxy(this.canvasElement), this.config, Comlink.proxy(this.folder), Comlink.proxy(this.stats), Comlink.proxy(this.simPanel));

        //this.initMesh();
    }

    update() {
    }

    async reset() {
        await this.sim.reset();
    }
}

export { ParallelClothDemo, ParallelClothDemoConfig };
