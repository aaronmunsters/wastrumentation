const counts = {}

Wasabi.analysis = {
    begin(location, type) {
        if (type !== "loop") return;
        counts[location.func] = counts[location.func] || {};
        counts[location.func][location.instr] = (counts[location.func][location.instr] || 0) + 1;
    }
}

Wasabi.analysisResult = () => counts;
