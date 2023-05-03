import * as Comlink from 'comlink';

// Define a transfer handler to allow sending events between a main thread
// DOM event listener on the input element and our worker, which will
// forward them to listeners downstream such as for OrbitControls and
// Grabber (raycast picking), which are running on the worker thread
Comlink.transferHandlers.set("event", {
    canHandle(obj: Event): obj is Event {
        // Cannot use instanceof `MouseEvent` et al. due to not being defined in worker, here and below
        // also, error with using obj.type in [XYZ], so we list out manually
        return obj instanceof Event && (obj.type === "pointerdown" || obj.type === "pointermove" || obj.type === "pointerup"
            || obj.type === "mousedown" || obj.type === "mousemove" || obj.type === "mouseup"
            || obj.type === "wheel");
    },
    serialize(obj: Event): [any, []] {
        if (obj.type === "pointerdown" || obj.type === "pointermove" || obj.type === "pointerup"
            || obj.type === "mousedown" || obj.type === "mousemove" || obj.type === "mouseup") {
            let e = obj as MouseEvent;
            return [
                {
                    type: e.type,
                    ctrlKey: e.ctrlKey,
                    metaKey: e.metaKey,
                    shiftKey: e.shiftKey,
                    button: e.button,
                    clientX: e.clientX,
                    clientY: e.clientY,
                },
                [],
            ];
        } else if (obj.type === "wheel") {
            let e = obj as WheelEvent;
            return [
                {
                    type: e.type,
                    deltaX: e.deltaX,
                    deltaY: e.deltaY,
                },
                [],
            ];
        } else {
            return [undefined, []];
        }
    },
    deserialize(obj: Event): Event {
        return obj;
    },
});