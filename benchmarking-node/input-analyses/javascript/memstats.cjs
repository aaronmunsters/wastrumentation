// READS & WRITES keep count per byte in linear memory
let READS = [];
let WRITES = [];

// grow if out of bounds and set the bits
function increaseAt(buffer, address, targetSize) {
    if (buffer.length <= address + targetSize) {
        buffer.length = address + targetSize;
        buffer.fill(0, buffer.length, address + targetSize);
    }
    for (let i = 0; i < targetSize; i++) buffer[address + i] += 1;
}

Wasabi.analysis = {
    load: (_loc, op, memarg, _value) => {
        const effectiveAddr = memarg.addr + memarg.offset;
        let size;
        switch (op) {
            case 'i32.load8_s':
            case 'i32.load8_u':
            case 'i64.load8_s':
            case 'i64.load8_u':
                size = 1;
                break;
            case 'i32.load16_s':
            case 'i32.load16_u':
            case 'i64.load16_s':
            case 'i64.load16_u':
                size = 2;
                break;
            case 'i32.load':
            case 'f32.load':
            case 'i64.load32_s':
            case 'i64.load32_u':
                size = 4;
                break;
            case 'i64.load':
            case 'f64.load':
                size = 8;
                break;
        }
        increaseAt(READS, effectiveAddr, size);
    },

    store: (_loc, op, memarg, _value) => {
        const effectiveAddr = memarg.addr + memarg.offset;size
        let size;
        switch (op) {
            case 'i32.store8':
            case 'i64.store8':
                size = 1;
                break;
            case 'i32.store16':
            case 'i64.store16':
                size = 2;
                break;
            case 'i32.store':
            case 'f32.store':
            case 'i64.store32':
                size = 4;
                break;
            case 'i64.store':
            case 'f64.store':
                size = 8;
                break;
        }
        increaseAt(WRITES, effectiveAddr, size);
    }
};

Wasabi.analysisResult = {
    reads: READS,
    writes: WRITES,
};
