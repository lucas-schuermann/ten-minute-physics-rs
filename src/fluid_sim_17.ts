import GUI from 'lil-gui';
import * as THREE from 'three';

import { FluidSimulation, SceneType, Field } from '../pkg';
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
        const renderBufPtr = this.sim.render_buffer;
        const bufSize = this.scene.width * this.scene.height * 4;
        this.buf = new Uint8ClampedArray(memory.buffer, renderBufPtr, bufSize);
        this.id = new ImageData(this.buf, this.scene.width, this.scene.height);

        // LVSTODO set other props to match scene conifgs
        if (this.props.scene === SceneType[SceneType.WindTunnel] || this.props.scene === SceneType[SceneType.HiresTunnel]) {
            this.props.showObstacle = true;
        } else {
            this.props.showObstacle = false;
        }
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
            numCells: this.sim.num_cells,
            numIters: this.sim.params.num_iters,
            density: this.sim.params.density,
            overRelaxation: this.sim.params.over_relaxation,
            showObstacle: false,
            showStreamlines: false,
            showVelocities: false,
            showPressure: this.sim.show_pressure,
            showSmoke: this.sim.show_smoke,
        };
        folder.add(this.props, 'scene', enumToValueList(SceneType)).onChange((_: string) => {
            this.reset();
        });
        folder.add(this.props, 'numCells').name('cells').disable();
        folder.add(this.props, 'numIters').name('substeps').onChange((v: number) => (this.sim.params.num_iters = v));
        folder.add(this.props, 'density').disable();
        folder.add(this.props, 'overRelaxation').decimals(1).name('over relaxation').onChange((v: number) => (this.sim.params.over_relaxation = v));
        folder.add(this.props, 'showObstacle').name('show obstacle').listen();
        folder.add(this.props, 'showStreamlines').name('show streamlines');
        folder.add(this.props, 'showVelocities').name('show velocities');
        folder.add(this.props, 'showPressure').name('show pressure').onFinishChange((v: boolean) => (this.sim.show_pressure = v));
        folder.add(this.props, 'showSmoke').name('show smoke').onFinishChange((v: boolean) => (this.sim.show_smoke = v));
        folder.add(this.props, 'animate').listen();

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

        // draw fluid (smoke and pressure from render buffer)
        c.putImageData(this.id, 0, 0);

        if (this.props.showVelocities) {

            c.strokeStyle = "#000000";
            let scale = 0.02;

            const uPtr = this.sim.u;
            const vPtr = this.sim.v;
            const bufSize = this.sim.num_cells;
            const ua = new Float32Array(memory.buffer, uPtr, bufSize); // LVSTODO move to class param, memory leak
            const va = new Float32Array(memory.buffer, vPtr, bufSize);
            const h = this.sim.params.h;
            const n = this.sim.num_cells_y;

            for (var i = 0; i < this.sim.num_cells_x; i++) {
                for (var j = 0; j < this.sim.num_cells_y; j++) {

                    const u = ua[i * n + j];
                    const v = va[i * n + j];

                    c.beginPath();

                    let x0 = this.sim.c_x(i * h);
                    let x1 = this.sim.c_x(i * h + u * scale);
                    y = this.sim.c_y((j + 0.5) * h);

                    c.moveTo(x0, y);
                    c.lineTo(x1, y);
                    c.stroke();

                    x = this.sim.c_x((i + 0.5) * h);
                    let y0 = this.sim.c_y(j * h);
                    let y1 = this.sim.c_y(j * h + v * scale)

                    c.beginPath();
                    c.moveTo(x, y0);
                    c.lineTo(x, y1);
                    c.stroke();

                }
            }
        }

        if (this.props.showStreamlines) {

            var segLen = this.sim.params.h * 0.2;
            var numSegs = 15;

            c.strokeStyle = "#000000";

            for (var i = 1; i < this.sim.num_cells_x - 1; i += 5) {
                for (var j = 1; j < this.sim.num_cells_y - 1; j += 5) {

                    var x = (i + 0.5) * this.sim.params.h;
                    var y = (j + 0.5) * this.sim.params.h;

                    c.beginPath();
                    c.moveTo(this.sim.c_x(x), this.sim.c_y(y));

                    for (var n = 0; n < numSegs; n++) {
                        var u = this.sim.sample_field(x, y, Field.U);
                        var v = this.sim.sample_field(x, y, Field.V);
                        let l = Math.sqrt(u * u + v * v);
                        x += u / l * segLen;
                        y += v / l * segLen;
                        x += u * 0.01;
                        y += v * 0.01;
                        if (x > this.sim.num_cells_x * this.sim.params.h)
                            break;

                        c.lineTo(this.sim.c_x(x), this.sim.c_y(y));
                    }
                    c.stroke();
                }
            }
        }

        if (this.props.showObstacle) {
            let r = this.sim.obstacle_radius + this.sim.params.h;
            let o = this.sim.obstacle_pos;
            if (this.props.showPressure) {
                c.fillStyle = "#000000";
            } else {
                c.fillStyle = "#DDDDDD";
            }

            c.beginPath();
            c.arc(this.sim.c_x(o[0]), this.sim.c_y(o[1]), this.sim.c_scale * r, 0.0, 2.0 * Math.PI);
            c.closePath();
            c.fill();

            c.lineWidth = 3.0;
            c.strokeStyle = "#000000";
            c.beginPath();
            c.arc(this.sim.c_x(o[0]), this.sim.c_y(o[1]), this.sim.c_scale * r, 0.0, 2.0 * Math.PI);
            c.closePath();
            c.stroke();
            c.lineWidth = 1.0;
        }
        // 
        //         if (this.props.showPressure) {
        //             // LVSTODO display pressure text info
        //         }
    }

    private setMousePos(x: number, y: number, reset: boolean) {
        const mx = x - this.mouseOffset.x;
        const my = y - this.mouseOffset.y;
        this.sim.set_obstacle_from_canvas(mx, my, reset, this.props.scene === SceneType[SceneType.Paint]);
        this.props.showObstacle = true;
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

export { FluidDemo, FluidDemoConfig };
