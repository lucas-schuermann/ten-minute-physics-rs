import GUI from 'lil-gui';
import * as THREE from 'three';

import { FluidSimulation, SceneType } from '../pkg';
import { memory } from '../pkg/index_bg.wasm';
import { Demo, Scene2D, Scene2DConfig, enumToValueList } from './lib';

const DEFAULT_SCENE = SceneType.WindTunnel;

type FluidDemoProps = {
    scene: string; // enum string value
    animate: boolean;
    numCells: number;
    numIters: number;
    density: number;
    overRelaxation: number;
    showObstacle: boolean;
    showStreamlines: boolean;
    showVelocities: boolean;
    showPressure: boolean;
    showSmoke: boolean;
};

const FluidDemoConfig: Scene2DConfig = {
    kind: '2D',
}

class FluidDemo implements Demo<FluidSimulation, FluidDemoProps> {
    sim: FluidSimulation;
    scene: Scene2D;
    props: FluidDemoProps;

    private rust_wasm: any;
    private id: ImageData;
    private buf: Uint8ClampedArray;
    private mouseDown: boolean;
    private mouseOffset: THREE.Vector2;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene2D, folder: GUI) {
        this.rust_wasm = rust_wasm;
        this.sim = new rust_wasm.FluidSimulation(DEFAULT_SCENE, scene.width, scene.height);
        this.scene = scene;
        this.initControls(folder, canvas);
    }

    init() {
        // LVSTODO safety comment
        const renderBufPtr = this.sim.render_buffer_ptr();
        const bufSize = this.scene.width * this.scene.height * 4;
        this.buf = new Uint8ClampedArray(memory.buffer, renderBufPtr, bufSize);
        this.id = new ImageData(this.buf, this.scene.width, this.scene.height);
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.draw();
        }
    }

    reset() {
        this.sim.free();
        this.sim = new this.rust_wasm.FluidSimulation(Object.values(SceneType).indexOf(this.props.scene), this.scene.width, this.scene.height);
        this.init();
        this.draw();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            scene: SceneType[DEFAULT_SCENE],
            animate: true,
            numCells: this.sim.num_cells(),
            numIters: this.sim.num_iters(),
            density: this.sim.density(),
            overRelaxation: this.sim.over_relaxation(),
            showObstacle: false,
            showStreamlines: false,
            showVelocities: false,
            showPressure: false,
            showSmoke: false,
        };
        folder.add(this.props, 'scene', enumToValueList(SceneType)).onChange((_: string) => {
            this.reset();
        });
        folder.add(this.props, 'numCells').name('cells').disable();
        folder.add(this.props, 'numIters').name('substeps').disable();
        folder.add(this.props, 'density').disable();
        folder.add(this.props, 'overRelaxation').decimals(1).name('over relaxation').disable();
        folder.add(this.props, 'showObstacle').name('show obstacle');
        folder.add(this.props, 'showStreamlines').name('show streamlines');
        folder.add(this.props, 'showVelocities').name('show velocities');
        folder.add(this.props, 'showPressure').name('show pressure');
        folder.add(this.props, 'showSmoke').name('show smoke');
        folder.add(this.props, 'animate');

        // scene interaction
        this.mouseDown = false;
        let rect = canvas.getBoundingClientRect();
        this.mouseOffset = new THREE.Vector2(rect.left - canvas.clientLeft, rect.top - canvas.clientTop);
        canvas.addEventListener('mousedown', e => { this.startDrag(e.x, e.y) });
        canvas.addEventListener('mouseup', _ => { this.endDrag() });
        canvas.addEventListener('mousemove', e => { this.drag(e.x, e.y) });
    }

    private draw() {
        const c = this.scene.context;

        // draw fluid (smoke and pressure)
        c.putImageData(this.id, 0, 0);

        //         if (this.props.showVelocities) {
        // 
        //             c.strokeStyle = "#000000";
        //             const scale = 0.02;
        //             for (var i = 0; i < f.numX; i++) {
        //                 for (var j = 0; j < f.numY; j++) {
        // 
        //                     var u = f.u[i * n + j];
        //                     var v = f.v[i * n + j];
        // 
        //                     c.beginPath();
        // 
        //                     let x0 = cX(i * h);
        //                     let x1 = cX(i * h + u * scale);
        //                     let y = cY((j + 0.5) * h);
        // 
        //                     c.moveTo(x0, y);
        //                     c.lineTo(x1, y);
        //                     c.stroke();
        // 
        //                     let x = cX((i + 0.5) * h);
        //                     let y0 = cY(j * h);
        //                     let y1 = cY(j * h + v * scale)
        // 
        //                     c.beginPath();
        //                     c.moveTo(x, y0);
        //                     c.lineTo(x, y1);
        //                     c.stroke();
        // 
        //                 }
        //             }
        //         }
        // 
        //         if (this.props.showStreamlines) {
        // 
        //             var segLen = f.h * 0.2;
        //             var numSegs = 15;
        // 
        //             c.strokeStyle = "#000000";
        // 
        //             for (var i = 1; i < f.numX - 1; i += 5) {
        //                 for (var j = 1; j < f.numY - 1; j += 5) {
        // 
        //                     var x = (i + 0.5) * f.h;
        //                     var y = (j + 0.5) * f.h;
        // 
        //                     c.beginPath();
        //                     c.moveTo(cX(x), cY(y));
        // 
        //                     for (var n = 0; n < numSegs; n++) {
        //                         var u = f.sampleField(x, y, U_FIELD);
        //                         var v = f.sampleField(x, y, V_FIELD);
        //                         l = Math.sqrt(u * u + v * v);
        //                         // x += u/l * segLen;
        //                         // y += v/l * segLen;
        //                         x += u * 0.01;
        //                         y += v * 0.01;
        //                         if (x > f.numX * f.h)
        //                             break;
        // 
        //                         c.lineTo(cX(x), cY(y));
        //                     }
        //                     c.stroke();
        //                 }
        //             }
        //         }

        //         if (this.props.showObstacle) {
        //             let r, ox, oy = this.sim.obstacle();
        //             if (this.props.showPressure) {
        //                 c.fillStyle = "#000000";
        //             } else {
        //                 c.fillStyle = "#DDDDDD";
        //             }
        // 
        //             c.beginPath();
        //             c.arc(
        //                 cX(ox), cY(oy), C_SCALE * r, 0.0, 2.0 * Math.PI);
        //             c.closePath();
        //             c.fill();
        // 
        //             c.lineWidth = 3.0;
        //             c.strokeStyle = "#000000";
        //             c.beginPath();
        //             c.arc(
        //                 cX(ox), cY(oy), C_SCALE * r, 0.0, 2.0 * Math.PI);
        //             c.closePath();
        //             c.stroke();
        //             c.lineWidth = 1.0;
        //         }
        // 
        //         if (this.props.showPressure) {
        //             // LVSTODO display pressure
        //         }
    }

    private setMousePos(x: number, y: number, reset: boolean) {
        const mx = x - this.mouseOffset.x;
        const my = y - this.mouseOffset.y;
        this.sim.set_obstacle(mx, my, reset, this.props.scene === SceneType[SceneType.Paint]);
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

export { FluidDemo, FluidDemoConfig };
