export function set_volume(v) {
  console.log(v);
  console.log(window.audio);
  return (window.audio.volume = v);
}
