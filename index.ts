import GUI from 'lil-gui';
import * as Stats from 'stats.js';


import { SelfCollisionDemo, SelfCollisionDemoConfig } from './src/self_collision_15';
import { ClothDemo, ClothDemoConfig } from './src/cloth_14';
import { HashDemo, HashDemoConfig } from './src/hashing_11';
import { Demo, Scene, Scene2DCanvas, Scene2DWebGL, Scene3D, SceneConfig, Scene2DConfig, Scene3DConfig, initThreeScene } from './src/lib';
import { SoftBodiesDemo, SoftBodiesDemoConfig } from './src/softbodies_10';
import { SkinnedSoftbodyDemo, SkinnedSoftbodyDemoConfig } from './src/softbody_skinning_12';
import { FluidDemo, FluidDemoConfig } from './src/fluid_sim_17';
import { FlipDemo, FlipDemoConfig } from './src/flip_18';
import { BodyChainDemo, BodyChainDemoConfig } from './src/body_chain_challenge';
import { PositionBasedFluidDemo, PositionBasedFluidDemoConfig } from './src/fluid_2d_challenge';
import { ParallelClothDemo, ParallelClothDemoConfig } from './src/parallel_cloth_16';

import('./pkg').then(async rust_wasm => {
    const $ = (id: string) => document.getElementById(id);

    const demos: Record<string, { title: string, config: SceneConfig, demo: any }> = {
        '10-SoftBodies': {
            title: 'Soft Body Simulation',
            config: SoftBodiesDemoConfig,
            demo: SoftBodiesDemo,
        },
        '11-Hashing': {
            title: 'Spatial Hashing',
            config: HashDemoConfig,
            demo: HashDemo,
        },
        '12-SoftbodySkinning': {
            title: 'Soft Body Skinning',
            config: SkinnedSoftbodyDemoConfig,
            demo: SkinnedSoftbodyDemo,
        },
        '14-Cloth': {
            title: 'Cloth Simulation',
            config: ClothDemoConfig,
            demo: ClothDemo,
        },
        '15-SelfCollision': {
            title: 'Cloth Self Collision Handling',
            config: SelfCollisionDemoConfig,
            demo: SelfCollisionDemo,
        },
        '16-ParallelCloth': {
            title: 'Parallel Cloth Solver',
            config: ParallelClothDemoConfig,
            demo: ParallelClothDemo,
        },
        '17-FluidSimulation': {
            title: 'Euler Fluid',
            config: FluidDemoConfig,
            demo: FluidDemo,
        },
        '18-Flip': {
            title: 'Flip Fluid',
            config: FlipDemoConfig,
            demo: FlipDemo,
        },
        'Chall-Body-Chain': {
            title: 'Chain of 100 Bodies',
            config: BodyChainDemoConfig,
            demo: BodyChainDemo,
        },
        'Chall-2D-Fluid': {
            title: '2D Particle Fluid',
            config: PositionBasedFluidDemoConfig,
            demo: PositionBasedFluidDemo,
        }
    };
    const demoNames = Object.keys(demos);
    let canvas = $('canvas') as HTMLCanvasElement;
    let demo: Demo<any, any>;
    let scene: Scene;

    const replaceCanvas = () => {
        // some demos modify text color for contrast; reset
        document.getElementById('info').removeAttribute("style");
        // replace canvas element so we can get a new rendering context
        let newCanvas = document.createElement('canvas');
        canvas.parentNode.replaceChild(newCanvas, canvas);
        canvas = newCanvas;
    }

    const init2DScene = (config: Scene2DConfig): Scene2DCanvas | Scene2DWebGL => {
        replaceCanvas();

        canvas.width = window.innerWidth;
        canvas.height = window.innerHeight;
        let context;
        let kind = config.kind;
        if (kind === "2DCanvas") {
            context = canvas.getContext('2d', { desynchronized: true });
            return { kind, width: canvas.width, height: canvas.height, context };
        } else if (kind === "2DWebGL") {
            context = canvas.getContext('webgl2', { antialias: true, desynchronized: true, powerPreference: "high-performance" });
            return { kind, width: canvas.width, height: canvas.height, context };
        } else {
            throw "unreachable";
        }
    }

    const init3DScene = (config: Scene3DConfig): Scene3D => {
        console.log("LVSTEST: INIT 3d scene");

        replaceCanvas();

        let scene = initThreeScene(canvas, canvas, config);
        scene.renderer.setPixelRatio(window.devicePixelRatio);
        scene.renderer.setSize(window.innerWidth, window.innerHeight);
        return scene;
    };

    let resizeTimer: NodeJS.Timeout; // limit 2d resize events to once per 250ms
    window.addEventListener('resize', () => {
        // LVSTODO how to handle offscreen canvas?
        if (scene.kind === "3D") {
            // for 3d, THREE.js can non-destructively update the renderer
            const width = window.innerWidth;
            const height = window.innerHeight;
            scene.camera.aspect = width / height;
            scene.camera.updateProjectionMatrix();
            scene.renderer.setSize(width, height);
        } else {
            clearTimeout(resizeTimer);
            resizeTimer = setTimeout(() => {
                // for 2d, we generally need to reload the demo
                initDemo(props.demoSelection);
            }, 250);
        }
    });

    // attach perf stats window
    const stats = new Stats();
    stats.dom.style.position = 'absolute';
    const simPanel = stats.addPanel(new Stats.Panel('MS (Sim)', '#ff8', '#221'));
    let maxSimMs = 1;
    stats.showPanel(stats.dom.children.length - 1); // ms per sim step
    $('container').appendChild(stats.dom);

    // populate controls window
    const props = {
        demoSelection: demoNames.at(-1), // default to latest demo
        reset: () => demo.reset(),
    }
    const gui = new GUI({ autoPlace: false });
    gui.domElement.style.opacity = '0.9';
    $('gui').appendChild(gui.domElement);
    const generalFolder = gui.addFolder('General');
    let demoFolder: GUI;
    const initDemo = async (sid: string) => {
        if (demoFolder) demoFolder.destroy();
        demoFolder = gui.addFolder('Demo Settings');
        const config = demos[sid].config;
        if (config.kind === "3D") {
            if (config.offscreen === true) {
                console.log("LVSTEST: init 3d offscreen");
                scene = { kind: "3D", offscreen: true };
            } else {
                scene = init3DScene(config);
            }
        } else {
            scene = init2DScene(config); // LVSTODO why is this dumb
        }
        $('title').innerText = demos[sid].title;
        if (!(config.kind === "3D" && config.offscreen)) {
            demo = new demos[sid].demo(rust_wasm, canvas, scene, demoFolder);
        } else {
            console.log("LVS INIT");
            replaceCanvas();
            canvas.width = 1200;
            canvas.height = 800;
            demo = new demos[sid].demo(rust_wasm, canvas.transferControlToOffscreen(), canvas, config, demoFolder, stats, simPanel);
        }
        await demo.init();
    }
    generalFolder.add(props, 'demoSelection', demoNames).name('select demo').onFinishChange(await initDemo);
    generalFolder.add(props, 'reset').name('reset simulation');

    // default init
    await initDemo(props.demoSelection);

    // main loop
    const animate = () => {
        if (scene.kind === "3D" && scene.offscreen) {
            requestAnimationFrame(animate); // noop for offscreen canvas, main loop in web worker
            return;
        };

        stats.begin(); // collect perf data for stats.js
        let simTimeMs = performance.now();
        demo.update();
        simTimeMs = performance.now() - simTimeMs;
        if (scene.kind === "3D") {
            scene.renderer.render(scene.scene, scene.camera);
        } else {
            demo.draw();
        }
        simPanel.update(simTimeMs, (maxSimMs = Math.max(maxSimMs, simTimeMs)));
        stats.end();
        requestAnimationFrame(animate);
    }
    requestAnimationFrame(animate);
}).catch(console.error);

export { initThreeScene };
