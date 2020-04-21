export function set_volume(v) {
  return (window.audio.volume = v);
}

export function dfhh() {
  window.audio2.currentTime = 0;
  window.audio2.play();
}

export function duue() {
  window.audio3.currentTime = 0;
  window.audio3.play();
}

export function quipp() {
  window.audio4.currentTime = 0;
  window.audio4.play();
}

export function exeunt(message) {
  const exit = document.createElement("div");
  const text = document.createElement("p");
  text.innerText = message;
  text.style.color = "rgba(0,0,0,0)";
  exit.appendChild(text);
  document.body.appendChild(exit).focus();
  exit.className = "exeunt";
}
