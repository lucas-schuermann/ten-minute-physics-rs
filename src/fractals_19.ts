import GUI from 'lil-gui';
import * as THREE from 'three';

import { FractalsSimulation, FractalsSceneType } from '../pkg';
import { Demo, Scene2DCanvas, Scene2DConfig, enumToValueList } from './lib';

const DEFAULT_SCENE = FractalsSceneType.Julia;
const MAX_SCROLL_SCALE_RATE = 0.10;

type FractalsDemoProps = {
    scene: string; // enum string value
    drawMono: boolean;
    maxIters: number;
    dragAction: 'ChangeFractalC' | 'Move';
};

const FractalsDemoConfig: Scene2DConfig = {
    kind: '2DCanvas',
}

class FractalsDemo implements Demo<FractalsSimulation, FractalsDemoProps> {
    sim: FractalsSimulation;
    scene: Scene2DCanvas;
    props: FractalsDemoProps;

    private mouseDown: boolean;
    private lastMouse: THREE.Vector2;
    private imageData: ImageData;

    constructor(rust_wasm: any, _: WebAssembly.Memory, canvas: HTMLCanvasElement, scene: Scene2DCanvas, folder: GUI) {
        this.sim = new rust_wasm.FractalsSimulation(DEFAULT_SCENE, scene.width, scene.height, scene.context);
        this.scene = scene;
        this.initControls(folder, canvas);
        this.imageData = scene.context.createImageData(scene.width, scene.height);
    }

    init() {
        this.props.maxIters = this.sim.max_iters;
    }

    // noop since all work is done in draw() upon redraw = true
    update() { }

    reset() {
        this.sim.reset();
        this.init();
    }

    draw() {
        this.sim.draw_buffer(this.imageData.data as unknown as Uint8Array); // wasm-bindgen doesn't support Uint8ClampedArray
        this.scene.context.putImageData(this.imageData, 0, 0);
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            scene: FractalsSceneType[DEFAULT_SCENE],
            drawMono: this.sim.draw_mono,
            maxIters: this.sim.max_iters,
            dragAction: 'Move',
        };
        folder.add(this.props, 'scene', enumToValueList(FractalsSceneType)).onChange((n: string) => {
            this.sim.scene_type = Object.values(FractalsSceneType).indexOf(n);
            this.sim.redraw = true;
        });
        folder.add(this.props, 'maxIters').min(1).max(1000).step(1).name('max iterations').onChange((v: number) => {
            this.sim.max_iters = v;
            this.sim.redraw = true;
        }).listen();
        folder.add(this.props, 'drawMono').name('draw mono').onFinishChange((v: boolean) => {
            this.sim.draw_mono = v;
            this.sim.redraw = true;
        });
        folder.add(this.props, 'dragAction', ['Move', 'ChangeFractalC']).name('drag action').listen();

        // scene interaction
        this.mouseDown = false;
        this.lastMouse = new THREE.Vector2(0, 0);
        canvas.addEventListener('mousedown', e => { this.startDrag(e.x, e.y) });
        canvas.addEventListener('touchstart', e => { this.startDrag(e.touches[0].clientX, e.touches[0].clientY) });
        canvas.addEventListener('mouseup', _ => { this.endDrag() });
        canvas.addEventListener('touchend', _ => { this.endDrag() });
        canvas.addEventListener('mousemove', e => { this.drag(e.x, e.y, e.shiftKey) });
        canvas.addEventListener('touchmove', e => {
            e.preventDefault();
            e.stopImmediatePropagation();
            this.drag(e.touches[0].clientX, e.touches[0].clientY, e.shiftKey);
        });
        canvas.addEventListener('wheel', e => {
            e.preventDefault();
            const rate = Math.min(MAX_SCROLL_SCALE_RATE, e.deltaY / 1000);
            this.sim.scale *= 1 + rate;
            this.sim.redraw = true;
        });
    }

    private startDrag(x: number, y: number) {
        this.mouseDown = true;
        this.lastMouse.x = x;
        this.lastMouse.y = y;
    }

    private drag(x: number, y: number, shift: boolean) {
        if (this.mouseDown) {
            const dx = x - this.lastMouse.x;
            const dy = y - this.lastMouse.y;
            this.sim.handle_drag(dx, dy, shift || this.props.dragAction === 'ChangeFractalC'); // sets redraw
        }
        this.lastMouse.x = x;
        this.lastMouse.y = y;
    }

    private endDrag() {
        this.mouseDown = false;
    }
}

export { FractalsDemo, FractalsDemoConfig };
