const globals = {}

Wasabi.analysis = {
    global(_loc, op, globalIndex, _v) {
        globals[globalIndex] = globals[globalIndex] || {r: 0, w: 0};
        switch (op) {
            case "global.set":
                globals[globalIndex].w++;
                return;
            case "global.get":
                globals[globalIndex].r++;
                return;
        }
    },
}

Wasabi.analysisResult = () => globals;
