import init, { add } from "./pkg/matrix_step_action.js";

init().then(() => {
    assert(add(2, 2) === 4);
});
