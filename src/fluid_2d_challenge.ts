import GUI, { Controller } from 'lil-gui';

import { PositionBasedFluidSimulation } from '../pkg';
import { Demo, Scene2DWebGL, Scene2DConfig } from './lib';


const DEFAULT_DAM_SIZE: [number, number] = [10, 1000];

type PositionBasedFluidDemoProps = {
    animate: boolean;
    numParticles: number;
    viscosity: number;
    substeps: number;
    singleColor: boolean;
    block: () => void;
    reset10x1000: () => void;
    reset10x200: () => void;
    reset40x100: () => void;
    reset100x100: () => void;
};

const PositionBasedFluidDemoConfig: Scene2DConfig = {
    kind: '2DWebGL',
}

class PositionBasedFluidDemo implements Demo<PositionBasedFluidSimulation, PositionBasedFluidDemoProps> {
    sim: PositionBasedFluidSimulation;
    scene: Scene2DWebGL;
    props: PositionBasedFluidDemoProps;

    lastDamSize: [number, number];
    particlesController: Controller;

    constructor(rust_wasm: any, _memory: WebAssembly.Memory, _canvas: HTMLCanvasElement, scene: Scene2DWebGL, folder: GUI) {
        const useDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
        this.lastDamSize = DEFAULT_DAM_SIZE;
        this.sim = new rust_wasm.PositionBasedFluidSimulation(scene.context, scene.width, scene.height, useDarkMode, ...this.lastDamSize);
        this.scene = scene;
        this.initControls(folder);
    }

    init() { }

    update() {
        if (this.props.animate) {
            this.sim.step();
        }
    }

    reset() {
        this.sim.reset(...this.lastDamSize);
        this.setInfo();
    }

    draw() {
        this.sim.draw();
    }

    private setInfo() {
        this.props.numParticles = this.sim.num_particles;
        this.particlesController.updateDisplay();
    }

    private initControls(folder: GUI) {
        const reset = (x: number, y: number) => {
            this.lastDamSize = [x, y];
            this.sim.reset(x, y);
            this.setInfo();
        }
        this.props = {
            animate: true,
            numParticles: this.sim.num_particles,
            viscosity: 0, // see `pbd_fluid_solver::DEFAULT_VISCOSITY`
            substeps: 10,
            singleColor: false,
            block: () => {
                this.sim.add_block();
                this.setInfo();
            },
            reset10x1000: () => reset(10, 1000),
            reset10x200: () => reset(10, 200),
            reset40x100: () => reset(40, 100),
            reset100x100: () => reset(100, 100),
        };
        this.particlesController = folder.add(this.props, 'numParticles').name('particles').disable().listen();
        folder.add(this.props, 'viscosity', 0, 0.75, 0.005).onChange((v: number) => this.sim.viscosity = v);
        folder.add(this.props, 'substeps', 5, 10, 1).onChange((v: number) => this.sim.solver_substeps = v);
        folder.add(this.props, 'singleColor').name('draw single color').onFinishChange((v: boolean) => this.sim.draw_single_color = v);
        folder.add(this.props, 'animate');
        folder.add(this.props, 'block').name('add block');
        folder.add(this.props, 'reset10x1000').name('reset 10x1000 block');
        folder.add(this.props, 'reset10x200').name('reset 10x200 block');
        folder.add(this.props, 'reset40x100').name('reset 40x100 block');
        folder.add(this.props, 'reset100x100').name('reset 100x100 block');
    }
}

export { PositionBasedFluidDemo, PositionBasedFluidDemoConfig };
