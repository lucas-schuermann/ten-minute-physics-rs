import GUI from 'lil-gui';
import * as THREE from 'three';

import { FlipSimulation } from '../pkg';
import { Demo, Scene2DWebGL, Scene2DConfig } from './lib';


type FlipDemoProps = {
    animate: boolean;
    numCells: number;
    numParticles: number;
    numParticleIters: number;
    numPressureIters: number;
    flipRatio: number;
    density: number;
    overRelaxation: number;
    compensateDrift: boolean;
    separateParticles: boolean;
    showObstacle: boolean;
    showParticles: boolean;
    showGrid: boolean;
};

const FlipDemoConfig: Scene2DConfig = {
    kind: '2DWebGL',
}

class FlipDemo implements Demo<FlipSimulation, FlipDemoProps> {
    sim: FlipSimulation;
    scene: Scene2DWebGL;
    props: FlipDemoProps;

    private mouseDown: boolean;
    private mouseOffset: THREE.Vector2;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, scene: Scene2DWebGL, folder: GUI) {
        this.sim = new rust_wasm.FlipSimulation(scene.width, scene.height, scene.context);
        this.scene = scene;
        this.initControls(folder, canvas);
    }

    init() {
        this.props.animate = true;
        this.props.numCells = this.sim.num_cells;
        this.props.numParticles = this.sim.num_particles;
        this.props.numParticleIters = this.sim.num_particle_iters;
        this.props.numPressureIters = this.sim.num_pressure_iters;
        this.props.flipRatio = this.sim.flip_ratio;
        this.props.density = this.sim.density;
        this.props.overRelaxation = this.sim.over_relaxation;
        this.props.separateParticles = this.sim.separate_particles;
        this.props.compensateDrift = this.sim.compensate_drift;
        this.props.showObstacle = this.sim.show_obstacle;
        this.props.showParticles = this.sim.show_particles;
        this.props.showGrid = this.sim.show_grid;
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
        }
    }

    reset() {
        // LVSTODO
    }

    draw() {
        this.sim.draw();
    }

    private initControls(folder: GUI, canvas: HTMLCanvasElement) {
        this.props = {
            animate: false,
            numCells: this.sim.num_cells,
            numParticles: this.sim.num_particles,
            numParticleIters: this.sim.num_particle_iters,
            numPressureIters: this.sim.num_pressure_iters,
            flipRatio: this.sim.flip_ratio,
            density: this.sim.density,
            overRelaxation: this.sim.over_relaxation,
            separateParticles: this.sim.separate_particles,
            compensateDrift: this.sim.compensate_drift,
            showObstacle: this.sim.show_obstacle,
            showParticles: this.sim.show_particles,
            showGrid: this.sim.show_grid,
        };
        folder.add(this.props, 'numCells').name('cells').disable().listen();
        folder.add(this.props, 'numParticles').name('particles').disable().listen();
        folder.add(this.props, 'numParticleIters').name('particle substeps').disable().listen();
        folder.add(this.props, 'numPressureIters').name('pressure substeps').disable().listen();
        folder.add(this.props, 'density').disable().listen();
        folder.add(this.props, 'flipRatio').decimals(2).min(0.00).max(1.00).name('flip ratio').onChange((v: number) => (this.sim.flip_ratio = v)).listen();
        folder.add(this.props, 'overRelaxation').decimals(2).min(1.00).max(1.99).name('over relaxation').onChange((v: number) => (this.sim.over_relaxation = v)).listen();
        folder.add(this.props, 'separateParticles').name('separate particles').onChange((v: boolean) => (this.sim.separate_particles = v));
        folder.add(this.props, 'compensateDrift').name('compensate drift').onChange((v: boolean) => (this.sim.compensate_drift = v));
        const sub = folder.addFolder('Rendering');
        sub.add(this.props, 'showObstacle').name('show obstacle').onFinishChange((v: boolean) => (this.sim.show_obstacle = v)).listen();
        sub.add(this.props, 'showParticles').name('show particles').onFinishChange((v: boolean) => (this.sim.show_particles = v)).listen();
        sub.add(this.props, 'showGrid').name('show grid').onFinishChange((v: boolean) => (this.sim.show_grid = v)).listen();
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

export { FlipDemo, FlipDemoConfig };
