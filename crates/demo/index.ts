import GUI from 'lil-gui';
import * as Stats from 'stats.js';
import * as THREE from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';

import { SelfCollisionDemo, SelfCollisionDemoConfig } from './src/self_collision_15';
import { ClothDemo, ClothDemoConfig } from './src/cloth_14';
import { HashDemo, HashDemoConfig } from './src/hashing_11';
import { Demo, Scene, SceneConfig } from './src/lib';
import { SoftBodiesDemo, SoftBodiesDemoConfig } from './src/softbodies_10';
import { SkinnedSoftbodyDemo, SkinnedSoftbodyDemoConfig } from './src/softbody_skinning_12';

import('./pkg').then(rust_wasm => {
    const $ = (id: string) => document.getElementById(id);
    const canvas = $('canvas') as HTMLCanvasElement;

    const demos: Record<string, { config: SceneConfig, demo: any }> = {
        '10-softbodies': {
            config: SoftBodiesDemoConfig,
            demo: SoftBodiesDemo,
        },
        '11-hashing': {
            config: HashDemoConfig,
            demo: HashDemo,
        },
        '12-softbody-skinning': {
            config: SkinnedSoftbodyDemoConfig,
            demo: SkinnedSoftbodyDemo,
        },
        '14-cloth': {
            config: ClothDemoConfig,
            demo: ClothDemo,
        },
        '15-self-collision': {
            config: SelfCollisionDemoConfig,
            demo: SelfCollisionDemo,
        }
    };
    const demoNames = Object.keys(demos);
    let demo: Demo<any, any>;
    let scene: Scene;

    const initThreeScene = (config: SceneConfig): Scene => {
        let scene: THREE.Scene;
        let camera: THREE.Camera;
        let renderer: THREE.WebGLRenderer;
        let controls: OrbitControls;

        // LVSTODO: handle retina
        canvas.width = window.innerWidth * 0.4;
        canvas.height = window.innerHeight * 0.35;

        scene = new THREE.Scene();

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
        renderer = new THREE.WebGLRenderer({ canvas: canvas, antialias: true, powerPreference: "high-performance" });
        renderer.shadowMap.enabled = true;
        renderer.setPixelRatio(window.devicePixelRatio);
        renderer.setSize(canvas.width, canvas.height);

        // Camera
        camera = new THREE.PerspectiveCamera(70, canvas.width / canvas.height, 0.01, 100);
        camera.position.set(0, config.cameraYZ[0], config.cameraYZ[1]);
        camera.updateMatrixWorld();
        scene.add(camera);

        controls = new OrbitControls(camera, renderer.domElement);
        controls.zoomSpeed = 2.0;
        controls.panSpeed = 0.4;
        controls.target = config.cameraLookAt;
        controls.update();

        return { scene, camera, renderer, controls };
    };

    // attach perf stats window
    const stats = new Stats();
    stats.dom.style.position = 'absolute';
    stats.showPanel(1); // ms per frame
    $('container').appendChild(stats.dom);

    // populate controls window
    const gui = new GUI({ autoPlace: false });
    gui.domElement.style.opacity = '0.9';
    let props = {
        demoSelection: demoNames[0],
        reset: () => demo.reset(),
    }
    const folder = gui.addFolder('General');
    $('gui').appendChild(gui.domElement);
    let demoFolder: GUI;
    const initDemo = (s: string) => {
        // initialize renderer
        if (demoFolder) demoFolder.destroy();
        demoFolder = gui.addFolder('Demo Settings');
        scene = initThreeScene(demos[s].config);
        demo = new demos[s].demo(rust_wasm, canvas, scene, demoFolder);
        demo.init();
    }
    folder.add(props, 'demoSelection', demoNames).name('select demo').onFinishChange(initDemo);
    folder.add(props, 'reset').name('reset simulation');

    // default init
    initDemo(demoNames[0]);

    // main loop
    const step = () => {
        stats.begin(); // collect perf data for stats.js
        demo.update();
        scene.renderer.render(scene.scene, scene.camera);
        stats.end();
        requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
}).catch(console.error);
