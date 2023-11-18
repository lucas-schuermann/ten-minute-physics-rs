import GUI from 'lil-gui';
import * as THREE from 'three';

import { FireSimulation } from '../pkg';
import { Demo, Scene2DCanvas, Scene2DConfig } from './lib';

type FireDemoProps = {
    animate: boolean;
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
        //this.props.numCells = this.sim.num_cells;
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
        };
        folder.add(this.props, 'animate').listen();

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
        // this.props.showObstacle = true;
        // this.sim.show_obstacle = true;
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
