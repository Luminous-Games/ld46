import * as game from "luminous_ld46";

const canvas = document.createElement("canvas");
canvas.width = window.innerWidth;
canvas.height = window.innerHeight;
canvas.id = "canvas";
document.body.appendChild(canvas);
game.run();
