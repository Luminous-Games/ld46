import spritesheet from "./images/spritesheet.png";
import tuustid from "./images/tuustid.png";
import ui from "./images/ui.png";
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

let loadCount = 0;
let loadify = () => {
  ++loadCount;
  if (loadCount === 3) {
    game.run();
  }
};

const img = document.getElementById("spritesheet");
img.src = spritesheet;
img.onload = loadify;
const img2 = document.getElementById("tuustid");
img2.src = tuustid;
img2.onload = loadify;
const img3 = document.getElementById("ui");
img3.src = ui;
img3.onload = loadify;

