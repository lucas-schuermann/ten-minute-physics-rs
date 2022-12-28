import { Controller } from 'lil-gui';
import * as THREE from 'three';
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls";

type Demo<S, T> = {
    sim: S;
    scene: Scene;
    props: T;

    init(): void;
    update(): void;
    reset(): void;
}

type Scene = {
    scene: THREE.Scene;
    camera: THREE.Camera;
    renderer: THREE.Renderer;
    controls: OrbitControls;
}

type SceneConfig = {
    cameraYZ: [number, number];
    cameraLookAt: THREE.Vector3;
};

type GrabberSim = {
    start_grab(v: Float32Array): void;
    move_grabbed(v: Float32Array): void;
    end_grab(v: Float32Array): void;
}

type GrabberProps = {
    animate: boolean;
}

class Grabber {
    sim: GrabberSim;
    scene: Scene;
    props: GrabberProps;
    animateController: Controller;

    mousePos: THREE.Vector2;
    mouseDown: boolean;
    intersectedObject: boolean;

    private raycaster: THREE.Raycaster;
    private distance: number;
    private prevPos: THREE.Vector3;
    private vel: THREE.Vector3;
    private time: number;
    private rect: DOMRect;

    constructor(sim: GrabberSim, canvas: HTMLCanvasElement, scene: Scene, props: GrabberProps, animateController: Controller) {
        this.sim = sim;
        this.scene = scene;
        this.props = props;
        this.animateController = animateController;
        this.raycaster = new THREE.Raycaster();
        this.raycaster.layers.set(1);
        this.raycaster.params.Line.threshold = 0.1;
        this.intersectedObject = false;
        this.distance = 0.0;
        this.mousePos = new THREE.Vector2();
        this.mouseDown = false;
        this.prevPos = new THREE.Vector3();
        this.vel = new THREE.Vector3();
        this.time = 0.0;
        this.rect = scene.renderer.domElement.getBoundingClientRect();

        const onPointer = (e: MouseEvent) => {
            e.preventDefault();
            if (e.type == "pointerdown") {
                this.mouseDown = true;
                this.start(e.clientX, e.clientY);
                if (this.intersectedObject) {
                    this.scene.controls.saveState();
                    this.scene.controls.enabled = false;
                }
            } else if (e.type == "pointermove" && this.mouseDown) {
                this.move(e.clientX, e.clientY);
            } else if (e.type == "pointerup") {
                this.mouseDown = false;
                this.scene.controls.enabled = true;
                if (this.intersectedObject) {
                    this.end();
                    this.scene.controls.reset();
                }
            }
        }
        canvas.addEventListener('pointerdown', onPointer, false);
        canvas.addEventListener('pointermove', onPointer, false);
        canvas.addEventListener('pointerup', onPointer, false);
    }
    increaseTime(dt: number) {
        this.time += dt;
    }
    updateRaycaster(x: number, y: number) {
        this.mousePos.x = ((x - this.rect.left) / this.rect.width) * 2 - 1;
        this.mousePos.y = -((y - this.rect.top) / this.rect.height) * 2 + 1;
        this.raycaster.setFromCamera(this.mousePos, this.scene.camera);
    }
    start(x: number, y: number) {
        this.intersectedObject = false;
        this.updateRaycaster(x, y);
        const intersects = this.raycaster.intersectObjects(this.scene.scene.children);
        if (intersects.length > 0) {
            const obj = intersects[0].object.userData;
            if (obj) {
                this.intersectedObject = true;
                this.distance = intersects[0].distance;
                let pos = this.raycaster.ray.origin.clone();
                pos.addScaledVector(this.raycaster.ray.direction, this.distance);
                this.sim.start_grab(pos.toArray() as unknown as Float32Array);
                this.prevPos.copy(pos);
                this.vel.set(0.0, 0.0, 0.0);
                this.time = 0.0;
                this.props.animate = true;
                this.animateController.updateDisplay();
            }
        }
    }
    move(x: number, y: number) {
        if (this.intersectedObject) {
            this.updateRaycaster(x, y);
            const pos = this.raycaster.ray.origin.clone();
            pos.addScaledVector(this.raycaster.ray.direction, this.distance);

            this.vel.copy(pos);
            this.vel.sub(this.prevPos);
            if (this.time > 0.0) {
                this.vel.divideScalar(this.time);
            } else {
                this.vel.set(0.0, 0.0, 0.0);
            }
            this.prevPos.copy(pos);
            this.time = 0.0;

            this.sim.move_grabbed(pos.toArray() as unknown as Float32Array); // LVSTODO types
        }
    }
    end() {
        if (this.intersectedObject) {
            this.sim.end_grab(this.vel.toArray() as unknown as Float32Array);
            this.intersectedObject = false;
        }
    }
}

export { Demo, Scene, SceneConfig, Grabber };
