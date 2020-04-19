import spritesheet from "./images/spritesheet.png";
import * as game from "luminous_ld46";

// const aspect = 16 / 8;

function resize(canvas) {
  const w = document.body.clientWidth;
  const h = document.body.clientHeight;
  // if (w / aspect < h) {
  //   canvas.width = w;
  //   canvas.height = w / aspect;
  // } else {
  //   canvas.width = h * aspect;
  //   canvas.height = h;
  // }
  canvas.width = w;
  canvas.height = h;
}

const canvas = document.createElement("canvas");
resize(canvas);
canvas.id = "canvas";
canvas.style.background = "red";
document.body.appendChild(canvas);

window.addEventListener("resize", function (event) {
  const canvas = document.getElementById("canvas");
  resize(canvas);
});

const img = document.getElementById("texture");
img.src = spritesheet;
img.onload = () => {
  game.run();
};
