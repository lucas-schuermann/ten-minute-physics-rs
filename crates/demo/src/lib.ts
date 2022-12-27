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
    cameraZ: number;
    cameraLookAt: THREE.Vector3;
};

export { Demo, Scene, SceneConfig };