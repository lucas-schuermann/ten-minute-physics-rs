import GUI from 'lil-gui'
import * as THREE from 'three';

import { memory } from '../pkg/index_bg.wasm';
import { ClothSim } from '../pkg';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';

type ClothDemoProps = {
    triangles: number;
    vertices: number;
    animate: boolean;
    showEdges: boolean;
    substeps: number;
    bendingCompliance: number;
    stretchingCompliance: number;
    reset: () => void;
};

class ClothDemo {
    sim: ClothSim;
    canvas: HTMLCanvasElement;
    grabber: Grabber;
    scene: THREE.Scene;
    camera: THREE.Camera;
    controls: OrbitControls;
    props: ClothDemoProps;
    guiFolder: GUI;
    edgeMesh: any;
    triMesh: any;
    positions: any;

    constructor(rust_wasm: any, canvas: HTMLCanvasElement, guiFolder: GUI, scene: THREE.Scene, camera: THREE.Camera, controls: OrbitControls) {
        this.sim = new rust_wasm.ClothSim(canvas);

        // bound
        this.scene = scene;
        this.camera = camera;
        this.controls = controls;
        this.canvas = canvas;

        // members
        this.guiFolder = guiFolder;
    }

    init(renderer: THREE.Renderer) {
        // grab handler
        this.grabber = new Grabber(this.sim, renderer, this.camera, this.scene);
        const onPointer = (e: MouseEvent) => {
            e.preventDefault();
            if (e.type == "pointerdown") {
                this.grabber.mouseDown = true;
                this.grabber.start(e.clientX, e.clientY);
                if (this.grabber.intersectedObject) {
                    this.controls.saveState();
                    this.controls.enabled = false;
                }
            } else if (e.type == "pointermove" && this.grabber.mouseDown) {
                this.grabber.move(e.clientX, e.clientY);
            } else if (e.type == "pointerup") {
                this.grabber.mouseDown = false;
                this.controls.enabled = true;
                if (this.grabber.intersectedObject) {
                    this.grabber.end();
                    this.controls.reset();
                }
            }
        }
        this.canvas.addEventListener('pointerdown', onPointer, false);
        this.canvas.addEventListener('pointermove', onPointer, false);
        this.canvas.addEventListener('pointerup', onPointer, false);

        this.initControls();
        this.initMesh();
    }

    initControls() {
        // populate controls
        this.props = {
            triangles: this.sim.num_tris(),
            vertices: this.sim.num_particles(),
            animate: true,
            showEdges: false,
            substeps: 15,
            bendingCompliance: 1,
            stretchingCompliance: 0,
            reset: () => {
                console.log('not yet implemented'); // LVSTODO
                //this.sim.reset();
                this.updateMesh();
            },
        };
        this.guiFolder.add(this.props, 'triangles').disable();
        this.guiFolder.add(this.props, 'vertices').disable();
        this.guiFolder.add(this.props, 'substeps').min(1).max(30).step(1).onChange((v: number) => this.sim.set_solver_substeps(v));
        this.guiFolder.add(this.props, 'bendingCompliance').name('bending compliance').min(0).max(10).step(0.1).onChange((v: number) => this.sim.set_bending_compliance(v));
        this.guiFolder.add(this.props, 'stretchingCompliance').name('stretching compliance').min(0).max(1).step(0.01).onChange((v: number) => this.sim.set_stretching_compliance(v));
        this.guiFolder.add(this.props, 'showEdges').name('show edges').onChange((s: boolean) => {
            this.edgeMesh.visible = s;
            this.triMesh.visible = !s;
        });
        this.guiFolder.add(this.props, 'animate');
        //this.guiFolder.add(this.props, 'reset').name('reset simulation');
    }

    initMesh() {
        const tri_ids = Array.from(this.sim.mesh_tri_ids());
        const edge_ids = Array.from(this.sim.mesh_edge_ids());

        // NOTE: ordering matters here. The sim.mesh_*() getter methods are lazily implemented and 
        // allocate into a new Vec to collect results into at runtime. This means a heap allocation
        // occurs and therefore the location in memory for particle positions changes. Here, we
        // store the pointer to the positions buffer location after these allocs. In the WASM
        // linear heap, it will be constant thereafter, so we don't need to touch the array moving 
        // forward.
        const positionsPtr = this.sim.particle_positions_ptr();
        this.positions = new Float32Array(memory.buffer, positionsPtr, this.sim.num_particles() * 3);

        // visual edge mesh
        let geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(edge_ids);
        const lineMaterial = new THREE.LineBasicMaterial({ color: 0xff0000, linewidth: 2 });
        this.edgeMesh = new THREE.LineSegments(geometry, lineMaterial);
        this.edgeMesh.visible = false;
        this.scene.add(this.edgeMesh);

        // visual tri mesh
        geometry = new THREE.BufferGeometry();
        geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
        geometry.setIndex(tri_ids);
        const visMaterial = new THREE.MeshPhongMaterial({ color: 0xff0000, side: THREE.DoubleSide });
        this.triMesh = new THREE.Mesh(geometry, visMaterial);
        this.triMesh.castShadow = true;
        this.triMesh.layers.enable(1);
        this.scene.add(this.triMesh);
        geometry.computeVertexNormals();

        this.updateMesh();
    }

    update() {
        if (this.props.animate) {
            this.sim.step();
            this.updateMesh();
            this.grabber.increaseTime(this.sim.dt());
        }
    }

    updateMesh() {
        this.triMesh.geometry.computeVertexNormals();
        this.triMesh.geometry.attributes.position.needsUpdate = true;
        this.triMesh.geometry.computeBoundingSphere();
        this.edgeMesh.geometry.attributes.position.needsUpdate = true;
    }
}

class Grabber {
    sim: ClothSim;
    raycaster: THREE.Raycaster;
    intersectedObject: boolean;
    distance: number;
    mousePos: THREE.Vector2;
    mouseDown: boolean;
    prevPos: THREE.Vector3;
    vel: THREE.Vector3;
    time: number;
    rect: DOMRect;
    camera: THREE.Camera;
    scene: THREE.Scene;

    constructor(sim: ClothSim, renderer: THREE.Renderer, camera: THREE.Camera, scene: THREE.Scene) {
        this.sim = sim;
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
        this.rect = renderer.domElement.getBoundingClientRect();
        this.camera = camera;
        this.scene = scene;
    }
    increaseTime(dt: number) {
        this.time += dt;
    }
    updateRaycaster(x: number, y: number) {
        this.mousePos.x = ((x - this.rect.left) / this.rect.width) * 2 - 1;
        this.mousePos.y = -((y - this.rect.top) / this.rect.height) * 2 + 1;
        this.raycaster.setFromCamera(this.mousePos, this.camera);
    }
    start(x: number, y: number) {
        this.intersectedObject = false;
        this.updateRaycaster(x, y);
        const intersects = this.raycaster.intersectObjects(this.scene.children);
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
                /*
                if (simulating == false) {
                    toggleSimulating();
                }
                */
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

export { ClothDemo };