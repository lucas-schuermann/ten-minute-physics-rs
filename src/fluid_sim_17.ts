import GUI from 'lil-gui';
import * as THREE from 'three';

import { FluidSimulation, SceneType } from '../pkg';
import { Demo, Scene2DCanvas, Scene2DConfig, enumToValueList } from './lib';

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
    kind: '2DCanvas',
}

class FluidDemo implements Demo<FluidSimulation, FluidDemoProps> {
    sim: FluidSimulation;
    scene: Scene2DCanvas;
    props: FluidDemoProps;

    private rust_wasm: any;
    private mouseDown: boolean;
    private mouseOffset: THREE.Vector2;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene2DCanvas, folder: GUI) {
        this.rust_wasm = rust_wasm;
        this.sim = new rust_wasm.FluidSimulation(DEFAULT_SCENE, scene.width, scene.height, scene.context);
        this.scene = scene;
        this.initControls(folder, canvas);
    }

    init() {
        this.props.numCells = this.sim.num_cells;
        this.props.numIters = this.sim.num_iters;
        this.props.density = this.sim.density;
        this.props.overRelaxation = this.sim.over_relaxation;
        this.props.showObstacle = this.sim.show_obstacle;
        this.props.showStreamlines = this.sim.show_streamlines;
        this.props.showVelocities = this.sim.show_velocities;
        this.props.showPressure = this.sim.show_pressure;
        this.props.showSmoke = this.sim.show_smoke;
        if (this.props.scene === SceneType[SceneType.WindTunnel] && this.props.showSmoke === true && this.props.showPressure === false) {
            document.getElementById('info').setAttribute("style", "color: #000;");
        } else {
            document.getElementById('info').removeAttribute("style");
        }
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
        }
    }

    reset() {
        this.sim.free();
        this.sim = new this.rust_wasm.FluidSimulation(Object.values(SceneType).indexOf(this.props.scene), this.scene.width, this.scene.height, this.scene.context);
        this.init();
    }

    draw() {
        this.sim.draw();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            scene: SceneType[DEFAULT_SCENE],
            animate: true,
            numCells: this.sim.num_cells,
            numIters: this.sim.num_iters,
            density: this.sim.density,
            overRelaxation: this.sim.over_relaxation,
            showObstacle: this.sim.show_obstacle,
            showStreamlines: this.sim.show_streamlines,
            showVelocities: this.sim.show_velocities,
            showPressure: this.sim.show_pressure,
            showSmoke: this.sim.show_smoke,
        };
        folder.add(this.props, 'scene', enumToValueList(SceneType)).onChange((_: string) => {
            this.reset();
        });
        folder.add(this.props, 'numCells').name('cells').disable().listen();
        folder.add(this.props, 'numIters').name('substeps').disable().listen();
        folder.add(this.props, 'density').disable().listen();
        folder.add(this.props, 'overRelaxation').decimals(2).min(1.00).max(1.99).name('over relaxation').onChange((v: number) => (this.sim.over_relaxation = v)).listen();
        const sub = folder.addFolder('Rendering');
        sub.add(this.props, 'showObstacle').name('show obstacle').onFinishChange((v: boolean) => (this.sim.show_obstacle = v)).listen();
        sub.add(this.props, 'showStreamlines').name('show streamlines').onFinishChange((v: boolean) => (this.sim.show_streamlines = v)).listen();
        sub.add(this.props, 'showVelocities').name('show velocities').onFinishChange((v: boolean) => (this.sim.show_velocities = v)).listen();
        sub.add(this.props, 'showPressure').name('show pressure').onFinishChange((v: boolean) => (this.sim.show_pressure = v)).listen();
        sub.add(this.props, 'showSmoke').name('show smoke').onFinishChange((v: boolean) => (this.sim.show_smoke = v)).listen();
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
        this.sim.set_obstacle_from_canvas(mx, my, reset, this.props.scene === SceneType[SceneType.Paint]);
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

export { FluidDemo, FluidDemoConfig };
