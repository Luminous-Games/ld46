import spritesheet from "./images/spritesheet.png";
import tuustid from "./images/tuustid.png";
import ui from "./images/ui.png";
import grass from "./images/tuustimaa.png";
import character from "./images/character.png";
import ludum46 from "./music/ludum46.m4a";
import dfhh from "./music/dfhh.mp3";
import duue from "./music/duue.mp3";
import quipp from "./music/quipp.mp3";
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
canvas.style.display = "none";
document.body.appendChild(canvas);

window.addEventListener("resize", function (event) {
    const canvas = document.getElementById("canvas");
    resize(canvas);
});

function startGame() {
    const audio = new Audio(ludum46);
    window.audio = audio;
    audio.loop = true;
    audio.volume = 0.2;
    audio.play();

    const audio2 = new Audio(dfhh);
    window.audio2 = audio2;
    audio2.loop = false;
    audio2.volume = 0.1;

    const audio3 = new Audio(duue);
    window.audio3 = audio3;
    audio3.loop = false;
    audio3.volume = 0.5;

    const audio4 = new Audio(quipp);
    window.audio4 = audio4;
    audio4.loop = false;
    audio4.volume = 0.2;

    const canvas = document.getElementById("canvas");
    canvas.style.display = "block";
    const tutorial = document.getElementById("tutorial");
    tutorial.style.display = "none";
    const mute = document.createElement("button");
    mute.style.position = "fixed";
    mute.style.top = "0";
    mute.style.right = "0";
    mute.onclick = function () {
        audio.muted = !audio.muted;
        audio2.muted = !audio2.muted;
        mute.textContent = audio.muted ? "Unmute" : "Mute";
    };
    mute.textContent = "Mute";
    document.body.appendChild(mute);

    game.run();
}

const startButton = document.getElementById("start_button");

let loadCount = 0;
let loadify = () => {
    ++loadCount;
    if (loadCount === 4) {
        startButton.onclick = startGame;
        startButton.disabled = false;
    }
};

const img0 = document.getElementById("character");
img0.src = character;

const img = document.getElementById("spritesheet");
img.src = spritesheet;
img.onload = loadify;
const img2 = document.getElementById("tuustid");
img2.src = tuustid;
img2.onload = loadify;
const img3 = document.getElementById("ui");
img3.src = ui;
img3.onload = loadify;
const img4 = document.getElementById("grass");
img4.src = grass;
img4.onload = loadify;

export function set_volume(v) {
    return (audio.volume = v);
}
