import GUI from 'lil-gui';
import * as THREE from 'three';

import { FireSimulation } from '../pkg';
import { Demo, Scene2DCanvas, Scene2DConfig } from './lib';

type FireDemoProps = {
    animate: boolean;
    numCells: number;
    numIters: number;
    overRelaxation: number;
    swirlProbability: number;
    showObstacle: boolean;
    showSwirls: boolean;
    burningObstacle: boolean;
    burningFloor: boolean;
};

const FireDemoConfig: Scene2DConfig = {
    kind: '2DCanvas',
}

class FireDemo implements Demo<FireSimulation, FireDemoProps> {
    sim: FireSimulation;
    scene: Scene2DCanvas;
    props: FireDemoProps;

    private rust_wasm: any;
    private mouseDown: boolean;
    private mouseOffset: THREE.Vector2;
    private imageData: ImageData;

    constructor(rust_wasm: any, _: WebAssembly.Memory, canvas: HTMLCanvasElement, scene: Scene2DCanvas, folder: GUI) {
        this.rust_wasm = rust_wasm;
        this.sim = new rust_wasm.FireSimulation(scene.width, scene.height, scene.context);
        this.scene = scene;
        this.initControls(folder, canvas);
        this.imageData = scene.context.createImageData(scene.width, scene.height);
    }

    init() {
        this.props.numCells = this.sim.num_cells;
        this.props.numIters = this.sim.num_iters;
        this.props.overRelaxation = this.sim.over_relaxation;
        this.props.swirlProbability = this.sim.swirl_probability;
        this.props.showObstacle = this.sim.show_obstacle;
        this.props.showSwirls = this.sim.show_swirls;
        this.props.burningObstacle = this.sim.burning_obstacle;
        this.props.burningFloor = this.sim.burning_floor;
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
        }
    }

    reset() {
        this.sim.free();
        this.sim = new this.rust_wasm.FireSimulation(this.scene.width, this.scene.height, this.scene.context);
        this.init();
    }

    draw() {
        this.sim.draw_buffer(this.imageData.data as unknown as Uint8Array); // wasm-bindgen doesn't support Uint8ClampedArray
        this.scene.context.putImageData(this.imageData, 0, 0);
        this.sim.draw_canvas();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            animate: true,
            numCells: this.sim.num_cells,
            numIters: this.sim.num_iters,
            overRelaxation: this.sim.over_relaxation,
            swirlProbability: this.sim.swirl_probability,
            showObstacle: this.sim.show_obstacle,
            showSwirls: this.sim.show_swirls,
            burningObstacle: this.sim.burning_obstacle,
            burningFloor: this.sim.burning_floor,
        };
        folder.add(this.props, 'numCells').name('cells').disable().listen();
        folder.add(this.props, 'numIters').name('substeps').disable().listen();
        folder.add(this.props, 'overRelaxation').decimals(2).min(1.00).max(1.99).name('over relaxation').onChange((v: number) => (this.sim.over_relaxation = v)).listen();
        folder.add(this.props, 'swirlProbability').decimals(0).min(0).max(100).name('swirl probability').onChange((v: number) => (this.sim.swirl_probability = v)).listen();
        folder.add(this.props, 'burningObstacle').name('burning obstacle').onFinishChange((v: boolean) => {
            this.sim.burning_obstacle = v;
            if (this.sim.burning_obstacle) {
                this.props.showObstacle = true;
                this.sim.show_obstacle = true;
            }
        }).listen();
        folder.add(this.props, 'burningFloor').name('burning floor').onFinishChange((v: boolean) => (this.sim.burning_floor = v)).listen();
        const sub = folder.addFolder('Rendering');
        sub.add(this.props, 'showObstacle').name('show obstacle').onFinishChange((v: boolean) => (this.sim.show_obstacle = v)).listen();
        sub.add(this.props, 'showSwirls').name('show swirls').onFinishChange((v: boolean) => (this.sim.show_swirls = v)).listen();
        sub.add(this.props, 'animate').listen();

        // scene interaction
        this.mouseDown = false;
        let rect = canvas.getBoundingClientRect();
        this.mouseOffset = new THREE.Vector2(rect.left - canvas.clientLeft, rect.top - canvas.clientTop);
        canvas.addEventListener('mousedown', e => { this.startDrag(e.x, e.y) });
        canvas.addEventListener('touchstart', e => { this.startDrag(e.touches[0].clientX, e.touches[0].clientY) });
        canvas.addEventListener('mouseup', _ => { this.endDrag() });
        canvas.addEventListener('touchend', _ => { this.endDrag() });
        canvas.addEventListener('mousemove', e => { this.drag(e.x, e.y) });
        canvas.addEventListener('touchmove', e => {
            e.preventDefault();
            e.stopImmediatePropagation();
            this.drag(e.touches[0].clientX, e.touches[0].clientY);
        });
    }

    private setMousePos(x: number, y: number, reset: boolean) {
        const mx = x - this.mouseOffset.x;
        const my = y - this.mouseOffset.y;
        this.sim.set_obstacle_from_canvas(mx, my, reset);
        this.props.showObstacle = true;
        this.sim.show_obstacle = true;
        this.props.animate = true;
    }

    private startDrag(x: number, y: number) {
        this.mouseDown = true;
        this.setMousePos(x, y, true);
    }

    private drag(x: number, y: number) {
        if (this.mouseDown) {
            this.setMousePos(x, y, false);
        }
    }

    private endDrag() {
        this.mouseDown = false;
    }
}

export { FireDemo, FireDemoConfig };
