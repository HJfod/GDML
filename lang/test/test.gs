
let y = 5;

fun what(a: int, b: int) -> int {
    if a > b {
        return a + b;
    }
    else {
        return a - b;
    }
}

let x = what(2, 5) + 6;
