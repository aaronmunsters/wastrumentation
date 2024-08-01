const [_node, _script, llama_js_bin_name, model_bin_path, verbose] = process.argv;
const llama = require(`./${llama_js_bin_name}`);

let stdout = [];
let stderr = [];

llama["FS"].init(
    /* stdin callback */
    () => { console.log("Error: attempt to read from stdin"); process.exit(); },
    /* stdout callback */
    (ascii_or_null) => { ascii_or_null !== null && stdout.push(ascii_or_null); progress(); },
    /* stderr callback */
    (ascii_or_null) => { ascii_or_null !== null && stderr.push(ascii_or_null); },
);

llama['onRuntimeInitialized'] = () => {
    const c_function_name = 'inference';
    const c_return_type = 'number';
    const c_argument_types = ['string', 'string']; // argument types
    const inference =
        llama.cwrap(c_function_name, c_return_type, c_argument_types);

    inference(model_bin_path, "Aaron is a computer scientist");

    const stdout_string = String.fromCharCode(...stdout);
    const stderr_string = String.fromCharCode(...stderr);

    // stderr output should be of the following form:
    //           achieved tok/s: 18.400924

    // so we pattern match on it:
    const tokens_per_second = stderr_string.matchAll(/achieved tok\/s: (\d+(\.\d+)?)/g);
    const capture_first = tokens_per_second.next();

    // assert there's a first match
    console.assert(!capture_first.done);
    console.assert(capture_first.value.index === 0);
    const [ _, reported_value_string ] = capture_first.value;
    const reported_value = Number.parseFloat(reported_value_string);
    // assert there's no further matches
    const capture_second = tokens_per_second.next();
    console.assert(capture_second.value === undefined);
    console.assert(capture_second.done);

    console.log(reported_value);
}

function progress() {
    const total_characters = 905
    // there are +/- ${total_characters} characters to be written
    verbose && console.log(`${Math.floor(stdout.length*100/total_characters)}%`);
}
