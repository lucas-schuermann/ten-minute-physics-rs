"use strict";(self.webpackChunk=self.webpackChunk||[]).push([[235],{235:(_,t,e)=>{e.r(t),e.d(t,{BodyChainSimulation:()=>B,ClothSimulation:()=>I,FireSimulation:()=>j,FlipSimulation:()=>E,FluidSceneType:()=>M,FluidSimulation:()=>R,FractalsSceneType:()=>F,FractalsSimulation:()=>U,HashSimulation:()=>D,HeightFieldWaterSimulation:()=>$,ParallelClothSimulation:()=>q,ParallelClothSolverKind:()=>L,PositionBasedFluidSimulation:()=>z,SelfCollisionSimulation:()=>J,SkinnedSoftbodySimulation:()=>H,SoftBodiesSimulation:()=>V,default:()=>Z,initSync:()=>X,initThreadPool:()=>O,wbg_rayon_PoolBuilder:()=>N,wbg_rayon_start_worker:()=>W});var i=e(517);let s;const n=new Array(128).fill(void 0);function r(_){return n[_]}n.push(void 0,null,!0,!1);let o=n.length;function a(_){const t=r(_);return function(_){_<132||(n[_]=o,o=_)}(_),t}let l=null;function b(){return null!==l&&l.buffer===s.memory.buffer||(l=new Uint8Array(s.memory.buffer)),l}const g="undefined"!=typeof TextDecoder?new TextDecoder("utf-8",{ignoreBOM:!0,fatal:!0}):{decode:()=>{throw Error("TextDecoder not available")}};function u(_,t){return _>>>=0,g.decode(b().slice(_,_+t))}function w(_){o===n.length&&n.push(n.length+1);const t=o;return o=n[t],n[t]=_,t}function c(_){const t=typeof _;if("number"==t||"boolean"==t||null==_)return`${_}`;if("string"==t)return`"${_}"`;if("symbol"==t){const t=_.description;return null==t?"Symbol":`Symbol(${t})`}if("function"==t){const t=_.name;return"string"==typeof t&&t.length>0?`Function(${t})`:"Function"}if(Array.isArray(_)){const t=_.length;let e="[";t>0&&(e+=c(_[0]));for(let i=1;i<t;i++)e+=", "+c(_[i]);return e+="]",e}const e=/\[object ([^\]]+)\]/.exec(toString.call(_));let i;if(!(e.length>1))return toString.call(_);if(i=e[1],"Object"==i)try{return"Object("+JSON.stringify(_)+")"}catch(_){return"Object"}return _ instanceof Error?`${_.name}: ${_.message}\n${_.stack}`:i}"undefined"!=typeof TextDecoder&&g.decode();let d=0;const p="undefined"!=typeof TextEncoder?new TextEncoder("utf-8"):{encode:()=>{throw Error("TextEncoder not available")}},m=function(_,t){const e=p.encode(_);return t.set(e),{read:_.length,written:e.length}};function f(_,t,e){if(void 0===e){const e=p.encode(_),i=t(e.length,1)>>>0;return b().subarray(i,i+e.length).set(e),d=e.length,i}let i=_.length,s=t(i,1)>>>0;const n=b();let r=0;for(;r<i;r++){const t=_.charCodeAt(r);if(t>127)break;n[s+r]=t}if(r!==i){0!==r&&(_=_.slice(r)),s=e(s,i,i=r+3*_.length,1)>>>0;const t=b().subarray(s+r,s+i);r+=m(_,t).written}return d=r,s}let h=null;function y(){return null!==h&&h.buffer===s.memory.buffer||(h=new Int32Array(s.memory.buffer)),h}let v=null;function k(){return null!==v&&v.buffer===s.memory.buffer||(v=new Float32Array(s.memory.buffer)),v}function S(_,t){const e=t(4*_.length,4)>>>0;return k().set(_,e/4),d=_.length,e}let A=null;function x(_,t){return _>>>=0,(null!==A&&A.buffer===s.memory.buffer||(A=new Uint32Array(s.memory.buffer)),A).subarray(_/4,_/4+t)}function P(_,t){const e=t(1*_.length,1)>>>0;return b().set(_,e/1),d=_.length,e}function T(_,t){return _>>>=0,k().subarray(_/4,_/4+t)}function O(_){return a(s.initThreadPool(_))}function W(_){s.wbg_rayon_start_worker(_)}function C(_){return null==_}const L=Object.freeze({COLORING:0,0:"COLORING",JACOBI:1,1:"JACOBI"}),M=Object.freeze({WindTunnel:0,0:"WindTunnel",HiresTunnel:1,1:"HiresTunnel",Tank:2,2:"Tank",Paint:3,3:"Paint"}),F=Object.freeze({Julia:0,0:"Julia",Mandelbrot:1,1:"Mandelbrot"});class B{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_bodychainsimulation_free(_)}get num_objects(){return s.__wbg_get_bodychainsimulation_num_objects(this.__wbg_ptr)>>>0}get num_substeps(){return s.__wbg_get_bodychainsimulation_num_substeps(this.__wbg_ptr)}get dt(){return s.__wbg_get_bodychainsimulation_dt(this.__wbg_ptr)}get rot_damping(){return s.__wbg_get_bodychainsimulation_rot_damping(this.__wbg_ptr)}get pos_damping(){return s.__wbg_get_bodychainsimulation_pos_damping(this.__wbg_ptr)}get compliance(){return s.__wbg_get_bodychainsimulation_compliance(this.__wbg_ptr)}constructor(_,t,e,i,n,r,o){const a=S(t,s.__wbindgen_malloc),l=d,b=S(e,s.__wbindgen_malloc),g=d,u=s.bodychainsimulation_new(_,a,l,b,g,i,n,r,o);return this.__wbg_ptr=u>>>0,this}reset(_,t){const e=S(_,s.__wbindgen_malloc),i=d,n=S(t,s.__wbindgen_malloc),r=d;s.bodychainsimulation_reset(this.__wbg_ptr,e,i,n,r)}step(){s.bodychainsimulation_step(this.__wbg_ptr)}get poses(){return s.bodychainsimulation_poses(this.__wbg_ptr)>>>0}set num_substeps(_){s.bodychainsimulation_set_num_substeps(this.__wbg_ptr,_)}set pos_damping(_){s.bodychainsimulation_set_pos_damping(this.__wbg_ptr,_)}set rot_damping(_){s.bodychainsimulation_set_rot_damping(this.__wbg_ptr,_)}set compliance(_){s.bodychainsimulation_set_compliance(this.__wbg_ptr,_)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.bodychainsimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.bodychainsimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.bodychainsimulation_end_grab(this.__wbg_ptr,_,e,i)}}class I{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_clothsimulation_free(_)}get num_particles(){return s.__wbg_get_clothsimulation_num_particles(this.__wbg_ptr)>>>0}get num_tris(){return s.__wbg_get_clothsimulation_num_tris(this.__wbg_ptr)>>>0}get num_substeps(){return s.__wbg_get_clothsimulation_num_substeps(this.__wbg_ptr)}get dt(){return s.__wbg_get_clothsimulation_dt(this.__wbg_ptr)}get bending_compliance(){return s.__wbg_get_clothsimulation_bending_compliance(this.__wbg_ptr)}set bending_compliance(_){s.__wbg_set_clothsimulation_bending_compliance(this.__wbg_ptr,_)}get stretching_compliance(){return s.__wbg_get_clothsimulation_stretching_compliance(this.__wbg_ptr)}set stretching_compliance(_){s.__wbg_set_clothsimulation_stretching_compliance(this.__wbg_ptr,_)}constructor(_,t,e){const i=s.clothsimulation_new(_,t,e);return this.__wbg_ptr=i>>>0,this}get pos(){return s.bodychainsimulation_poses(this.__wbg_ptr)>>>0}get edge_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.clothsimulation_edge_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get tri_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.clothsimulation_tri_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}reset(){s.clothsimulation_reset(this.__wbg_ptr)}set solver_substeps(_){s.clothsimulation_set_solver_substeps(this.__wbg_ptr,_)}step(){s.clothsimulation_step(this.__wbg_ptr)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.clothsimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.clothsimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.clothsimulation_end_grab(this.__wbg_ptr,_,e,i)}}class j{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_firesimulation_free(_)}get dt(){return s.__wbg_get_firesimulation_dt(this.__wbg_ptr)}get num_iters(){return s.__wbg_get_firesimulation_num_iters(this.__wbg_ptr)>>>0}set num_iters(_){s.__wbg_set_firesimulation_num_iters(this.__wbg_ptr,_)}get over_relaxation(){return s.__wbg_get_firesimulation_over_relaxation(this.__wbg_ptr)}set over_relaxation(_){s.__wbg_set_firesimulation_over_relaxation(this.__wbg_ptr,_)}get burning_obstacle(){return 0!==s.__wbg_get_firesimulation_burning_obstacle(this.__wbg_ptr)}set burning_obstacle(_){s.__wbg_set_firesimulation_burning_obstacle(this.__wbg_ptr,_)}get burning_floor(){return 0!==s.__wbg_get_firesimulation_burning_floor(this.__wbg_ptr)}set burning_floor(_){s.__wbg_set_firesimulation_burning_floor(this.__wbg_ptr,_)}get num_cells(){return s.__wbg_get_firesimulation_num_cells(this.__wbg_ptr)>>>0}get swirl_probability(){return s.__wbg_get_firesimulation_swirl_probability(this.__wbg_ptr)}set swirl_probability(_){s.__wbg_set_firesimulation_swirl_probability(this.__wbg_ptr,_)}get show_obstacle(){return 0!==s.__wbg_get_firesimulation_show_obstacle(this.__wbg_ptr)}set show_obstacle(_){s.__wbg_set_firesimulation_show_obstacle(this.__wbg_ptr,_)}get show_swirls(){return 0!==s.__wbg_get_firesimulation_show_swirls(this.__wbg_ptr)}set show_swirls(_){s.__wbg_set_firesimulation_show_swirls(this.__wbg_ptr,_)}constructor(_,t,e){const i=s.firesimulation_new(_,t,w(e));return this.__wbg_ptr=i>>>0,this}step(){s.firesimulation_step(this.__wbg_ptr)}draw_buffer(_){var t=P(_,s.__wbindgen_malloc),e=d;s.firesimulation_draw_buffer(this.__wbg_ptr,t,e,w(_))}draw_canvas(){s.firesimulation_draw_canvas(this.__wbg_ptr)}set_obstacle_from_canvas(_,t,e){s.firesimulation_set_obstacle_from_canvas(this.__wbg_ptr,_,t,e)}}class E{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_flipsimulation_free(_)}get density(){return s.__wbg_get_flipsimulation_density(this.__wbg_ptr)}get dt(){return s.__wbg_get_flipsimulation_dt(this.__wbg_ptr)}get num_substeps(){return s.__wbg_get_flipsimulation_num_substeps(this.__wbg_ptr)>>>0}set num_substeps(_){s.__wbg_set_flipsimulation_num_substeps(this.__wbg_ptr,_)}get flip_ratio(){return s.__wbg_get_flipsimulation_flip_ratio(this.__wbg_ptr)}set flip_ratio(_){s.__wbg_set_flipsimulation_flip_ratio(this.__wbg_ptr,_)}get over_relaxation(){return s.__wbg_get_bodychainsimulation_dt(this.__wbg_ptr)}set over_relaxation(_){s.__wbg_set_flipsimulation_over_relaxation(this.__wbg_ptr,_)}get compensate_drift(){return 0!==s.__wbg_get_flipsimulation_compensate_drift(this.__wbg_ptr)}set compensate_drift(_){s.__wbg_set_flipsimulation_compensate_drift(this.__wbg_ptr,_)}get separate_particles(){return 0!==s.__wbg_get_flipsimulation_separate_particles(this.__wbg_ptr)}set separate_particles(_){s.__wbg_set_flipsimulation_separate_particles(this.__wbg_ptr,_)}get particle_num_cells(){return s.__wbg_get_flipsimulation_particle_num_cells(this.__wbg_ptr)>>>0}get num_particles(){return s.__wbg_get_flipsimulation_num_particles(this.__wbg_ptr)>>>0}get num_cells(){return s.__wbg_get_flipsimulation_num_cells(this.__wbg_ptr)>>>0}get show_obstacle(){return 0!==s.__wbg_get_flipsimulation_show_obstacle(this.__wbg_ptr)}set show_obstacle(_){s.__wbg_set_flipsimulation_show_obstacle(this.__wbg_ptr,_)}get show_particles(){return 0!==s.__wbg_get_flipsimulation_show_particles(this.__wbg_ptr)}set show_particles(_){s.__wbg_set_flipsimulation_show_particles(this.__wbg_ptr,_)}get show_grid(){return 0!==s.__wbg_get_flipsimulation_show_grid(this.__wbg_ptr)}set show_grid(_){s.__wbg_set_flipsimulation_show_grid(this.__wbg_ptr,_)}constructor(_,t,e){try{const r=s.__wbindgen_add_to_stack_pointer(-16);s.flipsimulation_new(r,_,t,w(e));var i=y()[r/4+0],n=y()[r/4+1];if(y()[r/4+2])throw a(n);return this.__wbg_ptr=i>>>0,this}finally{s.__wbindgen_add_to_stack_pointer(16)}}draw(){s.flipsimulation_draw(this.__wbg_ptr)}step(){s.flipsimulation_step(this.__wbg_ptr)}set_obstacle_from_canvas(_,t,e){s.flipsimulation_set_obstacle_from_canvas(this.__wbg_ptr,_,t,e)}}class R{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_fluidsimulation_free(_)}get density(){return s.__wbg_get_fluidsimulation_density(this.__wbg_ptr)}get dt(){return s.__wbg_get_firesimulation_over_relaxation(this.__wbg_ptr)}get num_iters(){return s.__wbg_get_fluidsimulation_num_iters(this.__wbg_ptr)>>>0}set num_iters(_){s.__wbg_set_fluidsimulation_num_iters(this.__wbg_ptr,_)}get over_relaxation(){return s.__wbg_get_flipsimulation_dt(this.__wbg_ptr)}set over_relaxation(_){s.__wbg_set_fluidsimulation_over_relaxation(this.__wbg_ptr,_)}get num_cells(){return s.__wbg_get_fluidsimulation_num_cells(this.__wbg_ptr)>>>0}get show_obstacle(){return 0!==s.__wbg_get_fluidsimulation_show_obstacle(this.__wbg_ptr)}set show_obstacle(_){s.__wbg_set_fluidsimulation_show_obstacle(this.__wbg_ptr,_)}get show_streamlines(){return 0!==s.__wbg_get_fluidsimulation_show_streamlines(this.__wbg_ptr)}set show_streamlines(_){s.__wbg_set_fluidsimulation_show_streamlines(this.__wbg_ptr,_)}get show_velocities(){return 0!==s.__wbg_get_fluidsimulation_show_velocities(this.__wbg_ptr)}set show_velocities(_){s.__wbg_set_fluidsimulation_show_velocities(this.__wbg_ptr,_)}get show_pressure(){return 0!==s.__wbg_get_fluidsimulation_show_pressure(this.__wbg_ptr)}set show_pressure(_){s.__wbg_set_fluidsimulation_show_pressure(this.__wbg_ptr,_)}get show_smoke(){return 0!==s.__wbg_get_fluidsimulation_show_smoke(this.__wbg_ptr)}set show_smoke(_){s.__wbg_set_fluidsimulation_show_smoke(this.__wbg_ptr,_)}constructor(_,t,e,i){const n=s.fluidsimulation_new(_,t,e,w(i));return this.__wbg_ptr=n>>>0,this}set_obstacle_from_canvas(_,t,e,i){s.fluidsimulation_set_obstacle_from_canvas(this.__wbg_ptr,_,t,e,i)}draw_buffer(_){var t=P(_,s.__wbindgen_malloc),e=d;s.fluidsimulation_draw_buffer(this.__wbg_ptr,t,e,w(_))}draw_canvas(){s.fluidsimulation_draw_canvas(this.__wbg_ptr)}step(){s.fluidsimulation_step(this.__wbg_ptr)}}class U{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_fractalssimulation_free(_)}get scene_type(){return s.__wbg_get_fractalssimulation_scene_type(this.__wbg_ptr)}set scene_type(_){s.__wbg_set_fractalssimulation_scene_type(this.__wbg_ptr,_)}get max_iters(){return s.__wbg_get_firesimulation_num_iters(this.__wbg_ptr)>>>0}set max_iters(_){s.__wbg_set_firesimulation_num_iters(this.__wbg_ptr,_)}get scale(){return s.__wbg_get_firesimulation_over_relaxation(this.__wbg_ptr)}set scale(_){s.__wbg_set_firesimulation_over_relaxation(this.__wbg_ptr,_)}get draw_mono(){return 0!==s.__wbg_get_fractalssimulation_draw_mono(this.__wbg_ptr)}set draw_mono(_){s.__wbg_set_fractalssimulation_draw_mono(this.__wbg_ptr,_)}get redraw(){return 0!==s.__wbg_get_fractalssimulation_redraw(this.__wbg_ptr)}set redraw(_){s.__wbg_set_fractalssimulation_redraw(this.__wbg_ptr,_)}constructor(_,t,e){const i=s.fractalssimulation_new(_,t,e);return this.__wbg_ptr=i>>>0,this}reset(){s.fractalssimulation_reset(this.__wbg_ptr)}draw_buffer(_){var t=P(_,s.__wbindgen_malloc),e=d;s.fractalssimulation_draw_buffer(this.__wbg_ptr,t,e,w(_))}handle_drag(_,t,e){s.fractalssimulation_handle_drag(this.__wbg_ptr,_,t,e)}}class D{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_hashsimulation_free(_)}get num_bodies(){return s.__wbg_get_hashsimulation_num_bodies(this.__wbg_ptr)>>>0}constructor(){const _=s.hashsimulation_new();return this.__wbg_ptr=_>>>0,this}reset(){s.hashsimulation_reset(this.__wbg_ptr)}step(){s.hashsimulation_step(this.__wbg_ptr)}static get radius(){return s.hashsimulation_radius()}get pos(){return s.hashsimulation_pos(this.__wbg_ptr)>>>0}get collisions(){return s.hashsimulation_collisions(this.__wbg_ptr)>>>0}}class ${__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_heightfieldwatersimulation_free(_)}get num_x(){return s.__wbg_get_heightfieldwatersimulation_num_x(this.__wbg_ptr)>>>0}get num_z(){return s.__wbg_get_heightfieldwatersimulation_num_z(this.__wbg_ptr)>>>0}get num_cells(){return s.__wbg_get_flipsimulation_num_cells(this.__wbg_ptr)>>>0}constructor(_,t,e,i,n){const r=s.heightfieldwatersimulation_new(_,t,e,i,n);return this.__wbg_ptr=r>>>0,this}get positions(){return s.heightfieldwatersimulation_positions(this.__wbg_ptr)>>>0}get ball_radii(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.heightfieldwatersimulation_ball_radii(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=T(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get ball_positions(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.heightfieldwatersimulation_ball_positions(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=T(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get uvs(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.heightfieldwatersimulation_uvs(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=T(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get indices(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.heightfieldwatersimulation_indices(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}step(_){s.heightfieldwatersimulation_step(this.__wbg_ptr,_)}reset(_){s.heightfieldwatersimulation_reset(this.__wbg_ptr,_)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.heightfieldwatersimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.heightfieldwatersimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.heightfieldwatersimulation_end_grab(this.__wbg_ptr,_,e,i)}}class q{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_parallelclothsimulation_free(_)}get num_particles(){return s.__wbg_get_parallelclothsimulation_num_particles(this.__wbg_ptr)>>>0}get num_tris(){return s.__wbg_get_parallelclothsimulation_num_tris(this.__wbg_ptr)>>>0}get num_dist_constraints(){return s.__wbg_get_parallelclothsimulation_num_dist_constraints(this.__wbg_ptr)>>>0}get num_substeps(){return s.__wbg_get_parallelclothsimulation_num_substeps(this.__wbg_ptr)}get dt(){return s.__wbg_get_parallelclothsimulation_dt(this.__wbg_ptr)}get solver_kind(){return s.__wbg_get_parallelclothsimulation_solver_kind(this.__wbg_ptr)}set solver_kind(_){s.__wbg_set_parallelclothsimulation_solver_kind(this.__wbg_ptr,_)}get obstacle_radius(){return s.__wbg_get_clothsimulation_stretching_compliance(this.__wbg_ptr)}constructor(_,t,e){const i=s.parallelclothsimulation_new(_,t,e);return this.__wbg_ptr=i>>>0,this}get pos(){return s.parallelclothsimulation_pos(this.__wbg_ptr)>>>0}get normals(){return s.parallelclothsimulation_normals(this.__wbg_ptr)>>>0}get obstacle_pos(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.parallelclothsimulation_obstacle_pos(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=T(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get tri_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.parallelclothsimulation_tri_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}reset(){s.parallelclothsimulation_reset(this.__wbg_ptr)}set num_substeps(_){s.parallelclothsimulation_set_num_substeps(this.__wbg_ptr,_)}step(){s.parallelclothsimulation_step(this.__wbg_ptr)}update_normals(){s.parallelclothsimulation_update_normals(this.__wbg_ptr)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.parallelclothsimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.parallelclothsimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.parallelclothsimulation_end_grab(this.__wbg_ptr,_,e,i)}}class z{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_positionbasedfluidsimulation_free(_)}constructor(_,t,e,i,n,r){try{const b=s.__wbindgen_add_to_stack_pointer(-16);s.positionbasedfluidsimulation_new(b,w(_),t,e,i,n,r);var o=y()[b/4+0],l=y()[b/4+1];if(y()[b/4+2])throw a(l);return this.__wbg_ptr=o>>>0,this}finally{s.__wbindgen_add_to_stack_pointer(16)}}set draw_single_color(_){s.positionbasedfluidsimulation_set_draw_single_color(this.__wbg_ptr,_)}get num_particles(){return s.positionbasedfluidsimulation_num_particles(this.__wbg_ptr)>>>0}set viscosity(_){s.positionbasedfluidsimulation_set_viscosity(this.__wbg_ptr,_)}set solver_substeps(_){s.positionbasedfluidsimulation_set_solver_substeps(this.__wbg_ptr,_)}step(){s.positionbasedfluidsimulation_step(this.__wbg_ptr)}add_block(){s.positionbasedfluidsimulation_add_block(this.__wbg_ptr)}reset(_,t){s.positionbasedfluidsimulation_reset(this.__wbg_ptr,_,t)}draw(){s.positionbasedfluidsimulation_draw(this.__wbg_ptr)}}class J{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_selfcollisionsimulation_free(_)}get num_particles(){return s.__wbg_get_selfcollisionsimulation_num_particles(this.__wbg_ptr)>>>0}get num_tris(){return s.__wbg_get_selfcollisionsimulation_num_tris(this.__wbg_ptr)>>>0}get dt(){return s.__wbg_get_selfcollisionsimulation_dt(this.__wbg_ptr)}get handle_collisions(){return 0!==s.__wbg_get_selfcollisionsimulation_handle_collisions(this.__wbg_ptr)}set handle_collisions(_){s.__wbg_set_selfcollisionsimulation_handle_collisions(this.__wbg_ptr,_)}get stretch_compliance(){return s.__wbg_get_selfcollisionsimulation_stretch_compliance(this.__wbg_ptr)}set stretch_compliance(_){s.__wbg_set_selfcollisionsimulation_stretch_compliance(this.__wbg_ptr,_)}get shear_compliance(){return s.__wbg_get_selfcollisionsimulation_shear_compliance(this.__wbg_ptr)}set shear_compliance(_){s.__wbg_set_selfcollisionsimulation_shear_compliance(this.__wbg_ptr,_)}get bending_compliance(){return s.__wbg_get_selfcollisionsimulation_bending_compliance(this.__wbg_ptr)}set bending_compliance(_){s.__wbg_set_selfcollisionsimulation_bending_compliance(this.__wbg_ptr,_)}get friction(){return s.__wbg_get_selfcollisionsimulation_friction(this.__wbg_ptr)}set friction(_){s.__wbg_set_selfcollisionsimulation_friction(this.__wbg_ptr,_)}constructor(_,t,e,i,n){const r=s.selfcollisionsimulation_new(_,t,e,i,n);return this.__wbg_ptr=r>>>0,this}get pos(){return s.bodychainsimulation_poses(this.__wbg_ptr)>>>0}get edge_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.clothsimulation_edge_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get tri_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.clothsimulation_tri_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}reset(_){s.selfcollisionsimulation_reset(this.__wbg_ptr,_)}set solver_substeps(_){s.selfcollisionsimulation_set_solver_substeps(this.__wbg_ptr,_)}step(){s.selfcollisionsimulation_step(this.__wbg_ptr)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.selfcollisionsimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.selfcollisionsimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.selfcollisionsimulation_end_grab(this.__wbg_ptr,_,e,i)}}class H{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_skinnedsoftbodysimulation_free(_)}get num_particles(){return s.__wbg_get_skinnedsoftbodysimulation_num_particles(this.__wbg_ptr)>>>0}get num_tris(){return s.__wbg_get_skinnedsoftbodysimulation_num_tris(this.__wbg_ptr)>>>0}get num_tets(){return s.__wbg_get_skinnedsoftbodysimulation_num_tets(this.__wbg_ptr)>>>0}get num_surface_verts(){return s.__wbg_get_skinnedsoftbodysimulation_num_surface_verts(this.__wbg_ptr)>>>0}get dt(){return s.__wbg_get_skinnedsoftbodysimulation_dt(this.__wbg_ptr)}get edge_compliance(){return s.__wbg_get_selfcollisionsimulation_shear_compliance(this.__wbg_ptr)}set edge_compliance(_){s.__wbg_set_selfcollisionsimulation_shear_compliance(this.__wbg_ptr,_)}get vol_compliance(){return s.__wbg_get_selfcollisionsimulation_bending_compliance(this.__wbg_ptr)}set vol_compliance(_){s.__wbg_set_selfcollisionsimulation_bending_compliance(this.__wbg_ptr,_)}constructor(_,t,e){const i=s.skinnedsoftbodysimulation_new(_,t,e);return this.__wbg_ptr=i>>>0,this}get pos(){return s.skinnedsoftbodysimulation_pos(this.__wbg_ptr)>>>0}get surface_pos(){return s.parallelclothsimulation_pos(this.__wbg_ptr)>>>0}get tet_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.skinnedsoftbodysimulation_tet_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get edge_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.skinnedsoftbodysimulation_edge_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}get surface_tri_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.skinnedsoftbodysimulation_surface_tri_ids(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}set solver_substeps(_){s.skinnedsoftbodysimulation_set_solver_substeps(this.__wbg_ptr,_)}reset(){s.skinnedsoftbodysimulation_reset(this.__wbg_ptr)}step(){s.skinnedsoftbodysimulation_step(this.__wbg_ptr)}squash(){s.skinnedsoftbodysimulation_squash(this.__wbg_ptr)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.skinnedsoftbodysimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.skinnedsoftbodysimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.skinnedsoftbodysimulation_end_grab(this.__wbg_ptr,_,e,i)}}class V{__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_softbodiessimulation_free(_)}constructor(_,t,e){const i=s.softbodiessimulation_new(_,t,e);return this.__wbg_ptr=i>>>0,this}get surface_tri_ids(){try{const i=s.__wbindgen_add_to_stack_pointer(-16);s.heightfieldwatersimulation_uvs(i,this.__wbg_ptr);var _=y()[i/4+0],t=y()[i/4+1],e=x(_,t).slice();return s.__wbindgen_free(_,4*t,4),e}finally{s.__wbindgen_add_to_stack_pointer(16)}}reset(){s.softbodiessimulation_reset(this.__wbg_ptr)}add_body(){s.softbodiessimulation_add_body(this.__wbg_ptr)}squash(){s.softbodiessimulation_squash(this.__wbg_ptr)}get num_particles_per_body(){return s.softbodiessimulation_num_particles_per_body(this.__wbg_ptr)>>>0}get num_tets(){return s.softbodiessimulation_num_tets(this.__wbg_ptr)>>>0}get dt(){return s.softbodiessimulation_dt(this.__wbg_ptr)}start_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.softbodiessimulation_start_grab(this.__wbg_ptr,_,e,i)}move_grabbed(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.softbodiessimulation_move_grabbed(this.__wbg_ptr,_,e,i)}end_grab(_,t){const e=S(t,s.__wbindgen_malloc),i=d;s.softbodiessimulation_end_grab(this.__wbg_ptr,_,e,i)}pos(_){return s.softbodiessimulation_pos(this.__wbg_ptr,_)>>>0}set solver_substeps(_){s.softbodiessimulation_set_solver_substeps(this.__wbg_ptr,_)}set edge_compliance(_){s.softbodiessimulation_set_edge_compliance(this.__wbg_ptr,_)}set volume_compliance(_){s.softbodiessimulation_set_volume_compliance(this.__wbg_ptr,_)}step(){s.softbodiessimulation_step(this.__wbg_ptr)}}class N{static __wrap(_){_>>>=0;const t=Object.create(N.prototype);return t.__wbg_ptr=_,t}__destroy_into_raw(){const _=this.__wbg_ptr;return this.__wbg_ptr=0,_}free(){const _=this.__destroy_into_raw();s.__wbg_wbg_rayon_poolbuilder_free(_)}numThreads(){return s.wbg_rayon_poolbuilder_numThreads(this.__wbg_ptr)>>>0}receiver(){return s.wbg_rayon_poolbuilder_receiver(this.__wbg_ptr)>>>0}build(){s.wbg_rayon_poolbuilder_build(this.__wbg_ptr)}}function G(){const _={};return _.wbg={},_.wbg.__wbindgen_object_drop_ref=function(_){a(_)},_.wbg.__wbg_createShader_ae014363ffc75c3a=function(_,t){const e=r(_).createShader(t>>>0);return C(e)?0:w(e)},_.wbg.__wbg_shaderSource_928e12db21ccefe3=function(_,t,e,i){r(_).shaderSource(r(t),u(e,i))},_.wbg.__wbg_compileShader_81181e6a219b7098=function(_,t){r(_).compileShader(r(t))},_.wbg.__wbg_getShaderParameter_37b950cbc20b6795=function(_,t,e){return w(r(_).getShaderParameter(r(t),e>>>0))},_.wbg.__wbindgen_boolean_get=function(_){const t=r(_);return"boolean"==typeof t?t?1:0:2},_.wbg.__wbg_getShaderInfoLog_20c948f5d991e6fd=function(_,t,e){const i=r(t).getShaderInfoLog(r(e));var n=C(i)?0:f(i,s.__wbindgen_malloc,s.__wbindgen_realloc),o=d;y()[_/4+1]=o,y()[_/4+0]=n},_.wbg.__wbg_createProgram_c835e8e8ff672d87=function(_){const t=r(_).createProgram();return C(t)?0:w(t)},_.wbg.__wbg_attachShader_06c432ad16c8823a=function(_,t,e){r(_).attachShader(r(t),r(e))},_.wbg.__wbg_linkProgram_bc5dc3f9357619ca=function(_,t){r(_).linkProgram(r(t))},_.wbg.__wbg_getProgramParameter_790db16915da3254=function(_,t,e){return w(r(_).getProgramParameter(r(t),e>>>0))},_.wbg.__wbg_getProgramInfoLog_056131faf2350ad7=function(_,t,e){const i=r(t).getProgramInfoLog(r(e));var n=C(i)?0:f(i,s.__wbindgen_malloc,s.__wbindgen_realloc),o=d;y()[_/4+1]=o,y()[_/4+0]=n},_.wbg.__wbg_random_9f310ce86e57ad05="function"==typeof Math.random?Math.random:("Math.random",()=>{throw new Error("Math.random is not defined")}),_.wbg.__wbindgen_copy_to_typed_array=function(_,t,e){var i,s;new Uint8Array(r(e).buffer,r(e).byteOffset,r(e).byteLength).set((i=_,s=t,i>>>=0,b().subarray(i/1,i/1+s)))},_.wbg.__wbindgen_string_new=function(_,t){return w(u(_,t))},_.wbg.__wbg_setlineWidth_52861f70ee5fc11d=function(_,t){r(_).lineWidth=t},_.wbg.__wbg_setstrokeStyle_1bf67b48c7e92f7c=function(_,t){r(_).strokeStyle=r(t)},_.wbg.__wbg_beginPath_dcaf84daa6be35fe=function(_){r(_).beginPath()},_.wbg.__wbg_arc_267b3955b82dae7d=function(){return function(_,t){try{return function(_,t,e,i,s,n){r(_).arc(t,e,i,s,n)}.apply(this,t)}catch(_){s.__wbindgen_exn_store(w(_))}}(0,arguments)},_.wbg.__wbg_closePath_45d0b0af592ad33e=function(_){r(_).closePath()},_.wbg.__wbg_stroke_abe71960396d06f0=function(_){r(_).stroke()},_.wbg.__wbg_viewport_8fc784fc0658898b=function(_,t,e,i,s){r(_).viewport(t,e,i,s)},_.wbg.__wbg_clearColor_d0e4ba6b3de36fbc=function(_,t,e,i,s){r(_).clearColor(t,e,i,s)},_.wbg.__wbg_getUniformLocation_a7c602314cbc2c05=function(_,t,e,i){const s=r(_).getUniformLocation(r(t),u(e,i));return C(s)?0:w(s)},_.wbg.__wbg_useProgram_fcb92641d4c3215f=function(_,t){r(_).useProgram(r(t))},_.wbg.__wbg_getAttribLocation_6c42e2cd1c2847f2=function(_,t,e,i){return r(_).getAttribLocation(r(t),u(e,i))},_.wbg.__wbg_createBuffer_6ead16b08a511599=function(_){const t=r(_).createBuffer();return C(t)?0:w(t)},_.wbg.__wbg_bindBuffer_c0ef32bca575b1bf=function(_,t,e){r(_).bindBuffer(t>>>0,r(e))},_.wbg.__wbg_bufferData_cbf46e29ed1643f0=function(_,t,e,i){r(_).bufferData(t>>>0,r(e),i>>>0)},_.wbg.__wbg_clear_7f98b4d14a417e94=function(_,t){r(_).clear(t>>>0)},_.wbg.__wbg_uniform1f_e051ff9c7bf1e081=function(_,t,e){r(_).uniform1f(r(t),e)},_.wbg.__wbg_uniform2f_3654c72e821a2089=function(_,t,e,i){r(_).uniform2f(r(t),e,i)},_.wbg.__wbg_uniform1i_f13bd7d6ad492b5a=function(_,t,e){r(_).uniform1i(r(t),e)},_.wbg.__wbg_vertexAttribPointer_0959b49dbd9a1b3e=function(_,t,e,i,s,n,o){r(_).vertexAttribPointer(t>>>0,e,i>>>0,0!==s,n,o)},_.wbg.__wbg_enableVertexAttribArray_224e3bb561570cc2=function(_,t){r(_).enableVertexAttribArray(t>>>0)},_.wbg.__wbindgen_memory=function(){return w(s.memory)},_.wbg.__wbg_buffer_344d9b41efe96da7=function(_){return w(r(_).buffer)},_.wbg.__wbg_newwithbyteoffsetandlength_4761a4dc62ec68a9=function(_,t,e){return w(new Float32Array(r(_),t>>>0,e>>>0))},_.wbg.__wbg_bufferSubData_5479137ae34eb123=function(_,t,e,i){r(_).bufferSubData(t>>>0,e,r(i))},_.wbg.__wbg_drawArrays_c5972b3d73095bf5=function(_,t,e,i){r(_).drawArrays(t>>>0,e,i)},_.wbg.__wbg_disableVertexAttribArray_eb9b9b0042076ad2=function(_,t){r(_).disableVertexAttribArray(t>>>0)},_.wbg.__wbg_uniform3fv_a5aa096fec7ac224=function(_,t,e,i){r(_).uniform3fv(r(t),T(e,i))},_.wbg.__wbg_uniform2fv_4d1cfba3bb56370b=function(_,t,e,i){r(_).uniform2fv(r(t),T(e,i))},_.wbg.__wbg_drawElements_510ac32d8abfd683=function(_,t,e,i,s){r(_).drawElements(t>>>0,e,i>>>0,s)},_.wbg.__wbg_uniformMatrix4fv_1826d923932cf3bb=function(_,t,e,i,s){r(_).uniformMatrix4fv(r(t),0!==e,T(i,s))},_.wbg.__wbg_moveTo_96f4d56b6d86d473=function(_,t,e){r(_).moveTo(t,e)},_.wbg.__wbg_lineTo_577441645a6c05ee=function(_,t,e){r(_).lineTo(t,e)},_.wbg.__wbg_setfillStyle_343558d6a1a50509=function(_,t){r(_).fillStyle=r(t)},_.wbg.__wbg_fill_530550bd1c480bcf=function(_){r(_).fill()},_.wbg.__wbg_newwithbyteoffsetandlength_2dfd4b7f2d9095c8=function(_,t,e){return w(new Uint16Array(r(_),t>>>0,e>>>0))},_.wbg.__wbindgen_debug_string=function(_,t){const e=f(c(r(t)),s.__wbindgen_malloc,s.__wbindgen_realloc),i=d;y()[_/4+1]=i,y()[_/4+0]=e},_.wbg.__wbindgen_throw=function(_,t){throw new Error(u(_,t))},_.wbg.__wbindgen_module=function(){return w(Y.__wbindgen_wasm_module)},_.wbg.__wbg_startWorkers_6fd3af285ea11136=function(_,t,e){return w((0,i.Q)(a(_),a(t),N.__wrap(e)))},_}function K(_,t){_.wbg.memory=t||new WebAssembly.Memory({initial:39,maximum:16384,shared:!0})}function Q(_,t){return s=_.exports,Y.__wbindgen_wasm_module=t,v=null,h=null,A=null,l=null,s.__wbindgen_start(),s}function X(_,t){if(void 0!==s)return s;const e=G();return K(e,t),_ instanceof WebAssembly.Module||(_=new WebAssembly.Module(_)),Q(new WebAssembly.Instance(_,e),_)}async function Y(_,t){if(void 0!==s)return s;void 0===_&&(_=new URL(e(275),e.b));const i=G();("string"==typeof _||"function"==typeof Request&&_ instanceof Request||"function"==typeof URL&&_ instanceof URL)&&(_=fetch(_)),K(i,t);const{instance:n,module:r}=await async function(_,t){if("function"==typeof Response&&_ instanceof Response){if("function"==typeof WebAssembly.instantiateStreaming)try{return await WebAssembly.instantiateStreaming(_,t)}catch(t){if("application/wasm"==_.headers.get("Content-Type"))throw t;console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n",t)}const e=await _.arrayBuffer();return await WebAssembly.instantiate(e,t)}{const e=await WebAssembly.instantiate(_,t);return e instanceof WebAssembly.Instance?{instance:e,module:_}:e}}(await _,i);return Q(n,r)}const Z=Y},275:(_,t,e)=>{_.exports=e.p+"b6e2c1046d6906e34608.wasm"}}]);