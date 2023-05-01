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
    draw?(): void;
    resize?(width: number, height: number): void;
    free?(): void;
}

type Scene = Scene2DCanvas | Scene2DWebGL | Scene3D;

type Scene2DCanvas = {
    kind: "2DCanvas";
    width: number;
    height: number;
    context: CanvasRenderingContext2D;
}

type Scene2DWebGL = {
    kind: "2DWebGL";
    width: number;
    height: number;
    context: WebGL2RenderingContext;
}

type Scene3D = {
    kind: "3D";
    scene?: THREE.Scene;
    camera?: THREE.PerspectiveCamera;
    renderer?: THREE.WebGLRenderer;
    controls?: OrbitControls;
    offscreen: boolean;
}

type SceneConfig = Scene2DConfig | Scene3DConfig;

type Scene2DConfig = {
    kind: "2DCanvas" | "2DWebGL";
}

type Scene3DConfig = {
    kind: "3D";
    cameraYZ: [number, number];
    cameraLookAt: THREE.Vector3;
    offscreen?: boolean;
};

type GrabberInterface = {
    start_grab(id: number, v: Float32Array): void;
    move_grabbed(id: number, v: Float32Array): void;
    end_grab(id: number, v: Float32Array): void;
}

type GrabberProps = {
    animate: boolean;
}

class Grabber {
    sim: GrabberInterface;
    scene: Scene3D;
    props: GrabberProps;
    animateController: Controller;

    private mousePos: THREE.Vector2;
    private mouseDown: boolean;
    private intersectedObjectId: null | number;

    private inputElement: HTMLElement;
    private raycaster: THREE.Raycaster;
    private distance: number;
    private prevPos: THREE.Vector3;
    private vel: THREE.Vector3;
    private time: number;

    constructor(sim: GrabberInterface, inputElement: HTMLElement, scene: Scene3D, props: GrabberProps, animateController: Controller) {
        this.sim = sim;
        this.scene = scene;
        this.props = props;
        this.animateController = animateController;
        this.raycaster = new THREE.Raycaster();
        this.raycaster.layers.set(1);
        this.raycaster.params.Line.threshold = 0.1;
        this.intersectedObjectId = null;
        this.distance = 0.0;
        this.mousePos = new THREE.Vector2();
        this.mouseDown = false;
        this.prevPos = new THREE.Vector3();
        this.vel = new THREE.Vector3();
        this.time = 0.0;

        const getXY = (e: MouseEvent | TouchEvent): [number, number] => {
            let me = e as MouseEvent;
            let te = e as TouchEvent;
            if (me.clientX) {
                return [me.clientX, me.clientY];
            } else {
                return [te.touches[0].clientX, te.touches[0].clientY];
            }
        }

        const onInput = (e: MouseEvent | TouchEvent) => {
            e.preventDefault();
            if (e.type === "mousedown" || e.type === "touchstart") {
                this.mouseDown = true;
                this.start(...getXY(e));
                if (this.intersectedObjectId !== null) {
                    this.scene.controls.saveState();
                    this.scene.controls.enabled = false;
                }
            } else if ((e.type === "mousemove" || e.type === "touchmove") && this.mouseDown) {
                this.move(...getXY(e));
            } else if (e.type === "mouseup" || e.type === "touchend") {
                this.mouseDown = false;
                this.scene.controls.enabled = true;
                if (this.intersectedObjectId !== null) {
                    this.end();
                    this.scene.controls.reset();
                }
            }
        }
        inputElement.addEventListener('mousedown', onInput, false);
        inputElement.addEventListener('touchstart', onInput, false);
        inputElement.addEventListener('mouseup', onInput, false);
        inputElement.addEventListener('touchend', onInput, false);
        inputElement.addEventListener('mousemove', onInput, false);
        inputElement.addEventListener('touchmove', onInput, false);
        this.inputElement = inputElement;
    }
    increaseTime(dt: number) {
        this.time += dt;
    }
    updateRaycaster(x: number, y: number) {
        const rect = this.inputElement.getBoundingClientRect();
        this.mousePos.x = ((x - rect.left) / rect.width) * 2 - 1;
        this.mousePos.y = -((y - rect.top) / rect.height) * 2 + 1;
        this.raycaster.setFromCamera(this.mousePos, this.scene.camera);
    }
    start(x: number, y: number) {
        this.intersectedObjectId = null;
        this.updateRaycaster(x, y);
        const intersects = this.raycaster.intersectObjects(this.scene.scene.children);
        if (intersects.length > 0) {
            const obj = intersects[0].object;
            if (!obj) {
                return;
            }
            this.intersectedObjectId = 'id' in obj.userData ? obj.userData.id : 0;
            this.distance = intersects[0].distance;
            const pos = this.raycaster.ray.origin.clone();
            pos.addScaledVector(this.raycaster.ray.direction, this.distance);
            this.sim.start_grab(this.intersectedObjectId, pos.toArray() as unknown as Float32Array);
            this.prevPos.copy(pos);
            this.vel.set(0.0, 0.0, 0.0);
            this.time = 0.0;
            this.props.animate = true;
            this.animateController.updateDisplay();
        }
    }
    move(x: number, y: number) {
        if (this.intersectedObjectId !== null) {
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

            this.sim.move_grabbed(this.intersectedObjectId, pos.toArray() as unknown as Float32Array);
        }
    }
    end() {
        if (this.intersectedObjectId !== null) {
            this.sim.end_grab(this.intersectedObjectId, this.vel.toArray() as unknown as Float32Array);
            this.intersectedObjectId = null;
        }
    }
}

const initThreeScene = (canvas: HTMLCanvasElement | OffscreenCanvas, inputElement: HTMLElement, config: Scene3DConfig, devicePixelRatio: number): Scene3D => {
    const scene = new THREE.Scene();

    // lights
    scene.add(new THREE.AmbientLight(0x505050));
    scene.fog = new THREE.Fog(0x000000, 0, 15);

    const spotLight = new THREE.SpotLight(0xffffff);
    spotLight.angle = Math.PI / 5;
    spotLight.penumbra = 0.2;
    spotLight.position.set(2, 3, 3);
    spotLight.castShadow = true;
    spotLight.shadow.camera.near = 3;
    spotLight.shadow.camera.far = 10;
    spotLight.shadow.mapSize.width = 1024;
    spotLight.shadow.mapSize.height = 1024;
    scene.add(spotLight);

    const dirLight = new THREE.DirectionalLight(0x55505a, 1);
    dirLight.position.set(0, 3, 0);
    dirLight.castShadow = true;
    dirLight.shadow.camera.near = 1;
    dirLight.shadow.camera.far = 10;
    dirLight.shadow.camera.right = 1;
    dirLight.shadow.camera.left = - 1;
    dirLight.shadow.camera.top = 1;
    dirLight.shadow.camera.bottom = - 1;
    dirLight.shadow.mapSize.width = 1024;
    dirLight.shadow.mapSize.height = 1024;
    scene.add(dirLight);

    // geometry
    const ground = new THREE.Mesh(
        new THREE.PlaneGeometry(20, 20, 1, 1),
        new THREE.MeshPhongMaterial({ color: 0xa0adaf, shininess: 150 })
    );
    ground.rotation.x = - Math.PI / 2; // rotates X/Y to X/Z
    ground.receiveShadow = true;
    scene.add(ground);
    const helper = new THREE.GridHelper(20, 20);
    const material = helper.material as THREE.Material;
    material.opacity = 1.0;
    material.transparent = true;
    helper.position.set(0, 0.002, 0);
    scene.add(helper);

    // renderer
    const renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true, powerPreference: "high-performance" });
    renderer.shadowMap.enabled = true;
    renderer.setPixelRatio(devicePixelRatio);

    if (config.offscreen) {
        renderer.setSize(inputElement.clientWidth, inputElement.clientHeight, false);
    } else {
        renderer.setSize(window.innerWidth, window.innerHeight);
    }

    // camera
    const camera = new THREE.PerspectiveCamera(70, inputElement.clientWidth / inputElement.clientHeight, 0.01, 100);
    camera.position.set(0, config.cameraYZ[0], config.cameraYZ[1]);
    camera.updateMatrixWorld();
    scene.add(camera);

    const controls = new OrbitControls(camera, inputElement as HTMLCanvasElement);
    controls.zoomSpeed = 2.0;
    controls.panSpeed = 0.4;
    controls.target.copy(config.cameraLookAt);
    controls.update();

    return { kind: '3D', scene, camera, renderer, controls, offscreen: config.offscreen };
};

const resizeThreeScene = (scene: Scene3D, width: number, height: number, updateStyle: boolean) => {
    scene.camera.aspect = width / height;
    scene.camera.updateProjectionMatrix();
    scene.renderer.setSize(width, height, updateStyle);
}


// returns ['EnumOne', 'EnumTwo', ...]
const enumToValueList = (e: any): any => Object.values(e).filter((i) => typeof i === 'string');

export { Demo, Scene, Scene2DCanvas, Scene2DWebGL, Scene3D, SceneConfig, Scene2DConfig, Scene3DConfig, Grabber, initThreeScene, resizeThreeScene, enumToValueList };
