import GUI from 'lil-gui';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import * as Stats from 'stats.js';
import * as THREE from 'three';

import { HashDemo } from './src/hash';
import { ClothDemo } from './src/cloth';

import('./pkg').then(rust_wasm => {
    let scene: THREE.Scene;
    let camera: THREE.Camera;
    let renderer: THREE.WebGLRenderer;
    let controls: OrbitControls;

    const $ = (id: string) => document.getElementById(id);
    //const useDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const canvas = $('canvas') as HTMLCanvasElement;

    const initThreeScene = () => {
        // TODO: handle retina
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
        camera.position.set(0, 1, 1);
        camera.updateMatrixWorld();
        scene.add(camera);

        controls = new OrbitControls(camera, renderer.domElement);
        controls.zoomSpeed = 2.0;
        controls.panSpeed = 0.4;
        controls.target = new THREE.Vector3(0.0, 0.6, 0.0);
        controls.update();
    };

    // attach perf stats window
    const stats = new Stats();
    stats.dom.style.position = 'absolute';
    stats.showPanel(1); // ms per frame
    $('container').appendChild(stats.dom);

    // populate controls window
    const gui = new GUI({ autoPlace: false });
    gui.domElement.style.opacity = '0.9';
    let demoSelection = {
        demoSelection: 'Hash',
    }
    const folder = gui.addFolder('General');
    $('gui').appendChild(gui.domElement);
    let demoFolder = gui.addFolder('Demo Settings');
    let demo: ClothDemo | HashDemo;
    const initDemo = (s: string) => {
        // initialize renderer
        initThreeScene();
        demoFolder.destroy();
        demoFolder = gui.addFolder('Demo Settings');
        switch (s) {
            case 'Hash':
                demo = new HashDemo(rust_wasm, canvas, demoFolder, scene, camera);
                demo.init();
                break;
            case 'Cloth':
                demo = new ClothDemo(rust_wasm, canvas, demoFolder, scene, camera, controls);
                demo.init(renderer);
                break;
        }
    }
    folder.add(demoSelection, 'demoSelection', ['Cloth', 'Hash']).name('demo').onFinishChange(initDemo);

    // default init
    initDemo('Hash');

    // main loop
    const step = () => {
        stats.begin(); // collect perf data for stats.js
        demo.update();
        renderer.render(scene, camera);
        stats.end();
        requestAnimationFrame(step);
    }
    requestAnimationFrame(step);
}).catch(console.error);
